# Tools

## Purpose

`tools/` contains repository-local validation, SDK/API checks, migration checks, and operator/developer tooling that is not shipped as application runtime code.

## Owner

SDKWork Knowledgebase maintainers.

## Allowed Content

- PowerShell, Node, or Rust verification tools.
- Deterministic migration or standards validation scripts.
- Reusable generators or analyzers that are invoked from repository verification.

## Forbidden Content

- Runtime application implementation code.
- Generated SDK transport output.
- Local caches, secrets, environment overrides, tokens, or machine-specific absolute paths.
- Thin user command wrappers that belong in `scripts/` unless they contain reusable validation logic.

## Related Specs

- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/TEST_SPEC.md`
- `../sdkwork-specs/DEPENDENCY_MANAGEMENT_SPEC.md`
- `../sdkwork-specs/CODE_STYLE_SPEC.md`

## Verification

Run from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1
powershell -ExecutionPolicy Bypass -File tools/verify_phase1.ps1
```
