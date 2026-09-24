# Formalini Rust client

The Rust client uses blocking `reqwest` and covers the six operations in the [published API docs](https://formalini.com/docs). It has not been published to crates.io.

```rust
let client = formalini_public_api_client::Client::new(std::env::var("FORMALINI_API_KEY")?)?;
let documents = client.documents().list(10, None)?;
```

`Client::with_base_url` configures an HTTPS endpoint and timeout. `Error::Api` preserves status and `Retry-After`. Upload uses repeated multipart `file` fields. Mutations are never retried automatically.
