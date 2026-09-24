# Image to JSON with Formalini

Upload one JPEG, PNG, WebP or HEIC page image to receive a `document` result. Keep the page flat and legible, and inspect the returned status and fields. A photo can contain more than one logical document; split inputs if your application needs one record per document.

```sh
export FORMALINI_API_KEY="your_api_key_here"
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@page.webp"
```

The public API upload limit is 100 MB per page and 25 pages per request. See [upload details](../../docs/upload.md) for asynchronous processing. The [published API docs](https://formalini.com/docs) show the response shape.
