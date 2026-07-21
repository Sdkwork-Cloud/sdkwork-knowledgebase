import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  buildCertificationExecutionPlan,
  loadProviderCertification,
  validateProviderCertification,
} from "./provider_certification.mjs";
import { validateLiveCertificationEvidenceRecord } from "./provider_live_certification.mjs";
import {
  CANONICAL_OPERATIONAL_POLICY,
  computeLoadSloMetrics,
  computeOutageRecoveryMetrics,
  validateLoadSloEvidenceRecord,
  validateOutageRecoveryEvidenceRecord,
} from "./provider_operational_evidence.mjs";

const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestPath = path.join(
  workspaceRoot,
  "external/knowledge-engines/provider-certification.manifest.json",
);

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function dateWithOffset(days) {
  const date = new Date();
  date.setUTCHours(0, 0, 0, 0);
  date.setUTCDate(date.getUTCDate() + days);
  return date.toISOString().slice(0, 10);
}

async function writeArtifact(root, reference, value) {
  const bytes = Buffer.from(`${JSON.stringify(value, null, 2)}\n`);
  const target = path.join(root, reference);
  await mkdir(path.dirname(target), { recursive: true });
  await writeFile(target, bytes);
  return sha256(bytes);
}

function operationalIdentity() {
  return {
    providerId: "dify",
    upstreamVersion: "1.2.3",
    adapterCommit: "a".repeat(40),
    verifiedAt: dateWithOffset(0),
  };
}

function loadRecord(identity, reference, digest, rawResults) {
  const metrics = computeLoadSloMetrics(rawResults);
  return {
    schemaVersion: 1,
    kind: "sdkwork.knowledge-engine-load-slo-evidence",
    status: "passed",
    ...identity,
    policyVersion: CANONICAL_OPERATIONAL_POLICY.policyVersion,
    expiresAt: dateWithOffset(30),
    environment: "release",
    workflowRunRef: "https://ci.example.test/runs/load-1",
    reviewedBy: "release-reviewer",
    dashboardRef: "https://metrics.example.test/provider/dify",
    alertEvaluationRef: "https://alerts.example.test/evaluations/dify",
    rawResultsRef: reference,
    rawResultsSha256: digest,
    aggregate: metrics.aggregate,
    operationMetrics: metrics.operationMetrics,
  };
}

function outageRecord(identity, reference, digest, rawTimeline) {
  return {
    schemaVersion: 1,
    kind: "sdkwork.knowledge-engine-outage-recovery-evidence",
    status: "passed",
    ...identity,
    policyVersion: CANONICAL_OPERATIONAL_POLICY.policyVersion,
    expiresAt: dateWithOffset(30),
    environment: "release",
    workflowRunRef: "https://ci.example.test/runs/outage-1",
    reviewedBy: "release-reviewer",
    dashboardRef: "https://metrics.example.test/provider/dify",
    runbookRef: "docs/runbooks/provider-outage.md",
    rawTimelineRef: reference,
    rawTimelineSha256: digest,
    scenarioMetrics: computeOutageRecoveryMetrics(rawTimeline),
  };
}

test("Provider certification manifest is complete and secret-safe", async () => {
  const manifest = await loadProviderCertification(manifestPath);
  assert.deepEqual(await validateProviderCertification(manifest, workspaceRoot), []);
  assert.equal(buildCertificationExecutionPlan(manifest).length, 10);
});

test("Provider certification rejects shell injection and missing dimensions", async () => {
  const manifest = await loadProviderCertification(manifestPath);
  const invalid = structuredClone(manifest);
  invalid.providers[0].contractCertification.command.args[2] = "sdkwork-knowledgebase-engine-dify;whoami";
  delete invalid.providers[0].contractCertification.evidence.isolation;
  const violations = await validateProviderCertification(invalid, workspaceRoot);
  assert.ok(violations.some((violation) => violation.includes("complete owned adapter crate")));
  assert.ok(violations.some((violation) => violation.includes("unsafe argument")));
  assert.ok(violations.some((violation) => violation.includes("evidence.isolation")));
});

test("Provider certification plan rejects unknown targets", async () => {
  const manifest = await loadProviderCertification(manifestPath);
  assert.throws(
    () => buildCertificationExecutionPlan(manifest, ["unknown-provider"]),
    /unknown Provider certification target/,
  );
});

