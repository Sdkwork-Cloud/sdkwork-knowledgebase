import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { AnonymizeKnowledgeAuditSubjectRequest, AnonymizeKnowledgeAuditSubjectResult, ExportKnowledgeAuditEventsRequest, KnowledgeAuditEventExport } from '../types/knowledge-compliance';
import type { CreateKnowledgeSourceRequest, IngestionJob, KnowledgeIndex, KnowledgeIndexRequest, KnowledgeOkfBundleFile, KnowledgeOkfProfileRequest, KnowledgeProviderHealth, KnowledgeRetrievalProfile, KnowledgeRetrievalProfileRequest, KnowledgeRetrievalTrace, KnowledgeSource, KnowledgeTenantStatus, OkfBundleExportRequest, OkfBundleImportRequest, OkfBundleImportResult, OkfBundleIndexRebuildRequest, OkfCandidateResult, OkfCandidateReviewRequest, OkfCompileJobRequest, OkfConceptPublishRequest, OkfConceptSummary, OkfIndexDocument, OkfLogEntry, OkfQualityRun, OkfQualityRunRequest, PageInfo } from '../types';


export class KnowledgeComplianceAuditEventsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }

/** Export knowledge audit events for a subject */
  async export(body: ExportKnowledgeAuditEventsRequest): Promise<KnowledgeAuditEventExport> {
    return this.client.post<KnowledgeAuditEventExport>(backendApiPath(`/knowledge/compliance/audit_events/export`), body, undefined, undefined, 'application/json');
  }

