# Docs

## Purpose

`docs/` contains repository documentation, migration evidence, implementation plans, runbooks, and developer-facing knowledgebase references.

## Owner

SDKWork Knowledgebase maintainers.

## Allowed Content

- Architecture and migration evidence under `docs/superpowers/`.
- Architecture decision records under `docs/adr/`.
- Changelogs under `docs/changelogs/`.
- Runbooks under `docs/runbooks/`.
- Developer and operator documentation.
- Historical implementation notes that are explicitly labeled as history or migration evidence.

## Forbidden Content

- Root SDKWORK standard copies.
- Generated SDK transport output.
- Runtime data, local cache files, secrets, tokens, private certificates, or customer data.
- Active API contracts that belong under `apis/`.

## Related Specs

- `../sdkwork-specs/DOCUMENTATION_SPEC.md`
- `../sdkwork-specs/MIGRATION_SPEC.md`
- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/SOUL.md`

## Verification

Run from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1
powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1
```
