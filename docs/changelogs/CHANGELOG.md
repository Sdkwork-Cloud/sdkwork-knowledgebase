# Knowledgebase Changelog

## Unreleased

### Added

- Added the `ADR-20260720` Provider Binding prelaunch readiness read model and one-shot Worker
  command. It is read-only, tenant/organization scoped, opaque-keyset paginated, secret-free, and
  does not infer Provider Bindings from historic source order. Operator procedure:
  `docs/runbooks/RUNBOOK-provider-binding-readiness.md`.
