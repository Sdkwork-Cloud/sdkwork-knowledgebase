import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { CompleteKnowledgeUploadSessionRequest, CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest, CreateKnowledgeSpaceContextBindingRequest, CreateKnowledgeSpaceRequest, CreateKnowledgeUploadSessionRequest, IngestionJob, KnowledgeAgentBinding, KnowledgeAgentBindingRequest, KnowledgeAgentChatRequest, KnowledgeAgentChatResponse, KnowledgeAgentProfile, KnowledgeAgentProfileRequest, KnowledgeBrowserPage, KnowledgeBrowserView, KnowledgeContextPack, KnowledgeContextPackRequest, KnowledgeDocument, KnowledgeDocumentList, KnowledgeDocumentVersion, KnowledgeDocumentVersionList, KnowledgeDriveImportRequest, KnowledgeDriveImportResult, KnowledgeIngestRequest, KnowledgeRetrievalRequest, KnowledgeRetrievalResult, KnowledgeSpace, KnowledgeSpaceContextBinding, KnowledgeSpaceContextBindingList, KnowledgeUploadSession, KnowledgeWikiFileEntry, KnowledgeWikiPageRevisionList, UpdateKnowledgeSpaceContextBindingRequest, WikiContextPackRequest, WikiFileAnswerRequest, WikiIndexDocument, WikiLogDocument, WikiPageSummary, WikiPageSummaryList, WikiQueryRequest, WikiQueryResult, WikiSchemaDocument } from '../types';


export class KnowledgeUploadSessionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a drive-delegated knowledge upload session */
  async create(body: CreateKnowledgeUploadSessionRequest): Promise<KnowledgeUploadSession> {
    return this.client.post<KnowledgeUploadSession>(appApiPath(`/knowledge/upload_sessions`), body, undefined, undefined, 'application/json');
  }

