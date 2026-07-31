# Component Specs

This directory is the local SDKWork component contract for `sdkwork-intelligence-knowledgebase-repository-sqlx`.

- Component root: `sdkwork-knowledgebase/crates/sdkwork-intelligence-knowledgebase-repository-sqlx`
- Canonical standards: `../../../sdkwork-specs/README.md`
- Machine-readable contract: `specs/component.spec.json`
- Organization ownership inventory: `specs/organization-scope-inventory.json`

Read `specs/component.spec.json` before changing this component's public exports, runtime entrypoints, SDK clients, generated artifacts, config keys, or verification commands.

Do not copy root standards into this directory. Link to files under `../../../sdkwork-specs/` instead.

Run `pnpm check:organization-scope-sql` after changing repository SQL. The check inventories every
organization-owned table in PostgreSQL RLS and rejects DML literals that include `tenant_id` for
those tables without also including `organization_id`.
