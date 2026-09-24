# Extract data from a form

For recurring forms, create a template in the Formalini app and pass its ID as `template_id`. The published docs say the response fields then use your configured names. Without a template, inspect the returned field names rather than assuming a fixed schema.

```sh
export FORMALINI_API_KEY="your_api_key_here"
export FORMALINI_TEMPLATE_ID="your_template_id_here"
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@completed-form.png" \
  -F "template_id=${FORMALINI_TEMPLATE_ID}"
```

The public docs do not specify a template creation API. See [custom field names](../../docs/custom-schemas.md) for that boundary. Check required values, especially handwriting and checkboxes, before using the result.
