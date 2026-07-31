# SDKWork Knowledgebase PC Desktop Specs

This directory owns the component contract for the Tauri desktop host package.

- Machine contract: `component.spec.json`
- Global standards: referenced from `component.spec.json#canonicalSpecs`
- Native commands are limited by the Tauri capability allowlist and the command inventory in the
  machine contract. Renderer scripts are `self` only; binary resource reads are bounded to 32 MiB,
  exports to 64 MiB, document source to 16 MiB, and secure values to 256 KiB.
- Remote fetches reject private/special-use networks, pin validated DNS results for every redirect,
  cap redirects and time, and admit at most two concurrent resource operations. Export save and PDF
  rendering each admit one operation and offload blocking file work.
- Verification: use the commands listed in `component.spec.json#verification.commands`
