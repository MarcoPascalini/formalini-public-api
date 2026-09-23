# Formalini Java client

This Java 11 client uses `java.net.http.HttpClient` and Gson. It covers the six operations in the [published API docs](https://formalini.com/docs). It is not published to Maven Central.

```java
var client = new com.formalini.FormaliniClient(System.getenv("FORMALINI_API_KEY"));
var documents = client.documents.list(10, null);
for (var document : documents) System.out.println(document.getAsJsonObject().get("id"));
```

For upload, pass image paths to `client.documents.upload(...)`. The client uses a caller-supplied `HttpClient` and timeout when needed, preserves status and `Retry-After` in `ApiException`, and never retries mutations. Multipart bodies are assembled in memory, so use raw HTTP for very large batches.

The repository includes a mock `HttpClient` request suite at `src/test/java/com/formalini/FormaliniClientTest.java`.
