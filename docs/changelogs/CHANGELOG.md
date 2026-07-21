# Knowledgebase Changelog

## Unreleased

### Added

- Added `sdkwork-knowledgebase-provider-secret-adapter` and a typed Provider credential access
  context. Development/test sources are restricted to the Knowledgebase and implementation
  environment namespace or canonical files under a per-Provider root; staging/production use managed `secret://`
  references only. Executable tests cover unrelated and cross-Provider locator rejection, root and
  symlink escape, production fail-closed policy, complete context propagation, bounded results,
  one total time budget, a concurrency bulkhead that contains timed-out blocking calls, intermediate
  plaintext cleanup, sanitized errors, and immediate rotation/revocation without a cache.
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

- Removed raw Provider credential parsing from the app route crate. `KnowledgebaseRuntime` now
  injects the resolver, exposes an explicit managed-resolver construction path, and rejects default
  staging/production construction when no approved managed resolver is supplied. A concrete
  production Vault/KMS backend and live operational drills remain release gates.
- Reduced the readiness report to stable space identifiers and non-active Binding counts; space
  names, source metadata, remote-resource identifiers, and credential material are excluded.
- Replaced the Provider migration cutover store's positional parameter list with a typed command
  and private SQL transition objects. Atomic cutover, retained predecessor, lease fencing, and
  rollback behavior remain covered by the repository migration tests; no Clippy suppression is
  used for these transition paths.
- Replaced static-message `format!` calls in the AnythingLLM, Flowise, Haystack, Open WebUI, and
  Qdrant adapters. All ten executable Provider crates pass strict all-target Clippy with warnings
  denied.
