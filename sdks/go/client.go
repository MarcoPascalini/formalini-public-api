package formalini

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"mime/multipart"
	"net/http"
	"net/textproto"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"time"
)

type APIError struct {
	Status     int
	Message    string
	RetryAfter string
}

func (e *APIError) Error() string { return e.Message }

type UsageLimitError struct{ *APIError }

func apiError(status int, message, retryAfter string) error {
	base := &APIError{Status: status, Message: message, RetryAfter: retryAfter}
	if status == http.StatusPaymentRequired {
		return &UsageLimitError{APIError: base}
	}
	return base
}

type Client struct {
	baseURL   string
	apiKey    string
	http      *http.Client
	Documents Documents
	Workflows Workflows
}

func New(apiKey string, options ...Option) (*Client, error) {
	if strings.TrimSpace(apiKey) == "" {
		return nil, errors.New("API key is required")
	}
	c := &Client{baseURL: "https://api.formalini.com/api/public/v1", apiKey: apiKey, http: &http.Client{Timeout: 60 * time.Second}}
	for _, option := range options {
		option(c)
	}
	if !strings.HasPrefix(c.baseURL, "https://") {
		return nil, errors.New("base URL must use HTTPS")
	}
	c.baseURL = strings.TrimRight(c.baseURL, "/")
	c.Documents.client = c
	c.Workflows.client = c
	return c, nil
}

type Option func(*Client)

func WithBaseURL(value string) Option          { return func(c *Client) { c.baseURL = value } }
func WithHTTPClient(value *http.Client) Option { return func(c *Client) { c.http = value } }

func (c *Client) request(ctx context.Context, method, path, contentType string, body io.Reader) (map[string]json.RawMessage, error) {
	req, err := http.NewRequestWithContext(ctx, method, c.baseURL+path, body)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Authorization", "Bearer "+c.apiKey)
	if contentType != "" {
		req.Header.Set("Content-Type", contentType)
	}
	response, err := c.http.Do(req)
	if err != nil {
		return nil, err
	}
	defer response.Body.Close()
	raw, err := io.ReadAll(io.LimitReader(response.Body, 16<<20))
	if err != nil {
		return nil, err
	}
	var decoded map[string]json.RawMessage
	if err := json.Unmarshal(raw, &decoded); err != nil || decoded == nil {
		if response.StatusCode >= 400 {
			return nil, apiError(response.StatusCode, fmt.Sprintf("Formalini returned HTTP %d", response.StatusCode), response.Header.Get("Retry-After"))
		}
		return nil, errors.New("unexpected Formalini response")
	}
	if response.StatusCode >= 400 {
		message := fmt.Sprintf("Formalini returned HTTP %d", response.StatusCode)
		var serverMessage string
		if json.Unmarshal(decoded["error"], &serverMessage) == nil && serverMessage != "" {
			message = serverMessage
		}
		return nil, apiError(response.StatusCode, message, response.Header.Get("Retry-After"))
	}
	return decoded, nil
}

func (c *Client) json(ctx context.Context, method, path string, payload any) (map[string]json.RawMessage, error) {
	raw, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}
	return c.request(ctx, method, path, "application/json", bytes.NewReader(raw))
}

type Documents struct{ client *Client }
type Workflows struct{ client *Client }

type Field struct {
	Name       string   `json:"name"`
	Value      any      `json:"value"`
	Type       string   `json:"type,omitempty"`
	Confidence *float64 `json:"confidence,omitempty"`
	Page       int      `json:"page,omitempty"`
}

type CreateDocument struct {
	Name         string   `json:"name"`
	Fields       []Field  `json:"fields,omitempty"`
	Category     string   `json:"category,omitempty"`
	Vendor       string   `json:"vendor,omitempty"`
	Amount       *float64 `json:"amount,omitempty"`
	DocumentDate string   `json:"document_date,omitempty"`
}

type UpdateDocument struct {
	Status string  `json:"status,omitempty"`
	Fields []Field `json:"fields,omitempty"`
}

func (d Documents) List(ctx context.Context, limit int, status string) ([]json.RawMessage, error) {
	if limit < 1 || limit > 100 {
		return nil, errors.New("limit must be between 1 and 100")
	}
	allowed := map[string]bool{"": true, "queued": true, "processing": true, "review": true, "approved": true, "exported": true, "failed": true}
	if !allowed[status] {
		return nil, errors.New("unsupported status")
	}
	query := url.Values{"limit": {fmt.Sprint(limit)}}
	if status != "" {
		query.Set("status", status)
	}
	result, err := d.client.request(ctx, http.MethodGet, "/documents?"+query.Encode(), "", nil)
	if err != nil {
		return nil, err
	}
	var documents []json.RawMessage
	if err := json.Unmarshal(result["documents"], &documents); err != nil || documents == nil {
		return nil, errors.New("expected a documents array")
	}
	return documents, nil
}

