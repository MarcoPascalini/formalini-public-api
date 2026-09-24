# List, create and retrieve documents

Use `GET /documents` with `documents.read` to retrieve recent workspace documents and their extracted `fields`. `limit` accepts 1–100 and defaults to 25. `status` can filter `queued`, `processing`, `review`, `approved`, `exported` or `failed`. The [published docs](https://formalini.com/docs) show a `documents` array ordered newest first; they do not describe cursor pagination or a total count.

Use `GET /documents/{id}` to retrieve one document. A 404 means the ID is not found in the key's workspace.

If you already possess structured data, `POST /documents` creates a document without uploading a page image. Supply `name` and optionally `category`, `vendor`, `amount`, `document_date`, and up to 400 `fields`. A document with fields enters review; one without enters the queue according to the published docs. See the official request example before mapping your field names and types.
