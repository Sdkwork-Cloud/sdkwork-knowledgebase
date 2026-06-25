# SDKWork Knowledgebase Backend Design

Date: 2026-06-01
Status: draft for review (superseded in part by OKF bundle design)
Owner: sdkwork-knowledgebase

> **OKF supersession (2026-06-19):** LLM Wiki (`wiki/*`, `kb_wiki_*`, `wiki.*` APIs) is removed from the implementation. Authoritative knowledge-bundle behavior is defined in [`2026-06-19-okf-knowledge-bundle-design.md`](2026-06-19-okf-knowledge-bundle-design.md). Sections below that still describe wiki tables, routes, or Drive paths are historical context only.

## 1. Purpose

Build a Rust backend knowledgebase application under the `sdkwork-knowledgebase` repository root.
The application must provide reusable knowledgebase services that can run as standalone app/backend APIs and can also be imported as dependency components by other SDKWork applications.

The first scope is backend only. Frontend UI and apps integration are explicitly out of scope for this design round.

## 2. Hard Decisions

### 2.1 Storage Is Drive-First

`sdkwork-drive` is the only approved lower-level file storage capability.

The knowledgebase service must not implement its own file system, S3, OSS, MinIO, Azure Blob, GCS, or local filesystem business storage path. Original documents, parsed artifacts, OCR outputs, page images, thumbnails, export archives, parse manifests, LLM Wiki Markdown files, schema files, `index.md`, `log.md`, local mirror packages, delta packages, and runtime bundles must be written through `sdkwork-drive`.

The knowledgebase service owns semantic metadata and indexes only:

- knowledge spaces
- collections and document metadata
- document versions
- stable drive object references
- parse and indexing state
- chunks
- embeddings and vector index metadata
- retrieval profiles
- retrieval traces and citations
- ACL, audit, retention, and outbox records

The knowledgebase service must store stable drive object references, not file bytes and not presigned URLs.

### 2.2 Recommended Architecture

Use an adapter-first Rust component architecture:

- SQL is the system of record for metadata, permissions, tasks, audit, and retrieval traces.
- `sdkwork-drive` is the system of record for file/object bytes.
- Vector, full-text, parser, OCR, embedding, and rerank capabilities are ports with replaceable adapters.
- PostgreSQL plus pgvector is the default production SQL/vector adapter.
- SQLite is the default local/test metadata adapter. SQLite vector support can use an adapter-specific implementation or external vector references.

This keeps the first version deployable while avoiding a permanent dependency on one vector/search engine.

## 3. Standards Alignment

This design follows the local SDKWork standards:

- API contracts: `../../../../sdkwork-specs/API_SPEC.md`
- SDK generation: `../../../../sdkwork-specs/SDK_SPEC.md`
- database contracts: `../../../../sdkwork-specs/DATABASE_SPEC.md`
- reusable modules: `../../../../sdkwork-specs/MODULE_SPEC.md`
- security: `../../../../sdkwork-specs/SECURITY_SPEC.md`
- performance: `../../../../sdkwork-specs/PERFORMANCE_SPEC.md`
- observability: `../../../../sdkwork-specs/OBSERVABILITY_SPEC.md`
- media/object references: `../../../../sdkwork-specs/MEDIA_RESOURCE_SPEC.md`

Relevant local reference projects:

- `../../../../sdkwork-claw-router`
- `../../../../sdkwork-drive`
- `..\sdkwork-sdk-generator`

Industry reference points used for product and architecture alignment:

- Microsoft Azure AI Search RAG overview: https://learn.microsoft.com/en-us/azure/search/retrieval-augmented-generation-overview
- Microsoft Azure AI Search vector search: https://learn.microsoft.com/en-us/azure/search/vector-search-overview
- Microsoft Graph connectors external items: https://learn.microsoft.com/en-us/graph/connecting-external-content-manage-items
- Google Vertex AI RAG Engine: https://cloud.google.com/vertex-ai/generative-ai/docs/rag-engine/rag-overview
- OpenAI File Search and vector stores: https://platform.openai.com/docs/assistants/tools/file-search
- llms.txt proposal: https://llmstxt.org/
- LLM-Wiki retrieval-as-reasoning paper: https://arxiv.org/abs/2605.25480
- Open LLM wiki compiler reference: https://github.com/atomicstrata/llm-wiki-compiler
- Local LLM Wiki pattern specification: `../../llm-wiki.md`

## 4. Non-Goals

- No frontend implementation in this phase.
- No apps package implementation in this phase.
- No handwritten app/backend SDK clients.
- No direct object storage implementation in the knowledgebase service.
- No file bytes in knowledgebase SQL tables.
- No presigned URL as persisted business state.
- No fake parsing, fake indexing, fake retrieval, or fake successful job completion.
- No generated wiki page is considered authoritative unless it has source lineage, revision history, and citation/provenance state.

## 5. Domain Model

Primary app-local domain: `knowledge`.

Domain record:

```yaml
domain: knowledge
status: app-local-extension
owner: sdkwork-knowledgebase
database_prefix: kb
api_tags:
  - knowledge
sdk_namespaces:
  - knowledge
capabilities:
  - spaces
  - collections
  - sources
  - documents
  - document_versions
  - drive_object_refs
  - ingestion_jobs
  - parse_artifacts
  - chunks
  - embeddings
  - retrieval_profiles
  - retrievals
  - citations
  - wiki_pages
  - wiki_page_revisions
  - wiki_links
  - wiki_claims
  - wiki_file_entries
  - wiki_schema_profiles
  - wiki_log_entries
  - wiki_queries
  - llms_txt_exports
  - local_mirror_packages
  - local_mirror_deltas
  - local_runtime_bundles
  - acl
  - audit_events
depends_on:
  - iam
  - content
  - intelligence
  - sdkwork-drive
rust_parity: required
sdk_generation: required
```

## 6. Rust Workspace Structure

```text
sdkwork-knowledgebase/
  Cargo.toml
  crates/
    sdkwork-knowledgebase-contract/
    sdkwork-knowledgebase-config/
    sdkwork-knowledgebase-core/
    sdkwork-knowledgebase-security/
    sdkwork-knowledgebase-http/
    sdkwork-knowledgebase-observability/
    sdkwork-knowledgebase-drive/
    sdkwork-knowledgebase-index-contract/
    sdkwork-knowledgebase-storage-sqlx/
    sdkwork-knowledgebase-index-pgvector/
    sdkwork-knowledgebase-test-support/
  services/
    sdkwork-knowledgebase-product/
    sdkwork-knowledgebase-app-api/
    sdkwork-knowledgebase-admin-api/
    sdkwork-knowledgebase-worker/
    sdkwork-knowledgebase-installer/
  docs/
    schema-registry/
    superpowers/specs/
    superpowers/plans/
  sdks/
    sdkwork-knowledgebase-app-sdk/
      openapi/
        knowledgebase-app-api.openapi.json
      sdkwork-knowledgebase-app-sdk-typescript/
        generated/server-openapi/
          sdkwork-sdk.json
          .sdkwork/
          custom/
        composed/
    sdkwork-knowledgebase-backend-sdk/
      openapi/
        knowledgebase-backend-api.openapi.json
      sdkwork-knowledgebase-backend-sdk-typescript/
        generated/server-openapi/
          sdkwork-sdk.json
          .sdkwork/
          custom/
        composed/
  tests/
  tools/
```

### 6.1 Crate Responsibilities

`sdkwork-knowledgebase-contract`

- Public DTOs, enum contracts, API schema structs, operation constants, and OpenAPI helper metadata.
- No SQL, no HTTP runtime, no drive provider implementation.

`sdkwork-knowledgebase-config`

- Typed runtime config.
- Deployment mode, database config, queue config, parser limits, retrieval limits, and drive object policy config.
- Shared modules do not read environment variables directly.

`sdkwork-knowledgebase-core`

- Domain primitives, IDs, state machines, domain errors, pagination, timestamps, and lifecycle helpers.
- No SQLx, no Axum, no drive implementation.

`sdkwork-knowledgebase-security`

- Subject context, tenant context, ACL evaluator, permission codes, security trimming decisions.
- Business services consume this instead of parsing tokens or headers.

`sdkwork-knowledgebase-http`

- Axum route composition, problem-details mapping, auth extraction, request limits, response envelopes, health routes.
- App and backend services mount route modules from here.

`sdkwork-knowledgebase-observability`

- Tracing setup, safe log fields, metrics names, redaction helpers.

`sdkwork-knowledgebase-drive`

- The only knowledgebase crate allowed to depend on `sdkwork-drive-storage-contract`.
- Adapts `DriveObjectStore` into knowledgebase storage ports.
- Owns drive object reference mapping and artifact read/write helpers.

`sdkwork-knowledgebase-index-contract`

- Ports for chunking, embedding, vector index, full-text index, rerank, and retrieval.
- Adapters can be pgvector, external vector DB, Tantivy, OpenSearch, or provider-specific RAG.

`sdkwork-knowledgebase-storage-sqlx`

- SQLx repositories for PostgreSQL and SQLite.
- Migrations, row mappers, query builders, and schema readiness checks.

`sdkwork-knowledgebase-index-pgvector`

- PostgreSQL pgvector implementation of the vector index port.
- This is an adapter, not a domain dependency.

`sdkwork-knowledgebase-test-support`

- Fake drive store, fake embedding service, fake parser, fake vector index, SQL fixtures, API test helpers.

### 6.2 Service Responsibilities

`sdkwork-knowledgebase-product`

- Application services and domain orchestration.
- Folders: `api`, `application`, `domain`, `ports`, `infrastructure`, `bootstrap`.
- Does not expose a network listener by itself.

`sdkwork-knowledgebase-app-api`

- `/app/v3/api/knowledge/**`
- User-facing document, upload, import, indexing status, retrieval, and citation APIs.

`sdkwork-knowledgebase-admin-api`

- `/backend/v3/api/knowledge/**`
- Operator/admin APIs for spaces, policies, retrieval profiles, index jobs, storage reconciliation, audit, and diagnostics.

`sdkwork-knowledgebase-worker`

- Async ingestion, parsing, OCR, chunking, embedding, indexing, reindexing, cleanup, and reconciliation.

`sdkwork-knowledgebase-installer`

- Schema install, local bootstrap, seed policies, readiness checks.

## 7. Drive Integration Design

### 7.1 Drive Contract Dependency

The current `sdkwork-drive` storage contract exposes:

- `DriveObjectStore`
- `DriveObjectLocator`
- `PutObjectRequest`
- `HeadObjectRequest`
- `ReadObjectRangeRequest`
- multipart upload requests
- presigned upload/download requests
- `DriveObjectStoreError`
- `DriveStorageProviderKind`
- `DriveStorageProviderCapabilities`

The knowledgebase service must use these concepts through a narrow port:

```rust
pub trait KnowledgeDriveStorage: Send + Sync {
    async fn put_artifact(&self, request: PutKnowledgeArtifactRequest)
        -> Result<KnowledgeDriveObjectSnapshot, KnowledgeStorageError>;

    async fn head_object(&self, reference: &KnowledgeDriveObjectRef)
        -> Result<KnowledgeDriveObjectSnapshot, KnowledgeStorageError>;

    async fn read_range(&self, request: ReadKnowledgeObjectRangeRequest)
        -> Result<KnowledgeObjectRangeStream, KnowledgeStorageError>;

    async fn presign_download(&self, request: PresignKnowledgeDownloadRequest)
        -> Result<KnowledgeDownloadGrant, KnowledgeStorageError>;
}
```

Only `sdkwork-knowledgebase-drive` implements this port by calling `DriveObjectStore`.

Drive workspace, space, node, and object-version metadata are owned by `sdkwork-drive`.
Knowledgebase production code must not read from, write to, join against, or otherwise depend on drive physical tables such as `dr_drive_space`, `dr_drive_node`, or `dr_drive_storage_object`.
When knowledgebase needs a browser-visible folder tree, file node binding, or knowledge drive space, `sdkwork-drive` must expose that operation through product services or generated SDK APIs, and `sdkwork-knowledgebase-drive` adapts those APIs into knowledgebase ports.
Knowledgebase SQL may store stable references such as `drive_space_id`, `drive_node_id`, bucket, object key, etag, checksum, and object role inside `kb_*` tables, but those values are locators and projections, not permission to couple to drive schemas.