test("live certification cannot be promoted without pinned current evidence", async () => {
  const manifest = await loadProviderCertification(manifestPath);
  const invalid = structuredClone(manifest);
  invalid.providers[0].liveCertification = {
    status: "certified",
    upstreamVersion: "latest",
    evidenceRef: "docs/releases/provider-certification/missing.json",
    evidenceSha256: "0".repeat(64),
    verifiedAt: "2020-01-01",
    licensingApproval: "pending",
    securityPrivacyApproval: "pending",
    sloEvidence: "missing",
  };
  const violations = await validateProviderCertification(invalid, workspaceRoot);
  assert.ok(violations.some((violation) => violation.includes("upstreamVersion must be pinned")));
  assert.ok(violations.some((violation) => violation.includes("evidenceRef is missing")));
  assert.ok(violations.some((violation) => violation.includes("licensingApproval must be approved")));
  assert.ok(violations.some((violation) => violation.includes("verifiedAt is missing, future-dated, or stale")));
});

test("live certification template cannot be accepted as release evidence", async () => {
  const manifest = await loadProviderCertification(manifestPath);
  const template = JSON.parse(await readFile(path.join(
    workspaceRoot,
    "docs/releases/provider-certification/live-certification-evidence.template.json",
  ), "utf8"));
  const violations = await validateLiveCertificationEvidenceRecord(
    template,
    {
      providerId: template.providerId,
      upstreamVersion: template.upstreamVersion,
      contractSuiteVersion: manifest.policy.contractSuiteVersion,
      verifiedAt: template.verifiedAt,
      licensingApproval: "approved",
      securityPrivacyApproval: "approved",
    },
    manifest.policy,
    workspaceRoot,
    "template",
  );
  assert.ok(violations.some((violation) => violation.includes("kind is invalid")));
  assert.ok(violations.some((violation) => violation.includes("status must be certified")));
  assert.ok(violations.some((violation) => violation.includes("adapterCommit must be a full Git commit")));
  assert.ok(violations.some((violation) => violation.includes("licensingApproval must match")));
});

