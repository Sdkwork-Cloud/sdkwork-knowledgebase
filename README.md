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
  sdkwork-knowledgebase-product        Product ports and pure services.
  sdkwork-knowledgebase-storage-sqlx   SQL migration registry and SQLite SQLx repositories.
sdks/
  sdkwork-knowledgebase-app-api        App OpenAPI skeleton.
  sdkwork-knowledgebase-backend-api    Backend OpenAPI skeleton.
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

This checks Rust formatting, workspace tests, and OpenAPI operationId standards.

## Implemented Product Slices

- Knowledge space creation can initialize LLM Wiki standard files through the drive storage port.
- LLM Wiki standard file rendering covers `AGENTS.md`, `wiki_schema.yaml`, `wiki/index.md`, and `wiki/log.md`.
- Ingestion jobs support idempotent creation and basic state transitions.
- API Markdown payload ingest writes `inbox/api/{ingest_id}/payload.md` through `KnowledgeDriveStorage` and rejects empty payloads or unsafe idempotency keys.
- Drive object import verifies the existing sdkwork-drive object with `head_object`, persists a stable `KnowledgeDriveObjectRef`, creates source/document/version metadata through product ports, and creates an idempotent ingest job.
- Source, document, document version, ingest request, and drive import DTOs are available in the contract crate.
- SQL migration skeletons include source, document, document version, ingestion job, and ingestion job item tables for PostgreSQL and SQLite.
- SQLite SQLx repositories persist the drive-import metadata chain: source, document, document version, stable drive object ref, and ingestion job rows.
- SQLite SQLx repositories now persist knowledge spaces and LLM Wiki file entries, so space creation can initialize `wiki/schema/AGENTS.md`, `wiki/schema/wiki_schema.yaml`, `wiki/index.md`, and `wiki/log.md` through the drive port and then mark the space as LLM Wiki initialized.
- Local mirror snapshot and delta manifest services create LLM Wiki-compatible `mirror_manifest.json` and `delta_manifest.json` artifacts, compute SHA-256 checksums, reject unsafe path segments, and persist those manifests only through `KnowledgeDriveStorage`.
- OpenAPI skeletons use SDKWork dotted operation IDs, including `wiki.index.retrieve`, `wiki.log.entries.create`, `driveImports.create`, `documents.versions.create`, and `sources.create`.
