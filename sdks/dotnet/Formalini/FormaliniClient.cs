using System.Net;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Formalini;

public class FormaliniApiException : Exception
{
    public HttpStatusCode StatusCode { get; }
    public TimeSpan? RetryAfter { get; }

    public FormaliniApiException(HttpStatusCode statusCode, string message, TimeSpan? retryAfter)
        : base(message)
    {
        StatusCode = statusCode;
        RetryAfter = retryAfter;
    }
}

public sealed class FormaliniUsageLimitException : FormaliniApiException
{
    public FormaliniUsageLimitException(string message) : base(HttpStatusCode.PaymentRequired, message, null) { }
}

public sealed class FormaliniClient
{
    private readonly HttpClient _http;
    private readonly string _apiKey;

    public DocumentResource Documents { get; }
    public WorkflowResource Workflows { get; }

    public FormaliniClient(HttpClient http, string apiKey, Uri? baseUrl = null)
    {
        if (string.IsNullOrWhiteSpace(apiKey)) throw new ArgumentException("API key is required.", nameof(apiKey));
        var uri = baseUrl ?? new Uri("https://api.formalini.com/api/public/v1/");
        if (uri.Scheme != Uri.UriSchemeHttps) throw new ArgumentException("Base URL must use HTTPS.", nameof(baseUrl));
        _http = http ?? throw new ArgumentNullException(nameof(http));
        _http.BaseAddress = uri;
        _apiKey = apiKey;
        Documents = new DocumentResource(this);
        Workflows = new WorkflowResource(this);
    }

    internal async Task<JsonDocument> SendAsync(HttpMethod method, string path, HttpContent? content, CancellationToken cancellationToken)
    {
        using var request = new HttpRequestMessage(method, path) { Content = content };
        request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", _apiKey);
        using var response = await _http.SendAsync(request, HttpCompletionOption.ResponseHeadersRead, cancellationToken);
        var body = await response.Content.ReadAsStringAsync(cancellationToken);
        if (!response.IsSuccessStatusCode)
        {
            string message = $"Formalini returned HTTP {(int)response.StatusCode}";
            try
            {
                using var error = JsonDocument.Parse(body);
                if (error.RootElement.TryGetProperty("error", out var value) && value.ValueKind == JsonValueKind.String)
                    message = value.GetString() ?? message;
            }
            catch (JsonException) { }
            if (response.StatusCode == HttpStatusCode.PaymentRequired) throw new FormaliniUsageLimitException(message);
            throw new FormaliniApiException(response.StatusCode, message, response.Headers.RetryAfter?.Delta);
        }
        try
        {
            var parsed = JsonDocument.Parse(body);
            if (parsed.RootElement.ValueKind != JsonValueKind.Object)
            {
                parsed.Dispose();
                throw new JsonException("Expected a JSON object.");
            }
            return parsed;
        }
        catch (JsonException error)
        {
            throw new InvalidDataException("Unexpected Formalini response.", error);
        }
    }
}

public sealed class DocumentResource
{
    private readonly FormaliniClient _client;
    internal DocumentResource(FormaliniClient client) => _client = client;

    public async Task<JsonDocument> ListAsync(int limit = 25, string? status = null, CancellationToken cancellationToken = default)
    {
        if (limit is < 1 or > 100) throw new ArgumentOutOfRangeException(nameof(limit));
        if (status is not null && status is not ("queued" or "processing" or "review" or "approved" or "exported" or "failed"))
            throw new ArgumentException("Unsupported status.", nameof(status));
        var path = $"documents?limit={limit}" + (status is null ? "" : $"&status={Uri.EscapeDataString(status)}");
        var result = await _client.SendAsync(HttpMethod.Get, path, null, cancellationToken);
        if (!result.RootElement.TryGetProperty("documents", out var documents) || documents.ValueKind != JsonValueKind.Array)
        {
            result.Dispose();
            throw new InvalidDataException("Expected a documents array.");
        }
        return result;
    }