test("load and SLO evidence is recomputed from bounded multi-tenant raw results", async (context) => {
  const temporaryRoot = await mkdtemp(path.join(tmpdir(), "sdkwork-provider-load-evidence-"));
  context.after(() => rm(temporaryRoot, { recursive: true, force: true }));
  const identity = operationalIdentity();
  const startedAt = `${identity.verifiedAt}T01:00:00.000Z`;
  const completedAt = `${identity.verifiedAt}T01:30:00.000Z`;
  const operations = CANONICAL_OPERATIONAL_POLICY.loadSlo.requiredOperations;
  const rawResults = {
    schemaVersion: 1,
    kind: "sdkwork.knowledge-engine-load-results",
    providerId: identity.providerId,
    upstreamVersion: identity.upstreamVersion,
    adapterCommit: identity.adapterCommit,
    startedAt,
    completedAt,
    concurrency: 16,
    samples: Array.from({ length: 10000 }, (_, index) => ({
      sequence: index + 1,
      operation: operations[index % operations.length],
      tenantHash: index % 2 === 0 ? "b".repeat(64) : "c".repeat(64),
      outcome: index < 50 ? "failure" : "success",
      latencyMs: 120 + (index % 20),
      isolationViolation: false,
    })),
  };
  const reference = "docs/releases/provider-certification/artifacts/test-load/raw-results.json";
  const digest = await writeArtifact(temporaryRoot, reference, rawResults);
  const record = loadRecord(identity, reference, digest, rawResults);
  assert.deepEqual(
    await validateLoadSloEvidenceRecord(
      record,
      identity,
      CANONICAL_OPERATIONAL_POLICY,
      temporaryRoot,
      "load",
    ),
    [],
  );

  const failingRawResults = structuredClone(rawResults);
  for (const sample of failingRawResults.samples.slice(0, 600)) {
    sample.outcome = "failure";
    sample.latencyMs = 6000;
  }
  failingRawResults.samples[700].isolationViolation = true;
  const failingReference = "docs/releases/provider-certification/artifacts/test-load/failing-results.json";
  const failingDigest = await writeArtifact(temporaryRoot, failingReference, failingRawResults);
  const failingRecord = loadRecord(identity, failingReference, failingDigest, failingRawResults);
  const violations = await validateLoadSloEvidenceRecord(
    failingRecord,
    identity,
    CANONICAL_OPERATIONAL_POLICY,
    temporaryRoot,
    "load",
  );
  assert.ok(violations.some((violation) => violation.includes("failureRate exceeds policy")));
  assert.ok(violations.some((violation) => violation.includes("p95LatencyMs exceeds policy")));
  assert.ok(violations.some((violation) => violation.includes("p99LatencyMs exceeds policy")));
  assert.ok(violations.some((violation) => violation.includes("crossTenantViolationCount exceeds policy")));

  const boundedPolicy = structuredClone(CANONICAL_OPERATIONAL_POLICY);
  boundedPolicy.loadSlo.maximumRawSamples = rawResults.samples.length - 1;
  const boundedViolations = await validateLoadSloEvidenceRecord(
    record,
    identity,
    boundedPolicy,
    temporaryRoot,
    "boundedLoad",
  );
  assert.ok(boundedViolations.some((violation) => violation.includes("bounded evidence policy")));

  const byteBoundedPolicy = structuredClone(CANONICAL_OPERATIONAL_POLICY);
  byteBoundedPolicy.maxArtifactBytes = 1;
  const byteBoundedViolations = await validateLoadSloEvidenceRecord(
    record,
    identity,
    byteBoundedPolicy,
    temporaryRoot,
    "byteBoundedLoad",
  );
  assert.ok(byteBoundedViolations.some((violation) => violation.includes("1 byte limit")));

  const futureIdentity = { ...identity, verifiedAt: dateWithOffset(1) };
  const futureRecord = { ...record, verifiedAt: futureIdentity.verifiedAt };
  const futureViolations = await validateLoadSloEvidenceRecord(
    futureRecord,
    futureIdentity,
    CANONICAL_OPERATIONAL_POLICY,
    temporaryRoot,
    "futureLoad",
  );
  assert.ok(futureViolations.some((violation) => violation.includes("non-future")));

  const staleRawResults = structuredClone(rawResults);
  const staleCompletion = Date.parse(staleRawResults.completedAt) - 86_400_000;
  staleRawResults.completedAt = new Date(staleCompletion).toISOString();
  staleRawResults.startedAt = new Date(staleCompletion - 1_800_000).toISOString();
  const staleReference = "docs/releases/provider-certification/artifacts/test-load/stale-results.json";
  const staleDigest = await writeArtifact(temporaryRoot, staleReference, staleRawResults);
  const staleRecord = loadRecord(identity, staleReference, staleDigest, staleRawResults);
  const staleViolations = await validateLoadSloEvidenceRecord(
    staleRecord,
    identity,
    CANONICAL_OPERATIONAL_POLICY,
    temporaryRoot,
    "staleLoad",
  );
  assert.ok(staleViolations.some((violation) => violation.includes("completedAt must match verifiedAt")));

  const malformedRawResults = { ...rawResults, samples: {} };
  const malformedReference = "docs/releases/provider-certification/artifacts/test-load/malformed-results.json";
  const malformedDigest = await writeArtifact(temporaryRoot, malformedReference, malformedRawResults);
  const malformedRecord = { ...record, rawResultsRef: malformedReference, rawResultsSha256: malformedDigest };
  const malformedViolations = await validateLoadSloEvidenceRecord(
    malformedRecord,
    identity,
    CANONICAL_OPERATIONAL_POLICY,
    temporaryRoot,
    "malformedLoad",
  );
  assert.ok(malformedViolations.some((violation) => violation.includes("samples must not be empty")));
});

