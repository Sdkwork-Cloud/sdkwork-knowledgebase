import { appApiPath } from './paths';
import type { HttpClient } from '../http/client';

import type { AgentProfilesBindingsListResponse, AgentProfilesBindingsResponse, AgentProfilesBindingsUpdateResponse, AgentProfilesChatResponse, AgentProfilesCreateResponse201, AgentProfilesRetrievalPreviewResponse, AgentProfilesRetrieveResponse, AgentProfilesUpdateResponse, ConsumeGroupKnowledgebaseLaunchTicketRequest, ContextBindingsRetrieveResponse, ContextBindingsUpdateResponse, ContextPacksCreateResponse201, CreateKnowledgeDocumentRequest, CreateKnowledgeDocumentVersionRequest, CreateKnowledgeSpaceContextBindingRequest, CreateKnowledgeSpaceRequest, DocumentsContentListResponse, DocumentsCreateResponse201, DocumentsListResponse, DocumentsRetrieveResponse, DocumentsUpdateResponse, DocumentsVersionsListResponse, DocumentsVersionsResponse, DriveImportsCreateResponse201, GitImportsCreateResponse201, GitSyncsCreateResponse201, GrantKnowledgeSpaceMemberRequest, GroupLaunchesConsumeResponse, IngestsCreateResponse201, IngestsRetrieveResponse, KnowledgeAgentBindingRequest, KnowledgeAgentChatRequest, KnowledgeAgentProfileRequest, KnowledgeBrowserView, KnowledgeContextPackRequest, KnowledgeDriveImportRequest, KnowledgeGitImportRequest, KnowledgeGitSyncRequest, KnowledgeIngestRequest, KnowledgeMarketSubscriptionRequest, KnowledgeMediaTaskRequest, KnowledgeRetrievalRequest, KnowledgeSpaceMemberSubjectType, KnowledgeWechatArticlesPreviewRequest, KnowledgeWechatArticlesPublishRequest, KnowledgeWechatReplaceAppletsRequest, KnowledgeWechatReplaceOfficialAccountsRequest, MarketListingsListResponse, MarketSubscriptionsCreateResponse201, MediaTasksCreateResponse201, OkfBundleExportCreateResponse201, OkfBundleExportRequest, OkfBundleExportRetrieveResponse, OkfBundleImportCreateResponse201, OkfBundleImportRequest, OkfBundleIndexListResponse, OkfBundleLogListResponse, OkfBundleProfileListResponse, OkfConceptsListResponse, OkfConceptsRetrieveResponse, OkfConceptsRevisionsListResponse, OkfConceptsUpdateResponse, OkfConceptUpsertRequest, OkfContextPackRequest, OkfContextPacksCreateResponse201, OkfFileAnswerRequest, OkfLintRunsCreateResponse201, OkfQualityRunRequest, OkfQueriesCreateResponse201, OkfQueriesFileAnswerResponse, OkfQueryRequest, RetrievalsCreateResponse201, RetrievalsRetrieveResponse, SpacesBrowserListResponse, SpacesContextBindingsListResponse, SpacesContextBindingsResponse, SpacesCreateResponse201, SpacesMembersListResponse, SpacesMembersResponse, SpacesRetrieveResponse, SpacesUpdateResponse, UpdateKnowledgeSpaceContextBindingRequest, UpdateKnowledgeSpaceRequest, WechatAppletsListResponse, WechatAppletsUpdateResponse, WechatArticlesPreviewResponse, WechatArticlesPublishResponse, WechatOfficialAccountsFanTagsListResponse, WechatOfficialAccountsListResponse, WechatOfficialAccountsUpdateResponse } from '../types';


export interface KnowledgeGroupLaunchesConsumeParams {
  idempotencyKey: string;
}

export interface KnowledgeDocumentsListParams {
  spaceId: string;
  cursor?: string;
  pageSize?: number;
}

export interface KnowledgeDocumentsVersionsListParams {
  cursor?: string;
  pageSize?: number;
}

export interface KnowledgeOkfConceptsListParams {
  spaceId: string;
  cursor?: string;
  pageSize?: number;
}

export interface KnowledgeOkfConceptsRevisionsListParams {
  cursor?: string;
  pageSize?: number;
}

