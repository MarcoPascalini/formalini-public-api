export class FormaliniApiError extends Error {
  constructor(status, message, retryAfter = null) {
    super(message);
    this.name = 'FormaliniApiError';
    this.status = status;
    this.retryAfter = retryAfter;
  }
}

export class FormaliniResponseError extends Error {
  constructor() {
    super('Unexpected Formalini response');
    this.name = 'FormaliniResponseError';
  }
}

const statuses = new Set(['queued', 'processing', 'review', 'approved', 'exported', 'failed']);
const events = new Set(['document.processed', 'document.needs_review', 'document.approved', 'document.exported']);
const imageTypes = new Set(['image/jpeg', 'image/png', 'image/webp', 'image/heic']);

export class FormaliniClient {
  constructor({ apiKey, baseUrl = 'https://api.formalini.com/api/public/v1', timeoutMs = 60_000, fetchImpl = fetch }) {
    if (typeof apiKey !== 'string' || !apiKey.trim()) throw new TypeError('apiKey must be non-empty');
    if (!baseUrl.startsWith('https://')) throw new TypeError('baseUrl must use HTTPS');
    if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) throw new TypeError('timeoutMs must be positive');
    this.apiKey = apiKey;
    this.baseUrl = baseUrl.replace(/\/+$/, '');
    this.timeoutMs = timeoutMs;
    this.fetchImpl = fetchImpl;
    this.documents = {
      list: options => this.listDocuments(options),
      create: options => this.createDocument(options),
      upload: options => this.uploadDocument(options),
      get: id => this.getDocument(id),
      update: (id, options) => this.updateDocument(id, options),
    };
    this.workflows = { dispatch: options => this.dispatch(options) };
  }

  async request(method, path, { body, signal } = {}) {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeoutMs);
    const abort = () => controller.abort();
    signal?.addEventListener('abort', abort, { once: true });
    if (signal?.aborted) controller.abort();
    try {
      const headers = { Authorization: `Bearer ${this.apiKey}` };
      if (body && !(body instanceof FormData)) headers['Content-Type'] = 'application/json';
      const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
        method, headers, body: body instanceof FormData ? body : body ? JSON.stringify(body) : undefined,
        signal: controller.signal,
      });
      let data;
      try { data = await response.json(); } catch { throw new FormaliniResponseError(); }
      if (!response.ok) {
        const message = typeof data?.error === 'string' ? data.error : `Formalini returned HTTP ${response.status}`;
        throw new FormaliniApiError(response.status, message, response.headers?.get('Retry-After') ?? null);
      }
      if (!data || typeof data !== 'object' || Array.isArray(data)) throw new FormaliniResponseError();
      return data;
    } finally {
      clearTimeout(timer);
      signal?.removeEventListener('abort', abort);
    }
  }

  async listDocuments({ limit = 25, status, signal } = {}) {
    if (!Number.isInteger(limit) || limit < 1 || limit > 100) throw new RangeError('limit must be an integer from 1 to 100');
    if (status !== undefined && !statuses.has(status)) throw new RangeError('unsupported status');
    const query = new URLSearchParams({ limit: String(limit) });
    if (status) query.set('status', status);
    const data = await this.request('GET', `/documents?${query}`, { signal });
    if (!Array.isArray(data.documents)) throw new FormaliniResponseError();
    return data.documents;
  }

  createDocument({ name, fields, ...optional }) {
    if (typeof name !== 'string' || !name.trim()) throw new TypeError('name is required');
    if (fields !== undefined && (!Array.isArray(fields) || fields.length > 400)) throw new RangeError('fields may contain at most 400 entries');
    if (Object.keys(optional).some(key => !['category', 'vendor', 'amount', 'document_date'].includes(key))) throw new TypeError('unsupported create option');
    return this.request('POST', '/documents', { body: { name, ...(fields !== undefined ? { fields } : {}), ...optional } });
  }

  uploadDocument({ files, name, templateId, asynchronous = false, signal }) {
    if (!Array.isArray(files) || files.length < 1 || files.length > 25) throw new RangeError('upload requires 1 to 25 page images');
    const form = new FormData();
    for (const file of files) {
      if (!(file instanceof Blob) || !imageTypes.has(file.type) || file.size > 100 * 1024 * 1024) {
        throw new TypeError('each file must be a JPEG, PNG, WebP or HEIC image of at most 100 MB');
      }
      form.append('file', file, file.name ?? 'page');
    }
    if (name !== undefined) form.append('name', name);
    if (templateId !== undefined) form.append('template_id', templateId);
    if (asynchronous) form.append('async', 'true');
    return this.request('POST', '/documents/upload', { body: form, signal });
  }

  getDocument(id) {
    if (!id) throw new TypeError('document id is required');
    return this.request('GET', `/documents/${encodeURIComponent(id)}`);
  }

  updateDocument(id, { status, fields } = {}) {
    if (!id || (status === undefined && fields === undefined)) throw new TypeError('document id and a change are required');
    if (status !== undefined && !['review', 'approved', 'exported'].includes(status)) throw new RangeError('unsupported update status');
    return this.request('PATCH', `/documents/${encodeURIComponent(id)}`, { body: { ...(status !== undefined ? { status } : {}), ...(fields !== undefined ? { fields } : {}) } });
  }

  dispatch({ event, documentId, payload }) {
    if (!events.has(event) || !documentId) throw new TypeError('valid event and documentId are required');
    return this.request('POST', '/workflows/dispatch', { body: { event, document_id: documentId, ...(payload !== undefined ? { payload } : {}) } });
  }
}
