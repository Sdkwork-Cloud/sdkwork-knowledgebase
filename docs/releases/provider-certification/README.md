# Provider Live Certification Evidence

This directory owns release evidence indexes for Provider and pinned upstream-version pairs. Local
adapter contract tests are not live certification.

Start from `live-certification-evidence.template.json`. The template is deliberately a `draft` with
a template-only kind and pending approvals, so it cannot pass the certification gate. A reviewer may
change it to the certified kind and status only after all six referenced release artifacts exist,
their SHA-256 digests are recorded, the upstream version and adapter commit are pinned, and licensing
plus security/privacy reviews are approved.

Quality evidence starts from `quality-evaluation-evidence.template.json` and is governed by its own
schema plus `specs/knowledge-engine-evaluation.spec.json`. The checked-in sample dataset is a
`contract-fixture`, not production evidence. A production record requires reviewed
`production-domain` coverage, the pinned raw results, a deterministic evaluation report, and exact
SHA-256 digests for all three files. The production policy bounds scored queries at 5,000,
rejection queries at 500, and each artifact at 32 MiB; future-dated evidence is rejected.

Load/SLO and outage-recovery evidence start from their respective templates and are governed by
`load-slo-evidence.schema.json`, `outage-recovery-evidence.schema.json`, and
`specs/knowledge-engine-operational-evidence.spec.json`. The gate loads the digest-bound raw request
samples and outage timelines, rejects secret-bearing or unknown fields, and recomputes aggregate,
per-operation, detection, and recovery results. Declared summaries cannot override the policy or
the raw evidence. The checked-in templates are examples only and cannot satisfy live certification.

A certifiable load run must cover `search`, `read_document`, and `health` across at least two
tenants for at least 30 minutes and 10,000 requests at concurrency 8 or greater. It must satisfy the
versioned failure-rate, availability, P95/P99 latency, per-operation sample, and zero cross-tenant
violation thresholds. Raw operational artifacts are limited to 32 MiB and load evidence to 100,000
samples; their completion date must match `verifiedAt`. Outage evidence must exercise timeout,
rate-limit, upstream 5xx,
authentication, malformed-response, and bulkhead-saturation scenarios with bounded detection and
recovery, fail-closed behavior, alert and trace correlation, no retry storm, no secret leak, and no
cross-tenant violation.

Store referenced reports under `artifacts/`. Do not store credentials, tokens, private runtime
configuration, raw authorization headers, or unbounded upstream response bodies in any evidence
file. All quality, operational, and live release artifacts use the shared bounded reader, which
normalizes repository paths, resolves real paths inside the artifact root, enforces size limits,
and computes SHA-256 before parsing. Add the completed evidence index path and its exact SHA-256 to
`external/knowledge-engines/provider-certification.manifest.json`; `pnpm
check:provider-certification` rejects missing, stale, mutable, mismatched, or draft evidence.

Authority: `live-certification-evidence.schema.json`,
`specs/knowledge-engine-operational-evidence.spec.json`, `RELEASE_SPEC.md`,
`SUPPLY_CHAIN_SECURITY_SPEC.md`, `SECURITY_SPEC.md`, `PRIVACY_SPEC.md`, and `TEST_SPEC.md`.
