# SDKWork Knowledgebase API Changelog

## App API (`/app/v3/api`)

### 0.1.0 (Phase 0 MVP)

Initial API surface for the SDKWork Knowledgebase App API.

**Spaces**
- `POST /app/v3/api/knowledge/spaces` — `spaces.create`
- `GET /app/v3/api/knowledge/spaces/{spaceId}` — `spaces.retrieve`
- `PATCH /app/v3/api/knowledge/spaces/{spaceId}` — `spaces.update`
- `DELETE /app/v3/api/knowledge/spaces/{spaceId}` — `spaces.delete`

**Space Members**
- `GET /app/v3/api/knowledge/spaces/{spaceId}/members` — `spaces.members.list`
- `POST /app/v3/api/knowledge/spaces/{spaceId}/members` — `spaces.members.members`
- `DELETE /app/v3/api/knowledge/spaces/{spaceId}/members` — `spaces.members.delete`

**Documents**
- `GET /app/v3/api/knowledge/documents?spaceId={spaceId}` — `documents.list`
- `POST /app/v3/api/knowledge/documents` — `documents.create`
- `GET /app/v3/api/knowledge/documents/{documentId}` — `documents.retrieve`
- `PATCH /app/v3/api/knowledge/documents/{documentId}` — `documents.update`
- `DELETE /app/v3/api/knowledge/documents/{documentId}` — `documents.delete`
- `GET /app/v3/api/knowledge/documents/{documentId}/content` — `documents.content.list`
- `GET /app/v3/api/knowledge/documents/{documentId}/versions` — `documents.versions.list`
- `POST /app/v3/api/knowledge/documents/{documentId}/versions` — `documents.versions.versions`

**Ingestion**
- `POST /app/v3/api/knowledge/ingests` — `ingests.create`
- `GET /app/v3/api/knowledge/ingests/{ingestId}` — `ingests.retrieve`
- `POST /app/v3/api/knowledge/drive_imports` — `driveImports.create`
- `POST /app/v3/api/knowledge/git_imports` — `gitImports.create`
- `POST /app/v3/api/knowledge/git_syncs` — `gitSyncs.create`

**OKF (Open Knowledge Format)**
- `GET /app/v3/api/knowledge/okf/concepts?spaceId={spaceId}` — `okf.concepts.list`
- `PUT /app/v3/api/knowledge/okf/concepts/upsert` — `okf.concepts.update`
- `GET /app/v3/api/knowledge/okf/concepts/{conceptId}` — `okf.concepts.retrieve`
- `DELETE /app/v3/api/knowledge/okf/concepts/{conceptId}` — `okf.concepts.delete`
- `GET /app/v3/api/knowledge/okf/concepts/{conceptId}/revisions` — `okf.concepts.revisions.list`
- `GET /app/v3/api/knowledge/okf/index?spaceId={spaceId}` — `okf.bundle.index.list`
- `GET /app/v3/api/knowledge/okf/log?spaceId={spaceId}` — `okf.bundle.log.list`
- `GET /app/v3/api/knowledge/okf/profile?spaceId={spaceId}` — `okf.bundle.profile.list`
- `POST /app/v3/api/knowledge/okf/queries` — `okf.queries.create`
- `POST /app/v3/api/knowledge/okf/queries/{queryId}/file_answer` — `okf.queries.fileAnswer`
- `POST /app/v3/api/knowledge/okf/context_packs` — `okf.contextPacks.create`
- `POST /app/v3/api/knowledge/okf/exports` — `okf.bundle.export.create`
- `GET /app/v3/api/knowledge/okf/exports/{exportId}` — `okf.bundle.export.retrieve`
- `POST /app/v3/api/knowledge/okf/imports` — `okf.bundle.import.create`
- `POST /app/v3/api/knowledge/okf/lint_runs` — `okf.lintRuns.create`

**Browser / Navigation**
- `GET /app/v3/api/knowledge/spaces/{spaceId}/browser?view=files` — `spaces.browser.list`

**Retrieval & RAG**
- `POST /app/v3/api/knowledge/retrievals` — `retrievals.create`
- `GET /app/v3/api/knowledge/retrievals/{retrievalId}` — `retrievals.retrieve`
- `POST /app/v3/api/knowledge/context_packs` — `contextPacks.create`