    public Task<JsonDocument> CreateAsync(CreateDocument request, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(request.Name)) throw new ArgumentException("Name is required.", nameof(request));
        if (request.Fields?.Count > 400) throw new ArgumentException("Too many fields.", nameof(request));
        return _client.SendAsync(HttpMethod.Post, "documents", JsonContent.Create(request), cancellationToken);
    }

    public async Task<JsonDocument> UploadAsync(IReadOnlyList<string> imagePaths, string? name = null, string? templateId = null, bool asynchronous = false, CancellationToken cancellationToken = default)
    {
        if (imagePaths.Count is < 1 or > 25) throw new ArgumentOutOfRangeException(nameof(imagePaths));
        using var form = new MultipartFormDataContent();
        foreach (var path in imagePaths)
        {
            var mime = Path.GetExtension(path).ToLowerInvariant() switch
            {
                ".jpg" or ".jpeg" => "image/jpeg",
                ".png" => "image/png",
                ".webp" => "image/webp",
                ".heic" => "image/heic",
                _ => throw new ArgumentException("Upload accepts JPEG, PNG, WebP or HEIC images.", nameof(imagePaths))
            };
            var file = new FileInfo(path);
            if (file.Length > 100L * 1024 * 1024) throw new ArgumentException("Each image must be at most 100 MB.", nameof(imagePaths));
            var part = new StreamContent(file.OpenRead());
            part.Headers.ContentType = new MediaTypeHeaderValue(mime);
            form.Add(part, "file", file.Name);
        }
        if (name is not null) form.Add(new StringContent(name), "name");
        if (templateId is not null) form.Add(new StringContent(templateId), "template_id");
        if (asynchronous) form.Add(new StringContent("true"), "async");
        return await _client.SendAsync(HttpMethod.Post, "documents/upload", form, cancellationToken);
    }

    public Task<JsonDocument> GetAsync(string id, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(id)) throw new ArgumentException("Document ID is required.", nameof(id));
        return _client.SendAsync(HttpMethod.Get, $"documents/{Uri.EscapeDataString(id)}", null, cancellationToken);
    }

    public Task<JsonDocument> UpdateAsync(string id, UpdateDocument request, CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(id)) throw new ArgumentException("Document ID is required.", nameof(id));
        if (request.Status is null && request.Fields is null) throw new ArgumentException("A change is required.", nameof(request));
        if (request.Status is not null && request.Status is not ("review" or "approved" or "exported"))
            throw new ArgumentException("Unsupported status.", nameof(request));
        return _client.SendAsync(HttpMethod.Patch, $"documents/{Uri.EscapeDataString(id)}", JsonContent.Create(request), cancellationToken);
    }
}

public sealed class WorkflowResource
{
    private readonly FormaliniClient _client;
    internal WorkflowResource(FormaliniClient client) => _client = client;

    public Task<JsonDocument> DispatchAsync(DispatchEvent request, CancellationToken cancellationToken = default)
    {
        if (request.Event is not ("document.processed" or "document.needs_review" or "document.approved" or "document.exported") || string.IsNullOrWhiteSpace(request.DocumentId))
            throw new ArgumentException("A documented event and document ID are required.", nameof(request));
        return _client.SendAsync(HttpMethod.Post, "workflows/dispatch", JsonContent.Create(request), cancellationToken);
    }
}

public sealed record CreateDocument(
    [property: JsonPropertyName("name")] string Name,
    [property: JsonPropertyName("fields")] IReadOnlyList<object>? Fields = null,
    [property: JsonPropertyName("category")] string? Category = null,
    [property: JsonPropertyName("vendor")] string? Vendor = null,
    [property: JsonPropertyName("amount")] decimal? Amount = null,
    [property: JsonPropertyName("document_date")] string? DocumentDate = null);

public sealed record UpdateDocument(
    [property: JsonPropertyName("status")] string? Status = null,
    [property: JsonPropertyName("fields")] IReadOnlyList<object>? Fields = null);

public sealed record DispatchEvent(
    [property: JsonPropertyName("event")] string Event,
    [property: JsonPropertyName("document_id")] string DocumentId,
    [property: JsonPropertyName("payload")] object? Payload = null);
