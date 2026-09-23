# Formalini .NET client

The `Formalini` project targets .NET 8 and covers the six operations in the [published API docs](https://formalini.com/docs). It has not been published to NuGet.

```csharp
using Formalini;

using var http = new HttpClient { Timeout = TimeSpan.FromSeconds(60) };
var client = new FormaliniClient(http, Environment.GetEnvironmentVariable("FORMALINI_API_KEY")!);
using var page = await client.Documents.ListAsync(limit: 10);
Console.WriteLine(page.RootElement.GetProperty("documents").GetArrayLength());
```

The client accepts an `HttpClient` for reuse, cancellation and testing. Responses are `JsonDocument` because the published docs do not enumerate every optional field; dispose them after use. Mutations are not retried automatically. Keep the key in a server-side secret store.
