# Repository Guidelines

<!-- SDKWORK-AGENTS-GENERATED: v2 -->

## SDKWORK Soul

Read `../../../sdkwork-specs/SOUL.md` before executing tasks in this app surface. Follow specs before memory, dictionary before context, stop on ambiguity, and evidence before completion.

## SDKWORK Standards

<!-- SDKWORK-PROGRESSIVE-LOADING: v1 -->
Resolve this standards root once and use it as the global authority for the current task:

- `../../../sdkwork-specs/README.md`
- `../../../sdkwork-specs/SOUL.md`
- `../../../sdkwork-specs/AGENTS_SPEC.md`

Read only the relevant README task-matrix row or navigation heading, then load the selected authority sections.
<!-- /SDKWORK-PROGRESSIVE-LOADING: v1 -->

Do not copy root standard text into this directory. If these relative paths do not resolve, stop and report the broken workspace layout.

## Application Identity

This is the PC browser/desktop app surface for the root Knowledgebase application. Read `../../sdkwork.app.config.json` only when the task touches app identity, runtime config, SDK wiring, release metadata, app-owned capabilities, packaging, or deployment.

## Local Dictionary Structure

- `AGENTS.md`: app-surface agent entrypoint and relative SDKWork spec index.
- `package.json`: app-surface scripts and dependencies governed by `PNPM_SCRIPT_SPEC.md`.
- `.env.example`: safe app-surface runtime env template.
- `etc/`: deployable-root source configuration for concrete browser and desktop runtime values.
- `config/`: browser and desktop runtime config examples.
- `packages/`: PC React packages and desktop host package.
- `src/`: app-surface bootstrap, shell entrypoint, and global styles.
- `vite.config.ts`, `tsconfig.json`: TypeScript and Vite build manifests.

## Spec Resolution Order

<!-- SDKWORK-PROGRESSIVE-LOADING: v1 -->
Use dynamic progressive loading for the current task: resolve the selected root and task category before reading broad source context.

1. Read this `AGENTS.md` routing material and classify the owned surface.
2. Read `../../AGENTS.md`, `../../sdkwork.app.config.json`, local package manifests, `specs/`, `etc/`, and `.sdkwork/` only when the task reaches the contract each item governs.
3. Locate only the relevant task-matrix row or navigation heading in `../../../sdkwork-specs/README.md`; do not load the full catalog.
4. Read only the selected task-specific global specs, then inspect implementation files.
<!-- /SDKWORK-PROGRESSIVE-LOADING: v1 -->

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

Build scripts, dev runners, and `pnpm clean` must follow `CODE_STYLE_SPEC.md` section 7 (Build Source Integrity And Self-Healing). Git-tracked build-critical source files must be verified before builds and self-healed from git when missing; `clean` must not delete them.

## Build, Test, and Verification

<!-- SDKWORK-VERIFICATION-ROUTING: v1 -->
Choose only the narrowest verification selected by the changed surface. This is not a default full-suite command list.
Run workspace-wide checks only when the change crosses that boundary.
`bootstrap-*`, `align-*`, `sync-*`, `--write`, and other mutating repair commands are not verification defaults; use them only for an explicitly scoped repair, migration, bootstrap, or alignment task and inspect the resulting diff.
<!-- /SDKWORK-VERIFICATION-ROUTING: v1 -->

Use `pnpm dev`, `pnpm dev:desktop`, `pnpm build`, `pnpm build:desktop`, and `pnpm lint` for app-surface work. From the repository root, use the canonical cross-surface lifecycle commands.

## Agent Execution Rules

<!-- SDKWORK-PROGRESSIVE-LOADING: v1 -->
Use dynamic progressive loading for the current task; treat indexes and cross-references as discovery, not as a startup bundle.
Keep `../../../sdkwork-specs/SOUL.md` and the task-selected standards authoritative; expand context only when evidence exposes a new contract boundary.
Language-specific specs are on-demand: only the touched language loads `../../../sdkwork-specs/RUST_CODE_SPEC.md`, `../../../sdkwork-specs/JAVA_CODE_SPEC.md`, `../../../sdkwork-specs/TYPESCRIPT_CODE_SPEC.md`, or `../../../sdkwork-specs/FRONTEND_CODE_SPEC.md`.
Package command standardization loads `../../../sdkwork-specs/PNPM_SCRIPT_SPEC.md` only when the current task changes package commands or scripts; GitHub packaging work loads `../../../sdkwork-specs/GITHUB_WORKFLOW_SPEC.md` only when it reaches that workflow boundary.
Do not infer a recursive workspace scan or a broad validation suite from the presence of a path alone.
<!-- /SDKWORK-PROGRESSIVE-LOADING: v1 -->

Do not preserve legacy script aliases or local guidance blocks when root SDKWork standards govern the behavior. Do not replace generated SDK integration with raw HTTP. Record exact verification commands and important outputs before reporting completion.

## Task-Specific Standards

API work loads `../../../sdkwork-specs/API_SPEC.md` and its validators. List/search work loads `../../../sdkwork-specs/PAGINATION_SPEC.md` and `check-pagination.mjs`. Source configuration work loads `../../../sdkwork-specs/SOURCE_CONFIG_SPEC.md` and `check-source-config-standard.mjs`. Link these authorities instead of copying their normative bodies into `AGENTS.md`.

## Human Review Rules

Request human review before breaking SDKWork standards, changing public naming, altering security/auth behavior, changing runtime config semantics, changing package/release metadata, deleting data/files, changing generated SDK ownership, or modifying deployment governance. Surface unresolved spec paths, app identity conflicts, component ownership conflicts, and API authority ambiguity instead of guessing.