export interface KnowledgeSpacesBrowserListParams {
  view: KnowledgeBrowserView;
  parentId?: string | null;
  cursor?: string | null;
  pageSize?: number;
}

export interface KnowledgeSpacesContextBindingsListParams {
  cursor?: string;
  pageSize?: number;
}

export interface KnowledgeSpacesMembersListParams {
  cursor?: string;
  pageSize?: number;
}

export interface KnowledgeSpacesMembersDeleteParams {
  subjectType: KnowledgeSpaceMemberSubjectType;
  subjectId: string;
}

export interface KnowledgeMarketListingsListParams {
  cursor?: string;
  pageSize?: number;
}

export class KnowledgeApi {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }

/** Consume a group knowledgebase launch ticket */
  async groupLaunchesConsume(body: ConsumeGroupKnowledgebaseLaunchTicketRequest, params: KnowledgeGroupLaunchesConsumeParams): Promise<GroupLaunchesConsumeResponse> {
    const requestHeaders = buildRequestHeaders(
      {
        'Idempotency-Key': { value: params.idempotencyKey, style: 'simple', explode: false },
      },
      {}
    );
    return this.client.post<GroupLaunchesConsumeResponse>(appApiPath(`/knowledge/group_launches/consume`), body, undefined, requestHeaders, 'application/json');
  }

/** Create a knowledge space */
  async spacesCreate(body: CreateKnowledgeSpaceRequest): Promise<SpacesCreateResponse201> {
    return this.client.post<SpacesCreateResponse201>(appApiPath(`/knowledge/spaces`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge space */
  async spacesRetrieve(spaceId: string): Promise<SpacesRetrieveResponse> {
    return this.client.get<SpacesRetrieveResponse>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`));
  }

/** Update a knowledge space */
  async spacesUpdate(spaceId: string, body: UpdateKnowledgeSpaceRequest): Promise<SpacesUpdateResponse> {
    return this.client.patch<SpacesUpdateResponse>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete a knowledge space */
  async spacesDelete(spaceId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}`));
  }

/** Import a drive object into knowledgebase */
  async driveImportsCreate(body: KnowledgeDriveImportRequest): Promise<DriveImportsCreateResponse201> {
    return this.client.post<DriveImportsCreateResponse201>(appApiPath(`/knowledge/drive_imports`), body, undefined, undefined, 'application/json');
  }

/** Import a Git repository into knowledgebase */
  async gitImportsCreate(body: KnowledgeGitImportRequest): Promise<GitImportsCreateResponse201> {
    return this.client.post<GitImportsCreateResponse201>(appApiPath(`/knowledge/git_imports`), body, undefined, undefined, 'application/json');
  }

/** Create an ingestion job */
  async ingestsCreate(body: KnowledgeIngestRequest): Promise<IngestsCreateResponse201> {
    return this.client.post<IngestsCreateResponse201>(appApiPath(`/knowledge/ingests`), body, undefined, undefined, 'application/json');
  }

/** Retrieve an ingestion job */
  async ingestsRetrieve(ingestId: string): Promise<IngestsRetrieveResponse> {
    return this.client.get<IngestsRetrieveResponse>(appApiPath(`/knowledge/ingests/${serializePathParameter(ingestId, { name: 'ingestId', style: 'simple', explode: false })}`));
  }

/** List knowledge documents */
  async documentsList(params: KnowledgeDocumentsListParams): Promise<DocumentsListResponse> {
    const query = buildQueryString([
      { name: 'spaceId', value: params.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<DocumentsListResponse>(appendQueryString(appApiPath(`/knowledge/documents`), query));
  }

/** Create a knowledge document */
  async documentsCreate(body: CreateKnowledgeDocumentRequest): Promise<DocumentsCreateResponse201> {
    return this.client.post<DocumentsCreateResponse201>(appApiPath(`/knowledge/documents`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge document */
  async documentsRetrieve(documentId: string): Promise<DocumentsRetrieveResponse> {
    return this.client.get<DocumentsRetrieveResponse>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}`));
  }

/** Update a knowledge document */
  async documentsUpdate(documentId: string, body: CreateKnowledgeDocumentRequest): Promise<DocumentsUpdateResponse> {
    return this.client.patch<DocumentsUpdateResponse>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete a knowledge document */
  async documentsDelete(documentId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}`));
  }

/** Retrieve authoritative knowledge document content */
  async documentsContentList(documentId: string): Promise<DocumentsContentListResponse> {
    return this.client.get<DocumentsContentListResponse>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}/content`));
  }

/** List document versions */
  async documentsVersionsList(documentId: string, params?: KnowledgeDocumentsVersionsListParams): Promise<DocumentsVersionsListResponse> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<DocumentsVersionsListResponse>(appendQueryString(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}/versions`), query));
  }

