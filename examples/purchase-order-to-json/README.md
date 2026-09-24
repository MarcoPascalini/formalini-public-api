# Purchase order data extraction

Upload purchase-order page images and inspect the document result before matching it with an order in your system. Names, identifiers and monetary amounts can vary by supplier; validate them against your own records. This example does not implement ERP posting or a complete purchasing workflow.

```sh
export FORMALINI_API_KEY="your_api_key_here"
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@purchase-order-page-1.png" \
  -F "name=Purchase order example"
```

Use `template_id` if you have configured consistent field names in the app. The [official API docs](https://formalini.com/docs) define the request and upload limits; the exact purchase-order field set depends on your template and document.