Stable logical artifacts such as `wiki/index.md`, `wiki/log.md`, and `wiki/**/current.md` can be rewritten as materialized projections. If the underlying Drive provider does not return a native object version, or returns a blank object version, `sdkwork-knowledgebase-drive` must synthesize the object version as `sha256:<content_digest>` before persisting a `kb_drive_object_ref` locator. If a write request supplies a checksum, the Drive adapter must compute the request-body SHA-256 and reject the request before any Drive write when the supplied checksum does not match the body. When a browser-visible Drive workspace file is ensured again with the same logical path but different content metadata, the version advancement must happen inside the `sdkwork-drive` product service or generated SDK API; Knowledgebase must not patch Drive tables directly.

Knowledge space creation is a compensating workflow:

1. Create the local `kb_space` record in an uninitialized active state.
2. Provision or find the dedicated Drive `knowledge_base` space through `sdkwork-drive` product services.
3. Bind `kb_space.drive_space_id`.
4. Persist standard LLM Wiki files and ensure browser-visible Drive workspace nodes through Drive-facing ports.
5. Mark `kb_space.llm_wiki_initialized = true`.

If any step after local space creation fails, the local `kb_space` row must be soft-deleted before the request returns. If a Drive space was created or found for this initialization attempt and a later step fails, Knowledgebase must release it through the Drive product service or generated SDK API after verifying it belongs to the expected `sdkwork-knowledgebase:<kb-space-uuid>` owner tuple. Cleanup failures must be reported explicitly; they must not be hidden behind a successful create response.

### 7.2 Object Roles

Every drive object reference has an explicit role:

```text
original_document
normalized_text
ocr_result
page_image
thumbnail
source_attachment
parse_manifest
chunk_manifest
export_archive
citation_snapshot
wiki_schema
wiki_index
wiki_log
wiki_page_markdown
wiki_revision_markdown
wiki_candidate_markdown
wiki_graph_jsonl
query_answer_output
context_pack
llms_txt
llms_full_txt
wiki_eval_report
local_mirror_snapshot
local_mirror_delta
local_runtime_bundle
```

Rules:

- `original_document` is immutable after document version creation.
- Parser outputs must be written as new drive objects.
- Parsed text may be stored in SQL chunks for retrieval, but the canonical full parse artifact remains in drive.
- Download URLs are short-lived grants and are never stored as system-of-record values.

### 7.3 Upload and Import Flows

Upload flow:

```text
client
  -> POST /app/v3/api/knowledge/upload_sessions
  -> KnowledgeUploadSessionService
  -> KnowledgeDriveStorage
  -> sdkwork-drive
  -> document draft or import token
```

Drive import flow:

```text
existing drive object locator
  -> POST /app/v3/api/knowledge/drive_imports
  -> head_object through sdkwork-drive
  -> create kb_drive_object_ref
  -> create kb_document
  -> create kb_document_version
  -> enqueue kb_ingestion_job
```

### 7.4 Forbidden Storage Bypasses

Knowledgebase production code must not:

- call `std::fs` for business file storage
- use cloud object storage SDKs directly
- construct presigned URLs directly
- store object bytes in SQL
- store signed URLs in SQL
- scatter drive object keys in arbitrary metadata JSON
- parse raw drive credentials
- log drive credentials or presign material
- issue SQL directly against `sdkwork-drive` physical tables
- infer drive table names or schema columns in knowledgebase tests or production adapters

## 8. LLM Wiki and LLM-Friendly Knowledge Standard

The knowledgebase must support two complementary layers:

1. Retrieval layer: source documents are parsed, chunked, embedded, and searched.
2. LLM wiki layer: important knowledge is compiled into stable wiki pages with links, revisions, claims, provenance, and export formats.

The LLM wiki layer is not a frontend feature. It is a backend knowledge representation layer that can serve RAG, agent context, documentation export, and human review workflows.

### 8.0 LLM Wiki Compatibility Profile

The local specification `docs/llm-wiki.md` is treated as the product compatibility profile for this application.
It describes a persistent, compounding Markdown wiki maintained by LLM agents on top of immutable raw sources.

The knowledgebase must be compatible with that profile at the file, workflow, and local-runtime levels.
The backend may keep additional SQL, graph, index, audit, and drive-object metadata, but it must be able to project a space into a plain LLM Wiki layout that an agent can understand without reading service internals.

Required LLM Wiki semantics:

- Raw sources are immutable and are the source of truth.
- The wiki is a persistent set of interlinked Markdown pages generated and maintained by LLM workflows.
- A schema/instruction layer tells the LLM how the wiki is structured, what page conventions apply, and how ingest/query/lint workflows must run.
- Ingest reads a new source, writes or updates a source summary, updates related entity/concept/topic pages, updates the content index, and appends a chronological log entry.
- Query reads the wiki first, uses source-backed citations, and can file valuable answers back into the wiki as new pages or page revisions.
- Lint detects contradictions, stale claims, orphan pages, missing pages for important concepts, missing cross-references, broken links, and knowledge gaps.
- `index.md` is the content-oriented wiki catalog.
- `log.md` is the append-only chronological activity record.
- YAML frontmatter, `[[wikilinks]]`, local image attachments, and Obsidian-style browsing are first-class compatibility targets.
- Git compatibility is provided by local mirror/export projection. `sdkwork-drive` remains the authoritative lower-level storage for service-managed bytes.

Compatibility verdict for this design:

- The architecture is compatible after applying the explicit rules in this section and sections 8.5, 8.6, 8.9, and 8.10.
- A design or implementation that only exposes retrieval chunks, vector search, JSON logs, or `_index.md` helper files is not fully LLM Wiki compatible.
- Full compatibility requires the drive-backed files `wiki/schema/AGENTS.md`, `wiki/index.md`, `wiki/log.md`, immutable raw source projections, and local mirror packages that expose root `AGENTS.md`, `schema/**`, `raw/**`, and `wiki/**` for offline use.

### 8.1 llms.txt Compatibility

The `llms.txt` proposal defines an LLM-friendly Markdown index for sites and docs. This system should support exporting a knowledge space into:

```text
/llms.txt
/llms-full.txt
```

The export is generated from curated wiki pages, not directly from raw chunks.

Rules:

- Export content is Markdown.
- Exported links must point to stable knowledge URLs or stable document/citation references.
- Generated files are stored in `sdkwork-drive` as artifacts with object roles `llms_txt` and `llms_full_txt`.
- SQL stores export metadata and object references only.
- Export jobs are asynchronous and auditable.

### 8.2 LLM Wiki Layer

The LLM wiki layer compiles source material into structured, revisioned pages.

Core concepts:

- wiki page: stable topic page in a knowledge space.
- wiki revision: immutable page version.
- wiki link: explicit relationship between pages.
- wiki claim: source-grounded atomic statement.
- wiki candidate: draft page or revision produced by ingest/compile and awaiting review.
- wiki evaluation: quality checks such as unsupported claim, duplicate page, stale source, broken link, weak citation, and policy violation.

The canonical Markdown body of a wiki revision is stored in `sdkwork-drive`.
SQL stores metadata, revision graph, claim/citation records, search projection, and review state.

### 8.3 LLM Wiki Quality Rules

Wiki content must be source-grounded.

Rules:

- Every generated wiki page revision must have source lineage.
- Important claims should link to citations.
- A page may be machine-generated, human-edited, or imported, but provenance must be explicit.
- The system must distinguish draft, review, published, stale, archived, and deleted pages.
- Generated pages must not silently overwrite human-approved content.
- Recompilation creates a new revision or candidate, not an in-place mutation.
- Links are first-class records so graph traversal and broken-link checks are possible.
- Export artifacts must be reproducible from page revisions and export profile.

### 8.4 Knowledge Operations

The backend must support these common knowledgebase operations:

```text
ingest
import
sync
parse
chunk
embed
index
retrieve
query
file_answer
compile
review
publish
reindex
delete
restore
export
lint
evaluate
reconcile
package
mirror
delta_update
```

Operation meanings:

`ingest`

- Accept uploaded file, drive object, drive folder, URL, API payload, or connector source.
- Create source/document/version records.
- Create a job and process idempotently.
- For LLM Wiki ingest, compile the source into a source summary, update affected entity/concept/topic pages, update `wiki/index.md`, and append `wiki/log.md`.
- Support supervised one-source-at-a-time ingest and batch ingest. Both modes use the same idempotent job model.

`compile`

- Convert parsed source material and retrieval results into wiki page candidates.
- Build structured pages, links, claims, glossary terms, and citations.

`review`

- Approve, reject, request changes, or publish machine-created candidates.
- Preserve review actor, result, and trace.

`query`

- Read `wiki/index.md` or the SQL-backed index projection first, then retrieve relevant wiki pages, chunks, claims, and citations under ACL.
- Synthesize an answer with citations and trace metadata.
- Support answer forms such as Markdown page, comparison table, report, Marp-compatible deck source, chart data, or context pack.

`file_answer`

- Convert a useful query answer into a wiki candidate or revision so exploration compounds inside the knowledgebase.
- Preserve the original query trace, cited pages, cited chunks, generated answer artifact, reviewer decision, and resulting page revision.

`context`

- Build bounded LLM context packs from selected pages, chunks, claims, and sources.
- Store generated context packs in drive when persisted.

`lint`

- Check broken links, missing citations, stale source references, duplicate pages, unsupported claims, and forbidden content.

`evaluate`

- Run retrieval and wiki quality evaluation datasets.
- Produce auditable quality reports.

`export`

- Generate `llms.txt`, `llms-full.txt`, Markdown bundles, JSONL context packs, or other supported artifacts through drive.

`reconcile`

- Verify drive objects, parse artifacts, indexes, and SQL metadata still agree.

`package`

- Build a complete local runnable mirror package for one knowledge space or selected collections.
- Include wiki files, source manifests, parsed artifacts selected by policy, graph/read models, SQLite metadata, local vector/full-text indexes, and runtime config.

`mirror`

- Create or refresh a local mirror of a remote knowledge space.
- The local mirror is read-only by default, with optional local-only overlays as a separate extension.

`delta_update`

- Apply incremental update packages to an existing local mirror.
- Validate base version, checksums, tombstones, schema version, and index compatibility before mutating local state.

### 8.5 LLM Wiki Drive Directory Layout

LLM Wiki files are persisted through `sdkwork-drive`. The structure below is a logical object-key layout inside a drive bucket. It may be backed by local filesystem, S3-compatible storage, or any future drive provider, but knowledgebase code only sees `DriveObjectStore`.

Default bucket:

```text
{knowledgebase_drive_bucket}
```

Default object-key root:

```text
knowledge/{tenant_id}/{space_uuid}/
```

Full logical layout:

