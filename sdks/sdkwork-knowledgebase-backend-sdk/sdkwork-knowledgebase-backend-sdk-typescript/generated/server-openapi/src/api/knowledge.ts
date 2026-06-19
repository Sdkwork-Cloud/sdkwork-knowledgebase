import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { CreateKnowledgeSourceRequest, IngestionJob, KnowledgeIndex, KnowledgeIndexRequest, KnowledgeProviderHealth, KnowledgeRetrievalProfile, KnowledgeRetrievalProfileRequest, KnowledgeRetrievalTrace, KnowledgeRetrievalTraceList, KnowledgeSource, KnowledgeSourceList, KnowledgeWikiFileEntry, KnowledgeWikiFileEntryList, KnowledgeWikiSchemaProfileRequest, WikiCandidateResult, WikiCandidateResultList, WikiCandidateReviewRequest, WikiCompileJobRequest, WikiExportRequest, WikiIndexDocument, WikiIndexRebuildRequest, WikiLogEntry, WikiPagePublishRequest, WikiPageSummary, WikiQualityRun, WikiQualityRunRequest } from '../types';


export class KnowledgeProviderHealthApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve provider health status */
  async retrieve(): Promise<KnowledgeProviderHealth> {
    return this.client.get<KnowledgeProviderHealth>(backendApiPath(`/knowledge/provider_health`));
  }
}

export class KnowledgeRetrievalTracesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List retrieval traces */
  async list(): Promise<KnowledgeRetrievalTraceList> {
    return this.client.get<KnowledgeRetrievalTraceList>(backendApiPath(`/knowledge/retrieval_traces`));
  }