**Agent Profiles**
- `POST /app/v3/api/knowledge/agent_profiles` — `agentProfiles.create`
- `GET /app/v3/api/knowledge/agent_profiles/{profileId}` — `agentProfiles.retrieve`
- `PATCH /app/v3/api/knowledge/agent_profiles/{profileId}` — `agentProfiles.update`
- `DELETE /app/v3/api/knowledge/agent_profiles/{profileId}` — `agentProfiles.delete`
- `GET /app/v3/api/knowledge/agent_profiles/{profileId}/bindings` — `agentProfiles.bindings.list`
- `POST /app/v3/api/knowledge/agent_profiles/{profileId}/bindings` — `agentProfiles.bindings.bindings`
- `PATCH /app/v3/api/knowledge/agent_profiles/{profileId}/bindings/{bindingId}` — `agentProfiles.bindings.update`
- `DELETE /app/v3/api/knowledge/agent_profiles/{profileId}/bindings/{bindingId}` — `agentProfiles.bindings.delete`
- `POST /app/v3/api/knowledge/agent_profiles/{profileId}/retrieval_preview` — `agentProfiles.retrievalPreview.retrievalPreview`
- `POST /app/v3/api/knowledge/agent_profiles/{profileId}/chat` — `agentProfiles.chat.chat`

**Context Bindings**
- `GET /app/v3/api/knowledge/spaces/{spaceId}/context_bindings` — `spaces.contextBindings.list`
- `POST /app/v3/api/knowledge/spaces/{spaceId}/context_bindings` — `spaces.contextBindings.contextBindings`
- `GET /app/v3/api/knowledge/context_bindings/{bindingId}` — `contextBindings.retrieve`
- `PATCH /app/v3/api/knowledge/context_bindings/{bindingId}` — `contextBindings.update`
- `DELETE /app/v3/api/knowledge/context_bindings/{bindingId}` — `contextBindings.delete`

**Upload Sessions**
- `POST /app/v3/api/knowledge/upload_sessions` — `uploadSessions.create`
- `POST /app/v3/api/knowledge/upload_sessions/{sessionId}/complete` — `uploadSessions.complete`

**WeChat Integration**
- `GET /app/v3/api/knowledge/wechat/official_accounts` — `wechat.officialAccounts.list`
- `PUT /app/v3/api/knowledge/wechat/official_accounts` — `wechat.officialAccounts.update`
- `GET /app/v3/api/knowledge/wechat/applets` — `wechat.applets.list`
- `PUT /app/v3/api/knowledge/wechat/applets` — `wechat.applets.update`
- `POST /app/v3/api/knowledge/wechat/articles/publish` — `wechat.articles.publish`
- `POST /app/v3/api/knowledge/wechat/articles/preview` — `wechat.articles.preview`

**Market / Commerce**
- `GET /app/v3/api/knowledge/market/listings` — `market.listings.list`
- `POST /app/v3/api/knowledge/market/subscriptions` — `market.subscriptions.create`
- `DELETE /app/v3/api/knowledge/market/subscriptions/{listingId}` — `market.subscriptions.delete`

**Site Deployment**
- `POST /app/v3/api/knowledge/site_deployments` — `siteDeployments.create`
- `GET /app/v3/api/knowledge/site_deployments/{deploymentId}/preview` — `siteDeployments.preview.list`

**Media Tasks**
- `POST /app/v3/api/knowledge/media_tasks` — `mediaTasks.create`

## Backend API (`/backend/v3/api`)

### 0.1.0 (Phase 0 MVP)

Initial backend admin API surface.

### 0.2.0 (Phase 3 — Tenant Status)

Tenant status endpoint for multi-tenant architecture.

**Tenant Status**
- `GET /backend/v3/api/knowledge/tenants/current` — `tenants.current.list`
  - Retrieves the caller's own tenant knowledgebase status.
  - **Security**: Tenant identity is derived from the authenticated principal's
    access token claims (`WebRequestPrincipal.tenant_id()`). No `tenant_id`
    is accepted in the request body or path parameter.
  - Response: `{ "tenant_name": string?, "status": "ACTIVE"|"SUSPENDED"|"ARCHIVED", "space_count": u64, "document_count": u64, "created_at": string? }`

**Note**: Tenant creation and management is handled by the IAM layer.
Knowledgebase only reports tenant-level statistics derived from the authenticated
principal's token claims.

## Open API (`/knowledge/v3/api`)

### 0.1.0 (Phase 0 MVP)

Initial public Open API surface.
