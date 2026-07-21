# External Knowledge Engine Catalog

SDKWork Knowledgebase registers third-party knowledge platforms, RAG engines, retrieval
frameworks, and vector stores through the Knowledge Engine SPI. Catalog presence, executable
adapter status, local contract certification, and live commercial certification are separate facts.

## Layout

```text
external/knowledge-engines/
|-- README.md
|-- catalog.manifest.json
|-- provider-certification.manifest.json
|-- vendors/
|   `-- {vendorId}/engine.manifest.json
`-- upstream/
    `-- {vendorId}/
```

`catalog.manifest.json` and each vendor manifest own discovery and capability metadata.
`provider-certification.manifest.json` owns the versioned contract and live-certification matrix.
`docs/releases/provider-certification/live-certification-evidence.schema.json` owns the live
release-evidence record. Its checked-in template is deliberately not a certifiable record.
Optional upstream source pins live under `upstream/{vendorId}` for compliance review, API diffing,
and compatibility work; they are not runtime dependencies.

## Integration Tiers

| Tier | Meaning |
| --- | --- |
| `catalog` | Discovery metadata only; no executable SDKWork adapter |
| `stub` | Non-production adapter skeleton with no advertised executable capability |
| `adapter` | Executable SDKWork adapter with the local versioned contract suite |
| `production` | Adapter plus current live, licensing, security/privacy, and SLO evidence |

An `adapter` result never implies production support. Production promotion is rejected unless the
live evidence required by the certification policy is complete and current.

## Submodule Policy

1. Add every supported engine to the catalog and its vendor manifest.
2. Add an upstream submodule only when local source is required for compliance or compatibility.
3. Use `external/knowledge-engines/upstream/{vendorId}` as the path.
4. Plan and validate pins with `node tools/sync_external_knowledge_engine_submodules.mjs --check`.

## Verification

```bash
node tools/check_external_knowledge_engine_catalog.mjs
node tools/run_provider_certification.mjs
node tools/run_provider_certification.mjs --execute
```

The first certification command checks suite version, six required evidence dimensions, evidence
source fingerprints, safe structured commands, and Provider coverage. `--execute` runs the complete
owned adapter crate for every executable Provider without a shell. A local contract status of
`passed` is not live certification; production tier additionally requires current upstream-version,
licensing, security/privacy, SLO, and environment evidence. The live gate validates the evidence
index digest plus individual quality, contract, load/SLO, outage-recovery, licensing, and
security/privacy artifact digests. The current matrix contains ten local passes and zero live
certifications.

Load/SLO and outage-recovery summaries are not trusted as self-asserted release facts. The live
gate validates their schemas, loads the SHA-256-bound raw request samples and scenario timelines,
recomputes the metrics and recovery intervals, rejects policy weakening and secret-bearing fields,
and binds the evidence to the Provider ID, upstream version, adapter commit, workflow, reviewer,
dashboard, and runbook. Passing validator fixtures prove the gate, not a production Provider run.
Quality and operational evidence share one bounded artifact reader, so symlink/path escape,
oversized evidence, digest mismatch, and future-dated promotion attempts fail closed before parsing.

Normative contracts:

- `specs/external-knowledge-engine-catalog.spec.json`
- `specs/knowledge-engine-spi.spec.json`
- `specs/knowledge-engine-operational-evidence.spec.json`
