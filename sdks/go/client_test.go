package formalini

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRequests(t *testing.T) {
	type observed struct{ method, path, auth, contentType, body string }
	var calls []observed
	status := 200
	responseBody := `{"documents":[{"id":"example"}],"ok":true}`
	transport := roundTripFunc(func(r *http.Request) (*http.Response, error) {
		var body []byte
		if r.Body != nil {
			body, _ = io.ReadAll(r.Body)
		}
		calls = append(calls, observed{r.Method, r.URL.String(), r.Header.Get("Authorization"), r.Header.Get("Content-Type"), string(body)})
		return &http.Response{StatusCode: status, Header: http.Header{"Content-Type": []string{"application/json"}}, Body: io.NopCloser(strings.NewReader(responseBody)), Request: r}, nil
	})
	client, err := New("test-key", WithBaseURL("https://api.formalini.test"), WithHTTPClient(&http.Client{Transport: transport}))
	if err != nil {
		t.Fatal(err)
	}
	ctx := context.Background()
	docs, err := client.Documents.List(ctx, 10, "approved")
	if err != nil || len(docs) != 1 {
		t.Fatalf("list: %v %d", err, len(docs))
	}
	if calls[0].path != "https://api.formalini.test/documents?limit=10&status=approved" || calls[0].auth != "Bearer test-key" {
		t.Fatalf("bad list request: %+v", calls[0])
	}

	_, err = client.Documents.Create(ctx, CreateDocument{Name: "Invoice", Fields: []Field{{Name: "total", Value: "4"}}})
	if err != nil || calls[1].method != "POST" {
		t.Fatalf("create: %v", err)
	}
	var created map[string]any
	if json.Unmarshal([]byte(calls[1].body), &created) != nil || created["name"] != "Invoice" {
		t.Fatal("bad create JSON")
	}
	_, err = client.Documents.Get(ctx, "doc-id")
	if err != nil || calls[2].path != "https://api.formalini.test/documents/doc-id" {
		t.Fatalf("get: %v", err)
	}
	_, err = client.Documents.Update(ctx, "doc-id", UpdateDocument{Status: "approved"})
	if err != nil || calls[3].method != "PATCH" {
		t.Fatalf("update: %v", err)
	}
	_, err = client.Workflows.Dispatch(ctx, "document.approved", "doc-id", nil)
	if err != nil || calls[4].path != "https://api.formalini.test/workflows/dispatch" {
		t.Fatalf("dispatch: %v", err)
	}

	image := filepath.Join(t.TempDir(), "page.jpg")
	if err := os.WriteFile(image, []byte("test image"), 0600); err != nil {
		t.Fatal(err)
	}
	_, err = client.Documents.Upload(ctx, []string{image}, "Example", "", true)
	if err != nil {
		t.Fatal(err)
	}
	if !strings.HasPrefix(calls[5].contentType, "multipart/form-data") || !strings.Contains(calls[5].body, `name="file"`) || !strings.Contains(calls[5].body, `name="async"`) {
		t.Fatalf("bad upload: %+v", calls[5])
	}

	status = 429
	responseBody = `{"error":"Rate limit reached"}`
	_, err = client.Documents.Get(ctx, "doc-id")
	var apiError *APIError
	if !errors.As(err, &apiError) || apiError.Status != 429 || apiError.Message != "Rate limit reached" {
		t.Fatalf("error mapping: %v", err)
	}
}

type roundTripFunc func(*http.Request) (*http.Response, error)

func (fn roundTripFunc) RoundTrip(request *http.Request) (*http.Response, error) { return fn(request) }

func TestValidation(t *testing.T) {
	if _, err := New(""); err == nil {
		t.Fatal("expected empty key rejection")
	}
	client, _ := New("test-key")
	if _, err := client.Documents.List(context.Background(), 0, ""); err == nil {
		t.Fatal("expected limit rejection")
	}
	if _, err := client.Documents.Upload(context.Background(), []string{"file.pdf"}, "", "", false); err == nil {
		t.Fatal("expected PDF rejection")
	}
}
