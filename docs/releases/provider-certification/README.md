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
SHA-256 digests for all three files.

Store referenced reports under `artifacts/`. Do not store credentials, tokens, private runtime
configuration, raw authorization headers, or unbounded upstream response bodies in any evidence
file. Add the completed evidence index path and its exact SHA-256 to
`external/knowledge-engines/provider-certification.manifest.json`; `pnpm
check:provider-certification` rejects missing, stale, mutable, mismatched, or draft evidence.

Authority: `live-certification-evidence.schema.json`, `RELEASE_SPEC.md`,
`SUPPLY_CHAIN_SECURITY_SPEC.md`, `SECURITY_SPEC.md`, `PRIVACY_SPEC.md`, and `TEST_SPEC.md`.
