# SDKWork Knowledgebase

Rust backend foundation for SDKWork Knowledgebase.

This repository is currently implementing backend foundation and early business services. Frontend UI and `apps` integration are intentionally out of scope for the current phase.

## Workspace

```text
apis/                                      Inactive placeholder for authored API contract sources.
apps/                                      Inactive placeholder; repository root is the primary app root.
crates/
  sdkwork-knowledgebase-contract           Public DTOs, enums, IDs, operation IDs, and LLM Wiki/local mirror contracts.
  sdkwork-knowledgebase-agent-provider     Thin adapter from Knowledgebase retrieval contracts to sdkwork-agent-kernel KnowledgeProvider.
  sdkwork-intelligence-knowledgebase-object-key-service
                                             Object key planning helpers.
  sdkwork-intelligence-knowledgebase-service
                                             Business services, ports, and pure use cases.
  sdkwork-intelligence-knowledgebase-repository-sqlx
                                             SQL migration registry and SQLite SQLx repositories.
  sdkwork-router-knowledgebase-app-api     App HTTP route boundary for generated App SDK operations.
  sdkwork-router-knowledgebase-backend-api Backend HTTP route boundary for generated Backend SDK operations.
  sdkwork-knowledgebase-drive              Adapter to sdkwork-drive storage contracts.
  sdkwork-knowledgebase-memory             Adapter from Knowledgebase context packs to sdkwork-memory SPI.
  sdkwork-knowledgebase-test-support       Test fakes and fixtures.
sdks/
  sdkwork-knowledgebase-app-sdk            App SDK family, app-api OpenAPI authority, and generated TypeScript SDK.
  sdkwork-knowledgebase-backend-sdk        Backend SDK family, backend-api OpenAPI authority, and generated TypeScript SDK.
jobs/ plugins/ examples/ configs/ deployments/ scripts/ tests/
                                             Inactive standard root capability placeholders.
```

This repository root is the primary SDKWork Knowledgebase application root and owns `sdkwork.app.config.json`. The `apps/` directory is intentionally a tracked placeholder for future secondary app surfaces.

This workspace follows the standard project root directory dictionary from `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`.

### apis/ vs sdks/

`apis/` is the standard project-root directory for authored API contracts and API review inputs. `sdks/` contains SDK family workspaces, materialized authority OpenAPI, derived `sdkgen` inputs, and generated SDK output. Currently `apis/` is an inactive placeholder; authority OpenAPI files are materialized under `sdks/`.

### plugins/ vs .sdkwork/plugins/

`plugins/` stores application/runtime plugin source packages. `.sdkwork/plugins/` stores repository/application agent plugin workspaces. They are distinct directories with different purposes.

### configs/ vs runtime config

`configs/` stores source-controlled safe config templates, schemas, profile examples, and non-secret defaults. Runtime user/private config is governed by `RUNTIME_DIRECTORY_SPEC.md` and must not be committed.
## Storage Rule

`sdkwork-drive` is the only lower-level file/object storage boundary.

Business logic must not write source files, parsed artifacts, LLM Wiki Markdown, schema files, `wiki/index.md`, `wiki/log.md`, local mirror packages, or delta packages through direct filesystem, S3, OSS, MinIO, Azure Blob, or GCS SDKs.

Business logic depends on `KnowledgeDriveStorage`. Only `crates/sdkwork-knowledgebase-drive` depends on `sdkwork-drive-storage-contract`.

## Memory Rule

`sdkwork-memory` is the external memory context boundary.

Context pack assembly depends on the `KnowledgeMemoryContextProvider` port. Only `crates/sdkwork-knowledgebase-memory` adapts that port to `sdkwork-memory-spi`; retrieval and API crates must not call Memory HTTP APIs or copy Memory SDK DTOs. Memory context is returned as `memoryFragments` and remains separate from knowledge `fragments`, so Memory entries are not treated as knowledge chunks or citations.

## LLM Wiki Standard Files

Phase 1 establishes renderers and contracts for:

```text
wiki/schema/AGENTS.md
wiki/schema/wiki_schema.yaml
wiki/index.md
wiki/log.md
```

Local mirror contracts expose:

```text
AGENTS.md
schema/**
raw/**
wiki/**
llms.txt
llms-full.txt
```

## Verification

Run:

```powershell
powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1
```

This checks Rust formatting, workspace tests, OpenAPI operationId standards, SDK family layout, and canonical `sdkwork-sdk-generator` usage.

## Runtime

Phase 1 ships SQLite-backed HTTP runtimes in `crates/sdkwork-router-knowledgebase-app-api`:

| Binary | Default listen | Surface |
|--------|----------------|---------|
| `sdkwork-knowledgebase-app-api` | `127.0.0.1:18081` | App API (34 operations) |
| `sdkwork-knowledgebase-backend-api` | `127.0.0.1:18082` | Backend API (25 operations) |
| `sdkwork-knowledgebase-open-api` | `127.0.0.1:18083` | Open API (8 operations) |

Run from the repository root:

```powershell
cargo run -p sdkwork-router-knowledgebase-app-api --bin sdkwork-knowledgebase-app-api
cargo run -p sdkwork-router-knowledgebase-app-api --bin sdkwork-knowledgebase-backend-api
cargo run -p sdkwork-router-knowledgebase-app-api --bin sdkwork-knowledgebase-open-api
```

Common environment variables:

