import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { CreateKnowledgeSourceRequest, IngestionJob, KnowledgeSource, KnowledgeSourceList, KnowledgeWikiFileEntry, KnowledgeWikiFileEntryList, KnowledgeWikiSchemaProfileRequest, WikiCandidateResult, WikiCandidateResultList, WikiCandidateReviewRequest, WikiCompileJobRequest, WikiExportRequest, WikiIndexDocument, WikiIndexRebuildRequest, WikiLogEntry, WikiPagePublishRequest, WikiPageSummary, WikiQualityRun, WikiQualityRunRequest } from '../types';


export class KnowledgeWikiEvalRunsApi {
  private client: HttpClient;
  
  constructor(client: HttpClient) { 
    this.client = client; 
  }


async create(body: WikiQualityRunRequest): Promise<WikiQualityRun> {
    return this.client.post<WikiQualityRun>(backendApiPath(`/knowledge/wiki_eval_runs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiLintRunsApi {
  private client: HttpClient;
  
  constructor(client: HttpClient) { 
    this.client = client; 
  }


async create(body: WikiQualityRunRequest): Promise<WikiQualityRun> {
    return this.client.post<WikiQualityRun>(backendApiPath(`/knowledge/wiki_lint_runs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiFileEntriesApi {
  private client: HttpClient;
  
  constructor(client: HttpClient) { 
    this.client = client; 
  }


async list(): Promise<KnowledgeWikiFileEntryList> {
    return this.client.get<KnowledgeWikiFileEntryList>(backendApiPath(`/knowledge/wiki_file_entries`));
  }
}

export class KnowledgeWikiExportsApi {
  private client: HttpClient;
  
  constructor(client: HttpClient) { 
    this.client = client; 
  }


async create(body: WikiExportRequest): Promise<KnowledgeWikiFileEntry> {
    return this.client.post<KnowledgeWikiFileEntry>(backendApiPath(`/knowledge/wiki_exports`), body, undefined, undefined, 'application/json');
  }

async retrieve(exportId: number): Promise<KnowledgeWikiFileEntry> {
    return this.client.get<KnowledgeWikiFileEntry>(backendApiPath(`/knowledge/wiki_exports/${serializePathParameter(exportId, { name: 'exportId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeWikiLogEntriesApi {
  private client: HttpClient;
  
  constructor(client: HttpClient) { 
    this.client = client; 
  }


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


async rebuild(body: WikiIndexRebuildRequest): Promise<WikiIndexDocument> {
    return this.client.post<WikiIndexDocument>(backendApiPath(`/knowledge/wiki_index/rebuild`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiSchemaProfilesApi {
  private client: HttpClient;
  
  constructor(client: HttpClient) { 
    this.client = client; 
  }


async create(body: KnowledgeWikiSchemaProfileRequest): Promise<KnowledgeWikiFileEntry> {
    return this.client.post<KnowledgeWikiFileEntry>(backendApiPath(`/knowledge/wiki_schema_profiles`), body, undefined, undefined, 'application/json');
  }

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


async publish(pageId: number, body: WikiPagePublishRequest): Promise<WikiPageSummary> {
    return this.client.post<WikiPageSummary>(backendApiPath(`/knowledge/wiki_pages/${serializePathParameter(pageId, { name: 'pageId', style: 'simple', explode: false })}/publish`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiCandidatesApi {
  private client: HttpClient;
  
  constructor(client: HttpClient) { 
    this.client = client; 
  }


async list(): Promise<WikiCandidateResultList> {
    return this.client.get<WikiCandidateResultList>(backendApiPath(`/knowledge/wiki_candidates`));
  }

async approve(candidateId: number, body: WikiCandidateReviewRequest): Promise<WikiCandidateResult> {
    return this.client.post<WikiCandidateResult>(backendApiPath(`/knowledge/wiki_candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/approve`), body, undefined, undefined, 'application/json');
  }

async reject(candidateId: number, body: WikiCandidateReviewRequest): Promise<WikiCandidateResult> {
    return this.client.post<WikiCandidateResult>(backendApiPath(`/knowledge/wiki_candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/reject`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWikiCompileJobsApi {
  private client: HttpClient;
  
  constructor(client: HttpClient) { 
    this.client = client; 
  }


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


async list(): Promise<KnowledgeSourceList> {
    return this.client.get<KnowledgeSourceList>(backendApiPath(`/knowledge/sources`));
  }

async create(body: CreateKnowledgeSourceRequest): Promise<KnowledgeSource> {
    return this.client.post<KnowledgeSource>(backendApiPath(`/knowledge/sources`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeApi {
  private client: HttpClient;
  public readonly sources: KnowledgeSourcesApi;
  public readonly wiki: KnowledgeWikiApi;
  
  constructor(client: HttpClient) { 
    this.client = client;
    this.sources = new KnowledgeSourcesApi(client);
    this.wiki = new KnowledgeWikiApi(client); 
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
