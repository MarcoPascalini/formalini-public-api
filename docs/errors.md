# API errors and rate limits

The [published API docs](https://formalini.com/docs) list 400 for invalid input, 401 for a missing/revoked/expired key, 403 for insufficient scope, 404 for an unknown document in the key's workspace, 429 for the per-minute rate limit, and 500 for server failure. Upload can also return 402 when credits, storage or the daily allowance is exhausted. The error body explains the reason, but its full schema is not published.

The default published rate limit is 60 requests per minute. Responses carry `X-RateLimit-Limit` and `X-RateLimit-Remaining`. A 429 carries `Retry-After`; wait for that interval before retrying safe reads. The provided SDKs do not retry mutations automatically because idempotency behavior is not documented.

Never include an Authorization header or full API key in logs, exceptions, screenshots or support requests. Keep uploaded document data private as well.
