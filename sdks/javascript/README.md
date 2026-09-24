# Formalini JavaScript client

This server-side Node.js client implements the six operations shown in the [published API documentation](https://formalini.com/docs). TypeScript declarations are included. It is not published to npm.

```js
import { FormaliniClient } from './sdks/javascript/src/index.js';

const client = new FormaliniClient({ apiKey: process.env.FORMALINI_API_KEY });
const documents = await client.documents.list({ limit: 10 });
console.log(documents.map(({ id, name }) => ({ id, name })));
```

Upload with `client.documents.upload({ files: [imageBlob], name: 'Invoice example' })`. The client also exposes `documents.create`, `documents.get`, `documents.update` and `workflows.dispatch`. `FormaliniApiError` preserves status and `Retry-After`. No mutation is retried automatically. Run `npm test` from this directory.

HTTP 402 raises `FormaliniUsageLimitError` with the server's message. Handle it separately when showing account usage guidance.

Never bundle this client with a secret key into browser or public mobile code.
