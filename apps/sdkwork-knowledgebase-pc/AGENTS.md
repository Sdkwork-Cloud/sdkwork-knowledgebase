# Repository Guidelines

<!-- SDKWORK-AGENTS-GENERATED: v2 -->

## SDKWORK Soul

Read `../../../sdkwork-specs/SOUL.md` before executing tasks in this app surface. Follow specs before memory, dictionary before context, stop on ambiguity, and evidence before completion.

## SDKWORK Standards

Canonical SDKWORK specs path from this app surface:

- `../../../sdkwork-specs/README.md`
- `../../../sdkwork-specs/SOUL.md`
- `../../../sdkwork-specs/AGENTS_SPEC.md`
- `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`
- `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`
- `../../../sdkwork-specs/CODE_STYLE_SPEC.md`
- `../../../sdkwork-specs/NAMING_SPEC.md`

Do not copy root standard text into this directory. If these relative paths do not resolve, stop and report the broken workspace layout.

## Application Identity

This is the PC browser/desktop app surface for the root Knowledgebase application. Read `../../sdkwork.app.config.json` only when the task touches app identity, runtime config, SDK wiring, release metadata, app-owned capabilities, packaging, or deployment.

## Local Dictionary Structure

- `AGENTS.md`: app-surface agent entrypoint and relative SDKWork spec index.
- `package.json`: app-surface scripts and dependencies governed by `PNPM_SCRIPT_SPEC.md`.
- `.env.example`: safe app-surface runtime env template.
- `config/`: browser and desktop runtime config examples.
- `packages/`: PC React packages and desktop host package.
- `src/`: app-surface bootstrap, shell entrypoint, and global styles.
- `vite.config.ts`, `tsconfig.json`: TypeScript and Vite build manifests.

## Spec Resolution Order

Use dynamic progressive loading:

1. Read this `AGENTS.md` and any nearer component-level `AGENTS.md`.
2. Read `../../AGENTS.md` only when repository-root rules or scripts are needed.
3. Read `../../sdkwork.app.config.json` only when app behavior, runtime config, SDK wiring, release, packaging, or app-owned capabilities are touched.
4. Read local package manifests and config examples only for the affected package/surface.
5. Read `../../../sdkwork-specs/README.md`, then only the task-specific root specs.
6. Inspect implementation files after the dictionary and relevant specs are clear.

Do not load the whole app surface or every root spec before identifying the task surface.

## Required Specs By Task Type

- Agent/workflow changes: `../../../sdkwork-specs/SOUL.md`, `../../../sdkwork-specs/AGENTS_SPEC.md`, `../../../sdkwork-specs/SDKWORK_WORKSPACE_SPEC.md`, `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`, and `../../../sdkwork-specs/TEST_SPEC.md`.
- Package script changes: `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md`, `../../../sdkwork-specs/APP_RUNTIME_TOPOLOGY_SPEC.md`, `../../../sdkwork-specs/CONFIG_SPEC.md`, and `../../../sdkwork-specs/TEST_SPEC.md`.
- Any code change: `../../../sdkwork-specs/CODE_STYLE_SPEC.md`, `../../../sdkwork-specs/NAMING_SPEC.md`, plus only the touched language/framework spec.
- TypeScript/Node code: `../../../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md`.
- Frontend/UI code: `../../../sdkwork-specs/FRONTEND_CODE_SPEC.md`, `../../../sdkwork-specs/FRONTEND_SPEC.md`, `../../../sdkwork-specs/UI_ARCHITECTURE_SPEC.md`, `../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`, and `../../../sdkwork-specs/APP_PC_REACT_UI_SPEC.md`.
- Desktop host or Tauri/native code: `../../../sdkwork-specs/APP_PC_ARCHITECTURE_SPEC.md`, `../../../sdkwork-specs/DESKTOP_APP_ARCHITECTURE_SPEC.md`, `../../../sdkwork-specs/CONFIG_SPEC.md`, and the touched language spec.
- SDK integration changes: `../../../sdkwork-specs/APP_SDK_INTEGRATION_SPEC.md`, `../../../sdkwork-specs/SDK_SPEC.md`, `../../../sdkwork-specs/SDK_WORKSPACE_GENERATION_SPEC.md`, and `../../../sdkwork-specs/TEST_SPEC.md`.
- Runtime/deployment/release changes: `../../../sdkwork-specs/CONFIG_SPEC.md`, `../../../sdkwork-specs/ENVIRONMENT_SPEC.md`, `../../../sdkwork-specs/DEPLOYMENT_SPEC.md`, `../../../sdkwork-specs/RELEASE_SPEC.md`, and `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md`.

Language-specific specs are on-demand; do not load Rust, Java, TypeScript, and frontend specs for unrelated tasks.

## Code Style Rules

Read `../../../sdkwork-specs/CODE_STYLE_SPEC.md` and `../../../sdkwork-specs/NAMING_SPEC.md` before code changes. Keep package boundaries focused. App packages use generated app SDKs or approved app SDK wrappers; user-facing packages must not call backend-admin APIs directly or replace generated SDK integration with raw HTTP.

## Build, Test, and Verification

Run app-surface commands from this directory only when the task is limited to this surface:

- `pnpm dev`: browser renderer development.
- `pnpm dev:desktop`: desktop renderer/native host development through the standard action-first script.
- `pnpm build`: browser bundle build.
- `pnpm build:desktop`: desktop package build through the standard action-first script.
- `pnpm lint`: TypeScript check.

From the repository root, use `pnpm dev:browser`, `pnpm dev:desktop`, `pnpm build`, `pnpm test`, `pnpm check`, and `pnpm verify` for cross-surface validation.

## Agent Execution Rules

Use the convention dictionary before broad source loading. Follow dynamic progressive loading: nearest AGENTS, relevant manifests/specs, task-specific root standards, then implementation. Do not preserve legacy script aliases or local guidance blocks when root SDKWork standards already govern the behavior. Do not replace generated SDK integration with raw HTTP. Record exact verification commands and important outputs before reporting completion.

## Human Review Rules

Request human review before breaking SDKWork standards, changing public naming, altering security/auth behavior, changing runtime config semantics, changing package/release metadata, deleting data/files, changing generated SDK ownership, or modifying deployment governance. Surface unresolved spec paths, app identity conflicts, component ownership conflicts, and API authority ambiguity instead of guessing.
