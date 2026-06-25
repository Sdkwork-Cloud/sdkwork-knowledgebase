# SDKWork OKF Knowledge Bundle

SDKWork Knowledgebase stores curated knowledge as an [Open Knowledge Format (OKF) v0.1](external/knowledge-catalog/okf/SPEC.md) bundle per space.

## Structure

```text
{space}/
├── sources/raw/                 # immutable source files
├── okf/                         # OKF bundle root
│   ├── index.md
│   ├── log.md
│   ├── schema/
│   │   ├── AGENTS.md
│   │   └── okf_profile.yaml
│   └── <domain>/<concept>.md
└── .sdkwork/governance/         # drafts and revisions (never exported)
```

Nested directories may include their own `index.md` files for progressive disclosure (OKF §6).

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

- Design: `docs/superpowers/specs/2026-06-19-okf-knowledge-bundle-design.md`
- Contract: `specs/okf-knowledge-bundle.spec.json`
