# Secure API integration

Keep `FORMALINI_API_KEY` in a server-side environment variable or secrets manager. Never embed it in browser JavaScript, public mobile apps, public repositories, or distributed client source. Never log Authorization headers or full credentials.

Give each key only the scopes needed: `documents.read`, `documents.write`, and/or `workflows.run`. The key is tied to one workspace. Review who can issue and revoke keys in the application. Rotate a key if it is exposed.

Uploads and API responses may contain personal or business information. Limit access to documents, avoid production data in tests, and apply your application's retention and logging rules. Verify `X-Paperflow-Signature` against the raw webhook body before trusting an event; [official examples](https://formalini.com/docs) cover Node.js, Python, PHP and C#.