```text
knowledge/{tenant_id}/{space_uuid}/
  manifest/
    space.yaml
    registry.json
    checksums.json
    acl_snapshot.json
    _index.md
    compatibility.md

  inbox/
    uploads/
      {upload_session_uuid}/
        original/{safe_file_name}
        upload_manifest.json
    drive-imports/
      {import_uuid}/
        import_manifest.json
    api/
      {ingest_uuid}/
        payload.json
        payload.md

  sources/
    raw/
      README.md
      {source_uuid}/
        original/{safe_file_name}
        source.yaml
        checksum.sha256
        extracted_metadata.json
        assets/
          {safe_asset_name}
    urls/
      {source_uuid}/
        fetched.html
        readable.md
        source.yaml
        headers.json
    repos/
      {source_uuid}/
        repo_manifest.json
        tree_index.json
        files/
    message_archives/
      {source_uuid}/
        archive_manifest.json
        normalized.jsonl
    media/
      {source_uuid}/
        media_manifest.json
        originals/
        thumbnails/

  parsed/
    {document_uuid}/
      versions/
        v{version_no}/
          normalized_text.md
          normalized_text.json
          parse_manifest.json
          ocr/
            ocr_result.json
            pages/
              page-{page_no}.md
          page_images/
            page-{page_no}.png
          tables/
            table-{table_no}.csv
            table-{table_no}.json
          figures/
            figure-{figure_no}.json
          chunks/
            chunks.jsonl
            chunk_manifest.json

  wiki/
    _index.md
    index.md
    log.md
    schema/
      AGENTS.md
      CLAUDE.md
      wiki_schema.yaml
      ingest_workflow.md
      query_workflow.md
      lint_workflow.md
      page_conventions.md
    pages/
      sources/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      entities/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      concepts/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      topics/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      references/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      how_to/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      faq/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      glossary/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      answers/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      comparisons/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      presentations/
        {page_slug}/
          current.md
          page.yaml
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml
      charts/
        {page_slug}/
          current.md
          page.yaml
          data/
          revisions/
            r{revision_no}.md
            r{revision_no}.yaml

  graph/
    links.jsonl
    backlinks.jsonl
    entities.jsonl
    claims.jsonl
    claim_citations.jsonl
    source_lineage.jsonl
    graph_manifest.json

  candidates/
    compile_jobs/
      {compile_job_uuid}/
        job_manifest.json
        candidates/
          {candidate_uuid}.md
          {candidate_uuid}.yaml
        diff/
          {page_slug}.diff.md
        review_report.json

  indexes/
    full_text/
      index_manifest.json
      documents.jsonl
    vector/
      index_manifest.json
      embeddings_manifest.json
    retrieval/
      retrieval_profiles.json
      routing_manifest.json
    wiki/
      page_index.jsonl
      topic_index.jsonl
      stale_index.jsonl

  datasets/
    _index.md
    {dataset_slug}/
      dataset.yaml
      schema.json
      sample.jsonl
      profile.md
      query_recipes.md
      external_locations.json

  inventory/
    _index.md
    items.jsonl
    source_candidates.jsonl
    corpora.jsonl
    entities.jsonl
    open_questions.jsonl
    next_actions.jsonl
    watch_items.jsonl

  context_packs/
    {context_pack_uuid}/
      context.md
      context.json
      citations.jsonl
      manifest.json

  eval/
    retrieval/
      {eval_run_uuid}/
        dataset.jsonl
        results.jsonl
        report.md
    wiki/
      {eval_run_uuid}/
        issues.jsonl
        scores.json
        report.md
    audit/
      {audit_run_uuid}/
        evidence_chain.jsonl
        drift_report.md
        truth_audit.md

  output/
    answers/
      {output_uuid}/
        answer.md
        answer.json
        citations.jsonl
        manifest.json
    reports/
      {output_uuid}/
        report.md
        manifest.json
    decks/
      {output_uuid}/
        deck.md
        assets/
        manifest.json
    charts/
      {output_uuid}/
        chart.md
        chart_data.json
        assets/
        manifest.json
    plans/
      {output_uuid}/
        plan.md
        manifest.json
    study_guides/
      {output_uuid}/
        study_guide.md
        manifest.json
    exports/
      llms.txt
      llms-full.txt
      markdown_bundle.zip
      context_pack.jsonl
      export_manifest.json

  mirror/
    snapshots/
      {snapshot_version}/
        mirror_manifest.json
        sqlite/
          knowledgebase.sqlite
          schema_version.txt
        drive_objects/
          objects_manifest.jsonl
          objects/
        indexes/
          full_text/
          vector/
          graph/
        runtime/
          runtime_config.toml
          startup_manifest.json
        package/
          knowledgebase-mirror-{snapshot_version}.tar.zst
          knowledgebase-mirror-{snapshot_version}.zip
          checksums.sha256
    deltas/
      {from_version}_to_{to_version}/
        delta_manifest.json
        added_objects.jsonl
        changed_objects.jsonl
        deleted_objects.jsonl
        sql_patch.jsonl
        index_patch/
        package/
          knowledgebase-delta-{from_version}-{to_version}.tar.zst
          checksums.sha256
    local_runtime/
      {runtime_bundle_version}/
        runtime_manifest.json
        bin/
        config/
        README.md

  logs/
    activity.jsonl
    ingestion.jsonl
    compile.jsonl
    audit.jsonl
```

Layout rules:

- `sources/raw/**` is immutable. Retraction or deletion creates tombstones and state changes; it does not edit raw source bytes in place.
- `sources/raw/**/assets/**` stores downloaded images and attachments from clipped or imported sources through `sdkwork-drive`; wiki pages reference these by stable local/drive-backed paths instead of fragile remote URLs when policy allows.
- `parsed/**`, `wiki/**`, `graph/**`, `context_packs/**`, `eval/**`, and `output/**` are drive-backed artifacts with SQL metadata.
- `wiki/schema/AGENTS.md` is required for Codex-compatible LLM Wiki operation. `wiki/schema/CLAUDE.md` may be generated for Claude-compatible operation, but `AGENTS.md` remains the canonical SDKWork agent instruction projection.
- `wiki/schema/wiki_schema.yaml` is the machine-readable source for page categories, frontmatter conventions, allowed workflows, lint rules, and output filing policy.
- `wiki/index.md` and `wiki/log.md` are required compatibility files. `_index.md` files are optional SDKWork helper indexes and must not replace the standard LLM Wiki files.
- `wiki/**/current.md` is a materialized convenience object. The authoritative immutable page body is `revisions/r{revision_no}.md` plus SQL revision metadata.
- Every directory that can be browsed by an LLM should have an `_index.md` or manifest entry.
- `wiki/log.md` is materialized from append-only log entries; direct in-place edits are not allowed.
- `wiki/pages/answers/**`, `wiki/pages/comparisons/**`, `wiki/pages/presentations/**`, and `wiki/pages/charts/**` are wiki pages when an answer/output has been filed back into the knowledgebase. `output/**` remains the generated artifact area for unfiled outputs.
- All object keys use server-generated UUID/version segments plus sanitized slugs. User supplied names are stored as metadata and safe display names; they are never trusted as raw object-key path segments.
- Large or mutable datasets stay outside the knowledgebase by default. `datasets/**` stores manifests, samples, profiles, and query recipes, not full data copies unless explicitly configured.
- `graph/*.jsonl` files are export/read-model artifacts. The SQL tables remain the system of record for links, claims, citations, and lineage.
- Archive is represented by state in SQL and may optionally move logical objects under `archive/{archived_at}/{space_uuid}/`. Normal query, compile, and export operations skip archived spaces unless explicitly requested.
- `mirror/**` contains local runnable mirror artifacts. These artifacts are generated from SQL and drive objects, then written back to drive so they can be downloaded, audited, and incrementally updated.
- Snapshot and delta packages must be content-addressable by checksum and must declare the exact source space version, schema version, object manifest, and index manifest.

### 8.6 Local Mirror and Offline Runtime

The system must support packaging an LLM Wiki knowledgebase for local execution.

Local mirror goals:

- run without the remote service after package download
- preserve wiki page navigation, retrieval, citations, source lineage, and `llms.txt` exports
- preserve the LLM Wiki three-layer projection: immutable raw sources, generated wiki Markdown, and schema/instruction files
- open as a plain Markdown folder in Obsidian or another local editor
- optionally initialize a local Git repository for user-managed history after unpacking, without making Git the service storage backend
- use local drive-compatible object storage layout for files
- use local SQLite metadata for portable runtime
- use local full-text and vector indexes when available
- support incremental updates without redownloading the whole knowledgebase

Local mirror package contents:

```text
mirror_manifest.json
AGENTS.md
CLAUDE.md
schema/**
sqlite/knowledgebase.sqlite
drive_objects/objects_manifest.jsonl
drive_objects/objects/**
raw/**
wiki/**
graph/**
indexes/**
llms.txt
llms-full.txt
runtime/runtime_config.toml
checksums.sha256
README.md
```

`mirror_manifest.json` fields:

```json
{
  "schemaVersion": "1.0.0",
  "spaceId": "space_uuid",
  "snapshotVersion": "2026.06.01.000001",
  "baseSnapshotVersion": null,
  "createdAt": "2026-06-01T00:00:00Z",
  "packageKind": "snapshot",
  "contentPolicy": {
    "includeRawSources": false,
    "includeParsedArtifacts": true,
    "includeWiki": true,
    "includeEmbeddings": true,
    "includeEvalReports": false
  },
  "llmWikiCompatibility": {
    "profile": "docs/llm-wiki.md",
    "agentInstructionPath": "AGENTS.md",
    "schemaPath": "schema/wiki_schema.yaml",
    "rawRoot": "raw/",
    "wikiRoot": "wiki/",
    "indexPath": "wiki/index.md",
    "logPath": "wiki/log.md"
  },
  "database": {
    "engine": "sqlite",
    "schemaVersion": "1.0.0",
    "file": "sqlite/knowledgebase.sqlite",
    "checksumSha256": "..."
  },
  "objectsManifest": "drive_objects/objects_manifest.jsonl",
  "indexManifests": [
    "indexes/full_text/index_manifest.json",
    "indexes/vector/index_manifest.json",
    "indexes/graph/index_manifest.json"
  ],
  "checksums": "checksums.sha256"
}
```

Incremental delta package contents:

```text
delta_manifest.json
sql_patch.jsonl
added_objects.jsonl
changed_objects.jsonl
deleted_objects.jsonl
objects/**
index_patch/**
checksums.sha256
```

`delta_manifest.json` fields:

```json
{
  "schemaVersion": "1.0.0",
  "spaceId": "space_uuid",
  "packageKind": "delta",
  "fromSnapshotVersion": "2026.06.01.000001",
  "toSnapshotVersion": "2026.06.01.000002",
  "createdAt": "2026-06-01T01:00:00Z",
  "requiresSchemaVersion": "1.0.0",
  "operations": {
    "sqlPatch": "sql_patch.jsonl",
    "addedObjects": "added_objects.jsonl",
    "changedObjects": "changed_objects.jsonl",
    "deletedObjects": "deleted_objects.jsonl",
    "indexPatch": "index_patch/"
  },
  "checksums": "checksums.sha256"
}
```

Delta update rules:

- A delta can be applied only when local `snapshotVersion == fromSnapshotVersion`.
- Every object add/change/delete must be verified by checksum.
- Changes to `AGENTS.md`, `schema/**`, `wiki/index.md`, and `wiki/log.md` are treated as first-class delta objects and must be applied atomically with their related SQL and graph changes.
- `wiki/log.md` deltas are append-only unless a signed repair package explicitly declares a log repair reason and audit reference.
- Deletes are represented by tombstones first; physical cleanup runs after successful apply.
- SQL patch operations must be idempotent and ordered.
- Index patches must declare compatibility with the local index adapter and embedding model.
- If schema versions differ, the local updater must run a schema migration before applying content changes.
- Failed delta apply must rollback to the previous local snapshot.
- Local update must emit an audit log and update local mirror manifest.

Local runtime modes:

```text
offline_readonly
offline_with_local_overlay
online_sync
```

`offline_readonly`

- local mirror can search, read wiki, retrieve chunks, and build context packs.
- no remote writeback.

`offline_with_local_overlay`

- user can create local-only notes, tags, and bookmarks in a separate overlay SQLite database.
- remote canonical knowledge remains unchanged.

`online_sync`

- runtime checks remote manifest, downloads applicable deltas, verifies, applies, and updates local manifest.

Local mirror operations:

```text
localMirror.snapshots.create
localMirror.snapshots.retrieve
localMirror.deltas.create
localMirror.deltas.apply
localMirror.deltas.verify
localMirror.runtimeBundles.create
localMirror.manifests.retrieve
```

### 8.7 Local Mirror Security

Local mirror packaging must preserve access rules.

Rules:

- A package is generated for a tenant, organization, principal, or explicit audience scope.
- Package manifest records `audience_scope` and `data_scope`.
- Sensitive sources can be excluded by content policy.
- Raw source inclusion is opt-in.
- Local packages should be encrypted when they contain private tenant data.
- Package and delta manifests must be signed or have a detached signature record when distributed outside trusted infrastructure.
- `AGENTS.md`, `schema/**`, `wiki/index.md`, and `wiki/log.md` must not leak provider credentials, remote tokens, presigned URLs, or hidden ACL material.
- `wiki/log.md` entries must respect the same privacy policy as retrieval traces. Raw user query text is redacted unless explicit retention is enabled for the space.
- Package download grants are short-lived and served through `sdkwork-drive`.
- Local runtime must not expose drive provider credentials from the source environment.
- ACL metadata must remain available locally for security trimming and citation visibility.

### 8.8 Local Mirror Database Profile

The local mirror uses SQLite as the portable metadata store.

Local SQLite includes:

- published wiki pages and revisions
- wiki links, claims, citations, source lineage
- document metadata and safe object refs
- chunks selected by package policy
- retrieval profiles
- local full-text index metadata
- vector index metadata or vectors, depending on adapter
- package manifest tables
- tombstones and delta apply log

Local SQLite does not store:

- source drive credentials
- remote tokens
- presigned URLs
- provider secrets
- raw file bytes

The local runtime uses a drive-compatible local object store rooted at the unpacked package directory.

### 8.9 LLM Wiki Markdown Contract

Published wiki page Markdown should be stable and agent-readable.

Recommended page shape:

