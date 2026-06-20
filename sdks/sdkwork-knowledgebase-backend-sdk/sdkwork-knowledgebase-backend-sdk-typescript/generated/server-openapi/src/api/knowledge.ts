import { backendApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { CreateKnowledgeSourceRequest, IngestionJob, KnowledgeIndex, KnowledgeIndexRequest, KnowledgeOkfBundleFile, KnowledgeOkfBundleFileList, KnowledgeOkfProfileRequest, KnowledgeProviderHealth, KnowledgeRetrievalProfile, KnowledgeRetrievalProfileRequest, KnowledgeRetrievalTrace, KnowledgeRetrievalTraceList, KnowledgeSource, KnowledgeSourceList, OkfBundleExportRequest, OkfBundleImportRequest, OkfBundleImportResult, OkfBundleIndexRebuildRequest, OkfCandidateResult, OkfCandidateResultList, OkfCandidateReviewRequest, OkfCompileJobRequest, OkfConceptPublishRequest, OkfConceptSummary, OkfIndexDocument, OkfLogEntry, OkfQualityRun, OkfQualityRunRequest } from '../types';


export class KnowledgeApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }

async sourcesList(): Promise<KnowledgeSourceList> {
    return this.client.get<KnowledgeSourceList>(backendApiPath(`/knowledge/sources`));
  }

async sourcesCreate(body: CreateKnowledgeSourceRequest): Promise<KnowledgeSource> {
    return this.client.post<KnowledgeSource>(backendApiPath(`/knowledge/sources`), body, undefined, undefined, 'application/json');
  }

async okfCompileJobsCreate(body: OkfCompileJobRequest): Promise<IngestionJob> {
    return this.client.post<IngestionJob>(backendApiPath(`/knowledge/okf/compile_jobs`), body, undefined, undefined, 'application/json');
  }

async okfCandidatesList(): Promise<OkfCandidateResultList> {
    return this.client.get<OkfCandidateResultList>(backendApiPath(`/knowledge/okf/candidates`));
  }

async okfCandidatesApprove(candidateId: number, body: OkfCandidateReviewRequest): Promise<OkfCandidateResult> {
    return this.client.post<OkfCandidateResult>(backendApiPath(`/knowledge/okf/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/approve`), body, undefined, undefined, 'application/json');
  }

async okfCandidatesReject(candidateId: number, body: OkfCandidateReviewRequest): Promise<OkfCandidateResult> {
    return this.client.post<OkfCandidateResult>(backendApiPath(`/knowledge/okf/candidates/${serializePathParameter(candidateId, { name: 'candidateId', style: 'simple', explode: false })}/reject`), body, undefined, undefined, 'application/json');
  }

async okfConceptsPublish(conceptId: number, body: OkfConceptPublishRequest): Promise<OkfConceptSummary> {
    return this.client.post<OkfConceptSummary>(backendApiPath(`/knowledge/okf/concepts/${serializePathParameter(conceptId, { name: 'conceptId', style: 'simple', explode: false })}/publish`), body, undefined, undefined, 'application/json');
  }

async okfProfileCreate(body: KnowledgeOkfProfileRequest): Promise<KnowledgeOkfBundleFile> {
    return this.client.post<KnowledgeOkfBundleFile>(backendApiPath(`/knowledge/okf/profile`), body, undefined, undefined, 'application/json');
  }