test("outage evidence recomputes every required recovery scenario and fails closed", async (context) => {
  const temporaryRoot = await mkdtemp(path.join(tmpdir(), "sdkwork-provider-outage-evidence-"));
  context.after(() => rm(temporaryRoot, { recursive: true, force: true }));
  const identity = operationalIdentity();
  const base = Date.parse(`${identity.verifiedAt}T02:00:00.000Z`);
  const rawTimeline = {
    schemaVersion: 1,
    kind: "sdkwork.knowledge-engine-outage-timeline",
    providerId: identity.providerId,
    upstreamVersion: identity.upstreamVersion,
    adapterCommit: identity.adapterCommit,
    scenarios: CANONICAL_OPERATIONAL_POLICY.outageRecovery.requiredScenarios.map((category, index) => {
      const injected = base + index * 600000;
      return {
        scenarioId: `scenario-${index + 1}`,
        category,
        injectedAt: new Date(injected).toISOString(),
        detectedAt: new Date(injected + 10000).toISOString(),
        recoveredAt: new Date(injected + 60000).toISOString(),
        failClosed: true,
        alertTriggered: true,
        traceCorrelationVerified: true,
        retryStormDetected: false,
        secretLeakCount: 0,
        crossTenantViolationCount: 0,
      };
    }),
  };
  const reference = "docs/releases/provider-certification/artifacts/test-outage/raw-timeline.json";
  const digest = await writeArtifact(temporaryRoot, reference, rawTimeline);
  const record = outageRecord(identity, reference, digest, rawTimeline);
  assert.deepEqual(
    await validateOutageRecoveryEvidenceRecord(
      record,
      identity,
      CANONICAL_OPERATIONAL_POLICY,
      temporaryRoot,
      "outage",
    ),
    [],
  );

  const failingTimeline = structuredClone(rawTimeline);
  const scenario = failingTimeline.scenarios[0];
  scenario.recoveredAt = new Date(Date.parse(scenario.injectedAt) + 600000).toISOString();
  scenario.failClosed = false;
  scenario.alertTriggered = false;
  scenario.traceCorrelationVerified = false;
  scenario.retryStormDetected = true;
  scenario.secretLeakCount = 1;
  scenario.crossTenantViolationCount = 1;
  const failingReference = "docs/releases/provider-certification/artifacts/test-outage/failing-timeline.json";
  const failingDigest = await writeArtifact(temporaryRoot, failingReference, failingTimeline);
  const failingRecord = outageRecord(identity, failingReference, failingDigest, failingTimeline);
  const violations = await validateOutageRecoveryEvidenceRecord(
    failingRecord,
    identity,
    CANONICAL_OPERATIONAL_POLICY,
    temporaryRoot,
    "outage",
  );
  for (const expected of [
    "recoverySeconds exceeds policy",
    "failClosed must be true",
    "alertTriggered must be true",
    "traceCorrelationVerified must be true",
    "retryStormDetected must be false",
    "secretLeakCount exceeds policy",
    "crossTenantViolationCount exceeds policy",
  ]) {
    assert.ok(violations.some((violation) => violation.includes(expected)), expected);
  }

  const staleTimeline = structuredClone(rawTimeline);
  for (const staleScenario of staleTimeline.scenarios) {
    staleScenario.injectedAt = new Date(Date.parse(staleScenario.injectedAt) - 86_400_000).toISOString();
    staleScenario.detectedAt = new Date(Date.parse(staleScenario.detectedAt) - 86_400_000).toISOString();
    staleScenario.recoveredAt = new Date(Date.parse(staleScenario.recoveredAt) - 86_400_000).toISOString();
  }
  const staleReference = "docs/releases/provider-certification/artifacts/test-outage/stale-timeline.json";
  const staleDigest = await writeArtifact(temporaryRoot, staleReference, staleTimeline);
  const staleRecord = outageRecord(identity, staleReference, staleDigest, staleTimeline);
  const staleViolations = await validateOutageRecoveryEvidenceRecord(
    staleRecord,
    identity,
    CANONICAL_OPERATIONAL_POLICY,
    temporaryRoot,
    "staleOutage",
  );
  assert.ok(staleViolations.some((violation) => violation.includes("recoveredAt must match verifiedAt")));

  const malformedTimeline = { ...rawTimeline, scenarios: {} };
  const malformedReference = "docs/releases/provider-certification/artifacts/test-outage/malformed-timeline.json";
  const malformedDigest = await writeArtifact(temporaryRoot, malformedReference, malformedTimeline);
  const malformedRecord = { ...record, rawTimelineRef: malformedReference, rawTimelineSha256: malformedDigest };
  const malformedViolations = await validateOutageRecoveryEvidenceRecord(
    malformedRecord,
    identity,
    CANONICAL_OPERATIONAL_POLICY,
    temporaryRoot,
    "malformedOutage",
  );
  assert.ok(malformedViolations.some((violation) => violation.includes("scenarios must be an array")));
});

test("operational evidence templates cannot satisfy live certification", async () => {
  const identity = operationalIdentity();
  const loadTemplate = JSON.parse(await readFile(path.join(
    workspaceRoot,
    "docs/releases/provider-certification/load-slo-evidence.template.json",
  ), "utf8"));
  const outageTemplate = JSON.parse(await readFile(path.join(
    workspaceRoot,
    "docs/releases/provider-certification/outage-recovery-evidence.template.json",
  ), "utf8"));
  const loadViolations = await validateLoadSloEvidenceRecord(
    loadTemplate,
    identity,
    CANONICAL_OPERATIONAL_POLICY,
    workspaceRoot,
    "loadTemplate",
  );
  const outageViolations = await validateOutageRecoveryEvidenceRecord(
    outageTemplate,
    identity,
    CANONICAL_OPERATIONAL_POLICY,
    workspaceRoot,
    "outageTemplate",
  );
  assert.ok(loadViolations.some((violation) => violation.includes("kind is invalid")));
  assert.ok(loadViolations.some((violation) => violation.includes("status must be passed")));
  assert.ok(outageViolations.some((violation) => violation.includes("kind is invalid")));
  assert.ok(outageViolations.some((violation) => violation.includes("status must be passed")));
});
