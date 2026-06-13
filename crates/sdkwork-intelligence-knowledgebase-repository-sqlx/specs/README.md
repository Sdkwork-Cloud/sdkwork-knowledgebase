# sdkwork-intelligence-knowledgebase-repository-sqlx

SQLx repository implementations and migration registry for SDKWork Knowledgebase.

This README is the SDKWork module entrypoint for $(System.Collections.Hashtable.Name). The machine-readable component contract is specs/component.spec.json; canonical standards are under ../../../sdkwork-specs/.

Run component verification from the repository root with:

`powershell
cargo test -p sdkwork-intelligence-knowledgebase-repository-sqlx
`
"@ | Set-Content -LiteralPath crates\sdkwork-intelligence-knowledgebase-repository-sqlx\README.md
  @"
# Component Specs

This directory is the local SDKWork component contract for $(System.Collections.Hashtable.Name).

- Component root: sdkwork-knowledgebase/crates/sdkwork-intelligence-knowledgebase-repository-sqlx
- Canonical standards: ../../../sdkwork-specs/README.md
- Machine-readable contract: specs/component.spec.json

Read specs/component.spec.json before changing this component's public exports, runtime entrypoints, SDK clients, generated artifacts, config keys, or verification commands.

Do not copy root standards into this directory. Link to files under ../../../sdkwork-specs/ instead.
