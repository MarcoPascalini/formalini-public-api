# PDF to JSON with Formalini

The public upload endpoint accepts page images, not PDF bytes. Render each page of a PDF to JPEG or PNG using a tool you trust, then send each page as a repeated `file` field. The endpoint accepts up to 25 page images and each page can be up to 100 MB.

```sh
export FORMALINI_API_KEY="your_api_key_here"
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@page-1.png" \
  -F "file=@page-2.png" \
  -F "name=Example PDF"
```

The synchronous response contains `document`, including an ID, status and extracted `fields`. If extraction takes too long for your request timeout, add `-F "async=true"` and poll `GET /documents/{id}`. One AI credit is spent per page read. See [errors](../../docs/errors.md) for 402 and 429 handling.

Only send documents you are authorized to process. The hosted service stores and processes the document under the API key's workspace.
