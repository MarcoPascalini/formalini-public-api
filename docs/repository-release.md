# Repository and release setup

Suggested GitHub repository: `formalini/document-ai-api`, if Marco controls that organization and the name is available. Suggested description: “Open-source SDKs and examples for extracting structured data from documents with the Formalini API.”

Suggested topics (within GitHub's 20-topic limit): `document-ai`, `document-processing`, `document-extraction`, `pdf-to-json`, `invoice-parser`, `ocr-api`, `receipt-parser`, `form-recognition`, `structured-data`, `api`, `python`, `javascript`, `php`, `dotnet`, `java`, `golang`, `ruby`, `rust`.

Suggested initial repository release tag: `v0.1.0`, after authenticated contract checks and a package-name/ownership check. No package should be uploaded automatically from CI.

Package names and namespace ownership have **not** been checked across PyPI, npm, Packagist, NuGet, Maven Central, RubyGems, and crates.io. Package metadata is a starting point, not permission to publish under those names. Go uses the suggested GitHub module path, which must match the eventual remote.

GitHub setup after repository creation: set the description and topics, upload a social preview following [the asset specification](../assets/README.md), enable branch protection for `main` with required Verify checks, enable Dependabot and security alerts, then run the [pre-publication contract checks](api-contract-report.md). A maintainer should create a controlled release after those checks.