```markdown
---
id: page_uuid
slug: topic-slug
title: Topic Title
pageType: topic
spaceId: space_uuid
revisionNo: 7
status: published
confidence: 0.86
sourceState: fresh
updatedAt: 2026-06-01T00:00:00Z
---

# Topic Title

## Summary

Short answer or topic summary.

## Key Claims

- Claim text. [source: citation-id]

## Details

Structured explanation with cross-links like [[Related Topic]] and normal Markdown links.

## Sources

- citation-id: document title, version, chunk, page, offset.

## Open Questions

- Known gap or unresolved contradiction.

## Maintenance

- Last compile job, stale indicators, reviewer, and audit notes.
```

Rules:

- Frontmatter is required for wiki page revisions.
- IDs in Markdown must match SQL metadata and drive object refs.
- `[[wikilinks]]` are allowed for LLM/Obsidian-style navigation, but normal Markdown links must also be exported where possible.
- Claims and sources must be machine-readable through SQL and graph JSONL exports; Markdown is a readable projection, not the only source of truth.

### 8.10 LLM Wiki Standard Files

The LLM Wiki profile requires a plain Markdown contract that can be used by coding agents and local Markdown tools.
These files are generated and stored through `sdkwork-drive`, then materialized into local mirror packages.

`wiki/schema/AGENTS.md`

- Required canonical instruction file for Codex-compatible operation.
- Defines the role of the LLM maintainer, the three-layer structure, page categories, frontmatter rules, citation rules, ingest workflow, query workflow, lint workflow, and local mirror behavior.
- Generated from `wiki/schema/wiki_schema.yaml` plus selected policy templates.
- Must be copied to package root as `AGENTS.md` for local mirrors.

`wiki/schema/CLAUDE.md`

- Optional compatibility projection for Claude Code workflows.
- Must be generated from the same schema source as `AGENTS.md`; it is not independently authored.

`wiki/index.md`

- Required content-oriented catalog of published wiki pages.
- Lists pages by category, link, one-line summary, status, last update, source count, and important tags.
- Updated whenever ingest, publish, archive, restore, or filed query-answer workflows change visible wiki content.
- Query workflows read this file first at small and moderate scale; larger spaces may use SQL/search indexes but must keep the file correct.

Recommended `index.md` shape:

```markdown
# Index

## Overview

- [[Overview]] - Current synthesis for this knowledge space.

## Sources

- [[Source Title]] - One-line source summary. sources: 1, updated: 2026-06-01

## Entities

- [[Entity Name]] - One-line entity summary. sources: 4, updated: 2026-06-01

## Concepts

- [[Concept Name]] - One-line concept summary. sources: 3, updated: 2026-06-01

## Open Questions

- [[Question Title]] - Known gap or unresolved contradiction.
```

`wiki/log.md`

- Required chronological append-only activity log.
- Covers ingest, query, filed answer, compile, review, publish, lint, eval, package, mirror, and delta update events.
- Uses a parseable heading prefix.

Required heading pattern:

```markdown
## [2026-06-01T00:00:00Z] ingest | Source Title
```

Each entry should include:

- actor or system actor
- affected sources/pages
- candidate or revision IDs
- citation summary
- review state
- warnings, contradictions, or open questions
- audit event ID

`raw/**`

- Obsidian-compatible projection of immutable raw source material and downloaded attachments in local mirror packages.
- Original source bytes remain under `sources/raw/**` in drive; package-level `raw/**` is a safe readable projection for local mirrors.
- Images and attachments use safe local paths and are referenced from Markdown with relative links when content policy permits.

Filed query outputs:

- A query answer is first stored under `output/answers/{output_uuid}/`.
- If the user or policy files it into the wiki, the system creates a candidate under `candidates/**` and then a published page revision under `wiki/pages/answers/**`, `wiki/pages/comparisons/**`, `wiki/pages/presentations/**`, or another configured page category.
- The filed page keeps links to the retrieval trace, citations, source pages, and generated artifact.

Git compatibility:

- Local mirror packages are plain files and may be initialized as Git repositories by users or local tooling.
- The backend must not depend on Git for authoritative storage, locking, history, or replication.
- Service-managed history remains SQL revision metadata plus immutable drive objects.

### 8.11 LLM Wiki Tool Surface

The backend API should support an agent-native tool surface on top of normal REST APIs.

Tool-level operations:

```text
wiki.search
wiki.read
wiki.followLinks
wiki.listBacklinks
wiki.getClaims
wiki.getSources
wiki.ingest
wiki.compile
wiki.reviewCandidate
wiki.publish
wiki.buildContextPack
wiki.lint
wiki.audit
wiki.export
wiki.fileAnswer
wiki.renderIndex
wiki.appendLog
wiki.getSchema
wiki.packageLocalMirror
wiki.applyLocalDelta
```

The REST and SDK operation IDs still follow SDKWork API rules. The tool names are semantic aliases for agent orchestration and can be exported as an MCP/OpenAPI tool catalog later.

## 9. Database Design

All database objects created by SDKWork Knowledgebase use the `kb_` prefix for tables, `idx_kb_` for non-unique indexes, and `uk_kb_` for unique indexes. Product/API names may keep `Knowledge*` terminology; physical database objects must stay under the `kb_` namespace.

### 9.0 Runtime ID Strategy

All persistent `kb_*` tables use `id` as an `int64` internal primary key. Runtime insert paths MUST generate and bind `id` explicitly before executing SQL. The database MUST NOT own ID creation through SQLite rowid autogeneration, `AUTOINCREMENT`, PostgreSQL `SERIAL`/`BIGSERIAL`, identity columns, or ad hoc sequence calls in repository SQL.

The default strategy is a service-side Snowflake ID generator:

```yaml
id_strategy:
  type: snowflake
  bit_layout: 41_bit_timestamp_delta_ms + 10_bit_node_id + 12_bit_sequence
  epoch_utc: 2025-01-01T00:00:00Z
  id_type: int64
  runtime_node_id_env: SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID
  node_id_range: 0..1023
  clock_rollback: reject_and_surface_error
  sequence_overflow: wait_next_millisecond
  node_conflict: runtime_config_must_assign_unique_node_id_per_process
  invalid_config: fail_closed_before_repository_use
  public_id: uuid
```

SQLite migrations define `id BIGINT NOT NULL` with table-level `PRIMARY KEY (id)` so omitted IDs fail instead of falling back to implicit rowid generation. PostgreSQL migrations define `id BIGINT PRIMARY KEY` without serial or identity defaults. Repository tests must prove every `INSERT INTO kb_*` statement declares the `id` column and that SQLite rejects inserts that omit `id`.

Runtime configuration:

- `SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID` configures the 10-bit Snowflake node id. Valid values are integers from `0` through `1023`.
- Local and test runs may omit the variable and use node id `0`. Production multi-instance deployments must assign a unique node id per process or pod; duplicate node ids are a deployment fault.
- Invalid configured values fail closed during generator initialization. Runtime ID generation errors, including clock rollback and signed-int64 overflow, must propagate as repository errors instead of falling back to database-generated IDs.
- Sequence overflow waits for the next millisecond. Operators should monitor ID generation failures, clock rollback errors, and node-id configuration drift alongside repository write errors.

Common columns for core L2 tables:

```text
id BIGINT NOT NULL
uuid VARCHAR(64) NOT NULL
tenant_id BIGINT NOT NULL
organization_id BIGINT NOT NULL DEFAULT 0
user_id BIGINT
owner_type VARCHAR(64)
owner_id BIGINT
status INTEGER NOT NULL
created_at TIMESTAMP NOT NULL
updated_at TIMESTAMP NOT NULL
version BIGINT NOT NULL DEFAULT 0
deleted_at TIMESTAMP
deleted_by BIGINT
```

API serialization:

- `id`, `tenantId`, `organizationId`, `userId`, sizes, and counters are strings in JSON.
- timestamps are ISO 8601 UTC.
- decimals are strings.
- DB columns use `lower_snake_case`; API fields use `lowerCamelCase`.

### 9.1 Core Tables

#### kb_space

System of record for a knowledgebase space.

Key columns:

```text
name VARCHAR(200) NOT NULL
description VARCHAR(2000)
drive_space_id VARCHAR(128)
visibility INTEGER NOT NULL
llm_wiki_initialized BOOLEAN NOT NULL DEFAULT false
wiki_log_sequence_counter BIGINT NOT NULL DEFAULT 0
default_collection_id BIGINT
default_retrieval_profile_id BIGINT
quota_policy_id BIGINT
metadata JSON
```

Indexes:

```text
uk_kb_space_uuid unique(uuid)
uk_kb_space_drive_space unique(tenant_id, drive_space_id) where drive_space_id is not null and status = active
idx_kb_space_tenant_status_updated(tenant_id, organization_id, status, updated_at)
idx_kb_space_owner(owner_type, owner_id, status)
```

#### kb_collection

Tree structure inside a space.

Key columns:

```text
space_id BIGINT NOT NULL
parent_id BIGINT NOT NULL DEFAULT 0
name VARCHAR(200) NOT NULL
path VARCHAR(2048) NOT NULL
level_no INTEGER NOT NULL
sort_order INTEGER NOT NULL DEFAULT 0
metadata JSON
```

Indexes:

```text
uk_kb_collection_uuid unique(uuid)
idx_kb_collection_space_parent_sort(tenant_id, space_id, parent_id, sort_order)
idx_kb_collection_path(tenant_id, space_id, path)
```

#### kb_source

Import source record.

Key columns:

```text
space_id BIGINT NOT NULL
source_type VARCHAR(64) NOT NULL
provider VARCHAR(128)
drive_bucket VARCHAR(256)
drive_prefix VARCHAR(1024)
sync_policy JSON
last_sync_at TIMESTAMP
last_sync_job_id BIGINT
metadata JSON
```

Source types:

```text
upload
drive_object
drive_folder
url
connector
api
```

Indexes:

```text
uk_kb_source_identity unique(tenant_id, space_id, source_type, coalesce(provider), coalesce(drive_bucket), coalesce(drive_prefix)) where status = active
```

#### kb_drive_object_ref

Stable reference to a `sdkwork-drive` object.

Key columns:

```text
drive_provider_kind VARCHAR(64) NOT NULL
drive_bucket VARCHAR(256) NOT NULL
drive_object_key VARCHAR(2048) NOT NULL
drive_object_version VARCHAR(256)
drive_etag VARCHAR(256)
content_type VARCHAR(256)
size_bytes BIGINT NOT NULL
checksum_sha256_hex VARCHAR(128)
drive_metadata JSON
object_role VARCHAR(64) NOT NULL
access_mode VARCHAR(64) NOT NULL
```

Indexes:

```text
uk_kb_drive_object_ref_uuid unique(uuid)
uk_kb_drive_object_ref_locator unique(tenant_id, space_id, drive_bucket, drive_object_key, coalesce(drive_object_version), object_role)
idx_kb_drive_object_locator(tenant_id, drive_bucket, drive_object_key, drive_object_version)
idx_kb_drive_object_role(tenant_id, object_role, created_at)
idx_kb_drive_object_drive_node(tenant_id, space_id, drive_space_id, drive_node_id, status)
```

Important rule: this table stores drive object references only. It does not store presigned URLs or provider credentials.

#### kb_document

Document master data.

Key columns:

```text
space_id BIGINT NOT NULL
collection_id BIGINT NOT NULL DEFAULT 0
source_id BIGINT
identity_scope VARCHAR(64) NOT NULL DEFAULT 'source_and_original_drive_node'
original_file_drive_node_id VARCHAR(128)
title VARCHAR(512) NOT NULL
mime_type VARCHAR(256)
language VARCHAR(32)
current_version_id BIGINT
visibility INTEGER NOT NULL
content_state INTEGER NOT NULL
index_state INTEGER NOT NULL
metadata JSON
```

Indexes:

```text
uk_kb_document_uuid unique(uuid)
uk_kb_document_identity unique(tenant_id, space_id, collection_id, identity_scope, coalesce(source_id), identity-dependent drive node key) where status = active
idx_kb_document_drive_node(tenant_id, space_id, original_file_drive_node_id, status)
idx_kb_document_space_collection_updated(tenant_id, space_id, collection_id, updated_at)
idx_kb_document_source(source_id, status)
idx_kb_document_title(tenant_id, space_id, title)
```

Document identity is an explicit strategy, not an accidental nullable-key side effect:

- `source_only` is used for a direct single Drive object import. It requires `source_id`; the active document identity is the source itself, and a late `original_file_drive_node_id` binding may enrich the document without creating a second document.
- `source_and_original_drive_node` is used for folder imports, connector imports, API-created documents, and other multi-document source scenarios. The same source may produce many active documents, so the original Drive node participates in identity.
- The default is `source_and_original_drive_node` because it is the safer strategy for multi-document sources. Direct Drive object import code must opt into `source_only`.