/** Create a document version */
  async documentsVersions(documentId: string, body: CreateKnowledgeDocumentVersionRequest): Promise<DocumentsVersionsResponse> {
    return this.client.post<DocumentsVersionsResponse>(appApiPath(`/knowledge/documents/${serializePathParameter(documentId, { name: 'documentId', style: 'simple', explode: false })}/versions`), body, undefined, undefined, 'application/json');
  }

/** List OKF concepts */
  async okfConceptsList(params: KnowledgeOkfConceptsListParams): Promise<OkfConceptsListResponse> {
    const query = buildQueryString([
      { name: 'spaceId', value: params.spaceId, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<OkfConceptsListResponse>(appendQueryString(appApiPath(`/knowledge/okf/concepts`), query));
  }

/** Retrieve an OKF concept */
  async okfConceptsRetrieve(conceptId: string): Promise<OkfConceptsRetrieveResponse> {
    return this.client.get<OkfConceptsRetrieveResponse>(appApiPath(`/knowledge/okf/concepts/${serializePathParameter(conceptId, { name: 'conceptId', style: 'simple', explode: false })}`));
  }

/** Delete an OKF concept */
  async okfConceptsDelete(conceptId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/okf/concepts/${serializePathParameter(conceptId, { name: 'conceptId', style: 'simple', explode: false })}`));
  }

/** List OKF concept revisions */
  async okfConceptsRevisionsList(conceptId: string, params?: KnowledgeOkfConceptsRevisionsListParams): Promise<OkfConceptsRevisionsListResponse> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<OkfConceptsRevisionsListResponse>(appendQueryString(appApiPath(`/knowledge/okf/concepts/${serializePathParameter(conceptId, { name: 'conceptId', style: 'simple', explode: false })}/revisions`), query));
  }

/** Retrieve the OKF bundle index */
  async okfBundleIndexList(): Promise<OkfBundleIndexListResponse> {
    return this.client.get<OkfBundleIndexListResponse>(appApiPath(`/knowledge/okf/index`));
  }

/** Retrieve the OKF bundle log */
  async okfBundleLogList(): Promise<OkfBundleLogListResponse> {
    return this.client.get<OkfBundleLogListResponse>(appApiPath(`/knowledge/okf/log`));
  }

/** Retrieve the OKF bundle profile */
  async okfBundleProfileList(): Promise<OkfBundleProfileListResponse> {
    return this.client.get<OkfBundleProfileListResponse>(appApiPath(`/knowledge/okf/profile`));
  }

/** Create an OKF query */
  async okfQueriesCreate(body: OkfQueryRequest): Promise<OkfQueriesCreateResponse201> {
    return this.client.post<OkfQueriesCreateResponse201>(appApiPath(`/knowledge/okf/queries`), body, undefined, undefined, 'application/json');
  }

/** File an answer for an OKF query */
  async okfQueriesFileAnswer(queryId: string, body: OkfFileAnswerRequest): Promise<OkfQueriesFileAnswerResponse> {
    return this.client.post<OkfQueriesFileAnswerResponse>(appApiPath(`/knowledge/okf/queries/${serializePathParameter(queryId, { name: 'queryId', style: 'simple', explode: false })}/file_answer`), body, undefined, undefined, 'application/json');
  }

/** Create an OKF context pack */
  async okfContextPacksCreate(body: OkfContextPackRequest): Promise<OkfContextPacksCreateResponse201> {
    return this.client.post<OkfContextPacksCreateResponse201>(appApiPath(`/knowledge/okf/context_packs`), body, undefined, undefined, 'application/json');
  }

/** List knowledge browser view */
  async spacesBrowserList(spaceId: string, params: KnowledgeSpacesBrowserListParams): Promise<SpacesBrowserListResponse> {
    const query = buildQueryString([
      { name: 'view', value: params.view, style: 'form', explode: true, allowReserved: false },
      { name: 'parentId', value: params.parentId, style: 'form', explode: true, allowReserved: false },
      { name: 'cursor', value: params.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SpacesBrowserListResponse>(appendQueryString(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/browser`), query));
  }