/** Retrieve a retrieval trace */
  async retrieve(traceId: string): Promise<KnowledgeRetrievalTrace> {
    return this.client.get<KnowledgeRetrievalTrace>(backendApiPath(`/knowledge/retrieval_traces/${serializePathParameter(traceId, { name: 'traceId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeRetrievalProfilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a retrieval profile */
  async create(body: KnowledgeRetrievalProfileRequest): Promise<KnowledgeRetrievalProfile> {
    return this.client.post<KnowledgeRetrievalProfile>(backendApiPath(`/knowledge/retrieval_profiles`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a retrieval profile */
  async retrieve(profileId: string): Promise<KnowledgeRetrievalProfile> {
    return this.client.get<KnowledgeRetrievalProfile>(backendApiPath(`/knowledge/retrieval_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`));
  }

/** Update a retrieval profile */
  async update(profileId: string, body: KnowledgeRetrievalProfileRequest): Promise<KnowledgeRetrievalProfile> {
    return this.client.patch<KnowledgeRetrievalProfile>(backendApiPath(`/knowledge/retrieval_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeIndexesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a knowledge index */
  async create(body: KnowledgeIndexRequest): Promise<KnowledgeIndex> {
    return this.client.post<KnowledgeIndex>(backendApiPath(`/knowledge/indexes`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge index */
  async retrieve(indexId: string): Promise<KnowledgeIndex> {
    return this.client.get<KnowledgeIndex>(backendApiPath(`/knowledge/indexes/${serializePathParameter(indexId, { name: 'indexId', style: 'simple', explode: false })}`));
  }

/** Rebuild a knowledge index */
  async rebuild(indexId: string, body: WikiIndexRebuildRequest): Promise<WikiIndexDocument> {
    return this.client.post<WikiIndexDocument>(backendApiPath(`/knowledge/indexes/${serializePathParameter(indexId, { name: 'indexId', style: 'simple', explode: false })}/rebuild`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiEvalRunsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a wiki eval run */
  async create(body: WikiQualityRunRequest): Promise<WikiQualityRun> {
    return this.client.post<WikiQualityRun>(backendApiPath(`/knowledge/wiki_eval_runs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiLintRunsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a wiki lint run */
  async create(body: WikiQualityRunRequest): Promise<WikiQualityRun> {
    return this.client.post<WikiQualityRun>(backendApiPath(`/knowledge/wiki_lint_runs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiFileEntriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List wiki file entries */
  async list(): Promise<KnowledgeWikiFileEntryList> {
    return this.client.get<KnowledgeWikiFileEntryList>(backendApiPath(`/knowledge/wiki_file_entries`));
  }
}

export class KnowledgeWikiExportsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a wiki export */
  async create(body: WikiExportRequest): Promise<KnowledgeWikiFileEntry> {
    return this.client.post<KnowledgeWikiFileEntry>(backendApiPath(`/knowledge/wiki_exports`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a wiki export */
  async retrieve(exportId: number): Promise<KnowledgeWikiFileEntry> {
    return this.client.get<KnowledgeWikiFileEntry>(backendApiPath(`/knowledge/wiki_exports/${serializePathParameter(exportId, { name: 'exportId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeWikiLogEntriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a wiki log entry */
  async create(body: WikiLogEntry): Promise<WikiLogEntry> {
    return this.client.post<WikiLogEntry>(backendApiPath(`/knowledge/wiki_log_entries`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiLogApi {
  private client: HttpClient;
  public readonly entries: KnowledgeWikiLogEntriesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.entries = new KnowledgeWikiLogEntriesApi(client);
  }

}

export class KnowledgeWikiIndexApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Rebuild the wiki index */
  async rebuild(body: WikiIndexRebuildRequest): Promise<WikiIndexDocument> {
    return this.client.post<WikiIndexDocument>(backendApiPath(`/knowledge/wiki_index/rebuild`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiSchemaProfilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a wiki schema profile */
  async create(body: KnowledgeWikiSchemaProfileRequest): Promise<KnowledgeWikiFileEntry> {
    return this.client.post<KnowledgeWikiFileEntry>(backendApiPath(`/knowledge/wiki_schema_profiles`), body, undefined, undefined, 'application/json');
  }

/** Update a wiki schema profile */
  async update(profileId: number, body: KnowledgeWikiSchemaProfileRequest): Promise<KnowledgeWikiFileEntry> {
    return this.client.patch<KnowledgeWikiFileEntry>(backendApiPath(`/knowledge/wiki_schema_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiSchemaApi {
  private client: HttpClient;
  public readonly profiles: KnowledgeWikiSchemaProfilesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.profiles = new KnowledgeWikiSchemaProfilesApi(client);
  }

}

export class KnowledgeWikiPagesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Publish a wiki page */
  async publish(pageId: number, body: WikiPagePublishRequest): Promise<WikiPageSummary> {
    return this.client.post<WikiPageSummary>(backendApiPath(`/knowledge/wiki_pages/${serializePathParameter(pageId, { name: 'pageId', style: 'simple', explode: false })}/publish`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiCandidatesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List wiki candidates */
  async list(): Promise<WikiCandidateResultList> {
    return this.client.get<WikiCandidateResultList>(backendApiPath(`/knowledge/wiki_candidates`));
  }

/** Approve a wiki candidate */
  async approve(candidateId: number, body: WikiCandidateReviewRequest): Promise<WikiCandidateResult> {
    return this.client.post<WikiCandidateResult>(backendApiPath(`/knowledge/wiki_candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/approve`), body, undefined, undefined, 'application/json');
  }

/** Reject a wiki candidate */
  async reject(candidateId: number, body: WikiCandidateReviewRequest): Promise<WikiCandidateResult> {
    return this.client.post<WikiCandidateResult>(backendApiPath(`/knowledge/wiki_candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/reject`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiCompileJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a wiki compile job */
  async create(body: WikiCompileJobRequest): Promise<IngestionJob> {
    return this.client.post<IngestionJob>(backendApiPath(`/knowledge/wiki_compile_jobs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiApi {
  private client: HttpClient;
  public readonly compileJobs: KnowledgeWikiCompileJobsApi;
  public readonly candidates: KnowledgeWikiCandidatesApi;
  public readonly pages: KnowledgeWikiPagesApi;
  public readonly schema: KnowledgeWikiSchemaApi;
  public readonly index: KnowledgeWikiIndexApi;
  public readonly log: KnowledgeWikiLogApi;
  public readonly exports: KnowledgeWikiExportsApi;
  public readonly fileEntries: KnowledgeWikiFileEntriesApi;
  public readonly lintRuns: KnowledgeWikiLintRunsApi;
  public readonly evalRuns: KnowledgeWikiEvalRunsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.compileJobs = new KnowledgeWikiCompileJobsApi(client);
    this.candidates = new KnowledgeWikiCandidatesApi(client);
    this.pages = new KnowledgeWikiPagesApi(client);
    this.schema = new KnowledgeWikiSchemaApi(client);
    this.index = new KnowledgeWikiIndexApi(client);
    this.log = new KnowledgeWikiLogApi(client);
    this.exports = new KnowledgeWikiExportsApi(client);
    this.fileEntries = new KnowledgeWikiFileEntriesApi(client);
    this.lintRuns = new KnowledgeWikiLintRunsApi(client);
    this.evalRuns = new KnowledgeWikiEvalRunsApi(client);
  }

}

export class KnowledgeSourcesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List knowledge sources */
  async list(): Promise<KnowledgeSourceList> {
    return this.client.get<KnowledgeSourceList>(backendApiPath(`/knowledge/sources`));
  }

/** Create a knowledge source */
  async create(body: CreateKnowledgeSourceRequest): Promise<KnowledgeSource> {
    return this.client.post<KnowledgeSource>(backendApiPath(`/knowledge/sources`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeApi {
  private client: HttpClient;
  public readonly sources: KnowledgeSourcesApi;
  public readonly wiki: KnowledgeWikiApi;
  public readonly indexes: KnowledgeIndexesApi;
  public readonly retrievalProfiles: KnowledgeRetrievalProfilesApi;
  public readonly retrievalTraces: KnowledgeRetrievalTracesApi;
  public readonly providerHealth: KnowledgeProviderHealthApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.sources = new KnowledgeSourcesApi(client);
    this.wiki = new KnowledgeWikiApi(client);
    this.indexes = new KnowledgeIndexesApi(client);
    this.retrievalProfiles = new KnowledgeRetrievalProfilesApi(client);
    this.retrievalTraces = new KnowledgeRetrievalTracesApi(client);
    this.providerHealth = new KnowledgeProviderHealthApi(client);
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
