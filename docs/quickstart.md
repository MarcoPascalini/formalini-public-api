# Process your first document

Create a free account at [Formalini](https://formalini.com/auth), then generate a scoped API key under Integrations. Keep the key in a server-side environment variable.

## Verify your key

```sh
export FORMALINI_API_KEY="your_api_key_here"
curl "https://api.formalini.com/api/public/v1/documents?limit=10" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}"
```

This requires `documents.read`. The published response has a `documents` array containing document IDs, names, statuses and extracted `fields`. Do not treat the number returned as a total across all documents.

## Upload page images

The upload endpoint accepts JPEG, PNG, WebP or HEIC images. It does **not** accept PDFs yet; render each PDF page to an image first. Repeat `file` for a multi-page document, up to 25 pages. Each image can be up to 100 MB.

```sh
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@invoice-page-1.jpg" \
  -F "name=Invoice example"
```

This requires `documents.write`. The published synchronous response contains a `document` object with an ID, status and extracted fields. For a faster initial response, send `-F "async=true"`; then poll `GET /documents/{id}`. Do not log the full response if it contains private document data.

See the [official examples](https://formalini.com/docs) for the latest contract. The [demo page](https://formalini.com/demo) shows a sample document and result; to try your own document, sign in.
