# Dispatch document data to a connected system

`POST /workflows/dispatch` sends an event to active webhooks subscribed to it. It requires `workflows.run`. The API does not itself define a destination in this request; configure subscriptions in the Formalini app.

```sh
export FORMALINI_API_KEY="your_api_key_here"
export FORMALINI_DOCUMENT_ID="your_document_id_here"
curl -X POST "https://api.formalini.com/api/public/v1/workflows/dispatch" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -H "Content-Type: application/json" \
  -d "{\"event\":\"document.approved\",\"document_id\":\"${FORMALINI_DOCUMENT_ID}\",\"payload\":{\"target\":\"example\"}}"
```

Allowed events are `document.processed`, `document.needs_review`, `document.approved`, and `document.exported`. The example payload is application-defined metadata. Receivers should verify `X-Paperflow-Signature` using the raw request body and webhook secret as described in the [official docs](https://formalini.com/docs).
