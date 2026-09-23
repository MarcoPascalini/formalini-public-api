# Correct extracted document data

Use the document ID returned by upload or listing. This example corrects a field named `total` and changes the status to `approved`; both actions require `documents.write`.

```sh
export FORMALINI_API_KEY="your_api_key_here"
export FORMALINI_DOCUMENT_ID="your_document_id_here"
curl -X PATCH "https://api.formalini.com/api/public/v1/documents/${FORMALINI_DOCUMENT_ID}" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -H "Content-Type: application/json" \
  -d '{"status":"approved","fields":[{"name":"total","value":"1249.50"}]}'
```

The [published response](https://formalini.com/docs) includes `ok` and `updated_fields`. Read the document again to inspect the corrected value. Status changes to `approved`, `exported`, or `review` can trigger subscribed webhooks; plan for that side effect before using this call.
