# Public API contract discovery

The [published Formalini API documentation](https://formalini.com/docs) was inspected on 2026-09-23. It is the primary contract for this repository. The adjacent local application checkout differs and appears older; its only `/api/public/v1` route is a listing endpoint with different base URL, scopes and response shape. Do not generate clients from that checkout.

## Published contract

Base URL: `https://api.formalini.com/api/public/v1`. Bearer keys are scoped to one workspace. Scopes: `documents.read`, `documents.write`, `workflows.run`. The docs show a default 60 requests per minute, `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `429` and `Retry-After`.

| Method | Path | Scope | Published behavior |
| --- | --- | --- | --- |
| GET | `/documents` | `documents.read` | `limit` 1–100, default 25; optional `status` (`queued`, `processing`, `review`, `approved`, `exported`, `failed`); newest first; response has `documents` array |
| POST | `/documents` | `documents.write` | JSON `name` required; optional `category`, `vendor`, `amount`, `document_date`, `fields` (up to 400) |
| POST | `/documents/upload` | `documents.write` | Multipart repeated `file` (1–25 page images), optional `name`, `template_id`, `async` |
| GET | `/documents/{id}` | `documents.read` | Document and extracted fields |
| PATCH | `/documents/{id}` | `documents.write` | JSON `status` and/or `fields` corrections by name |
| POST | `/workflows/dispatch` | `workflows.run` | JSON `event`, `document_id`, optional `payload` |

Upload accepts JPEG, PNG, WebP and HEIC, up to 100 MB **per page**. PDF is not accepted by this endpoint; render pages to JPEG or PNG before sending. Synchronous upload returns a document with fields. `async=true` returns 202 and requires polling `GET /documents/{id}`. A page read consumes one AI credit. A 402 can indicate exhausted credits, storage, or daily allowance. The docs do not define a separate machine-readable quota error code.

Dispatch events are `document.processed`, `document.needs_review`, `document.approved`, `document.exported`. Webhook deliveries include `X-Paperflow-Signature` with a SHA-256 HMAC of the raw body; verify before trusting a delivery.

Published errors include 400 invalid input, 401 missing/revoked/expired key, 403 insufficient scope, 404 missing workspace document, 429 rate limit, and 500 server failure. The upload page also documents 402. The docs show response examples, but do not fully define all optional fields, exact document state transitions, idempotency support, or every success status. SDKs preserve unknown response properties and do not retry mutations.

## Remaining verification

The published examples are not a substitute for sanitized responses from real authenticated calls or a machine-readable server schema. Before package publication, run opt-in integration tests against a dedicated workspace and verify status codes, error bodies, response variants, timeout behavior, correction semantics, and the deployed host. Do not use customer data.