The physical unique index must include `identity_scope`, `tenant_id`, `space_id`, `collection_id`, `COALESCE(source_id, 0)`, and a conditional Drive node expression:

```text
CASE
  WHEN identity_scope = 'source_only' THEN ''
  ELSE COALESCE(original_file_drive_node_id, '')
END
```

#### kb_document_version

Immutable document version.

Key columns:

```text
document_id BIGINT NOT NULL
version_no BIGINT NOT NULL
original_object_ref_id BIGINT NOT NULL
checksum_sha256_hex VARCHAR(128)
size_bytes BIGINT NOT NULL
mime_type VARCHAR(256)
parser_profile_id BIGINT
parse_state INTEGER NOT NULL
index_state INTEGER NOT NULL
submitted_by BIGINT
submitted_at TIMESTAMP NOT NULL
metadata JSON
```

Indexes:

```text
uk_kb_document_version_uuid unique(uuid)
uk_kb_document_version_no unique(document_id, version_no)
idx_kb_document_version_state(tenant_id, parse_state, index_state, updated_at)
```

### 9.2 Processing Tables

#### kb_ingestion_job

Async job for import, parse, embed, reindex, sync, and cleanup.

Key columns:

```text
space_id BIGINT NOT NULL
source_id BIGINT
job_type VARCHAR(64) NOT NULL
state INTEGER NOT NULL
priority INTEGER NOT NULL DEFAULT 0
progress INTEGER NOT NULL DEFAULT 0
requested_by BIGINT
idempotency_key VARCHAR(128) NOT NULL
request_id VARCHAR(64)
trace_id VARCHAR(128)
error_code VARCHAR(128)
error_detail VARCHAR(4000)
started_at TIMESTAMP
finished_at TIMESTAMP
metadata JSON
```

Indexes:

```text
uk_kb_ingestion_job_uuid unique(uuid)
uk_kb_ingestion_job_idempotency unique(tenant_id, space_id, idempotency_key)
idx_kb_ingestion_job_state_priority(state, priority, created_at)
idx_kb_ingestion_job_space(space_id, state, updated_at)
```

Rules:

- `create_or_get` insert paths use the unique idempotency key as the concurrency guard.
- For imports with side effects, `metadata.idempotency_fingerprint_sha256_hex` stores a stable request fingerprint. Reusing the same idempotency key for a different request must return a conflict before reading drive objects or creating metadata rows.
- `create_or_get` repository paths should try the insert first and then reread on `ON CONFLICT DO NOTHING`, so concurrent identical requests return the same row instead of leaking a unique-key error.

#### kb_ingestion_job_item

Per-document job item.

Key columns:

```text
job_id BIGINT NOT NULL
document_id BIGINT
document_version_id BIGINT
input_object_ref_id BIGINT
stage VARCHAR(64) NOT NULL
state INTEGER NOT NULL
attempt_count INTEGER NOT NULL DEFAULT 0
error_code VARCHAR(128)
error_detail VARCHAR(4000)
started_at TIMESTAMP
finished_at TIMESTAMP
```

#### kb_parse_artifact

Drive-backed parser output.

Key columns:

```text
document_id BIGINT NOT NULL
document_version_id BIGINT NOT NULL
artifact_type VARCHAR(64) NOT NULL
object_ref_id BIGINT NOT NULL
text_length BIGINT
page_count INTEGER
parser_name VARCHAR(128)
parser_version VARCHAR(64)
metadata JSON
```

Artifact types:

```text
normalized_text
ocr_result
page_image
parse_manifest
chunk_manifest
thumbnail
```

### 9.3 Index Tables

#### kb_chunk

Retrievable text chunk.

Key columns:

```text
space_id BIGINT NOT NULL
document_id BIGINT NOT NULL
document_version_id BIGINT NOT NULL
source_artifact_ref_id BIGINT
chunk_no INTEGER NOT NULL
content_text TEXT NOT NULL
token_count INTEGER NOT NULL
page_no INTEGER
start_offset BIGINT
end_offset BIGINT
heading_path VARCHAR(2048)
content_hash VARCHAR(128) NOT NULL
metadata JSON
```

Indexes:

```text
uk_kb_chunk_uuid unique(uuid)
uk_kb_chunk_no unique(document_version_id, chunk_no)
idx_kb_chunk_document(document_id, document_version_id, chunk_no)
idx_kb_chunk_space_status(tenant_id, space_id, status, updated_at)
idx_kb_chunk_content_hash(tenant_id, content_hash)
```

#### kb_chunk_embedding

Embedding metadata and default pgvector read model.

Key columns:

```text
space_id BIGINT NOT NULL
chunk_id BIGINT NOT NULL
document_id BIGINT NOT NULL
document_version_id BIGINT NOT NULL
embedding_model VARCHAR(256) NOT NULL
embedding_dimensions INTEGER NOT NULL
embedding_hash VARCHAR(128) NOT NULL
vector_provider VARCHAR(64) NOT NULL
vector_ref VARCHAR(512)
embedding_vector VECTOR
metadata JSON
```

Physical mapping:

- PostgreSQL with pgvector: `embedding_vector vector(n)` in adapter-managed migration.
- SQLite: `embedding_vector` omitted or stored as adapter-specific blob/reference, depending on selected local vector adapter.

Indexes:

```text
uk_kb_chunk_embedding_model unique(chunk_id, embedding_model)
idx_kb_embedding_space_model(tenant_id, space_id, embedding_model, status)
idx_kb_embedding_hash(tenant_id, embedding_hash)
```

#### kb_search_projection

Optional denormalized read model for hybrid search.

Key columns:

```text
space_id BIGINT NOT NULL
chunk_id BIGINT NOT NULL
document_id BIGINT NOT NULL
document_title_snapshot VARCHAR(512)
collection_path_snapshot VARCHAR(2048)
content_text TEXT NOT NULL
language VARCHAR(32)
acl_fingerprint VARCHAR(128)
source_version BIGINT NOT NULL
rebuild_version BIGINT NOT NULL
metadata JSON
```

### 9.4 Retrieval Tables

#### kb_retrieval_profile

Query strategy configuration.

Key columns:

```text
space_id BIGINT NOT NULL
name VARCHAR(200) NOT NULL
query_mode VARCHAR(64) NOT NULL
embedding_model VARCHAR(256)
rerank_model VARCHAR(256)
chunk_top_k INTEGER NOT NULL DEFAULT 20
final_top_k INTEGER NOT NULL DEFAULT 8
hybrid_weight DECIMAL(8,4)
filters_schema JSON
prompt_policy JSON
metadata JSON
```

Query modes:

```text
keyword
vector
hybrid
hybrid_rerank
```

#### kb_retrieval_trace

Auditable retrieval call record.

Key columns:

```text
space_id BIGINT NOT NULL
profile_id BIGINT
subject_user_id BIGINT
q_hash VARCHAR(128) NOT NULL
query_mode VARCHAR(64) NOT NULL
chunk_top_k INTEGER NOT NULL
final_top_k INTEGER NOT NULL
latency_ms BIGINT NOT NULL
result_count INTEGER NOT NULL
request_id VARCHAR(64)
trace_id VARCHAR(128)
metadata JSON
```

Important rule: store query hash and safe metadata by default. Raw query text storage requires an explicit privacy decision.

#### kb_citation

Returned citation record.

Key columns:

```text
retrieval_trace_id BIGINT NOT NULL
space_id BIGINT NOT NULL
document_id BIGINT NOT NULL
document_version_id BIGINT NOT NULL
chunk_id BIGINT NOT NULL
rank_no INTEGER NOT NULL
score DECIMAL(18,8)
quote_start_offset BIGINT
quote_end_offset BIGINT
snippet VARCHAR(2000)
metadata JSON
```

### 9.5 LLM Wiki Tables

#### kb_wiki_page

Stable topic page in a knowledge space.

Key columns:

```text
space_id BIGINT NOT NULL
collection_id BIGINT NOT NULL DEFAULT 0
slug VARCHAR(256) NOT NULL
title VARCHAR(512) NOT NULL
summary VARCHAR(2000)
page_type VARCHAR(64) NOT NULL
current_revision_id BIGINT
revision_counter BIGINT NOT NULL DEFAULT 0
source_state VARCHAR(64) NOT NULL
review_state VARCHAR(64) NOT NULL
publish_state VARCHAR(64) NOT NULL
metadata JSON
```

Page types:

```text
source
entity
topic
concept
how_to
reference
faq
glossary
answer
comparison
presentation
chart
index
policy
runbook
```

Indexes:

```text
uk_kb_wiki_page_uuid unique(uuid)
uk_kb_wiki_page_slug unique(tenant_id, space_id, slug)
uk_kb_wiki_page_path unique(tenant_id, space_id, logical_path)
idx_kb_wiki_page_state(tenant_id, space_id, publish_state, updated_at)
```

Rules:

- `revision_counter` is the per-page atomic reservation source for `revision_no`. Services must not compute revision numbers with an unprotected `MAX(revision_no) + 1` read.

#### kb_wiki_page_revision

Immutable wiki page revision.

Key columns:

```text
page_id BIGINT NOT NULL
revision_no BIGINT NOT NULL
markdown_object_ref_id BIGINT NOT NULL
author_type VARCHAR(64) NOT NULL
author_id VARCHAR(128) NOT NULL
generation_job_id BIGINT
source_fingerprint VARCHAR(128)
content_hash VARCHAR(128) NOT NULL
change_summary VARCHAR(2000)
review_state VARCHAR(64) NOT NULL
published_at TIMESTAMP
metadata JSON
```

The Markdown object is stored by `sdkwork-drive`.

Indexes:

```text
uk_kb_wiki_page_revision_no unique(tenant_id, page_id, revision_no)
```

#### kb_wiki_source_ref

Source lineage from page revisions to source documents, document versions, chunks, or external citations.

Key columns:

```text
page_revision_id BIGINT NOT NULL
source_type VARCHAR(64) NOT NULL
source_id BIGINT
document_id BIGINT
document_version_id BIGINT
chunk_id BIGINT
external_uri VARCHAR(2048)
source_role VARCHAR(64) NOT NULL
object_path VARCHAR(2048)
confidence DECIMAL(18,8)
metadata JSON
```

Source roles:

```text
primary
supporting
conflicting
background
```

#### kb_wiki_file_entry

SQL registry for LLM Wiki drive-backed files.

Key columns:

```text
space_id BIGINT NOT NULL
object_ref_id BIGINT NOT NULL
logical_path VARCHAR(2048) NOT NULL
entry_type VARCHAR(64) NOT NULL
artifact_role VARCHAR(64) NOT NULL
owner_type VARCHAR(64)
owner_id BIGINT
content_hash VARCHAR(128)
manifest JSON
```

Entry types:

```text
source
parsed_artifact
wiki_revision
wiki_current_projection
wiki_schema
wiki_index
wiki_log
graph_export
context_pack
eval_report
output_export
log
manifest
```

This table makes the drive directory structure queryable and auditable without making SQL the file store.

#### kb_wiki_schema_profile

Versioned LLM Wiki schema and agent-instruction profile for a knowledge space.

Key columns:

```text
space_id BIGINT NOT NULL
profile_version VARCHAR(128) NOT NULL
schema_object_ref_id BIGINT NOT NULL
agents_md_object_ref_id BIGINT NOT NULL
claude_md_object_ref_id BIGINT
ingest_workflow_object_ref_id BIGINT
query_workflow_object_ref_id BIGINT
lint_workflow_object_ref_id BIGINT
page_conventions_object_ref_id BIGINT
state VARCHAR(64) NOT NULL
activated_at TIMESTAMP
metadata JSON
```

Indexes:

```text
uk_kb_wiki_schema_profile_version unique(space_id, profile_version)
idx_kb_wiki_schema_profile_state(space_id, state, activated_at)
```

#### kb_wiki_log_entry

Append-only source for the materialized `wiki/log.md` file.

Key columns:

```text
space_id BIGINT NOT NULL
sequence_no BIGINT NOT NULL
event_type VARCHAR(64) NOT NULL
event_time TIMESTAMP NOT NULL
title VARCHAR(512) NOT NULL
actor_type VARCHAR(64) NOT NULL
actor_id VARCHAR(128)
resource_type VARCHAR(64)
resource_id BIGINT
markdown_object_ref_id BIGINT
audit_event_id BIGINT
privacy_level VARCHAR(64) NOT NULL
metadata JSON
```

Indexes:

