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

## Concept rules

- Each concept is one `.md` file with required frontmatter `type`.
- Concept ID = path under `okf/` without `.md` (example: `tables/users`).
- Cross-links use bundle-relative paths such as `/tables/users.md`.

## Storage and API

- File bytes flow only through `sdkwork-drive`.
- Database objects created by SDKWork Knowledgebase use the `kb_` prefix for application-owned tables and indexes.
- SQL stores `kb_okf_*` metadata and stable drive object references.
- HTTP operations use the `okf.*` operationId family.
- Agent provider id: `provider.knowledge.okf`.

## References

- Design: `docs/superpowers/specs/2026-06-19-okf-knowledge-bundle-design.md`
- Contract: `specs/okf-knowledge-bundle.spec.json`
