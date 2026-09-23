# Security

Report suspected vulnerabilities privately to contact@pascalini.com. Do not include live credentials or customer documents in reports.

Formalini secret API keys belong in server-side environment variables or a secrets manager. Do not embed them in browser-side JavaScript, public mobile applications, public repositories or distributed client source code. Never log the `Authorization` header or complete credentials.

The published API docs show a `pf_live_` prefix. An older local checkout uses `fm_live_`. Treat both as secret patterns.
