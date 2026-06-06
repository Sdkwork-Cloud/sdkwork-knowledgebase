# SDKWork Knowledgebase

Rust backend foundation for SDKWork Knowledgebase.

This repository is currently implementing backend foundation and early product services. Frontend UI and `apps` integration are intentionally out of scope for the current phase.

## Workspace

```text
crates/
  sdkwork-knowledgebase-contract       Public DTOs, enums, IDs, operation IDs, and LLM Wiki/local mirror contracts.
  sdkwork-knowledgebase-core           Core domain helpers.
  sdkwork-knowledgebase-drive          Adapter to sdkwork-drive storage contracts.
  sdkwork-knowledgebase-test-support   Test fakes and fixtures.
services/
  sdkwork-knowledgebase-app-api        App HTTP API route boundary for generated App SDK operations.
  sdkwork-knowledgebase-backend-api    Backend HTTP API route boundary for generated Backend SDK operations.
  sdkwork-knowledgebase-product        Product ports and pure services.
  sdkwork-knowledgebase-storage-sqlx   SQL migration registry and SQLite SQLx repositories.
sdks/
  sdkwork-knowledgebase-app-sdk        App SDK family, app-api OpenAPI authority, and generated TypeScript SDK.
  sdkwork-knowledgebase-backend-sdk    Backend SDK family, backend-api OpenAPI authority, and generated TypeScript SDK.
```

## Storage Rule

`sdkwork-drive` is the only lower-level file/object storage boundary.

Business logic must not write source files, parsed artifacts, LLM Wiki Markdown, schema files, `wiki/index.md`, `wiki/log.md`, local mirror packages, or delta packages through direct filesystem, S3, OSS, MinIO, Azure Blob, or GCS SDKs.

Product logic depends on `KnowledgeDriveStorage`. Only `crates/sdkwork-knowledgebase-drive` depends on `sdkwork-drive-storage-contract`.

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

## Runtime ID Configuration

All runtime inserts into `kb_*` tables bind explicit service-generated Snowflake IDs. The database must not generate knowledgebase primary keys through SQLite rowid autogeneration, `AUTOINCREMENT`, PostgreSQL `SERIAL`/`BIGSERIAL`, identity columns, or ad hoc repository sequence calls.

`SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID` configures the 10-bit Snowflake node id:

```powershell
$env:SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID = "42"
```

Valid values are `0` through `1023`. Local and test runs may omit the variable and use node id `0`. Production multi-instance deployments must assign a unique node id per process or pod; invalid configured values fail closed during generator initialization.

## Implemented Product Slices

- Knowledge space creation can initialize LLM Wiki standard files through the drive storage port.
- LLM Wiki standard file rendering covers `AGENTS.md`, `wiki_schema.yaml`, `wiki/index.md`, and `wiki/log.md`.
- Ingestion jobs support idempotent creation and basic state transitions.
- API Markdown payload ingest writes `inbox/api/{ingest_id}/payload.md` through `KnowledgeDriveStorage` and rejects empty payloads or unsafe idempotency keys.
- Drive object import verifies the existing sdkwork-drive object with `head_object`, persists a stable `KnowledgeDriveObjectRef`, creates source/document/version metadata through product ports, and creates an idempotent ingest job.
- Source, document, document version, ingest request, and drive import DTOs are available in the contract crate.
- SQL migration skeletons include source, document, document version, ingestion job, and ingestion job item tables for PostgreSQL and SQLite.
- SQLite SQLx repositories persist the drive-import metadata chain: source, document, document version, stable drive object ref, and ingestion job rows. `create_or_get` paths use `kb_*` unique indexes plus insert-first conflict handling so concurrent identical imports reuse existing metadata instead of creating duplicates.
- SQLite SQLx repositories now persist knowledge spaces and LLM Wiki file entries, so space creation can initialize `wiki/schema/AGENTS.md`, `wiki/schema/wiki_schema.yaml`, `wiki/index.md`, and `wiki/log.md` through the drive port and then mark the space as LLM Wiki initialized.
- Wiki page revision numbers and `wiki/log.md` sequence numbers are reserved with database-backed counters (`kb_wiki_page.revision_counter` and `kb_space.wiki_log_sequence_counter`) to avoid `MAX + 1` races under concurrent writes.
- Local mirror snapshot and delta manifest services create LLM Wiki-compatible `mirror_manifest.json` and `delta_manifest.json` artifacts, compute SHA-256 checksums, reject unsafe path segments, and persist those manifests only through `KnowledgeDriveStorage`.
- App and Backend OpenAPI authority files use SDKWork dotted operation IDs, including `wiki.index.retrieve`, `wiki.log.entries.create`, `driveImports.create`, `documents.versions.create`, and `sources.create`.
- Generated App and Backend TypeScript SDKs are produced with the canonical `sdkwork-sdk-generator` using the SDKWork v3 standard profile.
- App and Backend Rust API crates mount every generated OpenAPI operation path and return SDKWork v3 `application/problem+json` errors when an operation has not yet been wired to a concrete product implementation.

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
