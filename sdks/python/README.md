# Formalini Python client

Implements the six operations shown in the [published API documentation](https://formalini.com/docs). The package is not published to PyPI; install from this checkout with `pip install -e sdks/python`.

```python
import os
from formalini import Client

client = Client(os.environ["FORMALINI_API_KEY"])
for document in client.documents.list(limit=10):
    print(document["id"], document["name"])

result = client.documents.upload(["invoice-page-1.jpg"], name="Invoice example")
print(result["document"]["id"])
```

The client also supports `documents.create`, `documents.get`, `documents.update`, and `workflows.dispatch`. Responses remain dictionaries because the published examples do not define every optional field. `ApiError` exposes `status` and `retry_after`. Network and timeout errors propagate. Mutations are never retried automatically. Uploads currently assemble the multipart body in memory, so use the raw HTTP example for very large batches.

Run tests from the repository root with `PYTHONPATH=sdks/python/src python -m unittest discover -s sdks/python/tests -v`.
