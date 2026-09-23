# Formalini JavaScript client

This server-side Node.js client implements the six operations shown in the [published API documentation](https://formalini.com/docs). It is not published to npm.

```js
import { FormaliniClient } from './sdks/javascript/src/index.js';

const client = new FormaliniClient({ apiKey: process.env.FORMALINI_API_KEY });
const documents = await client.documents.list({ limit: 10 });
console.log(documents.map(({ id, name }) => ({ id, name })));
```

Upload with `client.documents.upload({ files: [imageBlob], name: 'Invoice example' })`. The client also exposes `documents.create`, `documents.get`, `documents.update` and `workflows.dispatch`. `FormaliniApiError` preserves status and `Retry-After`. No mutation is retried automatically. Run `npm test` from this directory.

Never bundle this client with a secret key into browser or public mobile code.
