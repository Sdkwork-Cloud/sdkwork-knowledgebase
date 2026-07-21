import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { ConsumeGroupKnowledgebaseLaunchTicketRequest, CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest, CreateKnowledgeSiteHostBindingRequest, CreateKnowledgeSpaceContextBindingRequest, CreateKnowledgeSpaceRequest, GrantKnowledgeSpaceMemberRequest, GroupKnowledgebaseLaunchTarget, IngestionJob, KnowledgeAgentBinding, KnowledgeAgentBindingRequest, KnowledgeAgentChatRequest, KnowledgeAgentChatResponse, KnowledgeAgentProfile, KnowledgeAgentProfileRequest, KnowledgeBrowserListData, KnowledgeBrowserView, KnowledgeContextPack, KnowledgeContextPackRequest, KnowledgeDocument, KnowledgeDocumentContent, KnowledgeDocumentVersion, KnowledgeDriveImportRequest, KnowledgeDriveImportResult, KnowledgeGitImportRequest, KnowledgeGitImportResult, KnowledgeGitSyncRequest, KnowledgeGitSyncResult, KnowledgeIngestRequest, KnowledgeMarketCatalogItem, KnowledgeMarketSubscriptionRequest, KnowledgeMarketSubscriptionResult, KnowledgeMediaTaskRequest, KnowledgeMediaTaskResult, KnowledgeOkfBundleFile, KnowledgeOkfConceptRevisionList, KnowledgeRetrievalRequest, KnowledgeRetrievalResult, KnowledgeSite, KnowledgeSiteHostBinding, KnowledgeSitePublicationResult, KnowledgeSiteRelease, KnowledgeSpace, KnowledgeSpaceContextBinding, KnowledgeSpaceMember, KnowledgeSpaceMemberSubjectType, KnowledgeWechatAppletList, KnowledgeWechatArticlesPreviewRequest, KnowledgeWechatArticlesPublishRequest, KnowledgeWechatFanTagList, KnowledgeWechatOfficialAccountList, KnowledgeWechatOperationResult, KnowledgeWechatReplaceAppletsRequest, KnowledgeWechatReplaceOfficialAccountsRequest, OkfBundleExportRequest, OkfBundleImportRequest, OkfBundleImportResult, OkfConceptSummary, OkfConceptSummaryList, OkfConceptUpsertRequest, OkfContextPackRequest, OkfFileAnswerRequest, OkfIndexDocument, OkfLogDocument, OkfProfileDocument, OkfQualityRun, OkfQualityRunRequest, OkfQueryRequest, OkfQueryResult, PageInfo, PublishKnowledgeSiteReleaseRequest, RollbackKnowledgeSiteReleaseRequest, SdkWorkCommandData, UpdateKnowledgeSpaceContextBindingRequest, UpdateKnowledgeSpaceRequest, UpsertKnowledgeSiteRequest } from '../types';


export interface KnowledgeSiteHostBindingsListParams {
  cursor?: string;
  pageSize?: number;
}

export interface KnowledgeSiteHostBindingsDeleteParams {
  expectedVersion: string;
}

export class KnowledgeSiteHostBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List knowledge site host bindings */
  async list(siteId: string, params?: KnowledgeSiteHostBindingsListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/knowledge/sites/${serializePathParameter(siteId, { name: 'siteId', style: 'simple', explode: false })}/host_bindings`), query));
  }

/** Create a custom-prefix or external-domain host binding */
  async create(siteId: string, body: CreateKnowledgeSiteHostBindingRequest): Promise<KnowledgeSiteHostBinding> {
    return this.client.post<KnowledgeSiteHostBinding>(appApiPath(`/knowledge/sites/${serializePathParameter(siteId, { name: 'siteId', style: 'simple', explode: false })}/host_bindings`), body, undefined, undefined, 'application/json');
  }

/** Delete a non-system knowledge site host binding */
  async delete(siteId: string, bindingId: string, params: KnowledgeSiteHostBindingsDeleteParams): Promise<void> {
    const query = buildQueryString([
      { name: 'expected_version', value: params.expectedVersion, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.delete<void>(appendQueryString(appApiPath(`/knowledge/sites/${serializePathParameter(siteId, { name: 'siteId', style: 'simple', explode: false })}/host_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`), query));
  }
}

export interface KnowledgeSiteReleasesListParams {
  cursor?: string;
  pageSize?: number;
}

export class KnowledgeSiteReleasesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List immutable knowledge site releases */
  async list(siteId: string, params?: KnowledgeSiteReleasesListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/knowledge/sites/${serializePathParameter(siteId, { name: 'siteId', style: 'simple', explode: false })}/releases`), query));
  }

