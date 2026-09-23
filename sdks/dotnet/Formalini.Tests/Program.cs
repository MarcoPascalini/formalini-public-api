using System.Net;
using System.Text;
using System.Text.Json;
using Formalini;

var handler = new RecordingHandler();
using var http = new HttpClient(handler);
var client = new FormaliniClient(http, "test-key");

using (var page = await client.Documents.ListAsync(10, "approved"))
{
    Check(page.RootElement.GetProperty("documents").GetArrayLength() == 1, "List response parsing");
}
Check(handler.LastMethod == HttpMethod.Get, "List method");
Check(handler.LastUri?.ToString() == "https://api.formalini.com/api/public/v1/documents?limit=10&status=approved", "List URL");
Check(handler.LastAuthorization == "Bearer test-key", "Bearer auth");

using (await client.Documents.CreateAsync(new CreateDocument("Invoice"))) { }
Check(handler.LastMethod == HttpMethod.Post && handler.LastUri?.AbsolutePath.EndsWith("/documents") == true, "Create request");
Check(JsonDocument.Parse(handler.LastBody!).RootElement.GetProperty("name").GetString() == "Invoice", "Create body");

using (await client.Documents.GetAsync("doc-id")) { }
Check(handler.LastUri?.AbsolutePath.EndsWith("/documents/doc-id") == true, "Get request");

using (await client.Documents.UpdateAsync("doc-id", new UpdateDocument(Status: "approved"))) { }
Check(handler.LastMethod == HttpMethod.Patch, "Update method");
Check(JsonDocument.Parse(handler.LastBody!).RootElement.GetProperty("status").GetString() == "approved", "Update body");

using (await client.Workflows.DispatchAsync(new DispatchEvent("document.approved", "doc-id"))) { }
Check(JsonDocument.Parse(handler.LastBody!).RootElement.GetProperty("document_id").GetString() == "doc-id", "Dispatch body");

var image = Path.Combine(Path.GetTempPath(), $"formalini-test-{Guid.NewGuid()}.jpg");
try
{
    await File.WriteAllBytesAsync(image, [1, 2, 3]);
    using (await client.Documents.UploadAsync([image], asynchronous: true)) { }
    Check(handler.LastBody?.Contains("name=file") == true, "Upload multipart file");
    Check(handler.LastBody?.Contains("name=async") == true, "Upload async field");
}
finally { File.Delete(image); }

handler.StatusCode = HttpStatusCode.Forbidden;
handler.ResponseBody = "{\"error\":\"Insufficient scope.\"}";
try
{
    using var ignored = await client.Documents.GetAsync("doc-id");
    throw new Exception("Expected API error");
}
catch (FormaliniApiException error)
{
    Check(error.StatusCode == HttpStatusCode.Forbidden && error.Message == "Insufficient scope.", "API error mapping");
}

Console.WriteLine("Formalini .NET request tests passed.");

static void Check(bool condition, string label)
{
    if (!condition) throw new Exception($"Failed: {label}");
}

sealed class RecordingHandler : HttpMessageHandler
{
    public HttpMethod? LastMethod { get; private set; }
    public Uri? LastUri { get; private set; }
    public string? LastAuthorization { get; private set; }
    public string? LastBody { get; private set; }
    public HttpStatusCode StatusCode { get; set; } = HttpStatusCode.OK;
    public string ResponseBody { get; set; } = "{\"documents\":[{\"id\":\"example\"}]}";

    protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
    {
        LastMethod = request.Method;
        LastUri = request.RequestUri;
        LastAuthorization = request.Headers.Authorization?.ToString();
        LastBody = request.Content is null ? null : await request.Content.ReadAsStringAsync(cancellationToken);
        return new HttpResponseMessage(StatusCode) { Content = new StringContent(ResponseBody, Encoding.UTF8, "application/json") };
    }
}
