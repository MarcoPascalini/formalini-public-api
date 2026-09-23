# Contributing

The `docs/` directory holds public guidance, `openapi/` records the [published contract](https://formalini.com/docs), and `sdks/` contains language clients. Start with the [contract report](docs/api-contract-report.md) before changing an operation.

For the Python client, use Python 3.10 or newer and run:

```sh
PYTHONPATH=sdks/python/src python -m unittest discover -s sdks/python/tests -v
npm test --prefix sdks/javascript
php sdks/php/tests/run.php
dotnet run --project sdks/dotnet/Formalini.Tests/Formalini.Tests.csproj
mvn -f sdks/java/pom.xml package
cd sdks/go && go test ./... && go vet ./...
```

Behavioral changes need focused mocked HTTP tests. Do not send production requests in unit tests. Add SDK methods only after confirming the server route, scope, request body, response, and errors. Keep language APIs idiomatic and avoid speculative models.

Use `FORMALINI_API_KEY` only as an environment variable. Never commit keys or customer data. Pull requests should explain the contract evidence, user-facing change, and test results. The source is licensed under Apache 2.0.
