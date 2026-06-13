# sdkwork-knowledgebase-memory

Domain: intelligence
Capability: knowledgebase memory adapter
Package type: rust-crate
Status: standard

## Public API

Adapts the `KnowledgeMemoryContextProvider` port to `sdkwork-memory-spi` for context pack assembly.

## Required SDK Surface

- `sdkwork-memory-spi` (workspace dependency)

## Configuration

Memory context provider is injected through service ports. No crate-local configuration keys.

## SaaS/Private/Local Behavior

This module follows the canonical standards linked from `specs/component.spec.json`.

## Security

Do not add secrets, live tokens, manual auth headers, or app-local credential handling to this module.

## Extension Points

Extension points are limited to declared public exports and the injected memory context provider port.

## Verification

```powershell
cargo test -p sdkwork-knowledgebase-memory
```

## Owner And Status

Owner: SDKWork Knowledgebase maintainers.
Status: standard.
