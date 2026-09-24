# Correct extracted data

`PATCH /documents/{id}` requires `documents.write`. Send `fields` as entries with `name` and corrected `value`, optionally with `status`. The [published example](https://formalini.com/docs) uses `status: "approved"` and a corrected `total` value. The response includes `ok` and `updated_fields`.

The published docs say changing status to `approved`, `exported` or `review` also fires the matching webhook. Check field values and your webhook subscriptions before sending a correction. The exact state transition rules are not fully published; preserve server errors instead of predicting every transition locally.

See the [correction example](../examples/correct-extracted-data/README.md).
