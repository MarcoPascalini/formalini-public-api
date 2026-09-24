import assert from 'node:assert/strict';
import test from 'node:test';
import { FormaliniApiError, FormaliniClient, FormaliniResponseError, FormaliniUsageLimitError } from '../src/index.js';

const ok = data => ({ ok: true, json: async () => data });

test('lists documents using published host and response', async () => {
  let observed;
  const client = new FormaliniClient({ apiKey: 'test-key', fetchImpl: async (url, options) => {
    observed = { url, options };
    return ok({ documents: [{ id: 'example' }] });
  }});
  assert.equal((await client.documents.list({ limit: 10, status: 'approved' }))[0].id, 'example');
  assert.equal(observed.url, 'https://api.formalini.com/api/public/v1/documents?limit=10&status=approved');
  assert.equal(observed.options.headers.Authorization, 'Bearer test-key');
});

test('constructs JSON create, update and dispatch requests', async () => {
  const calls = [];
  const client = new FormaliniClient({ apiKey: 'test-key', fetchImpl: async (url, options) => { calls.push({ url, options }); return ok({ ok: true }); } });
  await client.documents.create({ name: 'Invoice', fields: [] });
  await client.documents.get('doc-id');
  await client.documents.update('doc-id', { status: 'approved' });
  await client.workflows.dispatch({ event: 'document.approved', documentId: 'doc-id' });
  assert.deepEqual(calls.map(call => call.options.method), ['POST', 'GET', 'PATCH', 'POST']);
  assert.deepEqual(JSON.parse(calls[0].options.body), { name: 'Invoice', fields: [] });
  assert.deepEqual(JSON.parse(calls[2].options.body), { status: 'approved' });
  assert.deepEqual(JSON.parse(calls[3].options.body), { event: 'document.approved', document_id: 'doc-id' });
});

test('upload repeats image file fields', async () => {
  let body;
  const client = new FormaliniClient({ apiKey: 'test-key', fetchImpl: async (_url, options) => { body = options.body; return ok({ document: { id: 'example' } }); } });
  await client.documents.upload({ files: [new Blob(['a'], { type: 'image/jpeg' }), new Blob(['b'], { type: 'image/png' })], asynchronous: true });
  assert.equal(body.getAll('file').length, 2);
  assert.equal(body.get('async'), 'true');
  assert.throws(() => client.documents.upload({ files: [new Blob(['pdf'], { type: 'application/pdf' })] }), TypeError);
});

test('validates inputs and preserves rate-limit errors', async () => {
  const client = new FormaliniClient({ apiKey: 'test-key', fetchImpl: async () => ({ ok: false, status: 429, headers: new Headers({ 'Retry-After': '10' }), json: async () => ({ error: 'Rate limit reached' }) }) });
  assert.throws(() => new FormaliniClient({ apiKey: '' }), TypeError);
  await assert.rejects(client.documents.list({ limit: 0 }), RangeError);
  await assert.rejects(client.documents.list(), error => error instanceof FormaliniApiError && error.status === 429 && error.retryAfter === '10');
  const malformed = new FormaliniClient({ apiKey: 'test-key', fetchImpl: async () => ok({ wrong: [] }) });
  await assert.rejects(malformed.documents.list(), FormaliniResponseError);
  const limited = new FormaliniClient({ apiKey: 'test-key', fetchImpl: async () => ({ ok: false, status: 402, json: async () => ({ error: 'No credits remaining' }) }) });
  await assert.rejects(limited.documents.list(), FormaliniUsageLimitError);
});