- `SDKWORK_KNOWLEDGEBASE_DATABASE_URL` (default `sqlite://data/knowledgebase.db?mode=rwc`)
- `SDKWORK_KNOWLEDGEBASE_TENANT_ID` (default `1`)
- `SDKWORK_KNOWLEDGEBASE_ACTOR_ID` / `SDKWORK_KNOWLEDGEBASE_OPERATOR_ID` (optional dev actor)
- `SDKWORK_KNOWLEDGEBASE_DRIVE_STORAGE_ROOT` (default `data/drive-objects`)
- `SDKWORK_KNOWLEDGEBASE_APP_LISTEN` / `SDKWORK_KNOWLEDGEBASE_BACKEND_LISTEN` / `SDKWORK_KNOWLEDGEBASE_OPEN_LISTEN`

Local development injects request context through `dev_auth` middleware. Production deployments must replace this with Appbase-backed authentication.

## Runtime ID Configuration

All runtime inserts into `kb_*` tables bind explicit service-generated Snowflake IDs. The database must not generate knowledgebase primary keys through SQLite rowid autogeneration, `AUTOINCREMENT`, PostgreSQL `SERIAL`/`BIGSERIAL`, identity columns, or ad hoc repository sequence calls.

`SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID` configures the 10-bit Snowflake node id:

```powershell
$env:SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID = "42"
```

Valid values are `0` through `1023`. Local and test runs may omit the variable and use node id `0`. Production multi-instance deployments must assign a unique node id per process or pod; invalid configured values fail closed during generator initialization.

## Implemented Service Slices

- Knowledge space creation can initialize LLM Wiki standard files through the drive storage port.
- LLM Wiki standard file rendering covers `AGENTS.md`, `wiki_schema.yaml`, `wiki/index.md`, and `wiki/log.md`.
- Ingestion jobs support idempotent creation and basic state transitions.
- API Markdown payload ingest writes `inbox/api/{ingest_id}/payload.md` through `KnowledgeDriveStorage` and rejects empty payloads or unsafe idempotency keys.
- Drive object import verifies the existing sdkwork-drive object with `head_object`, persists a stable `KnowledgeDriveObjectRef`, creates source/document/version metadata through service ports, and creates an idempotent ingest job.
- Source, document, document version, ingest request, and drive import DTOs are available in the contract crate.
- RAG retrieval, context pack, citation, retrieval trace, knowledge-agent profile, and knowledge-agent binding DTOs are available in the contract crate.
- Context packs can include bounded `sdkwork-memory` context through an injected Memory provider and keep Memory fragments separate from retrieved knowledge chunks.
- SQL migration skeletons include source, document, document version, ingestion job, and ingestion job item tables for PostgreSQL and SQLite.
- SQL migrations include chunk, index, embedding, retrieval profile, retrieval trace, retrieval hit, knowledge-agent profile, and knowledge-agent knowledge binding tables for PostgreSQL and SQLite.
- SQLite SQLx repositories persist the drive-import metadata chain: source, document, document version, stable drive object ref, and ingestion job rows. `create_or_get` paths use `kb_*` unique indexes plus insert-first conflict handling so concurrent identical imports reuse existing metadata instead of creating duplicates.
- SQLite SQLx repositories now persist knowledge spaces and LLM Wiki file entries, so space creation can initialize `wiki/schema/AGENTS.md`, `wiki/schema/wiki_schema.yaml`, `wiki/index.md`, and `wiki/log.md` through the drive port and then mark the space as LLM Wiki initialized.
- Wiki page revision numbers and `wiki/log.md` sequence numbers are reserved with database-backed counters (`kb_wiki_page.revision_counter` and `kb_space.wiki_log_sequence_counter`) to avoid `MAX + 1` races under concurrent writes.
- Local mirror snapshot and delta manifest services create LLM Wiki-compatible `mirror_manifest.json` and `delta_manifest.json` artifacts, compute SHA-256 checksums, reject unsafe path segments, and persist those manifests only through `KnowledgeDriveStorage`.
- App and Backend OpenAPI authority files use SDKWork dotted operation IDs, including `wiki.index.retrieve`, `wiki.log.entries.create`, `driveImports.create`, `documents.versions.create`, and `sources.create`.
- Generated App and Backend TypeScript SDKs are produced with the canonical `sdkwork-sdk-generator` using the SDKWork v3 standard profile.
- App and Backend SDK families declare Appbase, Drive, and Memory dependency SDKs; dependency-owned Appbase, Drive, and Memory APIs are not generated into knowledgebase transports.
- App and Backend Rust API crates mount every generated OpenAPI operation path. The hosted SQLite runtime (`KnowledgebaseSqliteRuntime`) wires all 67 operations to concrete service implementations; trait default stubs in route crates remain only for library-only injection tests.
- The agent provider crate exposes `provider.knowledge.sdkwork-knowledgebase` as a typed `sdkwork-agent-kernel::KnowledgeProvider` adapter backed by an injected retrieval client.

## SDKWork Documentation Contract

Domain: intelligence
Capability: knowledgebase
Package type: rust-crate
Status: standard

### Public API

Public exports are declared in `specs/component.spec.json` under `contracts.publicExports`.

### Required SDK Surface

- None declared in `specs/component.spec.json`.

### Configuration

Configuration keys and runtime entrypoints are declared in `specs/component.spec.json`.

### SaaS/Private/Local Behavior

This module follows the canonical standards linked from `specs/component.spec.json`, including deployment and runtime configuration rules where applicable.

### Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module.

### Extension Points

Extension points are limited to declared public exports, runtime entrypoints, SDK clients, events, and config keys.

### Verification

- `cargo test`

### Owner And Status

Owner and lifecycle status are tracked in `specs/component.spec.json`.
