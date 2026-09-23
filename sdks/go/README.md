# Formalini Go client

The Go client uses the standard library, accepts `context.Context` on every operation, and covers the six operations in the [published API docs](https://formalini.com/docs). It is not published as a module release.

```go
client, err := formalini.New(os.Getenv("FORMALINI_API_KEY"))
if err != nil { log.Fatal(err) }
documents, err := client.Documents.List(context.Background(), 10, "")
```

`WithHTTPClient` and `WithBaseURL` support custom timeouts and testing. Upload streams page images as multipart. `APIError` preserves status and `Retry-After`. No mutation is retried automatically.