```text
uk_kb_wiki_log_entry_sequence unique(tenant_id, space_id, sequence_no)
idx_kb_wiki_log_event_time(space_id, event_type, event_time)
```

Rules:

- Rows are append-only except for tombstone-style redaction metadata required by retention or privacy policy.
- `kb_space.wiki_log_sequence_counter` is the per-space atomic reservation source for `sequence_no`. Services must not compute log sequence numbers with an unprotected `MAX(sequence_no) + 1` read.
- `wiki/log.md` is generated from these rows and stored through `sdkwork-drive`.
- Raw query text is not stored unless the space explicitly enables query retention.

#### kb_wiki_link

Explicit link graph between wiki pages.

Key columns:

```text
space_id BIGINT NOT NULL
source_page_id BIGINT NOT NULL
target_page_id BIGINT
target_slug VARCHAR(256)
link_type VARCHAR(64) NOT NULL
anchor_text VARCHAR(512)
is_broken BOOLEAN NOT NULL DEFAULT false
metadata JSON
```

Link types:

```text
related
parent
child
prerequisite
see_also
contrasts_with
supersedes
```

#### kb_wiki_claim

Source-grounded atomic statement.

Key columns:

```text
page_revision_id BIGINT NOT NULL
claim_no INTEGER NOT NULL
claim_text VARCHAR(4000) NOT NULL
claim_hash VARCHAR(128) NOT NULL
support_state VARCHAR(64) NOT NULL
confidence DECIMAL(18,8)
metadata JSON
```

Support states:

```text
supported
weakly_supported
unsupported
conflicting
unknown
```

#### kb_wiki_claim_citation

Claim-to-source citation relation.

Key columns:

```text
claim_id BIGINT NOT NULL
document_id BIGINT
document_version_id BIGINT
chunk_id BIGINT
source_ref_id BIGINT
quote_start_offset BIGINT
quote_end_offset BIGINT
score DECIMAL(18,8)
metadata JSON
```

#### kb_wiki_candidate

Draft page or revision produced by compile jobs.

Key columns:

```text
space_id BIGINT NOT NULL
page_id BIGINT
candidate_type VARCHAR(64) NOT NULL
compile_job_id BIGINT NOT NULL
markdown_object_ref_id BIGINT NOT NULL
state VARCHAR(64) NOT NULL
reviewer_id BIGINT
reviewed_at TIMESTAMP
review_note VARCHAR(2000)
metadata JSON
```

Candidate states:

```text
draft
ready_for_review
approved
rejected
changes_requested
published
superseded
```

Candidate types:

```text
source_summary
page_create
page_update
query_answer
comparison
presentation
chart
schema_update
index_rebuild
```

#### kb_wiki_export

Generated LLM-friendly export artifact.

Key columns:

```text
space_id BIGINT NOT NULL
export_type VARCHAR(64) NOT NULL
profile_id BIGINT
object_ref_id BIGINT NOT NULL
source_revision_fingerprint VARCHAR(128) NOT NULL
state VARCHAR(64) NOT NULL
generated_by BIGINT
generated_at TIMESTAMP NOT NULL
metadata JSON
```

Export types:

```text
llms_txt
llms_full_txt
markdown_bundle
context_pack_json
jsonl
```

#### kb_wiki_eval_run

Quality evaluation run.

Key columns:

```text
space_id BIGINT NOT NULL
eval_type VARCHAR(64) NOT NULL
state VARCHAR(64) NOT NULL
target_type VARCHAR(64) NOT NULL
target_id BIGINT
score DECIMAL(18,8)
started_at TIMESTAMP
finished_at TIMESTAMP
metadata JSON
```

#### kb_wiki_eval_issue

Issue found by lint/evaluation.

Key columns:

```text
eval_run_id BIGINT NOT NULL
issue_type VARCHAR(128) NOT NULL
severity VARCHAR(64) NOT NULL
resource_type VARCHAR(64) NOT NULL
resource_id BIGINT
message VARCHAR(2000) NOT NULL
evidence JSON
state VARCHAR(64) NOT NULL
```

Issue types:

```text
broken_link
missing_citation
unsupported_claim
conflicting_claim
stale_source
duplicate_page
orphan_page
missing_concept_page
missing_cross_reference
knowledge_gap
empty_page
oversized_page
forbidden_content
acl_mismatch
```

### 9.6 Local Mirror Tables

#### kb_local_mirror_package

Generated local runnable package.

Key columns:

```text
space_id BIGINT NOT NULL
package_type VARCHAR(64) NOT NULL
snapshot_version VARCHAR(128) NOT NULL
base_snapshot_version VARCHAR(128)
object_ref_id BIGINT NOT NULL
manifest_object_ref_id BIGINT NOT NULL
checksum_sha256_hex VARCHAR(128) NOT NULL
content_policy JSON NOT NULL
audience_scope VARCHAR(128) NOT NULL
data_scope VARCHAR(128) NOT NULL
state VARCHAR(64) NOT NULL
created_by BIGINT
generated_at TIMESTAMP NOT NULL
metadata JSON
```

Package types:

```text
snapshot
delta
runtime_bundle
```

#### kb_local_mirror_delta

Incremental update package from one snapshot version to another.

Key columns:

```text
space_id BIGINT NOT NULL
from_snapshot_version VARCHAR(128) NOT NULL
to_snapshot_version VARCHAR(128) NOT NULL
package_object_ref_id BIGINT NOT NULL
manifest_object_ref_id BIGINT NOT NULL
sql_patch_object_ref_id BIGINT
object_patch_manifest_ref_id BIGINT
index_patch_object_ref_id BIGINT
checksum_sha256_hex VARCHAR(128) NOT NULL
state VARCHAR(64) NOT NULL
metadata JSON
```

Indexes:

```text
uk_kb_local_delta_versions unique(space_id, from_snapshot_version, to_snapshot_version)
idx_kb_local_delta_target(space_id, to_snapshot_version, state)
```

#### kb_local_mirror_subscription

Local mirror update channel or audience registration.

Key columns:

```text
space_id BIGINT NOT NULL
subscriber_type VARCHAR(64) NOT NULL
subscriber_id VARCHAR(128) NOT NULL
audience_scope VARCHAR(128) NOT NULL
current_snapshot_version VARCHAR(128)
update_policy JSON
last_check_at TIMESTAMP
last_package_id BIGINT
state VARCHAR(64) NOT NULL
metadata JSON
```

#### kb_local_mirror_apply_log

Audit log for local or remote delta apply attempts.

Key columns:

```text
space_id BIGINT NOT NULL
package_id BIGINT NOT NULL
from_snapshot_version VARCHAR(128)
to_snapshot_version VARCHAR(128)
apply_state VARCHAR(64) NOT NULL
started_at TIMESTAMP
finished_at TIMESTAMP
error_code VARCHAR(128)
error_detail VARCHAR(4000)
applied_by VARCHAR(128)
metadata JSON
```

### 9.7 Security and Governance Tables

#### kb_acl_entry

ACL at space, collection, document, or document version level.

Key columns:

```text
resource_type VARCHAR(64) NOT NULL
resource_id BIGINT NOT NULL
principal_type VARCHAR(64) NOT NULL
principal_id VARCHAR(128) NOT NULL
effect VARCHAR(32) NOT NULL
permission VARCHAR(128) NOT NULL
expires_at TIMESTAMP
condition_json JSON
```

Resource types:

```text
space
collection
document
document_version
```

Principal types:

```text
user
role
organization
tenant
group
service_account
public
```

Effects:

```text
allow
deny
```

Permissions:

```text
knowledge.spaces.read
knowledge.spaces.write
knowledge.documents.read
knowledge.documents.write
knowledge.documents.delete
knowledge.retrievals.create
knowledge.admin
```

#### kb_permission_snapshot

Optional read model for retrieval security trimming.

Key columns:

```text
resource_type VARCHAR(64) NOT NULL
resource_id BIGINT NOT NULL
principal_fingerprint VARCHAR(128) NOT NULL
permission_fingerprint VARCHAR(128) NOT NULL
acl_version BIGINT NOT NULL
expires_at TIMESTAMP
metadata JSON
```

#### kb_audit_event

Append-oriented audit events.

Key columns:

```text
event_type VARCHAR(128) NOT NULL
actor_type VARCHAR(64) NOT NULL
actor_id VARCHAR(128) NOT NULL
resource_type VARCHAR(64) NOT NULL
resource_id BIGINT
result VARCHAR(64) NOT NULL
request_id VARCHAR(64)
trace_id VARCHAR(128)
ip_hash VARCHAR(128)
user_agent_hash VARCHAR(128)
payload JSON
```

#### kb_tag

Tag dictionary.

Key columns:

```text
space_id BIGINT NOT NULL
name VARCHAR(128) NOT NULL
color VARCHAR(32)
description VARCHAR(512)
sort_order INTEGER NOT NULL DEFAULT 0
```

#### kb_document_tag

Document-tag relation.

Key columns:

```text
space_id BIGINT NOT NULL
document_id BIGINT NOT NULL
tag_id BIGINT NOT NULL
```

#### kb_retention_policy

Retention and deletion policy.

Key columns:

```text
space_id BIGINT
name VARCHAR(200) NOT NULL
retention_days INTEGER
delete_mode VARCHAR(64) NOT NULL
archive_before_delete BOOLEAN NOT NULL DEFAULT false
legal_hold_supported BOOLEAN NOT NULL DEFAULT false
metadata JSON
```

#### kb_outbox_event

Transactional event publication.

Key columns:

```text
event_id VARCHAR(128) NOT NULL
event_type VARCHAR(128) NOT NULL
aggregate_type VARCHAR(64) NOT NULL
aggregate_id BIGINT NOT NULL
payload JSON NOT NULL
state INTEGER NOT NULL
attempt_count INTEGER NOT NULL DEFAULT 0
next_attempt_at TIMESTAMP
published_at TIMESTAMP
```

Event examples:

```text
knowledge.space.created
knowledge.document.created
knowledge.document.version_created
knowledge.ingest.started
knowledge.ingest.succeeded
knowledge.ingest.failed
knowledge.document.indexed
knowledge.document.deleted
knowledge.wiki.candidate_created
knowledge.wiki.revision_published
knowledge.wiki.export_generated
knowledge.wiki.eval_issue_found
knowledge.acl.changed
knowledge.drive_object.reconciled
```

## 10. State Machines

### 9.1 Document Content State

```text
draft
uploaded
parse_pending
parsing
parse_failed
parsed
index_pending
indexing
index_failed
indexed
archived
deleted
```

### 10.2 Ingestion Job State

```text
queued
running
waiting_retry
succeeded
failed
cancelled
```

### 10.8 Local Mirror Package State

```text
queued
building
ready
failed
revoked
expired
deleted
```

### 10.9 Local Mirror Delta Apply State

```text
pending
verifying
applying
applied
failed
rolled_back
skipped
```

### 10.3 Parse Artifact Lifecycle

```text
created
validated
superseded
deleted
```

### 10.4 Index State

```text
not_indexed
pending
building
ready
stale
failed
deleted
```

### 10.5 Wiki Candidate State

```text
draft
ready_for_review
approved
rejected
changes_requested
published
superseded
```

### 10.6 Wiki Page Publish State

```text
draft
review
published
stale
archived
deleted
```

### 10.7 Wiki Evaluation State

```text
queued
running
succeeded
failed
cancelled
```

## 11. Application Services

`KnowledgeSpaceService`

- Create, update, list, retrieve, archive, delete spaces.
- Enforce tenant and organization ownership.

`KnowledgeCollectionService`

- Manage collection tree.
- Validate tree cycles, path, sort order, and collection moves.

`KnowledgeDocumentService`

- Manage document master data and immutable versions.
- Does not write file bytes.

`KnowledgeDriveStorageService`

- Wrap `KnowledgeDriveStorage`.
- Create object references.
- Validate drive locator, checksum, MIME type, and size.
- Write parse artifacts to drive.

`KnowledgeUploadSessionService`

- Create upload sessions through drive.
- Complete upload into document version.
- Keep presign material out of persisted SQL.

`KnowledgeDriveImportService`

- Import existing drive objects.
- Verify the object with `head_object`.
- Create document version and ingestion job.

`KnowledgeIngestionService`

- Create jobs and job items.
- Enforce idempotency.
- Own retry and cancellation semantics.

`KnowledgeParseService`

- Parse source object into normalized text and artifacts.
- Store artifacts through drive.

`KnowledgeChunkingService`

- Create chunks with offsets, headings, token counts, and content hashes.

`KnowledgeEmbeddingService`

