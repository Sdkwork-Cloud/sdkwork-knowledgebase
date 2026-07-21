# Knowledgebase Changelog

## Unreleased

### Added

- Added versioned Provider load/SLO and outage-recovery evidence schemas, templates, and a shared
  operational evidence policy. Live certification now recomputes results from digest-bound raw
  request samples and outage timelines and rejects policy weakening, unsafe fields, oversized or
  escaped artifacts, future/mismatched dates, threshold violations, retry storms, secret leaks, and
  cross-tenant violations. Quality, operational, and live evidence now reuse one bounded artifact
  reader; production quality datasets are also capped at 5,000 scored and 500 rejection queries.
  These controls do not create production evidence; all live Provider certifications remain pending.
- Added the `ADR-20260720` Provider Binding prelaunch readiness read model and one-shot Worker
  command. It is read-only, tenant/organization scoped, opaque-keyset paginated, secret-free, and
  does not infer Provider Bindings from historic source order. Operator procedure:
  `docs/runbooks/RUNBOOK-provider-binding-readiness.md`.

### Changed

- Reduced the readiness report to stable space identifiers and non-active Binding counts; space
  names, source metadata, remote-resource identifiers, and credential material are excluded.
- Replaced the Provider migration cutover store's positional parameter list with a typed command
  and private SQL transition objects. Atomic cutover, retained predecessor, lease fencing, and
  rollback behavior remain covered by the repository migration tests; no Clippy suppression is
  used for these transition paths.
- Replaced static-message `format!` calls in the AnythingLLM, Flowise, Haystack, Open WebUI, and
  Qdrant adapters. All ten executable Provider crates pass strict all-target Clippy with warnings
  denied.