/** Create a knowledge retrieval */
  async retrievalsCreate(body: KnowledgeRetrievalRequest): Promise<RetrievalsCreateResponse201> {
    return this.client.post<RetrievalsCreateResponse201>(appApiPath(`/knowledge/retrievals`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge retrieval result */
  async retrievalsRetrieve(retrievalId: string): Promise<RetrievalsRetrieveResponse> {
    return this.client.get<RetrievalsRetrieveResponse>(appApiPath(`/knowledge/retrievals/${serializePathParameter(retrievalId, { name: 'retrievalId', style: 'simple', explode: false })}`));
  }

/** Create a knowledge context pack */
  async contextPacksCreate(body: KnowledgeContextPackRequest): Promise<ContextPacksCreateResponse201> {
    return this.client.post<ContextPacksCreateResponse201>(appApiPath(`/knowledge/context_packs`), body, undefined, undefined, 'application/json');
  }

/** Create a knowledge agent profile */
  async agentProfilesCreate(body: KnowledgeAgentProfileRequest): Promise<AgentProfilesCreateResponse201> {
    return this.client.post<AgentProfilesCreateResponse201>(appApiPath(`/knowledge/agent_profiles`), body, undefined, undefined, 'application/json');
  }

/** Retrieve a knowledge agent profile */
  async agentProfilesRetrieve(profileId: string): Promise<AgentProfilesRetrieveResponse> {
    return this.client.get<AgentProfilesRetrieveResponse>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`));
  }

/** Update a knowledge agent profile */
  async agentProfilesUpdate(profileId: string, body: KnowledgeAgentProfileRequest): Promise<AgentProfilesUpdateResponse> {
    return this.client.patch<AgentProfilesUpdateResponse>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete a knowledge agent profile */
  async agentProfilesDelete(profileId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}`));
  }

/** List agent profile bindings */
  async agentProfilesBindingsList(profileId: string): Promise<AgentProfilesBindingsListResponse> {
    return this.client.get<AgentProfilesBindingsListResponse>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/bindings`));
  }

/** Create an agent profile binding */
  async agentProfilesBindings(profileId: string, body: KnowledgeAgentBindingRequest): Promise<AgentProfilesBindingsResponse> {
    return this.client.post<AgentProfilesBindingsResponse>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/bindings`), body, undefined, undefined, 'application/json');
  }

/** Update an agent profile binding */
  async agentProfilesBindingsUpdate(profileId: string, bindingId: string, body: KnowledgeAgentBindingRequest): Promise<AgentProfilesBindingsUpdateResponse> {
    return this.client.patch<AgentProfilesBindingsUpdateResponse>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete an agent profile binding */
  async agentProfilesBindingsDelete(profileId: string, bindingId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`));
  }

/** Preview retrieval for an agent profile */
  async agentProfilesRetrievalPreview(profileId: string, body: KnowledgeRetrievalRequest): Promise<AgentProfilesRetrievalPreviewResponse> {
    return this.client.post<AgentProfilesRetrievalPreviewResponse>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/retrieval_preview`), body, undefined, undefined, 'application/json');
  }

/** Chat with a knowledge-backed agent profile */
  async agentProfilesChat(profileId: string, body: KnowledgeAgentChatRequest): Promise<AgentProfilesChatResponse> {
    return this.client.post<AgentProfilesChatResponse>(appApiPath(`/knowledge/agent_profiles/${serializePathParameter(profileId, { name: 'profileId', style: 'simple', explode: false })}/chat`), body, undefined, undefined, 'application/json');
  }

