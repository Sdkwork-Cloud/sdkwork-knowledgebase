> Owner: SDKWork maintainers  
> Status: **completed** — verified by `pnpm check`, `tools/verify_sdkwork_structure.ps1`, and `cargo test --workspace`.

**Goal:** Bring `sdkwork-knowledgebase` to the current `sdkwork-specs` repository structure and Rust naming standards without preserving legacy package identities.

**Outcome:** All authored Rust workspace members live under `crates/` with responsibility-specific names, route manifest evidence is materialized under `sdks/_route-manifests/`, and structure verification is wired into `tools/verify_phase1.ps1`.

**Verification:**

```bash
pnpm check
powershell -ExecutionPolicy Bypass -File tools/verify_sdkwork_structure.ps1
cargo test --workspace
```

## Completed tasks

- Structure verifier (`tools/verify_sdkwork_structure.ps1`) and `verify_phase1.ps1` integration
- Migration from `services/` to `crates/sdkwork-routes-knowledgebase-*` and intelligence crates
- Package rename to `sdkwork-intelligence-knowledgebase-*` and `sdkwork-routes-knowledgebase-*`
- SDKWork dictionary, component specs, and root README alignment
- Route manifest modules and JSON artifacts under `sdks/_route-manifests/`
- Legacy package name purge from active source, config, and test files
