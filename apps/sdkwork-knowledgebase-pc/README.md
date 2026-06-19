# SDKWork Knowledgebase PC

PC browser and optional Tauri desktop surface for SDKWork Knowledgebase.

Architecture: `../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`, `../../sdkwork-specs/APP_PC_REACT_UI_SPEC.md`.

## Packages

| Package | Role |
| --- | --- |
| Root Vite app | PC shell, routing, and feature module composition |
| `packages/sdkwork-knowledgebase-pc-search` | Search and media viewer module |
| `packages/sdkwork-knowledgebase-pc-desktop` | Tauri desktop host |

## Prerequisites

- Node.js 22+
- pnpm 10+
- Rust toolchain (for desktop builds and backend dev orchestration)

## Local development

From the repository root:

```powershell
pnpm dev:browser
pnpm dev:desktop
```

Or from this directory:

```powershell
pnpm install
cp .env.example .env.local
pnpm dev
```

Topology env keys are defined in `../../configs/topology/` and mirrored in `.env.example`.

## SDK integration

- App API: `@sdkwork/knowledgebase-app-sdk` (generated from `../../sdks/sdkwork-knowledgebase-app-sdk/`)
- Auth: `@sdkwork/auth-pc-react`, `@sdkwork/auth-runtime-pc-react` (appbase IAM)
- Do not call backend HTTP APIs directly from app packages.

## Verification

```powershell
pnpm lint
pnpm --dir ../.. typecheck
```
