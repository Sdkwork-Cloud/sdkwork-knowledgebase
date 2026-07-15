# RUNBOOK-group-knowledgebase-lifecycle

Status: active
Owner: SDKWork Knowledgebase and IM operators
Requirement: REQ-2026-0713
Specs: DOCUMENTATION_SPEC.md, DATABASE_SPEC.md, SECURITY_SPEC.md, PRIVACY_SPEC.md

## Trigger Conditions

- Current-Owner initialization or retry remains in provisioning or fails.
- A joined non-Guest Owner, Admin, or Member is denied unexpectedly, or a Guest, removed, left, or
  role-reduced user retains access.
- Ticket consumption fails, replays, expires unexpectedly, or shows a mismatch.
- IM membership synchronization, Drive grant/revoke, or group-space archive/restore is delayed.

## Safety Rules

- Do not log, paste, store, or request a raw group launch ticket. Investigate with trace id,
  binding id, conversation id, and ticket hash only.
- Do not manually modify `kb_group_knowledge_space_*`, `kb_space`, or IM ticket/link tables.
  Use the owning service command or forward-fix migration.
- Treat a failed or uncertain membership/ACL projection as deny. Do not grant broad tenant, group,
  or Drive access as a workaround.
- The Conversation aggregate is the roster authority. Do not repair group membership through a
  generic Knowledgebase context binding or legacy space-group id.

## Procedure

1. Confirm the caller's verified tenant, organization, Conversation id, actor id, role, and session
   context. Do not infer any of these from a URL, client state, or log message.
2. Check IM's managed group link and Knowledgebase's group binding by the complete scope. Confirm
   their lifecycle, binding version, membership epoch, source-event state, and latest safe error
   code agree.
3. For provisioning, inspect the provision lease and idempotent source event. Confirm Drive space
   initialization, required resource initialization, and direct ACL projection. Retry through the
   current Owner command only after the failed dependency is healthy; Admin and Member requests
   must not initialize or retry an absent/failed binding. Concurrent Owner retries must converge on
   the existing binding.
4. For unexpected access, first verify the IM current roster snapshot. Then verify the specialized
   group resolver and direct Drive grant/revoke state. A removed or Guest user must be denied even
   when a stale generic permission would otherwise allow access.
5. For ticket failure, compare only ticket hash metadata, expiry, actor, scope, binding version,
   membership epoch, consumed state, and trace id. A stale/replayed/stolen ticket is an expected
   denial; issue a new ticket through a fresh authenticated launch instead of changing the ledger.
6. For dissolution, confirm the IM Conversation lifecycle event is durable and deduplicated, then
   verify Knowledgebase binding and managed space are archived. Retain documents and audit data.
   An explicitly deleted binding requires the explicit owner restore/recreate workflow; never use
   ensure/launch as a recreation shortcut.

## Verification

- The active binding has exactly one active Owner and a role-to-access mapping consistent with IM.
- `active` lifecycle implies an active ACL projection and initialized Drive space.
- Only the current IM Owner can initialize or retry; an active binding permits launch only for
  joined non-Guest Owner, Admin, and Member roles.
- Generic space listing/retrieval does not expose the managed group space.
- The resolved group workspace is the binding's exact space, not a personal/team/public fallback.
- Browser URLs and desktop handoff state contain no raw ticket after capture/consumption.
- Ticket and ACL failure logs contain only safe metadata and a trace id.

## Recovery And Escalation

- Restore a failed dependency and replay the same durable source event through the service retry
  path. Do not hand-edit event inbox or ticket rows.
- If the binding/space lifecycle invariant is violated, take the affected binding out of service by
  marking it failed through the owning recovery command, preserve evidence, and escalate to the
  Knowledgebase and IM service owners.
- Follow [migration-rollback.md](migration-rollback.md) for schema rollback decisions. A deployed
  rollback must be evaluated as a coordinated IM and Knowledgebase forward-fix; do not roll back
  one side while retaining events that require the other contract.