async okfProfileUpdate(profileId: number, body: KnowledgeOkfProfileRequest): Promise<KnowledgeOkfBundleFile> {
    return this.client.patch<KnowledgeOkfBundleFile>(backendApiPath(`/knowledge/okf/profile/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

async okfBundleIndexRebuild(body: OkfBundleIndexRebuildRequest): Promise<OkfIndexDocument> {
    return this.client.post<OkfIndexDocument>(backendApiPath(`/knowledge/okf/index/rebuild`), body, undefined, undefined, 'application/json');
  }

async okfLogEntriesCreate(body: OkfLogEntry): Promise<OkfLogEntry> {
    return this.client.post<OkfLogEntry>(backendApiPath(`/knowledge/okf/log_entries`), body, undefined, undefined, 'application/json');
  }

async okfBundleExportCreate(body: OkfBundleExportRequest): Promise<KnowledgeOkfBundleFile> {
    return this.client.post<KnowledgeOkfBundleFile>(backendApiPath(`/knowledge/okf/exports`), body, undefined, undefined, 'application/json');
  }

async okfBundleExportRetrieve(exportId: number): Promise<KnowledgeOkfBundleFile> {
    return this.client.get<KnowledgeOkfBundleFile>(backendApiPath(`/knowledge/okf/exports/${serializePathParameter(exportId, { name: 'exportId', style: 'simple', explode: false })}`));
  }

async okfBundleFilesList(): Promise<KnowledgeOkfBundleFileList> {
    return this.client.get<KnowledgeOkfBundleFileList>(backendApiPath(`/knowledge/okf/bundle/files`));
  }

async okfLintRunsCreate(body: OkfQualityRunRequest): Promise<OkfQualityRun> {
    return this.client.post<OkfQualityRun>(backendApiPath(`/knowledge/okf/lint_runs`), body, undefined, undefined, 'application/json');
  }

async okfEvalRunsCreate(body: OkfQualityRunRequest): Promise<OkfQualityRun> {
    return this.client.post<OkfQualityRun>(backendApiPath(`/knowledge/okf/eval_runs`), body, undefined, undefined, 'application/json');
  }

async indexesCreate(body: KnowledgeIndexRequest): Promise<KnowledgeIndex> {
    return this.client.post<KnowledgeIndex>(backendApiPath(`/knowledge/indexes`), body, undefined, undefined, 'application/json');
  }

async indexesRetrieve(indexId: string): Promise<KnowledgeIndex> {
    return this.client.get<KnowledgeIndex>(backendApiPath(`/knowledge/indexes/${serializePathParameter(indexId, { name: 'indexId', style: 'simple', explode: false })}`));
  }

async indexesRebuild(indexId: string, body: OkfBundleIndexRebuildRequest): Promise<OkfIndexDocument> {
    return this.client.post<OkfIndexDocument>(backendApiPath(`/knowledge/indexes/${serializePathParameter(indexId, { name: 'indexId', style: 'simple', explode: false })}/rebuild`), body, undefined, undefined, 'application/json');
  }

async retrievalProfilesCreate(body: KnowledgeRetrievalProfileRequest): Promise<KnowledgeRetrievalProfile> {
    return this.client.post<KnowledgeRetrievalProfile>(backendApiPath(`/knowledge/retrieval_profiles`), body, undefined, undefined, 'application/json');
  }

async retrievalProfilesRetrieve(profileId: string): Promise<KnowledgeRetrievalProfile> {
    return this.client.get<KnowledgeRetrievalProfile>(backendApiPath(`/knowledge/retrieval_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`));
  }

async retrievalProfilesUpdate(profileId: string, body: KnowledgeRetrievalProfileRequest): Promise<KnowledgeRetrievalProfile> {
    return this.client.patch<KnowledgeRetrievalProfile>(backendApiPath(`/knowledge/retrieval_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

async retrievalTracesList(): Promise<KnowledgeRetrievalTraceList> {
    return this.client.get<KnowledgeRetrievalTraceList>(backendApiPath(`/knowledge/retrieval_traces`));
  }

async retrievalTracesRetrieve(traceId: string): Promise<KnowledgeRetrievalTrace> {
    return this.client.get<KnowledgeRetrievalTrace>(backendApiPath(`/knowledge/retrieval_traces/${serializePathParameter(traceId, { name: 'traceId', style: 'simple', explode: false })}`));
  }

async providerHealthRetrieve(): Promise<KnowledgeProviderHealth> {
    return this.client.get<KnowledgeProviderHealth>(backendApiPath(`/knowledge/provider_health`));
  }

async okfBundleImportCreate(body: OkfBundleImportRequest): Promise<OkfBundleImportResult> {
    return this.client.post<OkfBundleImportResult>(backendApiPath(`/knowledge/okf/imports`), body, undefined, undefined, 'application/json');
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
