# Process a handwritten page

The [Formalini demo](https://formalini.com/demo) shows a handwritten form with a low-confidence serial number flagged for review. For your own page, create an account and API key, then send a JPEG, PNG, WebP or HEIC image through the upload endpoint.

```sh
export FORMALINI_API_KEY="your_api_key_here"
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@handwritten-page.jpg" \
  -F "async=true"
```

Poll `GET /documents/{id}` after the 202 response. Inspect each returned field's confidence and validity rather than assuming the whole page is ready. Use the [correction endpoint](../../docs/correcting-data.md) for a value that needs a human decision.
