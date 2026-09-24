# Formalini API with cURL

Create a [free account and scoped key](https://formalini.com/auth). Set `FORMALINI_API_KEY` in your server-side shell. The key needs `documents.read` for reads, `documents.write` for creation/upload/correction, and `workflows.run` for dispatch.

```sh
export FORMALINI_API_KEY="your_api_key_here"
```

List documents:

```sh
curl "https://api.formalini.com/api/public/v1/documents?limit=10" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}"
```

Create a document with existing values:

```sh
curl -X POST "https://api.formalini.com/api/public/v1/documents" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -H "Content-Type: application/json" \
  --data '{"name":"Invoice example","fields":[{"name":"total","value":"42.00"}]}'
```

Upload a JPEG page for extraction. Repeat `-F "file=@..."` for multiple pages, up to 25. PDFs must first be rendered to page images.

```sh
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@invoice-page-1.jpg" \
  -F "name=Invoice example"
```

Retrieve the document using its returned ID:

```sh
curl "https://api.formalini.com/api/public/v1/documents/DOCUMENT_ID" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}"
```

Correct or approve it:

```sh
curl -X PATCH "https://api.formalini.com/api/public/v1/documents/DOCUMENT_ID" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -H "Content-Type: application/json" \
  --data '{"status":"approved"}'
```

Dispatch a documented event:

```sh
curl -X POST "https://api.formalini.com/api/public/v1/workflows/dispatch" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -H "Content-Type: application/json" \
  --data '{"event":"document.approved","document_id":"DOCUMENT_ID"}'
```

These examples follow the [published API documentation](https://formalini.com/docs). Inspect response status and body before using returned fields. HTTP 402 means a usage allowance was reached, and HTTP 429 means the request rate was exceeded; see [error guidance](../../docs/errors.md). Never put the key in browser code or a public repository.