/** Anonymize audit events for a subject */
  async anonymizeActor(body: AnonymizeKnowledgeAuditSubjectRequest): Promise<AnonymizeKnowledgeAuditSubjectResult> {
    return this.client.post<AnonymizeKnowledgeAuditSubjectResult>(backendApiPath(`/knowledge/compliance/audit_events/anonymize_actor`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeComplianceApi {
  private client: HttpClient;
  public readonly auditEvents: KnowledgeComplianceAuditEventsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.auditEvents = new KnowledgeComplianceAuditEventsApi(client);
  }
}

export class KnowledgeSpacesMembersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List knowledge space members */
  async list(spaceId: string, params?: { cursor?: string; pageSize?: number }): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'pageSize', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(backendApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/members`), query));
  }
}

export class KnowledgeSpacesApi {
  private client: HttpClient;
  public readonly members: KnowledgeSpacesMembersApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.members = new KnowledgeSpacesMembersApi(client);
  }


/** List knowledge spaces */
  async list(): Promise<Record<string, unknown>> {
    return this.client.get<Record<string, unknown>>(backendApiPath(`/knowledge/spaces`));
  }
}

export class KnowledgeTenantsCurrentApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve current tenant knowledgebase status */
  async retrieve(): Promise<KnowledgeTenantStatus> {
    return this.client.get<KnowledgeTenantStatus>(backendApiPath(`/knowledge/tenants/current`));
  }
}

export class KnowledgeTenantsApi {
  private client: HttpClient;
  public readonly current: KnowledgeTenantsCurrentApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.current = new KnowledgeTenantsCurrentApi(client);
  }

}

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
  async list(): Promise<Record<string, unknown>> {
    return this.client.get<Record<string, unknown>>(backendApiPath(`/knowledge/retrieval_traces`));
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


/** List knowledge indexes */
  async list(): Promise<Record<string, unknown>> {
    return this.client.get<Record<string, unknown>>(backendApiPath(`/knowledge/indexes`));
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
  async rebuild(indexId: string, body: OkfBundleIndexRebuildRequest): Promise<OkfIndexDocument> {
    return this.client.post<OkfIndexDocument>(backendApiPath(`/knowledge/indexes/${serializePathParameter(indexId, { name: 'indexId', style: 'simple', explode: false })}/rebuild`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfEvalRunsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF eval run */
  async create(body: OkfQualityRunRequest): Promise<OkfQualityRun> {
    return this.client.post<OkfQualityRun>(backendApiPath(`/knowledge/okf/eval_runs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfLintRunsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF lint run */
  async create(body: OkfQualityRunRequest): Promise<OkfQualityRun> {
    return this.client.post<OkfQualityRun>(backendApiPath(`/knowledge/okf/lint_runs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfLogEntriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF log entry */
  async create(body: OkfLogEntry): Promise<OkfLogEntry> {
    return this.client.post<OkfLogEntry>(backendApiPath(`/knowledge/okf/log_entries`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfLogApi {
  private client: HttpClient;
  public readonly entries: KnowledgeOkfLogEntriesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.entries = new KnowledgeOkfLogEntriesApi(client);
  }

}

export class KnowledgeOkfBundleImportApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Import an OKF bundle from drive staging */
  async create(body: OkfBundleImportRequest): Promise<OkfBundleImportResult> {
    return this.client.post<OkfBundleImportResult>(backendApiPath(`/knowledge/okf/imports`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfBundleFilesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List OKF bundle files */
  async list(): Promise<Record<string, unknown>> {
    return this.client.get<Record<string, unknown>>(backendApiPath(`/knowledge/okf/bundle/files`));
  }
}

export class KnowledgeOkfBundleExportApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF bundle export */
  async create(body: OkfBundleExportRequest): Promise<KnowledgeOkfBundleFile> {
    return this.client.post<KnowledgeOkfBundleFile>(backendApiPath(`/knowledge/okf/exports`), body, undefined, undefined, 'application/json');
  }

/** Retrieve an OKF bundle export */
  async retrieve(exportId: number): Promise<KnowledgeOkfBundleFile> {
    return this.client.get<KnowledgeOkfBundleFile>(backendApiPath(`/knowledge/okf/exports/${serializePathParameter(exportId, { name: 'exportId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeOkfBundleIndexApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Rebuild the OKF bundle index */
  async rebuild(body: OkfBundleIndexRebuildRequest): Promise<OkfIndexDocument> {
    return this.client.post<OkfIndexDocument>(backendApiPath(`/knowledge/okf/index/rebuild`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfBundleApi {
  private client: HttpClient;
  public readonly index: KnowledgeOkfBundleIndexApi;
  public readonly export: KnowledgeOkfBundleExportApi;
  public readonly files: KnowledgeOkfBundleFilesApi;
  public readonly import: KnowledgeOkfBundleImportApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.index = new KnowledgeOkfBundleIndexApi(client);
    this.export = new KnowledgeOkfBundleExportApi(client);
    this.files = new KnowledgeOkfBundleFilesApi(client);
    this.import = new KnowledgeOkfBundleImportApi(client);
  }

}

export class KnowledgeOkfProfileApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF profile */
  async create(body: KnowledgeOkfProfileRequest): Promise<KnowledgeOkfBundleFile> {
    return this.client.post<KnowledgeOkfBundleFile>(backendApiPath(`/knowledge/okf/profile`), body, undefined, undefined, 'application/json');
  }

/** Update an OKF profile */
  async update(profileId: number, body: KnowledgeOkfProfileRequest): Promise<KnowledgeOkfBundleFile> {
    return this.client.patch<KnowledgeOkfBundleFile>(backendApiPath(`/knowledge/okf/profile/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfConceptsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Publish an OKF concept */
  async publish(conceptId: number, body: OkfConceptPublishRequest): Promise<OkfConceptSummary> {
    return this.client.post<OkfConceptSummary>(backendApiPath(`/knowledge/okf/concepts/${serializePathParameter(conceptId, { name: 'conceptId', style: 'simple', explode: false })}/publish`), body, undefined, undefined, 'application/json');
  }
}

export interface KnowledgeOkfCandidatesListParams {
  spaceId: number;
}

export class KnowledgeOkfCandidatesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List OKF candidates */
  async list(params: KnowledgeOkfCandidatesListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'spaceId', value: params.spaceId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(backendApiPath(`/knowledge/okf/candidates`), query));
  }

/** Approve an OKF candidate */
  async approve(candidateId: number, body: OkfCandidateReviewRequest): Promise<OkfCandidateResult> {
    return this.client.post<OkfCandidateResult>(backendApiPath(`/knowledge/okf/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/approve`), body, undefined, undefined, 'application/json');
  }

/** Reject an OKF candidate */
  async reject(candidateId: number, body: OkfCandidateReviewRequest): Promise<OkfCandidateResult> {
    return this.client.post<OkfCandidateResult>(backendApiPath(`/knowledge/okf/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/reject`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfCompileJobsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF compile job */
  async create(body: OkfCompileJobRequest): Promise<IngestionJob> {
    return this.client.post<IngestionJob>(backendApiPath(`/knowledge/okf/compile_jobs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfApi {
  private client: HttpClient;
  public readonly compileJobs: KnowledgeOkfCompileJobsApi;
  public readonly candidates: KnowledgeOkfCandidatesApi;
  public readonly concepts: KnowledgeOkfConceptsApi;
  public readonly profile: KnowledgeOkfProfileApi;
  public readonly bundle: KnowledgeOkfBundleApi;
  public readonly log: KnowledgeOkfLogApi;
  public readonly lintRuns: KnowledgeOkfLintRunsApi;
  public readonly evalRuns: KnowledgeOkfEvalRunsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.compileJobs = new KnowledgeOkfCompileJobsApi(client);
    this.candidates = new KnowledgeOkfCandidatesApi(client);
    this.concepts = new KnowledgeOkfConceptsApi(client);
    this.profile = new KnowledgeOkfProfileApi(client);
    this.bundle = new KnowledgeOkfBundleApi(client);
    this.log = new KnowledgeOkfLogApi(client);
    this.lintRuns = new KnowledgeOkfLintRunsApi(client);
    this.evalRuns = new KnowledgeOkfEvalRunsApi(client);
  }

}

export class KnowledgeSourcesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List knowledge sources */
  async list(): Promise<Record<string, unknown>> {
    return this.client.get<Record<string, unknown>>(backendApiPath(`/knowledge/sources`));
  }

/** Create a knowledge source */
  async create(body: CreateKnowledgeSourceRequest): Promise<KnowledgeSource> {
    return this.client.post<KnowledgeSource>(backendApiPath(`/knowledge/sources`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeApi {
  private client: HttpClient;
  public readonly sources: KnowledgeSourcesApi;
  public readonly okf: KnowledgeOkfApi;
  public readonly indexes: KnowledgeIndexesApi;
  public readonly retrievalProfiles: KnowledgeRetrievalProfilesApi;
  public readonly retrievalTraces: KnowledgeRetrievalTracesApi;
  public readonly providerHealth: KnowledgeProviderHealthApi;
  public readonly tenants: KnowledgeTenantsApi;
  public readonly spaces: KnowledgeSpacesApi;
  public readonly compliance: KnowledgeComplianceApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.sources = new KnowledgeSourcesApi(client);
    this.okf = new KnowledgeOkfApi(client);
    this.indexes = new KnowledgeIndexesApi(client);
    this.retrievalProfiles = new KnowledgeRetrievalProfilesApi(client);
    this.retrievalTraces = new KnowledgeRetrievalTracesApi(client);
    this.providerHealth = new KnowledgeProviderHealthApi(client);
    this.tenants = new KnowledgeTenantsApi(client);
    this.spaces = new KnowledgeSpacesApi(client);
    this.compliance = new KnowledgeComplianceApi(client);
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
