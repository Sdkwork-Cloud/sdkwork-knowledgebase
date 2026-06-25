# Crates

## Purpose

`crates/` contains authored Rust workspace members for SDKWork Knowledgebase.

## Owner

SDKWork Knowledgebase maintainers.

## Allowed Content

- Rust route crates named `sdkwork-router-<capability>-<surface>`.
- Rust service crates named `sdkwork-<domain>-<capability>-service`.
- Rust SQLx repository crates named `sdkwork-<domain>-<capability>-repository-sqlx`.
- Rust adapter, test-support, and integration crates with focused component specs.

## Forbidden Content

- Legacy `services/` Rust package roots.
- Generated SDK transport output.
- Runtime data, secrets, caches, or local build artifacts.
- Compatibility wrapper crates that preserve forbidden legacy package names.

## Related Specs

- `../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`
- `../sdkwork-specs/RUST_CODE_SPEC.md`
- `../sdkwork-specs/NAMING_SPEC.md`
- `../sdkwork-specs/COMPONENT_SPEC.md`

## Verification

Run from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1
cargo metadata --no-deps
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --tests -- -D warnings
```