- Generate embeddings.
- Validate embedding dimensions and model identity.

`KnowledgeVectorIndexService`

- Upsert, delete, and search embeddings.
- Adapter-backed.

`KnowledgeRetrievalService`

- Execute keyword/vector/hybrid retrieval.
- Apply security trimming before final results.
- Return citations.

`KnowledgeWikiCompileService`

- Compile parsed source material, chunks, and retrieval results into wiki page candidates.
- Preserve source lineage, candidate state, generated Markdown drive object references, links, and claims.

`KnowledgeWikiReviewService`

- Approve, reject, request changes, publish, archive, or supersede wiki candidates and revisions.
- Prevent machine-generated drafts from silently overwriting human-approved revisions.

`KnowledgeWikiQueryService`

- Retrieve wiki pages, revisions, claims, links, and citations under ACL.
- Provide topic graph traversal for LLM context building.
- Read the content catalog projection (`wiki/index.md`) before falling back to search indexes when configured.
- Persist query outputs under `output/answers/**` and hand off useful answers to the compile/review workflow for filing into `wiki/pages/answers/**` or related page categories.

`KnowledgeWikiSchemaService`

- Manage LLM Wiki schema profiles, `AGENTS.md`, optional `CLAUDE.md`, workflow docs, and page conventions.
- Generate agent instruction files from the machine-readable schema profile.
- Store all schema files through `sdkwork-drive` and register them in `kb_wiki_file_entry`.

`KnowledgeWikiIndexLogService`

- Maintain `wiki/index.md` and append-only `wiki/log.md` compatibility files.
- Materialize these files from SQL page metadata, graph state, and `kb_wiki_log_entry`.
- Keep `_index.md` helper files synchronized where needed without replacing the standard files.

`KnowledgeContextPackService`

- Build bounded LLM context packs from wiki pages, chunks, claims, and citations.
- Store persisted context packs through `sdkwork-drive`.

`KnowledgeWikiExportService`

- Generate `llms.txt`, `llms-full.txt`, Markdown bundles, JSONL, and context pack exports.
- Store all export artifacts through `sdkwork-drive`.

`KnowledgeWikiLintService`

- Detect broken links, missing citations, unsupported claims, conflicting claims, duplicate pages, stale source references, and ACL mismatches.

`KnowledgeEvaluationService`

- Run retrieval and wiki quality evaluation datasets.
- Persist evaluation runs and issues.

`KnowledgeLocalMirrorPackageService`

- Build full local mirror snapshots, incremental deltas, and runtime bundles.
- Store generated packages, manifests, patches, and checksums through `sdkwork-drive`.

`KnowledgeLocalMirrorUpdateService`

- Verify and apply local delta packages.
- Enforce base snapshot, checksum, schema, tombstone, and rollback rules.

`KnowledgeLocalRuntimeService`

- Produce and validate local runtime config for offline and online-sync modes.
- Bind local SQLite metadata and drive-compatible local object storage.

`KnowledgePermissionService`

- Evaluate ACL/RBAC/ABAC and build optional permission snapshots.

`KnowledgeAuditService`

- Emit audit events and outbox events.

## 12. Ports

Important ports under `services/sdkwork-knowledgebase-product/src/ports`:

```text
knowledge_space_store.rs
knowledge_collection_store.rs
knowledge_document_store.rs
knowledge_drive_object_ref_store.rs
knowledge_ingestion_job_store.rs
knowledge_parse_artifact_store.rs
knowledge_chunk_store.rs
knowledge_embedding_store.rs
knowledge_acl_store.rs
knowledge_audit_store.rs
knowledge_outbox_store.rs
knowledge_drive_storage.rs
knowledge_parser.rs
knowledge_chunker.rs
knowledge_embedder.rs
knowledge_vector_index.rs
knowledge_full_text_index.rs
knowledge_reranker.rs
knowledge_permission_evaluator.rs
knowledge_wiki_page_store.rs
knowledge_wiki_revision_store.rs
knowledge_wiki_claim_store.rs
knowledge_wiki_link_store.rs
knowledge_wiki_candidate_store.rs
knowledge_wiki_export_store.rs
knowledge_wiki_eval_store.rs
knowledge_wiki_file_entry_store.rs
knowledge_wiki_schema_profile_store.rs
knowledge_wiki_log_entry_store.rs
knowledge_local_mirror_package_store.rs
knowledge_local_mirror_delta_store.rs
knowledge_local_mirror_subscription_store.rs
knowledge_local_mirror_apply_log_store.rs
knowledge_wiki_compiler.rs
knowledge_wiki_schema_renderer.rs
knowledge_wiki_index_renderer.rs
knowledge_wiki_log_renderer.rs
knowledge_context_pack_builder.rs
knowledge_wiki_linter.rs
knowledge_local_mirror_packager.rs
knowledge_local_delta_applier.rs
```

## 13. API Design

All shared app APIs use `/app/v3/api`.
All backend/admin APIs use `/backend/v3/api`.
OpenAPI is the source of truth.
Generated SDKs must use dotted resource operation IDs.
For LLM Wiki resources, operation IDs must keep `wiki` as the first resource segment and put the concrete resource below it.
Use `wiki.index.retrieve`, `wiki.log.retrieve`, `wiki.log.entries.create`, `wiki.schema.retrieve`, `wiki.schema.profiles.create`, `wiki.pages.list`, and `wiki.queries.fileAnswer`; do not use flattened names such as `wikiIndex.retrieve` or `wikiPages.list`.

### 12.1 App API

Tag: `knowledge`

```text
GET    /app/v3/api/knowledge/spaces
       operationId: spaces.list

POST   /app/v3/api/knowledge/spaces
       operationId: spaces.create

GET    /app/v3/api/knowledge/spaces/{spaceId}
       operationId: spaces.retrieve

PATCH  /app/v3/api/knowledge/spaces/{spaceId}
       operationId: spaces.update

GET    /app/v3/api/knowledge/spaces/{spaceId}/collections
       operationId: spaces.collections.list

POST   /app/v3/api/knowledge/spaces/{spaceId}/collections
       operationId: spaces.collections.create

POST   /app/v3/api/knowledge/upload_sessions
       operationId: uploadSessions.create

POST   /app/v3/api/knowledge/drive_imports
       operationId: driveImports.create

POST   /app/v3/api/knowledge/ingests
       operationId: ingests.create

GET    /app/v3/api/knowledge/ingests/{ingestId}
       operationId: ingests.retrieve

GET    /app/v3/api/knowledge/documents
       operationId: documents.list

POST   /app/v3/api/knowledge/documents
       operationId: documents.create

GET    /app/v3/api/knowledge/documents/{documentId}
       operationId: documents.retrieve

PATCH  /app/v3/api/knowledge/documents/{documentId}
       operationId: documents.update

DELETE /app/v3/api/knowledge/documents/{documentId}
       operationId: documents.delete

POST   /app/v3/api/knowledge/documents/{documentId}/versions
       operationId: documents.versions.create

GET    /app/v3/api/knowledge/documents/{documentId}/versions
       operationId: documents.versions.list

GET    /app/v3/api/knowledge/documents/{documentId}/download
       operationId: documents.download.retrieve

GET    /app/v3/api/knowledge/index_jobs/{jobId}
       operationId: indexJobs.retrieve

POST   /app/v3/api/knowledge/retrievals
       operationId: retrievals.create

GET    /app/v3/api/knowledge/wiki_pages
       operationId: wiki.pages.list

GET    /app/v3/api/knowledge/wiki_pages/{pageId}
       operationId: wiki.pages.retrieve

GET    /app/v3/api/knowledge/wiki_pages/{pageId}/revisions
       operationId: wiki.pages.revisions.list

GET    /app/v3/api/knowledge/wiki_index
       operationId: wiki.index.retrieve

GET    /app/v3/api/knowledge/wiki_log
       operationId: wiki.log.retrieve

GET    /app/v3/api/knowledge/wiki_schema
       operationId: wiki.schema.retrieve

POST   /app/v3/api/knowledge/wiki_queries
       operationId: wiki.queries.create

POST   /app/v3/api/knowledge/wiki_queries/{queryId}/file_answer
       operationId: wiki.queries.fileAnswer

POST   /app/v3/api/knowledge/wiki_context_packs
       operationId: wiki.contextPacks.create

GET    /app/v3/api/knowledge/local_mirror/manifests/{spaceId}
       operationId: localMirror.manifests.retrieve

GET    /app/v3/api/knowledge/local_mirror/snapshots
       operationId: localMirror.snapshots.list

GET    /app/v3/api/knowledge/local_mirror/snapshots/{snapshotId}
       operationId: localMirror.snapshots.retrieve

GET    /app/v3/api/knowledge/local_mirror/deltas
       operationId: localMirror.deltas.list

POST   /app/v3/api/knowledge/local_mirror/deltas/apply
       operationId: localMirror.deltas.apply
```

### 12.2 Backend API

Tag: `knowledge`

```text
GET    /backend/v3/api/knowledge/spaces
       operationId: spaces.adminList

PATCH  /backend/v3/api/knowledge/spaces/{spaceId}
       operationId: spaces.adminUpdate

GET    /backend/v3/api/knowledge/sources
       operationId: sources.list

POST   /backend/v3/api/knowledge/sources
       operationId: sources.create

GET    /backend/v3/api/knowledge/drive_object_refs
       operationId: driveObjectRefs.list

GET    /backend/v3/api/knowledge/drive_object_refs/{objectRefId}
       operationId: driveObjectRefs.retrieve

POST   /backend/v3/api/knowledge/drive_object_refs/{objectRefId}/reconcile
       operationId: driveObjectRefs.reconcile

GET    /backend/v3/api/knowledge/index_jobs
       operationId: indexJobs.list

POST   /backend/v3/api/knowledge/index_jobs
       operationId: indexJobs.create

POST   /backend/v3/api/knowledge/index_jobs/{jobId}/cancel
       operationId: indexJobs.cancel

POST   /backend/v3/api/knowledge/index_jobs/{jobId}/retry
       operationId: indexJobs.retry

GET    /backend/v3/api/knowledge/retrieval_profiles
       operationId: retrievalProfiles.list

POST   /backend/v3/api/knowledge/retrieval_profiles
       operationId: retrievalProfiles.create

PATCH  /backend/v3/api/knowledge/retrieval_profiles/{profileId}
       operationId: retrievalProfiles.update

GET    /backend/v3/api/knowledge/audit_events
       operationId: auditEvents.list

POST   /backend/v3/api/knowledge/wiki_compile_jobs
       operationId: wiki.compileJobs.create

GET    /backend/v3/api/knowledge/wiki_candidates
       operationId: wiki.candidates.list

POST   /backend/v3/api/knowledge/wiki_candidates/{candidateId}/approve
       operationId: wiki.candidates.approve

POST   /backend/v3/api/knowledge/wiki_candidates/{candidateId}/reject
       operationId: wiki.candidates.reject

POST   /backend/v3/api/knowledge/wiki_pages/{pageId}/publish
       operationId: wiki.pages.publish

POST   /backend/v3/api/knowledge/wiki_schema_profiles
       operationId: wiki.schema.profiles.create

PATCH  /backend/v3/api/knowledge/wiki_schema_profiles/{profileId}
       operationId: wiki.schema.profiles.update

POST   /backend/v3/api/knowledge/wiki_index/rebuild
       operationId: wiki.index.rebuild

POST   /backend/v3/api/knowledge/wiki_log_entries
       operationId: wiki.log.entries.create

POST   /backend/v3/api/knowledge/wiki_exports
       operationId: wiki.exports.create

GET    /backend/v3/api/knowledge/wiki_exports/{exportId}
       operationId: wiki.exports.retrieve

GET    /backend/v3/api/knowledge/wiki_file_entries
       operationId: wiki.fileEntries.list

POST   /backend/v3/api/knowledge/wiki_lint_runs
       operationId: wiki.lintRuns.create

POST   /backend/v3/api/knowledge/wiki_eval_runs
       operationId: wiki.evalRuns.create

POST   /backend/v3/api/knowledge/local_mirror/snapshots
       operationId: localMirror.snapshots.create

POST   /backend/v3/api/knowledge/local_mirror/deltas
       operationId: localMirror.deltas.create

POST   /backend/v3/api/knowledge/local_mirror/deltas/{deltaId}/verify
       operationId: localMirror.deltas.verify

POST   /backend/v3/api/knowledge/local_mirror/runtime_bundles
       operationId: localMirror.runtimeBundles.create

GET    /backend/v3/api/knowledge/local_mirror/subscriptions
       operationId: localMirror.subscriptions.list
```

