# Formalini Document AI API

**Documents in. Structured data out.** Turn page images, forms, invoices and receipts into structured data your application can use.

```text
PAGE IMAGE → FORMALINI → STRUCTURED DATA → YOUR APPLICATION
```

[See a worked demo](https://formalini.com/demo) · [Create a free account](https://formalini.com/auth) · [5-minute quickstart](docs/quickstart.md) · [API documentation](https://formalini.com/docs) · [Examples](examples/README.md)

Python · JavaScript · PHP · C# · Java · Go · Ruby · Rust · cURL

The [demo page](https://formalini.com/demo) shows a sample document and result. Sign in to process your own document. The API upload endpoint currently accepts JPEG, PNG, WebP and HEIC page images; render PDF pages to images before sending them.

## Verify your API key

Create a key under Integrations with `documents.read`, then run:

```sh
export FORMALINI_API_KEY="your_api_key_here"
curl "https://api.formalini.com/api/public/v1/documents?limit=10" \
  -H "Authorization: Bearer ${FORMALINI_API_KEY}"
```

The response contains a `documents` array. Each document can include its ID, name, status, and extracted fields. [Quickstart and upload example](docs/quickstart.md).

## SDKs

| Language | Status |
| --- | --- |
| [Python](sdks/python/README.md) | Six documented operations, mocked tests; unpublished |
| [JavaScript](sdks/javascript/README.md) | Six documented operations, mocked tests; unpublished |
| [PHP](sdks/php/README.md) | Six documented operations, mocked tests; unpublished |
| [C# / .NET](sdks/dotnet/README.md) | Six documented operations, mocked tests; unpublished |
| [Java](sdks/java/README.md) | Six documented operations, mocked tests; unpublished |
| [Go](sdks/go/README.md) | Six documented operations, mocked tests; unpublished |
| Ruby, Rust | Planned; no package or release claim |
| cURL | [Copyable requests](docs/quickstart.md) |

The SDK source, examples and documentation are licensed under [Apache 2.0](LICENSE). The hosted Formalini service requires an account and API key and is governed by its [service terms](https://formalini.com/legal/terms). Free access has [usage limits](https://formalini.com/pricing); higher usage may require credits or a paid plan.

**Keep secret API keys server-side.** Never embed them in browser JavaScript, public mobile apps, public repositories or distributed client source. Use environment variables or a secrets manager. Never log Authorization headers.

[Contract notes](docs/api-contract-report.md) · [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md)

Author: Marco Pascalini
