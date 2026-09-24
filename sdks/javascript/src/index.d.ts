export type DocumentStatus = 'queued' | 'processing' | 'review' | 'approved' | 'exported' | 'failed';
export type UpdateStatus = 'review' | 'approved' | 'exported';
export type WorkflowEvent = 'document.processed' | 'document.needs_review' | 'document.approved' | 'document.exported';
export type JsonObject = Record<string, unknown>;

export interface DocumentRecord extends JsonObject {
  id?: string;
  name?: string;
  status?: DocumentStatus;
  fields?: JsonObject[];
}

export interface CreateDocument {
  name: string;
  fields?: JsonObject[];
  category?: string;
  vendor?: string;
  amount?: number;
  document_date?: string;
}

export interface ClientOptions {
  apiKey: string;
  baseUrl?: string;
  timeoutMs?: number;
  fetchImpl?: typeof fetch;
}

export class FormaliniApiError extends Error {
  constructor(status: number, message: string, retryAfter?: string | null);
  status: number;
  retryAfter: string | null;
}
export class FormaliniUsageLimitError extends FormaliniApiError {
  constructor(message: string);
}
export class FormaliniResponseError extends Error {
  constructor();
}

export class FormaliniClient {
  constructor(options: ClientOptions);
  readonly documents: {
    list(options?: { limit?: number; status?: DocumentStatus; signal?: AbortSignal }): Promise<DocumentRecord[]>;
    create(options: CreateDocument): Promise<JsonObject>;
    upload(options: { files: Blob[]; name?: string; templateId?: string; asynchronous?: boolean; signal?: AbortSignal }): Promise<JsonObject>;
    get(id: string): Promise<JsonObject>;
    update(id: string, options: { status?: UpdateStatus; fields?: JsonObject[] }): Promise<JsonObject>;
  };
  readonly workflows: {
    dispatch(options: { event: WorkflowEvent; documentId: string; payload?: JsonObject }): Promise<JsonObject>;
  };
}