/** Build and atomically publish a knowledge site release */
  async create(siteId: string, body: PublishKnowledgeSiteReleaseRequest): Promise<KnowledgeSitePublicationResult> {
    return this.client.post<KnowledgeSitePublicationResult>(appApiPath(`/knowledge/sites/${serializePathParameter(siteId, { name: 'siteId', style: 'simple', explode: false })}/releases`), body, undefined, undefined, 'application/json');
  }

/** Retrieve an immutable knowledge site release */
  async retrieve(releaseId: string): Promise<KnowledgeSiteRelease> {
    return this.client.get<KnowledgeSiteRelease>(appApiPath(`/knowledge/site_releases/${serializePathParameter(releaseId, { name: 'releaseId', style: 'simple', explode: false })}`));
  }

/** Atomically activate a prior ready knowledge site release */
  async rollback(siteId: string, body: RollbackKnowledgeSiteReleaseRequest): Promise<KnowledgeSite> {
    return this.client.post<KnowledgeSite>(appApiPath(`/knowledge/sites/${serializePathParameter(siteId, { name: 'siteId', style: 'simple', explode: false })}/rollbacks`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeSitesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve a knowledge site by space */
  async retrieve(spaceId: string): Promise<KnowledgeSite> {
    return this.client.get<KnowledgeSite>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/site`));
  }

/** Create or update a knowledge site */
  async update(spaceId: string, body: UpsertKnowledgeSiteRequest): Promise<KnowledgeSite> {
    return this.client.put<KnowledgeSite>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/site`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeMediaTasksApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create a knowledge media task (image generation or speech-to-text) */
  async create(body: KnowledgeMediaTaskRequest): Promise<KnowledgeMediaTaskResult> {
    return this.client.post<KnowledgeMediaTaskResult>(appApiPath(`/knowledge/media_tasks`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeMarketSubscriptionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Subscribe to a knowledge market listing */
  async create(body: KnowledgeMarketSubscriptionRequest): Promise<KnowledgeMarketSubscriptionResult> {
    return this.client.post<KnowledgeMarketSubscriptionResult>(appApiPath(`/knowledge/market/subscriptions`), body, undefined, undefined, 'application/json');
  }

/** Unsubscribe from a knowledge market listing */
  async delete(listingId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/market/subscriptions/${serializePathParameter(listingId, { name: 'listingId', style: 'simple', explode: false })}`));
  }
}

export interface KnowledgeMarketListingsListParams {
  cursor?: string;
  pageSize?: number;
}

export class KnowledgeMarketListingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List knowledge market catalog listings */
  async list(params?: KnowledgeMarketListingsListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/knowledge/market/listings`), query));
  }
}

export class KnowledgeMarketApi {
  private client: HttpClient;
  public readonly listings: KnowledgeMarketListingsApi;
  public readonly subscriptions: KnowledgeMarketSubscriptionsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.listings = new KnowledgeMarketListingsApi(client);
    this.subscriptions = new KnowledgeMarketSubscriptionsApi(client);
  }

}

export class KnowledgeGitSyncsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Sync knowledge space documents to a Git repository */
  async create(body: KnowledgeGitSyncRequest): Promise<KnowledgeGitSyncResult> {
    return this.client.post<KnowledgeGitSyncResult>(appApiPath(`/knowledge/git_syncs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWechatArticlesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Publish WeChat articles */
  async publish(body: KnowledgeWechatArticlesPublishRequest): Promise<KnowledgeWechatOperationResult> {
    return this.client.post<KnowledgeWechatOperationResult>(appApiPath(`/knowledge/wechat/articles/publish`), body, undefined, undefined, 'application/json');
  }

/** Preview WeChat articles */
  async preview(body: KnowledgeWechatArticlesPreviewRequest): Promise<KnowledgeWechatOperationResult> {
    return this.client.post<KnowledgeWechatOperationResult>(appApiPath(`/knowledge/wechat/articles/preview`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWechatAppletsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List WeChat applets */
  async list(): Promise<KnowledgeWechatAppletList> {
    return this.client.get<KnowledgeWechatAppletList>(appApiPath(`/knowledge/wechat/applets`));
  }

/** Replace WeChat applets */
  async update(body: KnowledgeWechatReplaceAppletsRequest): Promise<KnowledgeWechatAppletList> {
    return this.client.put<KnowledgeWechatAppletList>(appApiPath(`/knowledge/wechat/applets`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWechatOfficialAccountsFanTagsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List WeChat official account fan tags */
  async list(accountId: string): Promise<KnowledgeWechatFanTagList> {
    return this.client.get<KnowledgeWechatFanTagList>(appApiPath(`/knowledge/wechat/official_accounts/${serializePathParameter(accountId, { name: 'accountId', style: 'simple', explode: false })}/fan_tags`));
  }
}

export class KnowledgeWechatOfficialAccountsApi {
  private client: HttpClient;
  public readonly fanTags: KnowledgeWechatOfficialAccountsFanTagsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.fanTags = new KnowledgeWechatOfficialAccountsFanTagsApi(client);
  }


/** List WeChat official accounts */
  async list(): Promise<KnowledgeWechatOfficialAccountList> {
    return this.client.get<KnowledgeWechatOfficialAccountList>(appApiPath(`/knowledge/wechat/official_accounts`));
  }

/** Replace WeChat official accounts */
  async update(body: KnowledgeWechatReplaceOfficialAccountsRequest): Promise<KnowledgeWechatOfficialAccountList> {
    return this.client.put<KnowledgeWechatOfficialAccountList>(appApiPath(`/knowledge/wechat/official_accounts`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeWechatApi {
  private client: HttpClient;
  public readonly officialAccounts: KnowledgeWechatOfficialAccountsApi;
  public readonly applets: KnowledgeWechatAppletsApi;
  public readonly articles: KnowledgeWechatArticlesApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.officialAccounts = new KnowledgeWechatOfficialAccountsApi(client);
    this.applets = new KnowledgeWechatAppletsApi(client);
    this.articles = new KnowledgeWechatArticlesApi(client);
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
  async chat(profileId: string, body: KnowledgeAgentChatRequest): Promise<KnowledgeAgentChatResponse> {
    return this.client.post<KnowledgeAgentChatResponse>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/chat`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeAgentProfilesRetrievalPreviewApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Preview retrieval for an agent profile */
  async retrievalPreview(profileId: string, body: KnowledgeRetrievalRequest): Promise<KnowledgeRetrievalResult> {
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
  async bindings(profileId: string, body: KnowledgeAgentBindingRequest): Promise<KnowledgeAgentBinding> {
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

export class KnowledgeOkfLintRunsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF bundle lint run */
  async create(body: OkfQualityRunRequest): Promise<OkfQualityRun> {
    return this.client.post<OkfQualityRun>(appApiPath(`/knowledge/okf/lint_runs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfContextPacksApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF context pack */
  async create(body: OkfContextPackRequest): Promise<KnowledgeOkfBundleFile> {
    return this.client.post<KnowledgeOkfBundleFile>(appApiPath(`/knowledge/okf/context_packs`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfQueriesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF query */
  async create(body: OkfQueryRequest): Promise<OkfQueryResult> {
    return this.client.post<OkfQueryResult>(appApiPath(`/knowledge/okf/queries`), body, undefined, undefined, 'application/json');
  }

/** File an answer for an OKF query */
  async fileAnswer(queryId: string, body: OkfFileAnswerRequest): Promise<OkfQueryResult> {
    return this.client.post<OkfQueryResult>(appApiPath(`/knowledge/okf/queries/${serializePathParameter(queryId, { name: 'queryId', style: 'simple', explode: false })}/file_answer`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfBundleImportApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Import an OKF bundle from drive staging */
  async create(body: OkfBundleImportRequest): Promise<OkfBundleImportResult> {
    return this.client.post<OkfBundleImportResult>(appApiPath(`/knowledge/okf/imports`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfBundleExportApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Create an OKF bundle export */
  async create(body: OkfBundleExportRequest): Promise<KnowledgeOkfBundleFile> {
    return this.client.post<KnowledgeOkfBundleFile>(appApiPath(`/knowledge/okf/exports`), body, undefined, undefined, 'application/json');
  }

/** Retrieve an OKF bundle export */
  async retrieve(exportId: string): Promise<KnowledgeOkfBundleFile> {
    return this.client.get<KnowledgeOkfBundleFile>(appApiPath(`/knowledge/okf/exports/${serializePathParameter(exportId, { name: 'exportId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeOkfBundleProfileApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve the OKF bundle profile */
  async list(): Promise<OkfProfileDocument> {
    return this.client.get<OkfProfileDocument>(appApiPath(`/knowledge/okf/profile`));
  }
}

export class KnowledgeOkfBundleLogApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve the OKF bundle log */
  async list(): Promise<OkfLogDocument> {
    return this.client.get<OkfLogDocument>(appApiPath(`/knowledge/okf/log`));
  }
}

export class KnowledgeOkfBundleIndexApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve the OKF bundle index */
  async list(): Promise<OkfIndexDocument> {
    return this.client.get<OkfIndexDocument>(appApiPath(`/knowledge/okf/index`));
  }
}

export class KnowledgeOkfBundleApi {
  private client: HttpClient;
  public readonly index: KnowledgeOkfBundleIndexApi;
  public readonly log: KnowledgeOkfBundleLogApi;
  public readonly profile: KnowledgeOkfBundleProfileApi;
  public readonly export: KnowledgeOkfBundleExportApi;
  public readonly import: KnowledgeOkfBundleImportApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.index = new KnowledgeOkfBundleIndexApi(client);
    this.log = new KnowledgeOkfBundleLogApi(client);
    this.profile = new KnowledgeOkfBundleProfileApi(client);
    this.export = new KnowledgeOkfBundleExportApi(client);
    this.import = new KnowledgeOkfBundleImportApi(client);
  }

}

export interface KnowledgeOkfConceptsRevisionsListParams {
  cursor?: string;
  pageSize?: number;
}

export class KnowledgeOkfConceptsRevisionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List OKF concept revisions */
  async list(conceptId: string, params?: KnowledgeOkfConceptsRevisionsListParams): Promise<KnowledgeOkfConceptRevisionList> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<KnowledgeOkfConceptRevisionList>(appendQueryString(appApiPath(`/knowledge/okf/concepts/${serializePathParameter(conceptId, { name: 'conceptId', style: 'simple', explode: false })}/revisions`), query));
  }
}

export interface KnowledgeOkfConceptsListParams {
  spaceId: string;
  cursor?: string;
  pageSize?: number;
}

export class KnowledgeOkfConceptsApi {
  private client: HttpClient;
  public readonly revisions: KnowledgeOkfConceptsRevisionsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.revisions = new KnowledgeOkfConceptsRevisionsApi(client);
  }


/** List OKF concepts */
  async list(params: KnowledgeOkfConceptsListParams): Promise<OkfConceptSummaryList> {
    const query = buildQueryString([
      { name: 'spaceId', value: params.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<OkfConceptSummaryList>(appendQueryString(appApiPath(`/knowledge/okf/concepts`), query));
  }

/** Retrieve an OKF concept */
  async retrieve(conceptId: string): Promise<OkfConceptSummary> {
    return this.client.get<OkfConceptSummary>(appApiPath(`/knowledge/okf/concepts/${serializePathParameter(conceptId, { name: 'conceptId', style: 'simple', explode: false })}`));
  }

/** Delete an OKF concept */
  async delete(conceptId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/okf/concepts/${serializePathParameter(conceptId, { name: 'conceptId', style: 'simple', explode: false })}`));
  }

/** Upsert an OKF concept revision */
  async update(body: OkfConceptUpsertRequest): Promise<OkfConceptSummary> {
    return this.client.put<OkfConceptSummary>(appApiPath(`/knowledge/okf/concepts/upsert`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeOkfApi {
  private client: HttpClient;
  public readonly concepts: KnowledgeOkfConceptsApi;
  public readonly bundle: KnowledgeOkfBundleApi;
  public readonly queries: KnowledgeOkfQueriesApi;
  public readonly contextPacks: KnowledgeOkfContextPacksApi;
  public readonly lintRuns: KnowledgeOkfLintRunsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.concepts = new KnowledgeOkfConceptsApi(client);
    this.bundle = new KnowledgeOkfBundleApi(client);
    this.queries = new KnowledgeOkfQueriesApi(client);
    this.contextPacks = new KnowledgeOkfContextPacksApi(client);
    this.lintRuns = new KnowledgeOkfLintRunsApi(client);
  }

}

export interface KnowledgeDocumentsVersionsListParams {
  cursor?: string;
  pageSize?: number;
}

export class KnowledgeDocumentsVersionsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List document versions */
  async list(documentId: string, params?: KnowledgeDocumentsVersionsListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}/versions`), query));
  }

/** Create a document version */
  async versions(documentId: string, body: CreateKnowledgeDocumentVersionRequest): Promise<KnowledgeDocumentVersion> {
    return this.client.post<KnowledgeDocumentVersion>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}/versions`), body, undefined, undefined, 'application/json');
  }
}

export class KnowledgeDocumentsContentApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Retrieve authoritative knowledge document content */
  async list(documentId: string): Promise<KnowledgeDocumentContent> {
    return this.client.get<KnowledgeDocumentContent>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}/content`));
  }
}

export interface KnowledgeDocumentsListParams {
  spaceId: string;
  cursor?: string;
  pageSize?: number;
}

export class KnowledgeDocumentsApi {
  private client: HttpClient;
  public readonly content: KnowledgeDocumentsContentApi;
  public readonly versions: KnowledgeDocumentsVersionsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.content = new KnowledgeDocumentsContentApi(client);
    this.versions = new KnowledgeDocumentsVersionsApi(client);
  }


/** List knowledge documents */
  async list(params: KnowledgeDocumentsListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'spaceId', value: params.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/knowledge/documents`), query));
  }

/** Create a knowledge document */
  async create(body: CreateKnowledgeDocumentRequest): Promise<KnowledgeDocument> {
    return this.client.post<KnowledgeDocument>(appApiPath(`/knowledge/documents`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge document */
  async retrieve(documentId: string): Promise<KnowledgeDocument> {
    return this.client.get<KnowledgeDocument>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}`));
  }

/** Update a knowledge document */
  async update(documentId: string, body: CreateKnowledgeDocumentRequest): Promise<KnowledgeDocument> {
    return this.client.patch<KnowledgeDocument>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete a knowledge document */
  async delete(documentId: string): Promise<void> {
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
  async retrieve(ingestId: string): Promise<IngestionJob> {
    return this.client.get<IngestionJob>(appApiPath(`/knowledge/ingests/${serializePathParameter(ingestId, { name: 'ingestId', style: 'simple', explode: false })}`));
  }
}

export class KnowledgeGitImportsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Import a Git repository into knowledgebase */
  async create(body: KnowledgeGitImportRequest): Promise<KnowledgeGitImportResult> {
    return this.client.post<KnowledgeGitImportResult>(appApiPath(`/knowledge/git_imports`), body, undefined, undefined, 'application/json');
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

export interface KnowledgeSpacesMembersListParams {
  cursor?: string;
  pageSize?: number;
}

export interface KnowledgeSpacesMembersDeleteParams {
  subjectType: KnowledgeSpaceMemberSubjectType;
  subjectId: string;
}

export class KnowledgeSpacesMembersApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List knowledge space members */
  async list(spaceId: string, params?: KnowledgeSpacesMembersListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/members`), query));
  }

/** Grant knowledge space member access */
  async members(spaceId: string, body: GrantKnowledgeSpaceMemberRequest): Promise<SdkWorkCommandData> {
    return this.client.post<SdkWorkCommandData>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/members`), body, undefined, undefined, 'application/json');
  }

/** Revoke knowledge space member access */
  async delete(spaceId: string, params: KnowledgeSpacesMembersDeleteParams): Promise<void> {
    const query = buildQueryString([
      { name: 'subjectType', value: params.subjectType, style: 'form', explode: true, allowReserved: false },
      { name: 'subjectId', value: params.subjectId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.delete<void>(appendQueryString(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/members`), query));
  }
}

export interface KnowledgeSpacesContextBindingsListParams {
  cursor?: string;
  pageSize?: number;
}

export class KnowledgeSpacesContextBindingsApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** List knowledge space context bindings */
  async list(spaceId: string, params?: KnowledgeSpacesContextBindingsListParams): Promise<Record<string, unknown>> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<Record<string, unknown>>(appendQueryString(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/context_bindings`), query));
  }

/** Create a knowledge space context binding */
  async contextBindings(spaceId: string, body: CreateKnowledgeSpaceContextBindingRequest): Promise<KnowledgeSpaceContextBinding> {
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


/** List knowledge browser view */
  async list(spaceId: string, params: KnowledgeSpacesBrowserListParams): Promise<KnowledgeBrowserListData> {
    const query = buildQueryString([
      { name: 'view', value: params.view, style: 'form', explode: true, allowReserved: false },
      { name: 'parentId', value: params.parentId, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<KnowledgeBrowserListData>(appendQueryString(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/browser`), query));
  }
}

export class KnowledgeSpacesApi {
  private client: HttpClient;
  public readonly browser: KnowledgeSpacesBrowserApi;
  public readonly contextBindings: KnowledgeSpacesContextBindingsApi;
  public readonly members: KnowledgeSpacesMembersApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.browser = new KnowledgeSpacesBrowserApi(client);
    this.contextBindings = new KnowledgeSpacesContextBindingsApi(client);
    this.members = new KnowledgeSpacesMembersApi(client);
  }


/** Create a knowledge space */
  async create(body: CreateKnowledgeSpaceRequest): Promise<KnowledgeSpace> {
    return this.client.post<KnowledgeSpace>(appApiPath(`/knowledge/spaces`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge space */
  async retrieve(spaceId: string): Promise<KnowledgeSpace> {
    return this.client.get<KnowledgeSpace>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`));
  }

/** Update a knowledge space */
  async update(spaceId: string, body: UpdateKnowledgeSpaceRequest): Promise<KnowledgeSpace> {
    return this.client.patch<KnowledgeSpace>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete a knowledge space */
  async delete(spaceId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`));
  }
}

export interface KnowledgeGroupLaunchesConsumeParams {
  idempotencyKey: string;
}

export class KnowledgeGroupLaunchesApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }


/** Consume a group knowledgebase launch ticket */
  async consume(body: ConsumeGroupKnowledgebaseLaunchTicketRequest, params: KnowledgeGroupLaunchesConsumeParams): Promise<GroupKnowledgebaseLaunchTarget> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<GroupKnowledgebaseLaunchTarget>(appApiPath(`/knowledge/group_launches/consume`), body, undefined, requestHeaders, 'application/json');
  }
}

export class KnowledgeApi {
  private client: HttpClient;
  public readonly groupLaunches: KnowledgeGroupLaunchesApi;
  public readonly spaces: KnowledgeSpacesApi;
  public readonly driveImports: KnowledgeDriveImportsApi;
  public readonly gitImports: KnowledgeGitImportsApi;
  public readonly ingests: KnowledgeIngestsApi;
  public readonly documents: KnowledgeDocumentsApi;
  public readonly okf: KnowledgeOkfApi;
  public readonly retrievals: KnowledgeRetrievalsApi;
  public readonly contextPacks: KnowledgeContextPacksApi;
  public readonly agentProfiles: KnowledgeAgentProfilesApi;
  public readonly contextBindings: KnowledgeContextBindingsApi;
  public readonly wechat: KnowledgeWechatApi;
  public readonly gitSyncs: KnowledgeGitSyncsApi;
  public readonly market: KnowledgeMarketApi;
  public readonly mediaTasks: KnowledgeMediaTasksApi;
  public readonly sites: KnowledgeSitesApi;
  public readonly siteReleases: KnowledgeSiteReleasesApi;
  public readonly siteHostBindings: KnowledgeSiteHostBindingsApi;

  constructor(client: HttpClient) {
    this.client = client;
    this.groupLaunches = new KnowledgeGroupLaunchesApi(client);
    this.spaces = new KnowledgeSpacesApi(client);
    this.driveImports = new KnowledgeDriveImportsApi(client);
    this.gitImports = new KnowledgeGitImportsApi(client);
    this.ingests = new KnowledgeIngestsApi(client);
    this.documents = new KnowledgeDocumentsApi(client);
    this.okf = new KnowledgeOkfApi(client);
    this.retrievals = new KnowledgeRetrievalsApi(client);
    this.contextPacks = new KnowledgeContextPacksApi(client);
    this.agentProfiles = new KnowledgeAgentProfilesApi(client);
    this.contextBindings = new KnowledgeContextBindingsApi(client);
    this.wechat = new KnowledgeWechatApi(client);
    this.gitSyncs = new KnowledgeGitSyncsApi(client);
    this.market = new KnowledgeMarketApi(client);
    this.mediaTasks = new KnowledgeMediaTasksApi(client);
    this.sites = new KnowledgeSitesApi(client);
    this.siteReleases = new KnowledgeSiteReleasesApi(client);
    this.siteHostBindings = new KnowledgeSiteHostBindingsApi(client);
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
function buildRequestHeaders(
  headers: Record<string, HeaderParameterSpec | undefined>,
  cookies: Record<string, HeaderParameterSpec | undefined> = {},
): Record<string, string> | undefined {
  const requestHeaders: Record<string, string> = {};

  for (const [name, parameter] of Object.entries(headers)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      requestHeaders[name] = serialized;
    }
  }

  const cookieHeader = buildCookieHeader(cookies);
  if (cookieHeader) {
    requestHeaders.Cookie = requestHeaders.Cookie
      ? `${requestHeaders.Cookie}; ${cookieHeader}`
      : cookieHeader;
  }

  return Object.keys(requestHeaders).length > 0 ? requestHeaders : undefined;
}

interface HeaderParameterSpec {
  value: unknown;
  style: string;
  explode: boolean;
  contentType?: string;
}

function buildCookieHeader(cookies: Record<string, HeaderParameterSpec | undefined>): string | undefined {
  const pairs: string[] = [];
  for (const [name, parameter] of Object.entries(cookies)) {
    const serialized = serializeParameterValue(parameter);
    if (serialized !== undefined) {
      pairs.push(`${encodeURIComponent(name)}=${encodeURIComponent(serialized)}`);
    }
  }
  return pairs.length > 0 ? pairs.join('; ') : undefined;
}

function serializeParameterValue(parameter: HeaderParameterSpec | undefined): string | undefined {
  const value = parameter?.value;
  if (value === undefined || value === null) {
    return undefined;
  }
  if (parameter?.contentType) {
    return JSON.stringify(value);
  }
  if (value instanceof Date) {
    return value.toISOString();
  }
  if (Array.isArray(value)) {
    return value.map((item) => serializeHeaderPrimitive(item)).join(',');
  }
  if (typeof value === 'object' && value !== null) {
    return serializeHeaderObject(value as Record<string, unknown>, parameter?.explode === true);
  }
  return serializeHeaderPrimitive(value);
}

function serializeHeaderObject(value: Record<string, unknown>, explode: boolean): string {
  const entries = Object.entries(value).filter(([, entryValue]) => entryValue !== undefined && entryValue !== null);
  if (explode) {
    return entries.map(([key, entryValue]) => `${key}=${serializeHeaderPrimitive(entryValue)}`).join(',');
  }
  return entries.flatMap(([key, entryValue]) => [key, serializeHeaderPrimitive(entryValue)]).join(',');
}

function serializeHeaderPrimitive(value: unknown): string {
  if (value instanceof Date) {
    return value.toISOString();
  }
  return String(value);
}
