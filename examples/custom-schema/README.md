# Extract fields using a Formalini template

Use an existing template ID to receive fields with your configured names. This keeps your application mapping stable across similar documents, while you still validate the returned values.

```sh
export FORMALINI_API_KEY="your_api_key_here"
export FORMALINI_TEMPLATE_ID="your_template_id_here"
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@page.jpg" \
  -F "template_id=${FORMALINI_TEMPLATE_ID}"
```

Template creation and schema definition are managed in the app; their public request format is not documented. See [custom field names](../../docs/custom-schemas.md).