func (d Documents) Create(ctx context.Context, input CreateDocument) (map[string]json.RawMessage, error) {
	if strings.TrimSpace(input.Name) == "" || len(input.Fields) > 400 {
		return nil, errors.New("name is required and fields are limited to 400")
	}
	return d.client.json(ctx, http.MethodPost, "/documents", input)
}

func (d Documents) Get(ctx context.Context, id string) (map[string]json.RawMessage, error) {
	if id == "" {
		return nil, errors.New("document ID is required")
	}
	return d.client.request(ctx, http.MethodGet, "/documents/"+url.PathEscape(id), "", nil)
}

func (d Documents) Update(ctx context.Context, id string, input UpdateDocument) (map[string]json.RawMessage, error) {
	if id == "" || (input.Status == "" && input.Fields == nil) {
		return nil, errors.New("document ID and a change are required")
	}
	if input.Status != "" && input.Status != "review" && input.Status != "approved" && input.Status != "exported" {
		return nil, errors.New("unsupported status")
	}
	return d.client.json(ctx, http.MethodPatch, "/documents/"+url.PathEscape(id), input)
}

func (d Documents) Upload(ctx context.Context, paths []string, name, templateID string, async bool) (map[string]json.RawMessage, error) {
	if len(paths) < 1 || len(paths) > 25 {
		return nil, errors.New("upload requires 1 to 25 page images")
	}
	types := map[string]string{".jpg": "image/jpeg", ".jpeg": "image/jpeg", ".png": "image/png", ".webp": "image/webp", ".heic": "image/heic"}
	for _, path := range paths {
		if types[strings.ToLower(filepath.Ext(path))] == "" {
			return nil, errors.New("upload accepts JPEG, PNG, WebP or HEIC images")
		}
		info, err := os.Stat(path)
		if err != nil {
			return nil, err
		}
		if info.Size() > 100<<20 {
			return nil, errors.New("each page must be at most 100 MB")
		}
	}
	reader, writer := io.Pipe()
	defer reader.Close()
	multipartWriter := multipart.NewWriter(writer)
	go func() {
		var err error
		defer func() {
			if err != nil {
				writer.CloseWithError(err)
			} else {
				writer.Close()
			}
		}()
		for _, path := range paths {
			var file *os.File
			file, err = os.Open(path)
			if err != nil {
				return
			}
			header := make(textproto.MIMEHeader)
			filename := strings.NewReplacer("\"", "_", "\r", "_", "\n", "_").Replace(filepath.Base(path))
			header.Set("Content-Disposition", fmt.Sprintf(`form-data; name="file"; filename="%s"`, filename))
			header.Set("Content-Type", types[strings.ToLower(filepath.Ext(path))])
			var part io.Writer
			part, err = multipartWriter.CreatePart(header)
			if err == nil {
				_, err = io.Copy(part, file)
			}
			file.Close()
			if err != nil {
				return
			}
		}
		if name != "" {
			err = multipartWriter.WriteField("name", name)
			if err != nil {
				return
			}
		}
		if templateID != "" {
			err = multipartWriter.WriteField("template_id", templateID)
			if err != nil {
				return
			}
		}
		if async {
			err = multipartWriter.WriteField("async", "true")
			if err != nil {
				return
			}
		}
		err = multipartWriter.Close()
	}()
	return d.client.request(ctx, http.MethodPost, "/documents/upload", multipartWriter.FormDataContentType(), reader)
}

func (w Workflows) Dispatch(ctx context.Context, event, documentID string, payload map[string]any) (map[string]json.RawMessage, error) {
	allowed := map[string]bool{"document.processed": true, "document.needs_review": true, "document.approved": true, "document.exported": true}
	if !allowed[event] || documentID == "" {
		return nil, errors.New("a documented event and document ID are required")
	}
	body := map[string]any{"event": event, "document_id": documentID}
	if payload != nil {
		body["payload"] = payload
	}
	return w.client.json(ctx, http.MethodPost, "/workflows/dispatch", body)
}