/** List knowledge space context bindings */
  async spacesContextBindingsList(spaceId: string, params?: KnowledgeSpacesContextBindingsListParams): Promise<SpacesContextBindingsListResponse> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SpacesContextBindingsListResponse>(appendQueryString(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/context_bindings`), query));
  }

/** Create a knowledge space context binding */
  async spacesContextBindings(spaceId: string, body: CreateKnowledgeSpaceContextBindingRequest): Promise<SpacesContextBindingsResponse> {
    return this.client.post<SpacesContextBindingsResponse>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/context_bindings`), body, undefined, undefined, 'application/json');
  }

/** List knowledge space members */
  async spacesMembersList(spaceId: string, params?: KnowledgeSpacesMembersListParams): Promise<SpacesMembersListResponse> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<SpacesMembersListResponse>(appendQueryString(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/members`), query));
  }

/** Grant knowledge space member access */
  async spacesMembers(spaceId: string, body: GrantKnowledgeSpaceMemberRequest): Promise<SpacesMembersResponse> {
    return this.client.post<SpacesMembersResponse>(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/members`), body, undefined, undefined, 'application/json');
  }

/** Revoke knowledge space member access */
  async spacesMembersDelete(spaceId: string, params: KnowledgeSpacesMembersDeleteParams): Promise<void> {
    const query = buildQueryString([
      { name: 'subjectType', value: params.subjectType, style: 'form', explode: true, allowReserved: false },
      { name: 'subjectId', value: params.subjectId, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.delete<void>(appendQueryString(appApiPath(`/knowledge/spaces/${serializePathParameter(spaceId, { name: 'spaceId', style: 'simple', explode: false })}/members`), query));
  }

/** Retrieve a knowledge space context binding */
  async contextBindingsRetrieve(bindingId: string): Promise<ContextBindingsRetrieveResponse> {
    return this.client.get<ContextBindingsRetrieveResponse>(appApiPath(`/knowledge/context_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`));
  }

/** Update a knowledge space context binding */
  async contextBindingsUpdate(bindingId: string, body: UpdateKnowledgeSpaceContextBindingRequest): Promise<ContextBindingsUpdateResponse> {
    return this.client.patch<ContextBindingsUpdateResponse>(appApiPath(`/knowledge/context_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`), body, undefined, undefined, 'application/json');
  }

/** Delete a knowledge space context binding */
  async contextBindingsDelete(bindingId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/context_bindings/${serializePathParameter(bindingId, { name: 'bindingId', style: 'simple', explode: false })}`));
  }

/** Upsert an OKF concept revision */
  async okfConceptsUpdate(body: OkfConceptUpsertRequest): Promise<OkfConceptsUpdateResponse> {
    return this.client.put<OkfConceptsUpdateResponse>(appApiPath(`/knowledge/okf/concepts/upsert`), body, undefined, undefined, 'application/json');
  }

/** Create an OKF bundle export */
  async okfBundleExportCreate(body: OkfBundleExportRequest): Promise<OkfBundleExportCreateResponse201> {
    return this.client.post<OkfBundleExportCreateResponse201>(appApiPath(`/knowledge/okf/exports`), body, undefined, undefined, 'application/json');
  }

/** Retrieve an OKF bundle export */
  async okfBundleExportRetrieve(exportId: string): Promise<OkfBundleExportRetrieveResponse> {
    return this.client.get<OkfBundleExportRetrieveResponse>(appApiPath(`/knowledge/okf/exports/${serializePathParameter(exportId, { name: 'exportId', style: 'simple', explode: false })}`));
  }

/** Import an OKF bundle from drive staging */
  async okfBundleImportCreate(body: OkfBundleImportRequest): Promise<OkfBundleImportCreateResponse201> {
    return this.client.post<OkfBundleImportCreateResponse201>(appApiPath(`/knowledge/okf/imports`), body, undefined, undefined, 'application/json');
  }

/** Create an OKF bundle lint run */
  async okfLintRunsCreate(body: OkfQualityRunRequest): Promise<OkfLintRunsCreateResponse201> {
    return this.client.post<OkfLintRunsCreateResponse201>(appApiPath(`/knowledge/okf/lint_runs`), body, undefined, undefined, 'application/json');
  }

/** List WeChat official accounts */
  async wechatOfficialAccountsList(): Promise<WechatOfficialAccountsListResponse> {
    return this.client.get<WechatOfficialAccountsListResponse>(appApiPath(`/knowledge/wechat/official_accounts`));
  }

/** Replace WeChat official accounts */
  async wechatOfficialAccountsUpdate(body: KnowledgeWechatReplaceOfficialAccountsRequest): Promise<WechatOfficialAccountsUpdateResponse> {
    return this.client.put<WechatOfficialAccountsUpdateResponse>(appApiPath(`/knowledge/wechat/official_accounts`), body, undefined, undefined, 'application/json');
  }

/** List WeChat official account fan tags */
  async wechatOfficialAccountsFanTagsList(accountId: string): Promise<WechatOfficialAccountsFanTagsListResponse> {
    return this.client.get<WechatOfficialAccountsFanTagsListResponse>(appApiPath(`/knowledge/wechat/official_accounts/${serializePathParameter(accountId, { name: 'accountId', style: 'simple', explode: false })}/fan_tags`));
  }

/** List WeChat applets */
  async wechatAppletsList(): Promise<WechatAppletsListResponse> {
    return this.client.get<WechatAppletsListResponse>(appApiPath(`/knowledge/wechat/applets`));
  }

/** Replace WeChat applets */
  async wechatAppletsUpdate(body: KnowledgeWechatReplaceAppletsRequest): Promise<WechatAppletsUpdateResponse> {
    return this.client.put<WechatAppletsUpdateResponse>(appApiPath(`/knowledge/wechat/applets`), body, undefined, undefined, 'application/json');
  }

/** Publish WeChat articles */
  async wechatArticlesPublish(body: KnowledgeWechatArticlesPublishRequest): Promise<WechatArticlesPublishResponse> {
    return this.client.post<WechatArticlesPublishResponse>(appApiPath(`/knowledge/wechat/articles/publish`), body, undefined, undefined, 'application/json');
  }

/** Preview WeChat articles */
  async wechatArticlesPreview(body: KnowledgeWechatArticlesPreviewRequest): Promise<WechatArticlesPreviewResponse> {
    return this.client.post<WechatArticlesPreviewResponse>(appApiPath(`/knowledge/wechat/articles/preview`), body, undefined, undefined, 'application/json');
  }

/** Sync knowledge space documents to a Git repository */
  async gitSyncsCreate(body: KnowledgeGitSyncRequest): Promise<GitSyncsCreateResponse201> {
    return this.client.post<GitSyncsCreateResponse201>(appApiPath(`/knowledge/git_syncs`), body, undefined, undefined, 'application/json');
  }

/** List knowledge market catalog listings */
  async marketListingsList(params?: KnowledgeMarketListingsListParams): Promise<MarketListingsListResponse> {
    const query = buildQueryString([
      { name: 'cursor', value: params?.cursor, style: 'form', explode: true, allowReserved: false },
      { name: 'page_size', value: params?.pageSize, style: 'form', explode: true, allowReserved: false },
    ]);
    return this.client.get<MarketListingsListResponse>(appendQueryString(appApiPath(`/knowledge/market/listings`), query));
  }

/** Subscribe to a knowledge market listing */
  async marketSubscriptionsCreate(body: KnowledgeMarketSubscriptionRequest): Promise<MarketSubscriptionsCreateResponse201> {
    return this.client.post<MarketSubscriptionsCreateResponse201>(appApiPath(`/knowledge/market/subscriptions`), body, undefined, undefined, 'application/json');
  }

/** Unsubscribe from a knowledge market listing */
  async marketSubscriptionsDelete(listingId: string): Promise<void> {
    return this.client.delete<void>(appApiPath(`/knowledge/market/subscriptions/${serializePathParameter(listingId, { name: 'listingId', style: 'simple', explode: false })}`));
  }

/** Create a knowledge media task (image generation or speech-to-text) */
  async mediaTasksCreate(body: KnowledgeMediaTaskRequest): Promise<MediaTasksCreateResponse201> {
    return this.client.post<MediaTasksCreateResponse201>(appApiPath(`/knowledge/media_tasks`), body, undefined, undefined, 'application/json');
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