/** Complete a knowledge upload session and start ingestion */
  async complete(sessionId: string, body: CompleteKnowledgeUploadSessionRequest): Promise<IngestionJob> {
    return this.client.post<IngestionJob>(appApiPath(`/knowledge/upload_sessions/${serializePathParameter(sessionId, { name: 'sessionId', style: 'simple', explode: false })}/complete`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeContextBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve a knowledge space context binding */
  async retrieve(bindingId: string): Promise<KnowledgeSpaceContextBinding> {
    return this.client.get<KnowledgeSpaceContextBinding>(appApiPath(`/knowledge/context_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`));
  }

/** Update a knowledge space context binding */
  async update(bindingId: string, body: UpdateKnowledgeSpaceContextBindingRequest): Promise<KnowledgeSpaceContextBinding> {
    return this.client.patch<KnowledgeSpaceContextBinding>(appApiPath(`/knowledge/context_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete a knowledge space context binding */
  async delete(bindingId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/context_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeAgentProfilesChatApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Chat with a knowledge-backed agent profile */
  async create(profileId: string, body: KnowledgeAgentChatRequest): Promise<KnowledgeAgentChatResponse> {
    return this.client.post<KnowledgeAgentChatResponse>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/chat`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeAgentProfilesRetrievalPreviewApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Preview retrieval for an agent profile */
  async create(profileId: string, body: KnowledgeRetrievalRequest): Promise<KnowledgeRetrievalResult> {
    return this.client.post<KnowledgeRetrievalResult>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/retrieval_preview`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeAgentProfilesBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List agent profile bindings */
  async list(profileId: string): Promise<unknown> {
    return this.client.get<unknown>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/bindings`));
  }

/** Create an agent profile binding */
  async create(profileId: string, body: KnowledgeAgentBindingRequest): Promise<KnowledgeAgentBinding> {
    return this.client.post<KnowledgeAgentBinding>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/bindings`), body, undefined, undefined, 'application/json');
  }

/** Update an agent profile binding */
  async update(profileId: string, bindingId: string, body: KnowledgeAgentBindingRequest): Promise<KnowledgeAgentBinding> {
    return this.client.patch<KnowledgeAgentBinding>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete an agent profile binding */
  async delete(profileId: string, bindingId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeAgentProfilesApi {
  private client: HttpClient;
  public readonly bindings: KnowledgeAgentProfilesBindingsApi;
  public readonly retrievalPreview: KnowledgeAgentProfilesRetrievalPreviewApi;
  public readonly chat: KnowledgeAgentProfilesChatApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.bindings = new KnowledgeAgentProfilesBindingsApi(client);
    this.retrievalPreview = new KnowledgeAgentProfilesRetrievalPreviewApi(client);
    this.chat = new KnowledgeAgentProfilesChatApi(client);
  }


/** Create a knowledge agent profile */
  async create(body: KnowledgeAgentProfileRequest): Promise<KnowledgeAgentProfile> {
    return this.client.post<KnowledgeAgentProfile>(appApiPath(`/knowledge/agent_profiles`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge agent profile */
  async retrieve(profileId: string): Promise<KnowledgeAgentProfile> {
    return this.client.get<KnowledgeAgentProfile>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`));
  }

/** Update a knowledge agent profile */
  async update(profileId: string, body: KnowledgeAgentProfileRequest): Promise<KnowledgeAgentProfile> {
    return this.client.patch<KnowledgeAgentProfile>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete a knowledge agent profile */
  async delete(profileId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeContextPacksApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a knowledge context pack */
  async create(body: KnowledgeContextPackRequest): Promise<KnowledgeContextPack> {
    return this.client.post<KnowledgeContextPack>(appApiPath(`/knowledge/context_packs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeRetrievalsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a knowledge retrieval */
  async create(body: KnowledgeRetrievalRequest): Promise<KnowledgeRetrievalResult> {
    return this.client.post<KnowledgeRetrievalResult>(appApiPath(`/knowledge/retrievals`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge retrieval result */
  async retrieve(retrievalId: string): Promise<KnowledgeRetrievalResult> {
    return this.client.get<KnowledgeRetrievalResult>(appApiPath(`/knowledge/retrievals/${serializePathParameter(retrievalId, { name: 'retrievalId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeWikiContextPacksApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a wiki context pack */
  async create(body: WikiContextPackRequest): Promise<KnowledgeWikiFileEntry> {
    return this.client.post<KnowledgeWikiFileEntry>(appApiPath(`/knowledge/wiki_context_packs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiQueriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a wiki query */
  async create(body: WikiQueryRequest): Promise<WikiQueryResult> {
    return this.client.post<WikiQueryResult>(appApiPath(`/knowledge/wiki_queries`), body, undefined, undefined, 'application/json');
  }

/** File an answer for a wiki query */
  async fileAnswer(queryId: number, body: WikiFileAnswerRequest): Promise<WikiQueryResult> {
    return this.client.post<WikiQueryResult>(appApiPath(`/knowledge/wiki_queries/${serializePathParameter(queryId, { name: 'queryId', style: 'simple', explode: false })}/file_answer`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiSchemaApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve the wiki schema */
  async retrieve(): Promise<WikiSchemaDocument> {
    return this.client.get<WikiSchemaDocument>(appApiPath(`/knowledge/wiki_schema`));
  }
}

export class KnowledgeWikiLogApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve the wiki log */
  async retrieve(): Promise<WikiLogDocument> {
    return this.client.get<WikiLogDocument>(appApiPath(`/knowledge/wiki_log`));
  }
}

export class KnowledgeWikiIndexApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve the wiki index */
  async retrieve(): Promise<WikiIndexDocument> {
    return this.client.get<WikiIndexDocument>(appApiPath(`/knowledge/wiki_index`));
  }
}

export class KnowledgeWikiPagesRevisionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List wiki page revisions */
  async list(pageId: number): Promise<KnowledgeWikiPageRevisionList> {
    return this.client.get<KnowledgeWikiPageRevisionList>(appApiPath(`/knowledge/wiki_pages/${serializePathParameter(pageId, { name: 'pageId', style: 'simple', explode: false })}/revisions`));
  }
}

export class KnowledgeWikiPagesApi {
  private client: HttpClient;
  public readonly revisions: KnowledgeWikiPagesRevisionsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.revisions = new KnowledgeWikiPagesRevisionsApi(client);
  }


/** List wiki pages */
  async list(): Promise<WikiPageSummaryList> {
    return this.client.get<WikiPageSummaryList>(appApiPath(`/knowledge/wiki_pages`));
  }

/** Retrieve a wiki page */
  async retrieve(pageId: number): Promise<WikiPageSummary> {
    return this.client.get<WikiPageSummary>(appApiPath(`/knowledge/wiki_pages/${serializePathParameter(pageId, { name: 'pageId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeWikiApi {
  private client: HttpClient;
  public readonly pages: KnowledgeWikiPagesApi;
  public readonly index: KnowledgeWikiIndexApi;
  public readonly log: KnowledgeWikiLogApi;
  public readonly schema: KnowledgeWikiSchemaApi;
  public readonly queries: KnowledgeWikiQueriesApi;
  public readonly contextPacks: KnowledgeWikiContextPacksApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.pages = new KnowledgeWikiPagesApi(client);
    this.index = new KnowledgeWikiIndexApi(client);
    this.log = new KnowledgeWikiLogApi(client);
    this.schema = new KnowledgeWikiSchemaApi(client);
    this.queries = new KnowledgeWikiQueriesApi(client);
    this.contextPacks = new KnowledgeWikiContextPacksApi(client);
  }

}

export class KnowledgeDocumentsVersionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List document versions */
  async list(documentId: number): Promise<KnowledgeDocumentVersionList> {
    return this.client.get<KnowledgeDocumentVersionList>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}/versions`));
  }

/** Create a document version */
  async create(documentId: number, body: CreateKnowledgeDocumentVersionRequest): Promise<KnowledgeDocumentVersion> {
    return this.client.post<KnowledgeDocumentVersion>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}/versions`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeDocumentsApi {
  private client: HttpClient;
  public readonly versions: KnowledgeDocumentsVersionsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.versions = new KnowledgeDocumentsVersionsApi(client);
  }


/** List knowledge documents */
  async list(): Promise<KnowledgeDocumentList> {
    return this.client.get<KnowledgeDocumentList>(appApiPath(`/knowledge/documents`));
  }

/** Create a knowledge document */
  async create(body: CreateKnowledgeDocumentRequest): Promise<KnowledgeDocument> {
    return this.client.post<KnowledgeDocument>(appApiPath(`/knowledge/documents`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge document */
  async retrieve(documentId: number): Promise<KnowledgeDocument> {
    return this.client.get<KnowledgeDocument>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}`));
  }

/** Update a knowledge document */
  async update(documentId: number, body: CreateKnowledgeDocumentRequest): Promise<KnowledgeDocument> {
    return this.client.patch<KnowledgeDocument>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete a knowledge document */
  async delete(documentId: number): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeIngestsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an ingestion job */
  async create(body: KnowledgeIngestRequest): Promise<IngestionJob> {
    return this.client.post<IngestionJob>(appApiPath(`/knowledge/ingests`), body, undefined, undefined, 'application/json');
  }

/** Retrieve an ingestion job */
  async retrieve(ingestId: number): Promise<IngestionJob> {
    return this.client.get<IngestionJob>(appApiPath(`/knowledge/ingests/${serializePathParameter(ingestId, { name: 'ingestId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeDriveImportsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Import a drive object into knowledgebase */
  async create(body: KnowledgeDriveImportRequest): Promise<KnowledgeDriveImportResult> {
    return this.client.post<KnowledgeDriveImportResult>(appApiPath(`/knowledge/drive_imports`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeSpacesContextBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List knowledge space context bindings */
  async list(spaceId: string): Promise<KnowledgeSpaceContextBindingList> {
    return this.client.get<KnowledgeSpaceContextBindingList>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/context_bindings`));
  }

/** Create a knowledge space context binding */
  async create(spaceId: string, body: CreateKnowledgeSpaceContextBindingRequest): Promise<KnowledgeSpaceContextBinding> {
    return this.client.post<KnowledgeSpaceContextBinding>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/context_bindings`), body, undefined, undefined, 'application/json');
  }
}

export interface KnowledgeSpacesBrowserListParams {
  view: KnowledgeBrowserView;
  parentId?: string | null;
  cursor?: string | null;
  pageSize?: number;
}

export class KnowledgeSpacesBrowserApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List browser view of a knowledge space */
  async list(spaceId: number, params: KnowledgeSpacesBrowserListParams): Promise<KnowledgeBrowserPage> {
    const query = buildQueryString([
      { name: 'view', value: params.view, style: 'form', explode: true, allowReserved: false },
      { name: 'parentId', value: params.parentId, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'pageSize', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<KnowledgeBrowserPage>(appendQueryString(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/browser`), query));
  }
}

export class KnowledgeSpacesApi {
  private client: HttpClient;
  public readonly browser: KnowledgeSpacesBrowserApi;
  public readonly contextBindings: KnowledgeSpacesContextBindingsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.browser = new KnowledgeSpacesBrowserApi(client);
    this.contextBindings = new KnowledgeSpacesContextBindingsApi(client);
  }


/** Create a knowledge space */
  async create(body: CreateKnowledgeSpaceRequest): Promise<KnowledgeSpace> {
    return this.client.post<KnowledgeSpace>(appApiPath(`/knowledge/spaces`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge space */
  async retrieve(spaceId: number): Promise<KnowledgeSpace> {
    return this.client.get<KnowledgeSpace>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeApi {
  private client: HttpClient;
  public readonly spaces: KnowledgeSpacesApi;
  public readonly driveImports: KnowledgeDriveImportsApi;
  public readonly ingests: KnowledgeIngestsApi;
  public readonly documents: KnowledgeDocumentsApi;
  public readonly wiki: KnowledgeWikiApi;
  public readonly retrievals: KnowledgeRetrievalsApi;
  public readonly contextPacks: KnowledgeContextPacksApi;
  public readonly agentProfiles: KnowledgeAgentProfilesApi;
  public readonly contextBindings: KnowledgeContextBindingsApi;
  public readonly uploadSessions: KnowledgeUploadSessionsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.spaces = new KnowledgeSpacesApi(client);
    this.driveImports = new KnowledgeDriveImportsApi(client);
    this.ingests = new KnowledgeIngestsApi(client);
    this.documents = new KnowledgeDocumentsApi(client);
    this.wiki = new KnowledgeWikiApi(client);
    this.retrievals = new KnowledgeRetrievalsApi(client);
    this.contextPacks = new KnowledgeContextPacksApi(client);
    this.agentProfiles = new KnowledgeAgentProfilesApi(client);
    this.contextBindings = new KnowledgeContextBindingsApi(client);
    this.uploadSessions = new KnowledgeUploadSessionsApi(client);
  }

}

export function createKnowledgeApi(client: HttpClient): KnowledgeApi {
  return new KnowledgeApi(client);
}

function appendQueryString(path: string, rawQueryString: string): string {
  const query = rawQueryString.replace(/^\?+/, '');
  if (!query) {
    return path;
  }
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

interface PathParameterSpec {
  name: string;
  style: string;
  explode: boolean;
}

function serializePathParameter(value: unknown, spec: PathParameterSpec): string {
  if (value === undefined || value === null) {
    return '';
  }

  const style = spec.style || 'simple';
  if (Array.isArray(value)) {
    return serializePathArray(spec.name, value, style, spec.explode);
  }
  if (typeof value === 'object') {
    return serializePathObject(spec.name, value as Record<string, unknown>, style, spec.explode);
  }
  return pathPrefix(spec.name, style, false) + encodePathValue(serializePathPrimitive(value));
}

function serializePathArray(name: string, values: unknown[], style: string, explode: boolean): string {
  const serialized = values
    .filter((item) => item !== undefined && item !== null)
    .map((item) => encodePathValue(serializePathPrimitive(item)));
  if (serialized.length === 0) {
    return pathPrefix(name, style, false);
  }
  if (style === 'matrix') {
    return explode
      ? serialized.map((item) => `;${name}=${item}`).join('')
      : `;${name}=${serialized.join(',')}`;
  }
  return pathPrefix(name, style, false) + serialized.join(explode ? '.' : ',');
}

function serializePathObject(name: string, value: Record<string, unknown>, style: string, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return pathPrefix(name, style, true);
  }
  if (style === 'matrix') {
    return explode
      ? entries.map(([key, entryValue]) => `;${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join('')
      : `;${name}=${entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',')}`;
  }
  const serialized = explode
    ? entries.map(([key, entryValue]) => `${encodePathValue(key)}=${encodePathValue(serializePathPrimitive(entryValue))}`).join(style === 'label' ? '.' : ',')
    : entries.flatMap(([key, entryValue]) => [encodePathValue(key), encodePathValue(serializePathPrimitive(entryValue))]).join(',');
  return pathPrefix(name, style, true) + serialized;
}

function pathPrefix(name: string, style: string, _objectValue: boolean): string {
  if (style === 'label') return '.';
  if (style === 'matrix') return `;${name}`;
  return '';
}

function encodePathValue(value: string): string {
  return encodeURIComponent(value);
}

function serializePathPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}
interface QueryParameterSpec {
  name: string;
  value: unknown;
  style: string;
  explode: boolean;
  allowReserved: boolean;
  contentType?: string;
}

function buildQueryString(parameters: QueryParameterSpec[]): string {
  const pairs: string[] = [];
  for (const parameter of parameters) {
    appendSerializedParameter(pairs, parameter);
  }
  return pairs.join('&');
}

function appendSerializedParameter(pairs: string[], parameter: QueryParameterSpec): void {
  if (parameter.value === undefined || parameter.value === null) {
    return;
  }

  if (parameter.contentType) {
    pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(JSON.stringify(parameter.value), parameter.allowReserved)}`);
    return;
  }

  const style = parameter.style || 'form';
  if (style === 'deepObject') {
    appendDeepObjectParameter(pairs, parameter.name, parameter.value, parameter.allowReserved);
    return;
  }

  if (Array.isArray(parameter.value)) {
    appendArrayParameter(pairs, parameter.name, parameter.value, style, parameter.explode, parameter.allowReserved);
    return;
  }

  if (typeof parameter.value === 'object') {
    appendObjectParameter(pairs, parameter.name, parameter.value as Record<string, unknown>, style, parameter.explode, parameter.allowReserved);
    return;
  }

  pairs.push(`${encodeQueryComponent(parameter.name)}=${encodeQueryValue(serializePrimitive(parameter.value), parameter.allowReserved)}`);
}

function appendArrayParameter(
  pairs: string[],
  name: string,
  value: unknown[],
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const values = value
    .filter((item) => item !== undefined && item !== null)
    .map((item) => serializePrimitive(item));
  if (values.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const item of values) {
      pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(item, allowReserved)}`);
    }
    return;
  }

  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(values.join(','), allowReserved)}`);
}

function appendObjectParameter(
  pairs: string[],
  name: string,
  value: Record<string, unknown>,
  style: string,
  explode: boolean,
  allowReserved: boolean,
): void {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (entries.length === 0) {
    return;
  }

  if (style === 'form' && explode) {
    for (const [key, entryValue] of entries) {
      pairs.push(`${encodeQueryComponent(key)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
    }
    return;
  }

  const serialized = entries.flatMap(([key, entryValue]) => [key, serializePrimitive(entryValue)]).join(',');
  pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serialized, allowReserved)}`);
}

function appendDeepObjectParameter(
  pairs: string[],
  name: string,
  value: unknown,
  allowReserved: boolean,
): void {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    pairs.push(`${encodeQueryComponent(name)}=${encodeQueryValue(serializePrimitive(value), allowReserved)}`);
    return;
  }

  for (const [key, entryValue] of Object.entries(value as Record<string, unknown>)) {
    if (entryValue === undefined || entryValue === null) {
      continue;
    }
    pairs.push(`${encodeQueryComponent(`${name}[${key}]`)}=${encodeQueryValue(serializePrimitive(entryValue), allowReserved)}`);
  }
}

function serializePrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
}

function encodeQueryComponent(value: string): string {
  return encodeURIComponent(value);
}

function encodeQueryValue(value: string, allowReserved: boolean): string {
  const encoded = encodeURIComponent(value);
  if (!allowReserved) {
    return encoded;
  }
  return encoded.replace(/%3A/gi, ':')
    .replace(/%2F/gi, '/')
    .replace(/%3F/gi, '?')
    .replace(/%23/gi, '#')
    .replace(/%5B/gi, '[')
    .replace(/%5D/gi, ']')
    .replace(/%40/gi, '@')
    .replace(/%21/gi, '!')
    .replace(/%24/gi, '$')
    .replace(/%26/gi, '&')
    .replace(/%27/gi, "'")
    .replace(/%28/gi, '(')
    .replace(/%29/gi, ')')
    .replace(/%2A/gi, '*')
    .replace(/%2B/gi, '+')
    .replace(/%2C/gi, ',')
    .replace(/%3B/gi, ';')
    .replace(/%3D/gi, '=');
}
