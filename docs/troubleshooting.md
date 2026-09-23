# Troubleshooting

**401 or 403:** Confirm that the key is active, is used as a bearer token, and has the operation's scope. Keys see only their own workspace.

**400 on upload:** Send page images as repeated multipart `file` fields. The public endpoint accepts JPEG, PNG, WebP and HEIC, not PDF. Check the 25-page limit and 100 MB per-page limit.

**402 on upload:** The account may be out of credits, storage or its daily allowance. Read the response and check [current usage terms](free-usage.md).

**429:** Respect `Retry-After`. Avoid automatically retrying create, upload, update or dispatch requests because idempotency is not documented.

**Long upload:** Use multipart `async=true` and poll `GET /documents/{id}`. Do not assume processing is complete when the initial request returns 202.
