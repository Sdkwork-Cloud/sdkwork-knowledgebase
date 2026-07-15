# ADR-20260713-group-knowledgebase-binding-and-launch

Status: accepted
Requirement: REQ-2026-0713
Owner: sdkwork-knowledgebase and sdkwork-im maintainers
Date: 2026-07-13
Specs: ARCHITECTURE_DECISION_SPEC.md, API_SPEC.md, SDK_SPEC.md, DATABASE_SPEC.md, MIGRATION_SPEC.md, IAM_SPEC.md, SECURITY_SPEC.md, PRIVACY_SPEC.md, APP_PC_ARCHITECTURE_SPEC.md, DESKTOP_APP_ARCHITECTURE_SPEC.md

## Context

The IM PC group chat uses an IM Conversation aggregate as its stable group identity. A legacy
space-group `group_id` is a different aggregate and cannot prove a one-to-one relationship with
every conversation. The existing generic Knowledgebase context-binding model allows more than one
space per context, has no organization scope, provisioning lease, roster epoch, or event replay
semantics, and cannot independently verify live IM membership.

The previous host-embed surface is a generic reusable UI capability. It cannot safely select or
authorize a managed group space, and an IM-owned iframe or native Webview does not satisfy the
requirement to open the full independent Knowledgebase application.

## Decision

1. Knowledgebase owns `kb_group_knowledge_space_binding` as the only authoritative group-space
   relation. Its key is `(tenant_id, organization_id, conversation_id)`, and it owns lifecycle,
   provision lease, source-event inbox, roster epoch, and ACL projection state.
2. Generic context-binding mutation for `chat_group` is rejected. Generic space query and mutation
   paths exclude group-managed spaces so no second authority or selector leakage exists.
3. IM calls a trusted ensure/synchronize/archive/restore service boundary using generated SDK/RPC
   integration. Only the current IM Owner may issue ensure or retry a failed provisioning; Admin
   and Member requests in absent or failed lifecycle state are denied by IM before this boundary.
   The browser never calls the internal boundary and never supplies actor, scope, role, or target
   space values.
4. Reservation creates only a hidden provisioning space plus binding. Knowledgebase initializes
   Drive and required space resources, projects direct user grants from the IM role snapshot, then
   atomically publishes the active lifecycle only when the ACL projection is active. Group content
   access requires both the synchronized role snapshot and direct Drive authorization.
5. IM stores a local projection/saga and a SHA-256 ticket ledger. Its ticket is short-lived,
   one-time, actor/scope/version/epoch bound, and is consumed through a trusted internal IM
   delegation after Knowledgebase has authenticated the interactive user session.
6. Browser launch uses a fragment-only ticket and a full standalone Knowledgebase route. Desktop
   launch uses a strict allowlisted deep link handled by the independent single-instance
   Knowledgebase Tauri process. The route's public base path is owned by Knowledgebase runtime
   config; IM only uses the configured public application URL.
7. Initialization and active-content access are separate. The current IM Owner alone initializes
   or retries failed provisioning. Once active, joined non-Guest Owner, Admin, and Member roles
   receive the mapped content access; Guest, left, removed, and non-member actors are denied.
   Roster and role changes synchronize/revoke grants; dissolve archives; explicit delete remains
   deleted until an explicit Owner restore/recreate command. No click silently recreates a deleted
   space.

## Alternatives

1. **Provision on group creation**: rejected because it produces unused/orphaned spaces.
2. **Use `im_chat_groups.group_id`**: rejected because it is not the Conversation group identity.
3. **Treat generic `chat_group` context binding as authority**: rejected because its cardinality,
   scope, and lifecycle cannot enforce the managed invariant.
4. **Render the group workspace in IM iframe/Webview**: rejected because it weakens process and
   authorization separation and is not the complete standalone Knowledgebase application.
5. **Pass a space id, token, or role in the launch URL/deep link**: rejected because it exposes
   authority in browser history, logs, app activation, or persistent local state.
6. **Trust only an asynchronous ACL projection**: rejected because removal and role downgrade
   require immediate denial; the managed resolver requires current snapshot and direct ACL checks.

## Consequences

- Knowledgebase gains a dedicated binding/repository/service boundary and RLS-protected tables.
- IM and Knowledgebase release compatible contracts together, but each system retains its own
  source of record and database ownership.
- Existing generic host embedding remains available only for unrelated generic integrations. It is
  explicitly not a managed group knowledgebase entrypoint and must not receive group launch data.
- Failed provisioning is retryable and observable. Archive is default on dissolve; destructive
  cleanup requires an explicit lifecycle command and retention policy.
- The capability requires health/readiness checks for the cross-service dependency, ticket
  consumption, Drive/ACL projection, and event retry. This decision does not assert deployment or
  release completion.

## Verification

Focused verification covers concurrent current-Owner reservation and retry, non-Owner denial before
activation, generic-binding rejection, provisioning visibility, active joined non-Guest role
mapping, guest/left denial, removal/role downgrade during ticket use, Drive/ACL failure, ticket
theft/replay/expiry, exact fixed workspace selection, fragment redaction, base-path routing, strict
deep links, and lifecycle archive/restore behavior. Repository verification also runs API
envelope/pattern, SDK materialization, schema/RLS, Rust, TypeScript, desktop, and browser gates.

## Supersedes / Superseded By

- Supersedes host-embed group identifiers and IM-owned Knowledgebase windows for managed group
  knowledge bases.
- Superseded by: none.
