# REQ-2026-0713-group-knowledgebase

Status: ready
Owner: sdkwork-knowledgebase and sdkwork-im maintainers
Source: managed IM Conversation group knowledgebase capability
Specs: REQUIREMENTS_SPEC.md, API_SPEC.md, SDK_SPEC.md, DATABASE_SPEC.md, IAM_SPEC.md, SECURITY_SPEC.md, PRIVACY_SPEC.md, APP_PC_ARCHITECTURE_SPEC.md, DESKTOP_APP_ARCHITECTURE_SPEC.md, DOCUMENTATION_SPEC.md

## Goal

Provide exactly one managed SDKWork Knowledgebase space for an IM Conversation group, on demand.
The group is not provisioned when it is created. Only the current IM Owner may initialize it or
retry failed provisioning through a trusted service boundary. Once the binding is active, joined
non-Guest Owners, Admins, and Members may open the complete standalone Knowledgebase browser or
desktop application for that exact space; Guest, left, removed, and non-member actors are denied.

## Scope

- The authoritative key is `(tenant_id, organization_id, conversation_id)`. `conversation_id` is
  the IM Conversation aggregate identity; legacy space-group identifiers are not accepted.
- Knowledgebase owns the dedicated group-to-space binding, space lifecycle, Drive initialization,
  direct access grants, event deduplication, and final content authorization.
- IM owns group existence, current roster, roles, owner transfer, removal, leave, and dissolution.
- IM's current Owner is the sole initialization and failed-provisioning retry authority. An owner
  transfer changes that authority; Knowledgebase does not derive it from a browser request or a
  stale role snapshot.
- User-facing ticket consumption is an authenticated Knowledgebase App API operation. Trusted IM
  provisioning and roster synchronization use an approved internal SDK/RPC adapter, never a
  browser-supplied actor or raw HTTP call.
- Group-managed spaces never appear in generic space lists, selectors, or generic retrieve/update/
  delete flows. The specialized group-launch flow resolves the exact server-authorized space.
- Browser launch uses an opaque ticket in a URL fragment only. Desktop launch accepts only the
  `sdkwork-knowledgebase://group-launch/<opaque-ticket>` deep link in the standalone Knowledgebase
  Tauri application.

## Non-Goals

- No space creation during group creation, group list hydration, Header rendering, or a member
  request when no binding exists.
- No use of generic `chat_group` context binding as a second group-space authority.
- No iframe, IM-owned Webview, arbitrary executable/URL launch, space id in a launch URL, raw
  ticket persistence, or session credential in browser/desktop launch state.
- No implicit recreation of an explicitly deleted group knowledgebase.
- No physical document deletion when IM dissolves a group; the default lifecycle action is archive
  with retained audit history.

## Acceptance Criteria

1. Concurrent current-Owner first-launch requests reserve one binding and converge on one space.
   Only the current Owner may retry failed provisioning; Admin and Member requests while the
   binding is absent or failed are denied before a new reservation or provisioning call. Owner
   retries are idempotent and failed provisioning is observable and safely retryable.
2. A provisioning space remains hidden from generic Knowledgebase access until Drive, required
   initialization, and direct ACL projection have completed. An active group binding has an active
   ACL projection.
3. Initialization authorization and active-content access are separate. The current Owner alone
   may initialize or retry. After activation, IM role mapping is fixed: Owner -> Owner, Admin ->
   Writer, Member and muted member -> Reader, Guest -> no access. Final authorization requires a
   current joined non-Guest group snapshot and direct storage ACL, so leave, removal, and role
   reduction deny access immediately.
4. Source events are deduplicated by scope and event id with a payload fingerprint. Roster changes,
   owner transfer, leave, removal, and dissolution advance/invalidate relevant launch state and
   synchronize grants/revocations without duplicate side effects.
5. The ticket consumer accepts only a syntactically valid opaque one-time ticket, derives caller
   tenant, organization, session, and actor from verified context, atomically consumes the ticket,
   and rejects theft, replay, expiration, stale binding version, stale membership epoch, inactive
   binding, and insufficient current role.
6. The browser clears a fragment ticket before authentication redirect state is constructed. The
   fixed workspace never falls back to personal, team, public, market, or arbitrary URL-selected
   spaces. Configured public web base paths are honored by route capture and navigation.
7. The desktop app parses a strict deep link, holds its ticket only in memory, forwards it only to
   the primary window of its single instance, and opens the fixed group workspace after normal
   authentication.
8. Generic context-binding commands reject `chat_group` mutation. Generic group-managed space
   mutation and deletion cannot bypass the managed lifecycle.
9. PostgreSQL and SQLite schema contracts, RLS, SDK materialization, tests, runbooks, and rollback
   guidance change together. No ticket, token, document content, or unredacted membership payload
   is written to logs, analytics, or audit error text.

## Quality Attributes

| Area | Requirement |
| --- | --- |
| Security | Fail closed on IM/Drive/ACL dependency failure; current-Owner-only initialization; least privilege; opaque, hash-stored one-time tickets; tenant and organization isolation. |
| Privacy | Persist only ticket hashes and minimum operational metadata; never persist raw tickets, session credentials, or document content in the integration ledger. |
| Reliability | Unique constraints, leases, idempotent inbox/outbox events, durable retry state, and archive-first lifecycle behavior. |
| Performance | O(1) binding lookup, bounded member projection work, no generic list scans for launch, and no client-side full-space fallback. |
| Operations | Metrics and alerts for provisioning failure, ticket consumption failure, ACL projection failure, and synchronization lag; logs redact opaque tickets. |

## Traceability

- Architecture: [ADR-20260713-group-knowledgebase-binding-and-launch.md](../../architecture/decisions/ADR-20260713-group-knowledgebase-binding-and-launch.md)
- Product canon: [PRD.md](../prd/PRD.md)
- Technical architecture: [TECH_ARCHITECTURE.md](../../architecture/tech/TECH_ARCHITECTURE.md)
- Operations: [RUNBOOK-group-knowledgebase-lifecycle.md](../../runbooks/RUNBOOK-group-knowledgebase-lifecycle.md)
