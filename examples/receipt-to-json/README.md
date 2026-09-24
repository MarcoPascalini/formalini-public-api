# Receipt to JSON with Formalini

Use a supported page image and inspect the `fields` in the returned `document`. Receipt photos can be uneven or cropped, so check merchant, date, currency and total before creating an expense in your own application. The API does not guarantee that every receipt exposes the same fields.

```sh
export FORMALINI_API_KEY="your_api_key_here"
curl -X POST "https://api.formalini.com/api/public/v1/documents/upload" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}" \
  -F "file=@receipt.jpg" \
  -F "name=Expense receipt"
```

If a value requires review, use the [correction API](../../docs/correcting-data.md) with its actual field name. Do not log the receipt or its extracted personal information. [Usage limits](../../docs/free-usage.md) apply per page read.
