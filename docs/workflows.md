# Workflow events and webhooks

`POST /workflows/dispatch` requires `workflows.run`. Send `event` and `document_id`, with optional application-defined `payload`. Allowed events are `document.processed`, `document.needs_review`, `document.approved`, and `document.exported`. Formalini sends the event to active webhooks subscribed to it.

Configure webhook URLs and subscriptions in the app. Deliveries use `X-Paperflow-Signature: sha256=<hex HMAC>` of the **raw request body** with the webhook secret. Verify before trusting the payload. The [official docs](https://formalini.com/docs) show verification in several languages and say failed deliveries can be replayed from the app.

See the [dispatch example](../examples/workflow-dispatch/README.md). Do not assume dispatch is idempotent; SDKs do not retry it automatically.
