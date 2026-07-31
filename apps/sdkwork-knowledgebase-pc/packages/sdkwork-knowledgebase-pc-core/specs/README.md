# SDKWork Knowledgebase PC Core Specs

This directory owns the component contract for `sdkwork-knowledgebase-pc-core`.

- Machine contract: `component.spec.json`
- Global standards: referenced from `component.spec.json#canonicalSpecs`
- The `./host` port is the only renderer-to-native bridge. Its command and event names are a
  compile-time allowlist; browser fallbacks are explicit per operation and never emulate native
  secure storage or local filesystem authority.
- Renderer binary resources are capped at 32 MiB and outbound export payloads at 64 MiB before IPC.
- Verification: use the commands listed in `component.spec.json#verification.commands`
