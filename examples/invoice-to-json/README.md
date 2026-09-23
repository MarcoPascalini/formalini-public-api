# Invoice to JSON with Formalini

Send a JPEG, PNG, WebP or HEIC invoice page with a `documents.write` key. For a multi-page invoice, repeat `file` once per page.

```sh
export FORMALINI_API_KEY="your_api_key_here"
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@invoice-page-1.jpg" \
  -F "name=Invoice example"
```

The [published example](https://formalini.com/docs) shows a `document` with `id`, `status`, `vendor`, `amount`, `currency`, `document_date`, `pages`, `credits_spent`, and `fields`. Those properties depend on what the document contains and how it is processed; check for missing values before using them. Inspect field confidence and correct values that require review before sending them onward.

If you already have invoice data, use `POST /documents` with a JSON `name` and optional `fields` instead of uploading an image. The [API docs](https://formalini.com/docs) give that request body.
