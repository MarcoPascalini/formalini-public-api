# Authentication and API keys

Create an account at [Formalini](https://formalini.com/auth). In the app, create a workspace-scoped API key under Integrations and grant only the scopes your integration needs:

| Scope | Operations |
| --- | --- |
| `documents.read` | List and retrieve documents |
| `documents.write` | Create, upload and update documents |
| `workflows.run` | Dispatch webhook events |

Send `Authorization: Bearer <FORMALINI_API_KEY>` with every API request. Keys are secret. Store them in server-side environment variables or a secrets manager. Never put them in browser JavaScript, public mobile apps, public repositories or distributed source. Rotate a compromised key in the app and update the server-side secret.
