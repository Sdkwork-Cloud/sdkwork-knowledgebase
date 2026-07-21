# Technical Architecture Directory

Proposed Live Wiki provider architecture: [TECH-live-wiki-resource-provider.md](TECH-live-wiki-resource-provider.md).

This directory owns the technical architecture Canon for the repository.

## Fixed Entry

- [TECH_ARCHITECTURE.md](TECH_ARCHITECTURE.md) — required entry document. Keep summary, status, and links here.
- [TECH-topology-standard.md](TECH-topology-standard.md) — runtime topology adoption for this application root.

## Splitting Rules

- Split large architecture content into sibling shards named `TECH-<kebab-topic>.md`.
- Every shard `MUST` be linked from `TECH_ARCHITECTURE.md`.
- Do not create competing architecture roots such as `docs/architecture/TECH_ARCHITECTURE.md`; that path is retired and redirect-only.

See `DOCUMENTATION_SPEC.md` section 2.2.
