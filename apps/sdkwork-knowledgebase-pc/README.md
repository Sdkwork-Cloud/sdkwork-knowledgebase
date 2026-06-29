# SDKWork Knowledgebase PC

PC browser and optional Tauri desktop surface for SDKWork Knowledgebase.

Architecture: `../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`, `../../sdkwork-specs/APP_PC_REACT_UI_SPEC.md`.

Launch acceptance: `../../docs/product/prd/PRD-mvp-launch.md`.

## Packages

| Package | Role |
| --- | --- |
| Root Vite app (`src/`) | Bootstrap, IAM shell, routing |
| `packages/sdkwork-knowledgebase-pc-core` | Session, runtime config, SDK registry, auth gate |
| `packages/sdkwork-knowledgebase-pc-commons` | Shared UI utilities, sanitizers, error boundaries |
| `packages/sdkwork-knowledgebase-pc-shell` | App shell, settings, global navigation |
| `packages/sdkwork-knowledgebase-pc-knowledgebase` | Editor, drive browser, WeChat, market |
| `packages/sdkwork-knowledgebase-pc-search` | Search and media viewer module |
| `packages/sdkwork-knowledgebase-pc-desktop` | Tauri desktop host |
| `packages/sdkwork-knowledgebase-pc-knowledge` | Host-managed embed surface (optional; not wired into default shell) |

Import rule: use package names (`@sdkwork/sdkwork-knowledgebase-pc-*`). Do not use `@packages/` deep paths or raw HTTP for business APIs.

## Config examples

| File | Profile |
| --- | --- |
| `config/browser/runtime-env.development.example.json` | Local browser dev |
| `config/browser/runtime-env.staging.example.json` | Staging (`enableDemoMode: false`) |
| `config/browser/runtime-env.production.example.json` | Production (`enableDemoMode: false`) |
| `config/desktop/sdkwork-knowledgebase-pc.development.toml.example` | Desktop dev host |
| `config/desktop/sdkwork-knowledgebase-pc.production.toml.example` | Desktop production host |

Repository-root `.env.postgres` supplies development database and IAM signing secrets (copy from `../../.env.postgres.example`).

## Prerequisites

- Node.js 22+
- pnpm 10+
- Rust toolchain (desktop builds and backend dev orchestration)

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

## SDK integration

- App API: `@sdkwork/knowledgebase-app-sdk` (composed facade over `../../sdks/sdkwork-knowledgebase-app-sdk/`)
- Drive: `@sdkwork/drive-app-sdk`
- Auth: `@sdkwork/auth-pc-react`, `@sdkwork/auth-runtime-pc-react` (Appbase IAM)
- Do not call backend HTTP APIs directly from app packages.

Production builds must not rely on demo/mock fallbacks (`VITE_SDKWORK_KNOWLEDGEBASE_ENABLE_DEMO_MODE=false` or unset in production).

## Verification

From repository root:

```powershell
pnpm lint
pnpm test:frontend
pnpm check:pc-app-hygiene
pnpm test:e2e:playwright
```

From this directory:

```powershell
pnpm lint
pnpm test:e2e
```

## Hygiene

One-off migration scripts (`update_*.cjs`, `fix_*.js`, etc.) are forbidden at the app root. Use package tests, i18n tooling, and standard build scripts only. Enforced by `../../tools/check_pc_app_hygiene.mjs`.