### 12.3 Important DTOs

All DTOs use lowerCamelCase.

```text
KnowledgeSpace
KnowledgeCollection
KnowledgeSource
KnowledgeDriveObjectRef
KnowledgeDocument
KnowledgeDocumentVersion
KnowledgeUploadSession
KnowledgeDriveImportRequest
KnowledgeIngestionJob
KnowledgeParseArtifact
KnowledgeChunk
KnowledgeRetrievalProfile
KnowledgeRetrievalRequest
KnowledgeRetrievalResult
KnowledgeCitation
KnowledgeIngestRequest
KnowledgeIngestJob
KnowledgeWikiPage
KnowledgeWikiPageRevision
KnowledgeWikiClaim
KnowledgeWikiLink
KnowledgeWikiCandidate
KnowledgeWikiCompileJob
KnowledgeWikiContextPack
KnowledgeWikiExport
KnowledgeWikiFileEntry
KnowledgeWikiEvalRun
KnowledgeWikiEvalIssue
KnowledgeLocalMirrorManifest
KnowledgeLocalMirrorSnapshot
KnowledgeLocalMirrorDelta
KnowledgeLocalMirrorPackage
KnowledgeLocalMirrorSubscription
KnowledgeLocalMirrorApplyLog
KnowledgeLocalRuntimeBundle
KnowledgeAclEntry
KnowledgeAuditEvent
```

`KnowledgeDriveObjectRef` exposes stable identity only. It may include safe delivery hints only when returned by a download/grant API.

## 14. SDK Generation

All SDKWork Knowledgebase HTTP SDKs MUST be generated by the canonical SDKWork generator:

```text
..\sdkwork-sdk-generator
```

The executable entrypoint is:

```text
..\sdkwork-sdk-generator\bin\sdkgen.js
```

SDK family roots:

```text
sdks/sdkwork-knowledgebase-app-sdk
sdks/sdkwork-knowledgebase-backend-sdk
```

OpenAPI authority documents:

```text
sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json
sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json
```

Generation commands must use the canonical `sdkwork-sdk-generator` entrypoint and the SDKWork v3 profile:

```powershell
node ..\sdkwork-sdk-generator\bin\sdkgen.js generate `
  -i sdks\sdkwork-knowledgebase-app-sdk\openapi\knowledgebase-app-api.openapi.json `
  -o sdks\sdkwork-knowledgebase-app-sdk\sdkwork-knowledgebase-app-sdk-typescript\generated\server-openapi `
  -n KnowledgebaseApp `
  -t app `
  -l typescript `
  --sdk-name sdkwork-knowledgebase-app-sdk `
  --package-name @sdkwork/knowledgebase-app-sdk `
  --api-prefix /app/v3/api `
  --fixed-sdk-version 0.1.0 `
  --standard-profile sdkwork-v3
```

```powershell
node ..\sdkwork-sdk-generator\bin\sdkgen.js generate `
  -i sdks\sdkwork-knowledgebase-backend-sdk\openapi\knowledgebase-backend-api.openapi.json `
  -o sdks\sdkwork-knowledgebase-backend-sdk\sdkwork-knowledgebase-backend-sdk-typescript\generated\server-openapi `
  -n KnowledgebaseBackend `
  -t backend `
  -l typescript `
  --sdk-name sdkwork-knowledgebase-backend-sdk `
  --package-name @sdkwork/knowledgebase-backend-sdk `
  --api-prefix /backend/v3/api `
  --fixed-sdk-version 0.1.0 `
  --standard-profile sdkwork-v3
```

Later Rust SDK generation uses the same generator with `-l rust`.

Generated SDK output must not be hand-edited. Each `generated/server-openapi` output root must retain the generator control plane:

- `sdkwork-sdk.json`
- `.sdkwork/sdkwork-generator-manifest.json`
- `.sdkwork/sdkwork-generator-changes.json`
- `.sdkwork/sdkwork-generator-report.json`
- `custom/` for handwritten extensions that survive regeneration

The generated package may be checked or built locally with `bin/publish-core.mjs`. Local verification can create `node_modules/`, `dist/`, and `package-lock.json` under `generated/server-openapi`; these are build artifacts, not generator-owned source artifacts, and must stay out of repository review and release source diffs.

`sdkwork-code-generator`, `openapi-generator`, `swagger-codegen`, copied generator code, local stubs, and direct edits to generated-owned files are not approved SDK generation paths for SDKWork Knowledgebase.

## 15. Security Design

Protected app/backend APIs require dual-token security according to SDKWork standards.

Security controls:

- Tenant and organization context comes from validated token context.
- Object-level authorization is enforced by `KnowledgePermissionService`.
- Retrieval uses security trimming before final results are returned.
- ACL deny entries have precedence over allow entries.
- Admin/backend operations require least privilege permissions.
- Audit events are emitted for document import, delete, permission changes, retrieval profile changes, reindex, and object reconciliation.
- Raw tokens, drive credentials, presign signing material, provider secrets, raw SQL, stack traces, and private object keys are redacted from logs and errors.
- Upload APIs enforce object size, MIME allow-list, checksum, and scan hooks.

Important permission codes:

```text
knowledge.spaces.read
knowledge.spaces.write
knowledge.documents.read
knowledge.documents.write
knowledge.documents.delete
knowledge.retrievals.create
knowledge.index_jobs.read
knowledge.index_jobs.write
knowledge.drive_object_refs.read
knowledge.drive_object_refs.reconcile
knowledge.audit_events.read
knowledge.admin
```

## 16. Performance Design

Default performance classes:

- Space/document list: P1.
- Retrieval: P1 with bounded topK and timeout.
- Upload/import/parse/embed/reindex: P2 async jobs.
- Audit and reconciliation: P2/P3.

Default limits:

```text
default_page_size: 20
max_page_size: 100 for user lists
max_admin_page_size: 200
default_retrieval_chunk_top_k: 20
max_retrieval_chunk_top_k: 100
default_final_top_k: 8
max_final_top_k: 50
max_upload_size: config-driven
interactive_timeout_ms: 30000
worker_task_timeout_ms: config-driven
```

Retrieval must avoid unbounded scans.
High-cardinality filters must map to indexed columns.
Full-text and vector search are read models and can be rebuilt.
Projection stores must enforce bounded batch size even when a higher service layer already paginates requests. Directory document projections by drive node id, wiki projections by drive node id, and wiki projections by logical path are capped at 200 items per call; larger requests must fail fast with a validation error instead of building unbounded `IN (...)` queries.

## 17. Observability Design

Every API request should include or produce:

- traceId
- requestId generated by server
- route template
- operationId
- tenant and organization context when safe
- latency
- status and error code

Worker jobs expose:

- job state
- item state
- attempt count
- stage latency
- parser failures
- embedding failures
- vector index failures
- drive read/write failures

Metrics:

```text
knowledge_api_requests_total
knowledge_api_request_duration_ms
knowledge_ingestion_jobs_total
knowledge_ingestion_job_duration_ms
knowledge_drive_object_operations_total
knowledge_parse_artifacts_total
knowledge_chunks_total
knowledge_embeddings_total
knowledge_retrieval_requests_total
knowledge_retrieval_duration_ms
knowledge_permission_denials_total
```

## 18. Error Model

HTTP errors use RFC 9457 problem details.

Common error codes:

```text
knowledge_space_not_found
knowledge_document_not_found
knowledge_document_version_conflict
knowledge_drive_object_not_found
knowledge_drive_object_integrity_failed
knowledge_upload_too_large
knowledge_unsupported_mime_type
knowledge_ingest_failed
knowledge_parse_failed
knowledge_embedding_failed
knowledge_index_failed
knowledge_retrieval_failed
knowledge_wiki_compile_failed
knowledge_wiki_candidate_not_found
knowledge_wiki_claim_unsupported
knowledge_wiki_export_failed
knowledge_local_mirror_package_failed
knowledge_local_mirror_delta_incompatible
knowledge_local_mirror_delta_checksum_failed
knowledge_local_mirror_delta_apply_failed
knowledge_local_mirror_schema_migration_required
knowledge_permission_denied
knowledge_rate_limited
```

Drive errors map from `DriveObjectStoreErrorKind` to problem details without leaking provider internals.

## 19. Testing and Verification

Required tests before implementation completion:

- Contract tests for OpenAPI app/backend specs.
- SDK generation smoke tests with `--standard-profile sdkwork-v3`.
- SQL schema tests for PostgreSQL and SQLite where supported.
- Repository tests for tenant predicates and index-backed queries.
- Drive adapter tests with fake `DriveObjectStore`.
- Drive adapter integrity tests proving caller-supplied checksums are verified against the request body, blank provider versions are treated as missing, and versionless providers produce deterministic `sha256:<content_digest>` object versions.
- Negative tests proving no direct file storage bypass in knowledgebase code.
- Projection store tests proving directory and wiki metadata batch lookups reject unbounded input before executing SQL.
- Upload/import idempotency tests.
- Parser/embedding/index worker state-machine tests.
- Retrieval tests for keyword, vector, hybrid, rerank, and citations.
- Ingest operation tests for upload, direct drive object import, idempotent retry, failed parse, and reindex.
- Wiki compile tests for source lineage, non-overwrite of approved revisions, claim citation mapping, and drive-backed Markdown artifacts.
- LLM Wiki compatibility tests proving generated spaces include `wiki/schema/AGENTS.md`, `wiki/schema/wiki_schema.yaml`, `wiki/index.md`, `wiki/log.md`, drive-backed raw source objects, and filed answer pages.
- LLM Wiki workflow tests proving ingest updates source summary pages, related pages, `wiki/index.md`, and append-only `wiki/log.md`.
- Query filing tests proving useful answers can be stored under `output/answers/**`, reviewed as candidates, and published as wiki pages with citations and retrieval trace lineage.
- llms.txt export tests proving output is generated from published wiki revisions and stored through drive.
- Wiki lint/evaluation tests for broken links, unsupported claims, stale sources, duplicate pages, orphan pages, missing concept pages, missing cross-references, knowledge gaps, and ACL mismatches.
- Local mirror snapshot tests proving package manifests, SQLite metadata, drive object manifests, indexes, and checksums are complete.
- Local mirror compatibility tests proving the unpacked package opens as a plain Markdown LLM Wiki with root `AGENTS.md`, schema files, raw source projection, `wiki/index.md`, `wiki/log.md`, links, assets, `llms.txt`, and `llms-full.txt`.
- Local delta tests for base-version mismatch, checksum failure, tombstones, idempotent apply, rollback, schema migration requirement, append-only log updates, and schema/index/log atomicity.
- Offline runtime tests proving a packaged mirror can search, read wiki pages, retrieve citations, build context packs, and file local overlay notes without remote service access.
- ACL negative tests for wrong tenant, wrong organization, missing permission, deny override, and deleted document.
- Audit tests for admin mutations and permission changes.

Verification commands will be finalized after scaffolding, but the target gate is:

```powershell
cargo fmt --all --check
cargo test --workspace
node ..\sdkwork-sdk-generator\bin\sdkgen.js generate --help
```

Once OpenAPI files exist, SDK generation checks become mandatory.

## 20. Open Questions

1. Should the first production database require PostgreSQL only, or should SQLite be an officially supported local/private runtime from v0.1?
2. Which embedding provider should be the default test adapter: deterministic local fake only, or a real configurable provider behind a port?
3. Should raw query text be retained in `kb_retrieval_trace`, or only a hash plus safe metadata by default?
4. Should drive folder sync be first-phase or second-phase after direct upload and direct drive object import?
5. Should ACL be document-only in v0.1, or support space, collection, document, and document version immediately?
6. Should wiki compile be enabled for every ingest by default, or require an explicit compile job after document indexing?
7. Should `llms.txt` exports include only approved published wiki pages, or allow draft/private exports for internal agents?
8. Should local mirror packages include raw source files by default, or default to wiki/parsed artifacts only for privacy and package size?
9. Should delta packages be generated eagerly after every publish/index change, or lazily on mirror subscriber demand?
10. Should local vector search use bundled pgvector-compatible data, SQLite vector extension, Tantivy-style local index, or adapter-specific files in v0.1?

## 21. Next Step

After this design is approved:

1. Create an implementation plan under `docs/superpowers/plans/`.
2. Scaffold the Rust workspace.
3. Write failing tests first for the drive storage boundary and database schema contracts.
4. Implement the minimal backend foundation.
5. Generate app/backend OpenAPI and SDK artifacts through `sdkwork-sdk-generator`.
