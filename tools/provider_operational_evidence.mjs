import { readFile } from "node:fs/promises";
import path from "node:path";

import {
  isCertificationArtifactReference,
  normalizeCertificationArtifactReference,
  readBoundedCertificationArtifact,
} from "./provider_certification_artifact.mjs";

export const OPERATIONAL_EVIDENCE_SPEC_PATH =
  "specs/knowledge-engine-operational-evidence.spec.json";
export const LOAD_SLO_EVIDENCE_SCHEMA_PATH =
  "docs/releases/provider-certification/load-slo-evidence.schema.json";
export const OUTAGE_RECOVERY_EVIDENCE_SCHEMA_PATH =
  "docs/releases/provider-certification/outage-recovery-evidence.schema.json";

export const CANONICAL_OPERATIONAL_POLICY = Object.freeze({
  schemaVersion: 1,
  kind: "sdkwork.knowledge-engine-operational-evidence-policy",
  policyVersion: "1.0.0",
  maxEvidenceAgeDays: 90,
  maxArtifactBytes: 33_554_432,
  loadSlo: Object.freeze({
    minimumDurationSeconds: 1800,
    minimumTotalRequests: 10000,
    maximumRawSamples: 100000,
    minimumRequestsPerOperation: 100,
    minimumConcurrency: 8,
    requiredOperations: Object.freeze(["search", "read_document", "health"]),
    maximumFailureRate: 0.01,
    minimumAvailability: 0.995,
    maximumP95LatencyMs: 2000,
    maximumP99LatencyMs: 5000,
    minimumTenantCount: 2,
    maximumCrossTenantViolations: 0,
  }),
  outageRecovery: Object.freeze({
    requiredScenarios: Object.freeze([
      "timeout",
      "rate_limit",
      "upstream_5xx",
      "authentication",
      "malformed_response",
      "bulkhead_saturation",
    ]),
    maximumDetectionSeconds: 60,
    maximumRecoverySeconds: 300,
    maximumSecretLeakCount: 0,
    maximumCrossTenantViolations: 0,
    requireFailClosed: true,
    requireAlert: true,
    requireTraceCorrelation: true,
    allowRetryStorm: false,
  }),
});

const LOAD_REQUIRED_FIELDS = Object.freeze([
  "schemaVersion",
  "kind",
  "status",
  "providerId",
  "upstreamVersion",
  "adapterCommit",
  "policyVersion",
  "verifiedAt",
  "expiresAt",
  "environment",
  "workflowRunRef",
  "reviewedBy",
  "dashboardRef",
  "alertEvaluationRef",
  "rawResultsRef",
  "rawResultsSha256",
  "aggregate",
  "operationMetrics",
]);

const OUTAGE_REQUIRED_FIELDS = Object.freeze([
  "schemaVersion",
  "kind",
  "status",
  "providerId",
  "upstreamVersion",
  "adapterCommit",
  "policyVersion",
  "verifiedAt",
  "expiresAt",
  "environment",
  "workflowRunRef",
  "reviewedBy",
  "dashboardRef",
  "runbookRef",
  "rawTimelineRef",
  "rawTimelineSha256",
  "scenarioMetrics",
]);

const datePattern = /^\d{4}-\d{2}-\d{2}$/u;
const gitCommitPattern = /^[a-f0-9]{40}$/u;
const sha256Pattern = /^[a-f0-9]{64}$/u;
const forbiddenKeyPattern = /(credential|secret|password|api.?key|access.?token|authorization)/iu;

function hasExactKeys(value, fields) {
  return value
    && typeof value === "object"
    && !Array.isArray(value)
    && JSON.stringify(Object.keys(value).sort()) === JSON.stringify([...fields].sort());
}

function addViolation(violations, condition, message) {
  if (!condition) violations.push(message);
}

function roundMetric(value) {
  return Number(value.toFixed(6));
}

function percentile(values, quantile) {
  if (values.length === 0) return 0;
  const ordered = [...values].sort((left, right) => left - right);
  const index = Math.max(0, Math.ceil(ordered.length * quantile) - 1);
  return ordered[index];
}

