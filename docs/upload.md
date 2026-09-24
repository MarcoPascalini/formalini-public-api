# Upload page images

`POST /documents/upload` uses `multipart/form-data` and requires `documents.write`. Repeat `file` once per page, from 1 to 25 pages. Supported formats are JPEG, PNG, WebP and HEIC, up to 100 MB per page. This endpoint does not accept PDF; render PDF pages to JPEG or PNG first.

Optional fields are `name`, `template_id` and `async`. With `async=true`, the API returns 202 and you poll `GET /documents/{id}`. Synchronous extraction may take several seconds, so configure a generous timeout. The response's `document` contains the ID, status and extracted data when processing completes. Each page read spends one AI credit.

See the [quickstart](quickstart.md) for a copyable request and [PDF to JSON example](../examples/pdf-to-json/README.md) for multi-page input. Handle 402 usage failures and 429 rate limiting as described in [errors](errors.md).
