> Migrated from `docs/okf-knowledge-bundle.md` on 2026-06-24.
> Owner: SDKWork maintainers

SDKWork Knowledgebase stores curated knowledge as an [Open Knowledge Format (OKF) v0.1](external/knowledge-catalog/okf/SPEC.md) bundle per space.

## Structure

```text
{space}/
|-- sources/raw/                 # immutable original source files
|-- okf/                         # generated OKF bundle root
|   |-- index.md
|   |-- log.md
|   |-- schema/
|   |   |-- AGENTS.md
|   |   `-- okf_profile.yaml
|   `-- <domain>/<concept>.md
|-- output/                      # generated answers, reports, and exports
`-- .sdkwork/governance/         # drafts and revisions (never exported)
```

Nested directories under `okf/` may include their own `index.md` files for progressive disclosure (OKF section 6).

## Browser views

The knowledge browser API exposes bounded Drive views so the UI never has to infer OKF storage layout from raw Drive folders.

| Browser view | Drive root | UI purpose |
| --- | --- | --- |
| `files` | `sources/raw` for OKF spaces; Drive root for non-OKF external spaces | Knowledgebase file list, uploads, folders, imports, and asset scans over original files |
| `okf_bundle` | `okf` | OKF concept and bundle tools that need generated bundle files |
| `outputs` | `output` | Generated answers, reports, export outputs, and operational artifacts |

Frontend knowledgebase file lists MUST call `spaces.browser.list` with `view=files`. For OKF spaces this lists original raw source files under `sources/raw`; it must not show the generated `okf/` bundle tree, `output/`, `.sdkwork/`, or raw Drive root system folders.

Root uploads and root folder creation MUST use `spaces.browser.list(..., view=files)` response `data.parentId` as the Drive `parentNodeId`. Clients must not hard-code `sources/raw`, and they must not upload to Drive root. OKF concept copy, move, and bundle inspection workflows that need generated concepts MUST call `view=okf_bundle` explicitly.

## Concept rules

- Each concept is one `.md` file with required frontmatter `type`.
- Concept ID = path under `okf/` without `.md` (example: `tables/users`).
- Cross-links use bundle-relative paths such as `/tables/users.md`.
- External catalog bundles (for example StackOverflow samples) are canonicalized on import: lowercase segments, dots become underscores.

## Workflows

| Workflow | Purpose |
|----------|---------|
| **ingest** | Read raw sources, upsert concepts, rebuild indexes, append log |
| **compile** | Validate source/bundle, append log, rebuild indexes, refresh schema files |
| **eval** | Lint bundle, append log, rebuild indexes, refresh schema files |
| **query** | Read `index.md` first, retrieve concepts, file approved answers |
| **lint** | Conformance, broken links, orphans, missing citations, stale claims |

Workflow steps are declared in `okf/schema/okf_profile.yaml` and executed through `okf.*` API operations and backend compile/eval jobs.

## Export modes

| Mode | Contents |
|------|----------|
| `okf_strict` (default) | Bundle root view: all hierarchical `index.md`, `log.md`, `schema/*`, published concepts; strips `sdkwork` frontmatter |
| `okf_with_sources` | `okf_strict` plus `raw/` mirror of `sources/raw/` |

Exports include `export_manifest.yaml` for drive-import round-trips and local mirror compatibility.

## Storage and API

- File bytes flow only through `sdkwork-drive`.
- Database objects created by SDKWork Knowledgebase use the `kb_` prefix for application-owned tables and indexes.
- SQL stores `kb_okf_*` metadata and stable drive object references.
- `spaces.browser.list` returns standard list data as `KnowledgeBrowserListData`: `items`, `pageInfo`, `spaceId`, `driveSpaceId`, resolved `parentId`, `view`, and `pageSize`.
- HTTP operations use the `okf.*` operationId family.
- Agent provider id: `provider.knowledge.okf`.

## Observability

Prometheus counters are exposed on API server `/metrics`:

- `kb_okf_concept_publish_total`
- `kb_okf_concept_upsert_total`
- `kb_okf_bundle_lint_issues_total`
- `kb_okf_conformance_failures_total`
- `kb_okf_bundle_import_total`
- `kb_okf_bundle_export_total`

Structured audit log lines use `audit_event` fields: `okf.concept.published`, `okf.concept.upserted`, `okf.bundle.imported`, `okf.bundle.exported`, `okf.bundle.lint.completed`, `knowledge.document.visibility_changed`, `knowledge.space.member_granted`, and `knowledge.space.member_revoked`.

## References

- Design: [TECH-2026-06-19-okf-knowledge-bundle-design.md](TECH-2026-06-19-okf-knowledge-bundle-design.md)
- Contract: `specs/okf-knowledge-bundle.spec.json`