function collectForbiddenKeys(value, location, violations) {
  if (Array.isArray(value)) {
    value.forEach((item, index) => collectForbiddenKeys(item, `${location}[${index}]`, violations));
    return;
  }
  if (!value || typeof value !== "object") return;
  for (const [key, nested] of Object.entries(value)) {
    if (key !== "secretLeakCount" && forbiddenKeyPattern.test(key)) {
      violations.push(`${location}.${key}: secret-bearing fields are forbidden`);
    }
    collectForbiddenKeys(nested, `${location}.${key}`, violations);
  }
}

function normalizeRepositoryPath(value) {
  return normalizeCertificationArtifactReference(value);
}

function validateIdentity(record, expected, policy, kind, location, violations) {
  addViolation(violations, record?.schemaVersion === 1, `${location}.schemaVersion must be 1`);
  addViolation(violations, record?.kind === kind, `${location}.kind is invalid`);
  addViolation(violations, record?.status === "passed", `${location}.status must be passed`);
  addViolation(violations, record?.providerId === expected.providerId, `${location}.providerId mismatch`);
  addViolation(violations, record?.upstreamVersion === expected.upstreamVersion, `${location}.upstreamVersion mismatch`);
  addViolation(violations, record?.adapterCommit === expected.adapterCommit, `${location}.adapterCommit mismatch`);
  addViolation(violations, gitCommitPattern.test(record?.adapterCommit ?? ""), `${location}.adapterCommit must be a full Git commit`);
  addViolation(violations, record?.policyVersion === policy.policyVersion, `${location}.policyVersion mismatch`);
  addViolation(violations, record?.verifiedAt === expected.verifiedAt, `${location}.verifiedAt mismatch`);
  addViolation(violations, record?.environment === "release", `${location}.environment must be release`);
  addViolation(violations, /^https:\/\//u.test(record?.workflowRunRef ?? ""), `${location}.workflowRunRef must be HTTPS`);
  addViolation(violations, /^https:\/\//u.test(record?.dashboardRef ?? ""), `${location}.dashboardRef must be HTTPS`);
  addViolation(violations, typeof record?.reviewedBy === "string" && record.reviewedBy.length > 0, `${location}.reviewedBy is required`);

  const verifiedTime = datePattern.test(record?.verifiedAt ?? "")
    ? Date.parse(`${record.verifiedAt}T00:00:00Z`)
    : Number.NaN;
  const expiryTime = datePattern.test(record?.expiresAt ?? "")
    ? Date.parse(`${record.expiresAt}T00:00:00Z`)
    : Number.NaN;
  addViolation(
    violations,
    Number.isFinite(verifiedTime)
      && Number.isFinite(expiryTime)
      && verifiedTime <= Date.now()
      && expiryTime > verifiedTime
      && expiryTime <= verifiedTime + policy.maxEvidenceAgeDays * 86_400_000
      && Date.now() <= expiryTime + 86_399_999,
    `${location}.verifiedAt/expiresAt must be current, non-future, and within operational evidence policy`,
  );
}

async function loadJsonArtifact(
  record,
  refField,
  digestField,
  policy,
  workspaceRoot,
  location,
  violations,
) {
  const reference = normalizeRepositoryPath(record?.[refField]);
  addViolation(
    violations,
    isCertificationArtifactReference(reference),
    `${location}.${refField} must reference a certification artifact`,
  );
  addViolation(
    violations,
    sha256Pattern.test(record?.[digestField] ?? ""),
    `${location}.${digestField} must be a SHA-256 digest`,
  );
  if (!isCertificationArtifactReference(reference)) {
    return undefined;
  }
  try {
    const { bytes, digest } = await readBoundedCertificationArtifact(
      reference,
      workspaceRoot,
      policy.maxArtifactBytes,
    );
    addViolation(violations, digest === record[digestField], `${location}.${digestField} does not match ${reference}`);
    try {
      return JSON.parse(bytes.toString("utf8"));
    } catch {
      violations.push(`${location}.${refField} must contain valid JSON`);
    }
  } catch (error) {
    violations.push(`${location}.${refField} is missing or invalid: ${reference}: ${error.message}`);
  }
  return undefined;
}

function metricForSamples(operation, samples, durationSeconds, concurrency) {
  const successCount = samples.filter((sample) => sample.outcome === "success").length;
  const failureCount = samples.length - successCount;
  const latencies = samples.map((sample) => sample.latencyMs);
  return {
    operation,
    requestCount: samples.length,
    successCount,
    failureCount,
    availability: roundMetric(successCount / samples.length),
    failureRate: roundMetric(failureCount / samples.length),
    p95LatencyMs: percentile(latencies, 0.95),
    p99LatencyMs: percentile(latencies, 0.99),
    throughputPerSecond: roundMetric(samples.length / durationSeconds),
    concurrency,
  };
}

export function computeLoadSloMetrics(rawResults, policy = CANONICAL_OPERATIONAL_POLICY) {
  const startedAt = Date.parse(rawResults.startedAt);
  const completedAt = Date.parse(rawResults.completedAt);
  const durationSeconds = (completedAt - startedAt) / 1000;
  const samples = rawResults.samples;
  const operationMetrics = policy.loadSlo.requiredOperations.map((operation) => metricForSamples(
    operation,
    samples.filter((sample) => sample.operation === operation),
    durationSeconds,
    rawResults.concurrency,
  ));
  const aggregateMetric = metricForSamples("aggregate", samples, durationSeconds, rawResults.concurrency);
  return {
    aggregate: {
      durationSeconds,
      totalRequests: aggregateMetric.requestCount,
      concurrency: rawResults.concurrency,
      successCount: aggregateMetric.successCount,
      failureCount: aggregateMetric.failureCount,
      availability: aggregateMetric.availability,
      failureRate: aggregateMetric.failureRate,
      p95LatencyMs: aggregateMetric.p95LatencyMs,
      p99LatencyMs: aggregateMetric.p99LatencyMs,
      throughputPerSecond: aggregateMetric.throughputPerSecond,
      tenantCount: new Set(samples.map((sample) => sample.tenantHash)).size,
      crossTenantViolationCount: samples.filter((sample) => sample.isolationViolation).length,
    },
    operationMetrics,
  };
}

function validateLoadThresholds(metrics, policy, location, violations) {
  const threshold = policy.loadSlo;
  const aggregate = metrics.aggregate;
  addViolation(violations, aggregate.durationSeconds >= threshold.minimumDurationSeconds, `${location}.aggregate.durationSeconds is below policy`);
  addViolation(violations, aggregate.totalRequests >= threshold.minimumTotalRequests, `${location}.aggregate.totalRequests is below policy`);
  addViolation(violations, aggregate.concurrency >= threshold.minimumConcurrency, `${location}.aggregate.concurrency is below policy`);
  addViolation(violations, aggregate.failureRate <= threshold.maximumFailureRate, `${location}.aggregate.failureRate exceeds policy`);
  addViolation(violations, aggregate.availability >= threshold.minimumAvailability, `${location}.aggregate.availability is below policy`);
  addViolation(violations, aggregate.p95LatencyMs <= threshold.maximumP95LatencyMs, `${location}.aggregate.p95LatencyMs exceeds policy`);
  addViolation(violations, aggregate.p99LatencyMs <= threshold.maximumP99LatencyMs, `${location}.aggregate.p99LatencyMs exceeds policy`);
  addViolation(violations, aggregate.tenantCount >= threshold.minimumTenantCount, `${location}.aggregate.tenantCount is below policy`);
  addViolation(violations, aggregate.crossTenantViolationCount <= threshold.maximumCrossTenantViolations, `${location}.aggregate.crossTenantViolationCount exceeds policy`);
  for (const metric of metrics.operationMetrics) {
    const metricLocation = `${location}.operationMetrics.${metric.operation}`;
    addViolation(violations, metric.requestCount >= threshold.minimumRequestsPerOperation, `${metricLocation}.requestCount is below policy`);
    addViolation(violations, metric.failureRate <= threshold.maximumFailureRate, `${metricLocation}.failureRate exceeds policy`);
    addViolation(violations, metric.p95LatencyMs <= threshold.maximumP95LatencyMs, `${metricLocation}.p95LatencyMs exceeds policy`);
    addViolation(violations, metric.p99LatencyMs <= threshold.maximumP99LatencyMs, `${metricLocation}.p99LatencyMs exceeds policy`);
  }
}

export async function validateLoadSloEvidenceRecord(record, expected, policy, workspaceRoot, location) {
  const violations = [];
  collectForbiddenKeys(record, location, violations);
  addViolation(violations, hasExactKeys(record, LOAD_REQUIRED_FIELDS), `${location} fields must match the load/SLO evidence contract`);
  validateIdentity(record, expected, policy, "sdkwork.knowledge-engine-load-slo-evidence", location, violations);
  addViolation(violations, /^https:\/\//u.test(record?.alertEvaluationRef ?? ""), `${location}.alertEvaluationRef must be HTTPS`);
  const rawResults = await loadJsonArtifact(
    record,
    "rawResultsRef",
    "rawResultsSha256",
    policy,
    workspaceRoot,
    location,
    violations,
  );
  if (!rawResults) return violations;
  collectForbiddenKeys(rawResults, `${location}.rawResults`, violations);
  addViolation(
    violations,
    hasExactKeys(rawResults, [
      "schemaVersion",
      "kind",
      "providerId",
      "upstreamVersion",
      "adapterCommit",
      "startedAt",
      "completedAt",
      "concurrency",
      "samples",
    ]),
    `${location}.rawResults fields are invalid`,
  );
  addViolation(violations, rawResults?.schemaVersion === 1, `${location}.rawResults.schemaVersion must be 1`);
  addViolation(violations, rawResults?.kind === "sdkwork.knowledge-engine-load-results", `${location}.rawResults.kind is invalid`);
  addViolation(violations, rawResults?.providerId === expected.providerId, `${location}.rawResults.providerId mismatch`);
  addViolation(violations, rawResults?.upstreamVersion === expected.upstreamVersion, `${location}.rawResults.upstreamVersion mismatch`);
  addViolation(violations, rawResults?.adapterCommit === expected.adapterCommit, `${location}.rawResults.adapterCommit mismatch`);
  addViolation(violations, Number.isInteger(rawResults?.concurrency) && rawResults.concurrency > 0, `${location}.rawResults.concurrency is invalid`);
  const startedAt = Date.parse(rawResults?.startedAt ?? "");
  const completedAt = Date.parse(rawResults?.completedAt ?? "");
  addViolation(violations, Number.isFinite(startedAt) && Number.isFinite(completedAt) && completedAt > startedAt, `${location}.rawResults time range is invalid`);
  addViolation(violations, Array.isArray(rawResults?.samples) && rawResults.samples.length > 0, `${location}.rawResults.samples must not be empty`);
  addViolation(
    violations,
    Array.isArray(rawResults?.samples)
      && rawResults.samples.length <= policy.loadSlo.maximumRawSamples,
    `${location}.rawResults.samples exceeds the bounded evidence policy`,
  );
  addViolation(
    violations,
    Number.isFinite(completedAt)
      && new Date(completedAt).toISOString().slice(0, 10) === expected.verifiedAt,
    `${location}.rawResults.completedAt must match verifiedAt`,
  );
  const sequences = new Set();
  const samples = Array.isArray(rawResults?.samples) ? rawResults.samples : [];
  for (const [index, sample] of samples.entries()) {
    const sampleLocation = `${location}.rawResults.samples[${index}]`;
    addViolation(
      violations,
      hasExactKeys(sample, [
        "sequence",
        "operation",
        "tenantHash",
        "outcome",
        "latencyMs",
        "isolationViolation",
      ]),
      `${sampleLocation} fields are invalid`,
    );
    addViolation(violations, Number.isSafeInteger(sample?.sequence) && sample.sequence > 0, `${sampleLocation}.sequence is invalid`);
    addViolation(violations, !sequences.has(sample?.sequence), `${sampleLocation}.sequence is duplicated`);
    sequences.add(sample?.sequence);
    addViolation(violations, policy.loadSlo.requiredOperations.includes(sample?.operation), `${sampleLocation}.operation is invalid`);
    addViolation(violations, ["success", "failure"].includes(sample?.outcome), `${sampleLocation}.outcome is invalid`);
    addViolation(violations, Number.isFinite(sample?.latencyMs) && sample.latencyMs >= 0, `${sampleLocation}.latencyMs is invalid`);
    addViolation(violations, sha256Pattern.test(sample?.tenantHash ?? ""), `${sampleLocation}.tenantHash must be SHA-256`);
    addViolation(violations, typeof sample?.isolationViolation === "boolean", `${sampleLocation}.isolationViolation is required`);
  }
  if (violations.some((violation) => violation.includes(`${location}.rawResults`))) return violations;
  const computed = computeLoadSloMetrics(rawResults, policy);
  addViolation(violations, JSON.stringify(record?.aggregate) === JSON.stringify(computed.aggregate), `${location}.aggregate differs from raw result recomputation`);
  addViolation(violations, JSON.stringify(record?.operationMetrics) === JSON.stringify(computed.operationMetrics), `${location}.operationMetrics differ from raw result recomputation`);
  validateLoadThresholds(computed, policy, location, violations);
  return violations;
}

export function computeOutageRecoveryMetrics(rawTimeline, policy = CANONICAL_OPERATIONAL_POLICY) {
  return policy.outageRecovery.requiredScenarios.map((category) => {
    const scenario = rawTimeline.scenarios.find((candidate) => candidate.category === category);
    const injectedAt = Date.parse(scenario.injectedAt);
    return {
      scenarioId: scenario.scenarioId,
      category,
      detectionSeconds: (Date.parse(scenario.detectedAt) - injectedAt) / 1000,
      recoverySeconds: (Date.parse(scenario.recoveredAt) - injectedAt) / 1000,
      failClosed: scenario.failClosed,
      alertTriggered: scenario.alertTriggered,
      traceCorrelationVerified: scenario.traceCorrelationVerified,
      retryStormDetected: scenario.retryStormDetected,
      secretLeakCount: scenario.secretLeakCount,
      crossTenantViolationCount: scenario.crossTenantViolationCount,
    };
  });
}

function validateOutageThresholds(metrics, policy, location, violations) {
  const threshold = policy.outageRecovery;
  for (const metric of metrics) {
    const metricLocation = `${location}.scenarioMetrics.${metric.category}`;
    addViolation(violations, metric.detectionSeconds <= threshold.maximumDetectionSeconds, `${metricLocation}.detectionSeconds exceeds policy`);
    addViolation(violations, metric.recoverySeconds <= threshold.maximumRecoverySeconds, `${metricLocation}.recoverySeconds exceeds policy`);
    addViolation(violations, !threshold.requireFailClosed || metric.failClosed, `${metricLocation}.failClosed must be true`);
    addViolation(violations, !threshold.requireAlert || metric.alertTriggered, `${metricLocation}.alertTriggered must be true`);
    addViolation(violations, !threshold.requireTraceCorrelation || metric.traceCorrelationVerified, `${metricLocation}.traceCorrelationVerified must be true`);
    addViolation(violations, threshold.allowRetryStorm || !metric.retryStormDetected, `${metricLocation}.retryStormDetected must be false`);
    addViolation(violations, metric.secretLeakCount <= threshold.maximumSecretLeakCount, `${metricLocation}.secretLeakCount exceeds policy`);
    addViolation(violations, metric.crossTenantViolationCount <= threshold.maximumCrossTenantViolations, `${metricLocation}.crossTenantViolationCount exceeds policy`);
  }
}

export async function validateOutageRecoveryEvidenceRecord(record, expected, policy, workspaceRoot, location) {
  const violations = [];
  collectForbiddenKeys(record, location, violations);
  addViolation(violations, hasExactKeys(record, OUTAGE_REQUIRED_FIELDS), `${location} fields must match the outage evidence contract`);
  validateIdentity(record, expected, policy, "sdkwork.knowledge-engine-outage-recovery-evidence", location, violations);
  addViolation(
    violations,
    normalizeRepositoryPath(record?.runbookRef) === "docs/runbooks/provider-outage.md",
    `${location}.runbookRef must reference the canonical Provider outage runbook`,
  );
  const rawTimeline = await loadJsonArtifact(
    record,
    "rawTimelineRef",
    "rawTimelineSha256",
    policy,
    workspaceRoot,
    location,
    violations,
  );
  if (!rawTimeline) return violations;
  collectForbiddenKeys(rawTimeline, `${location}.rawTimeline`, violations);
  addViolation(
    violations,
    hasExactKeys(rawTimeline, [
      "schemaVersion",
      "kind",
      "providerId",
      "upstreamVersion",
      "adapterCommit",
      "scenarios",
    ]),
    `${location}.rawTimeline fields are invalid`,
  );
  addViolation(violations, rawTimeline?.schemaVersion === 1, `${location}.rawTimeline.schemaVersion must be 1`);
  addViolation(violations, rawTimeline?.kind === "sdkwork.knowledge-engine-outage-timeline", `${location}.rawTimeline.kind is invalid`);
  addViolation(violations, rawTimeline?.providerId === expected.providerId, `${location}.rawTimeline.providerId mismatch`);
  addViolation(violations, rawTimeline?.upstreamVersion === expected.upstreamVersion, `${location}.rawTimeline.upstreamVersion mismatch`);
  addViolation(violations, rawTimeline?.adapterCommit === expected.adapterCommit, `${location}.rawTimeline.adapterCommit mismatch`);
  addViolation(violations, Array.isArray(rawTimeline?.scenarios), `${location}.rawTimeline.scenarios must be an array`);
  const categories = new Set();
  const scenarioIds = new Set();
  const scenarios = Array.isArray(rawTimeline?.scenarios) ? rawTimeline.scenarios : [];
  for (const [index, scenario] of scenarios.entries()) {
    const scenarioLocation = `${location}.rawTimeline.scenarios[${index}]`;
    addViolation(
      violations,
      hasExactKeys(scenario, [
        "scenarioId",
        "category",
        "injectedAt",
        "detectedAt",
        "recoveredAt",
        "failClosed",
        "alertTriggered",
        "traceCorrelationVerified",
        "retryStormDetected",
        "secretLeakCount",
        "crossTenantViolationCount",
      ]),
      `${scenarioLocation} fields are invalid`,
    );
    addViolation(violations, typeof scenario?.scenarioId === "string" && scenario.scenarioId.length > 0, `${scenarioLocation}.scenarioId is required`);
    addViolation(violations, !scenarioIds.has(scenario?.scenarioId), `${scenarioLocation}.scenarioId is duplicated`);
    scenarioIds.add(scenario?.scenarioId);
    addViolation(violations, policy.outageRecovery.requiredScenarios.includes(scenario?.category), `${scenarioLocation}.category is invalid`);
    addViolation(violations, !categories.has(scenario?.category), `${scenarioLocation}.category is duplicated`);
    categories.add(scenario?.category);
    const injectedAt = Date.parse(scenario?.injectedAt ?? "");
    const detectedAt = Date.parse(scenario?.detectedAt ?? "");
    const recoveredAt = Date.parse(scenario?.recoveredAt ?? "");
    addViolation(violations, Number.isFinite(injectedAt) && detectedAt >= injectedAt && recoveredAt >= detectedAt, `${scenarioLocation} timeline is invalid`);
    addViolation(
      violations,
      Number.isFinite(recoveredAt)
        && new Date(recoveredAt).toISOString().slice(0, 10) === expected.verifiedAt,
      `${scenarioLocation}.recoveredAt must match verifiedAt`,
    );
    for (const field of ["failClosed", "alertTriggered", "traceCorrelationVerified", "retryStormDetected"]) {
      addViolation(violations, typeof scenario?.[field] === "boolean", `${scenarioLocation}.${field} is required`);
    }
    for (const field of ["secretLeakCount", "crossTenantViolationCount"]) {
      addViolation(violations, Number.isInteger(scenario?.[field]) && scenario[field] >= 0, `${scenarioLocation}.${field} is invalid`);
    }
  }
  addViolation(
    violations,
    JSON.stringify([...categories].sort()) === JSON.stringify([...policy.outageRecovery.requiredScenarios].sort()),
    `${location}.rawTimeline scenarios must exactly cover the required outage categories`,
  );
  if (violations.some((violation) => violation.includes(`${location}.rawTimeline`))) return violations;
  const computed = computeOutageRecoveryMetrics(rawTimeline, policy);
  addViolation(violations, JSON.stringify(record?.scenarioMetrics) === JSON.stringify(computed), `${location}.scenarioMetrics differ from raw timeline recomputation`);
  validateOutageThresholds(computed, policy, location, violations);
  return violations;
}

export async function loadOperationalEvidencePolicy(policy, workspaceRoot, violations = []) {
  addViolation(
    violations,
    policy?.operationalEvidenceSpec === OPERATIONAL_EVIDENCE_SPEC_PATH,
    `policy.operationalEvidenceSpec must be ${OPERATIONAL_EVIDENCE_SPEC_PATH}`,
  );
  try {
    const loaded = JSON.parse(await readFile(path.join(workspaceRoot, OPERATIONAL_EVIDENCE_SPEC_PATH), "utf8"));
    addViolation(
      violations,
      JSON.stringify(loaded) === JSON.stringify(CANONICAL_OPERATIONAL_POLICY),
      "operational evidence policy must match the canonical non-weakenable policy",
    );
    return loaded;
  } catch {
    violations.push(`operational evidence policy is missing or invalid: ${OPERATIONAL_EVIDENCE_SPEC_PATH}`);
    return CANONICAL_OPERATIONAL_POLICY;
  }
}

async function validateSchema(pathname, kind, requiredFields, workspaceRoot, violations) {
  try {
    const schema = JSON.parse(await readFile(path.join(workspaceRoot, pathname), "utf8"));
    addViolation(violations, schema?.type === "object", `${pathname}: schema must describe an object`);
    addViolation(violations, schema?.additionalProperties === false, `${pathname}: schema must reject unknown fields`);
    addViolation(violations, schema?.properties?.schemaVersion?.const === 1, `${pathname}: schemaVersion must be 1`);
    addViolation(violations, schema?.properties?.kind?.const === kind, `${pathname}: schema kind is invalid`);
    addViolation(violations, JSON.stringify(schema?.required) === JSON.stringify(requiredFields), `${pathname}: required fields must match the validator contract`);
  } catch {
    violations.push(`${pathname}: schema is missing or invalid`);
  }
}

export async function validateOperationalEvidenceSchemas(policy, workspaceRoot) {
  const violations = [];
  addViolation(violations, policy?.loadSloEvidenceSchema === LOAD_SLO_EVIDENCE_SCHEMA_PATH, `policy.loadSloEvidenceSchema must be ${LOAD_SLO_EVIDENCE_SCHEMA_PATH}`);
  addViolation(violations, policy?.outageRecoveryEvidenceSchema === OUTAGE_RECOVERY_EVIDENCE_SCHEMA_PATH, `policy.outageRecoveryEvidenceSchema must be ${OUTAGE_RECOVERY_EVIDENCE_SCHEMA_PATH}`);
  await loadOperationalEvidencePolicy(policy, workspaceRoot, violations);
  await Promise.all([
    validateSchema(LOAD_SLO_EVIDENCE_SCHEMA_PATH, "sdkwork.knowledge-engine-load-slo-evidence", LOAD_REQUIRED_FIELDS, workspaceRoot, violations),
    validateSchema(OUTAGE_RECOVERY_EVIDENCE_SCHEMA_PATH, "sdkwork.knowledge-engine-outage-recovery-evidence", OUTAGE_REQUIRED_FIELDS, workspaceRoot, violations),
  ]);
  return violations;
}
