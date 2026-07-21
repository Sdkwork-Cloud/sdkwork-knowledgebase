import { readFile } from "node:fs/promises";
import path from "node:path";

import {
  evaluateRetrieval,
  validateRetrievalDataset,
  validateRetrievalResults,
} from "./evaluate_knowledge_engine_retrieval.mjs";
import {
  isCertificationArtifactReference,
  normalizeCertificationArtifactReference,
  readBoundedCertificationArtifact,
} from "./provider_certification_artifact.mjs";

export const QUALITY_EVIDENCE_SCHEMA_PATH =
  "docs/releases/provider-certification/quality-evaluation-evidence.schema.json";

const ARTIFACT_PAIRS = Object.freeze([
  ["datasetRef", "datasetSha256"],
  ["resultsRef", "resultsSha256"],
  ["evaluationReportRef", "evaluationReportSha256"],
]);
const METRIC_FIELDS = Object.freeze([
  "recallAtK",
  "mrr",
  "ndcgAtK",
  "citationCorrectness",
  "failureRate",
  "p95LatencyMs",
  "emptyQueryPassRate",
  "scoredQueryCount",
]);
const allowedFields = new Set([
  "schemaVersion",
  "kind",
  "status",
  "providerId",
  "upstreamVersion",
  "adapterCommit",
  "datasetId",
  "datasetVersion",
  "datasetClassification",
  "datasetRef",
  "datasetSha256",
  "resultsRef",
  "resultsSha256",
  "evaluationReportRef",
  "evaluationReportSha256",
  "environment",
  "workflowRunRef",
  "verifiedAt",
  "expiresAt",
  "reviewedBy",
  "metrics",
]);
const datePattern = /^\d{4}-\d{2}-\d{2}$/u;
const gitCommitPattern = /^[a-f0-9]{40}$/u;
const sha256Pattern = /^[a-f0-9]{64}$/u;
const forbiddenKeyPattern = /(credential|secret|password|api.?key|access.?token|authorization)/iu;

function addViolation(violations, condition, message) {
  if (!condition) violations.push(message);
}

function collectForbiddenKeys(value, location, violations) {
  if (Array.isArray(value)) {
    value.forEach((item, index) => collectForbiddenKeys(item, `${location}[${index}]`, violations));
    return;
  }
  if (!value || typeof value !== "object") return;
  for (const [key, nested] of Object.entries(value)) {
    if (forbiddenKeyPattern.test(key)) violations.push(`${location}.${key}: secret-bearing fields are forbidden`);
    collectForbiddenKeys(nested, `${location}.${key}`, violations);
  }
}

async function loadVerifiedArtifact(
  record,
  refField,
  digestField,
  productionPolicy,
  workspaceRoot,
  location,
  violations,
) {
  const reference = normalizeCertificationArtifactReference(record?.[refField]);
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
  if (!isCertificationArtifactReference(reference)) return undefined;
  try {
    const { bytes, digest } = await readBoundedCertificationArtifact(
      reference,
      workspaceRoot,
      productionPolicy.maxArtifactBytes,
    );
    addViolation(violations, record[digestField] === digest, `${location}.${digestField} does not match ${reference}`);
    return bytes;
  } catch (error) {
    violations.push(`${location}.${refField} is missing or invalid: ${reference}: ${error.message}`);
    return undefined;
  }
}

export async function validateQualityEvidenceSchema(spec, workspaceRoot) {
  const violations = [];
  addViolation(
    violations,
    spec?.productionEvidenceSchema === QUALITY_EVIDENCE_SCHEMA_PATH,
    `productionEvidenceSchema must be ${QUALITY_EVIDENCE_SCHEMA_PATH}`,
  );
  try {
    const schema = JSON.parse(await readFile(path.join(workspaceRoot, QUALITY_EVIDENCE_SCHEMA_PATH), "utf8"));
    addViolation(violations, schema?.type === "object", "quality evidence schema must describe an object");
    addViolation(violations, schema?.additionalProperties === false, "quality evidence schema must reject unknown fields");
    addViolation(violations, schema?.properties?.schemaVersion?.const === 1, "quality evidence schemaVersion must be 1");
    addViolation(
      violations,
      schema?.properties?.kind?.const === "sdkwork.knowledge-engine-quality-evaluation-evidence",
      "quality evidence schema kind is invalid",
    );
    addViolation(
      violations,
      JSON.stringify(schema?.required) === JSON.stringify([...allowedFields]),
      "quality evidence schema required fields must match the validator contract",
    );
    addViolation(
      violations,
      JSON.stringify(schema?.properties?.metrics?.required) === JSON.stringify(spec?.productionPolicy?.requiredMetrics),
      "quality evidence schema metrics must match the evaluation policy",
    );
    addViolation(
      violations,
      schema?.properties?.metrics?.properties?.scoredQueryCount?.minimum
        === spec?.productionPolicy?.minimumScoredQueries,
      "quality evidence schema minimum query count must match the evaluation policy",
    );
    addViolation(
      violations,
      schema?.properties?.metrics?.properties?.scoredQueryCount?.maximum
        === spec?.productionPolicy?.maximumScoredQueries,
      "quality evidence schema maximum query count must match the evaluation policy",
    );
  } catch {
    violations.push(`quality evidence schema is missing or invalid: ${QUALITY_EVIDENCE_SCHEMA_PATH}`);
  }
  return violations;
}

export async function validateProductionQualityEvidenceRecord(
  record,
  expected,
  productionPolicy,
  workspaceRoot,
  location = "qualityEvidence",
) {
  const violations = [];
  collectForbiddenKeys(record, location, violations);
  addViolation(violations, record?.schemaVersion === 1, `${location}.schemaVersion must be 1`);
  addViolation(
    violations,
    record?.kind === "sdkwork.knowledge-engine-quality-evaluation-evidence",
    `${location}.kind is invalid`,
  );
  addViolation(violations, record?.status === "passed", `${location}.status must be passed`);
  addViolation(
    violations,
    Object.keys(record ?? {}).every((field) => allowedFields.has(field)),
    `${location} contains an unknown field`,
  );
  addViolation(
    violations,
    /^[a-z0-9][a-z0-9_-]*$/u.test(record?.providerId ?? "") && record.providerId === expected.providerId,
    `${location}.providerId mismatch or invalid`,
  );
  addViolation(
    violations,
    typeof record?.upstreamVersion === "string"
      && record.upstreamVersion.length > 0
      && !/latest|[*xX]/u.test(record.upstreamVersion)
      && record.upstreamVersion === expected.upstreamVersion,
    `${location}.upstreamVersion must match a pinned version`,
  );
  addViolation(
    violations,
    gitCommitPattern.test(record?.adapterCommit ?? "") && record.adapterCommit === expected.adapterCommit,
    `${location}.adapterCommit must match a full Git commit`,
  );
  addViolation(violations, record?.datasetClassification === "production-domain", `${location}.datasetClassification must be production-domain`);
  addViolation(violations, typeof record?.datasetId === "string" && record.datasetId.length > 0, `${location}.datasetId is required`);
  addViolation(violations, /^\d+\.\d+\.\d+$/u.test(record?.datasetVersion ?? ""), `${location}.datasetVersion must be semantic version`);
  addViolation(violations, record?.environment === "release", `${location}.environment must be release`);
  addViolation(violations, /^https:\/\//u.test(record?.workflowRunRef ?? ""), `${location}.workflowRunRef must be HTTPS`);
  const reviewers = Array.isArray(record?.reviewedBy) ? record.reviewedBy : [];
  addViolation(
    violations,
    reviewers.length >= productionPolicy.minimumReviewers
      && new Set(reviewers).size === reviewers.length
      && reviewers.every((reviewer) => typeof reviewer === "string" && reviewer.length > 0),
    `${location}.reviewedBy requires ${productionPolicy.minimumReviewers} distinct reviewers`,
  );
  addViolation(
    violations,
    JSON.stringify(Object.keys(record?.metrics ?? {})) === JSON.stringify(METRIC_FIELDS),
    `${location}.metrics must contain the canonical ordered metrics`,
  );

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
      && expiryTime <= verifiedTime + productionPolicy.maximumEvidenceAgeDays * 86_400_000
      && Date.now() <= expiryTime + 86_399_999,
    `${location}.verifiedAt/expiresAt must be current, non-future, and within the quality evidence policy`,
  );

  const [datasetBytes, resultsBytes, reportBytes] = await Promise.all(
    ARTIFACT_PAIRS.map(([refField, digestField]) => loadVerifiedArtifact(
      record,
      refField,
      digestField,
      productionPolicy,
      workspaceRoot,
      location,
      violations,
    )),
  );
  if (datasetBytes && resultsBytes && reportBytes) {
    try {
      const dataset = JSON.parse(datasetBytes.toString("utf8"));
      const results = JSON.parse(resultsBytes.toString("utf8"));
      const report = JSON.parse(reportBytes.toString("utf8"));
      validateRetrievalDataset(dataset, productionPolicy);
      validateRetrievalResults(dataset, results);
      const computed = evaluateRetrieval(dataset, results, productionPolicy);
      addViolation(violations, dataset.datasetId === record.datasetId, `${location}.datasetId mismatch`);
      addViolation(violations, dataset.version === record.datasetVersion, `${location}.datasetVersion mismatch`);
      addViolation(violations, results.providerId === record.providerId, `${location}: result providerId mismatch`);
      addViolation(violations, results.providerVersion === record.upstreamVersion, `${location}: result providerVersion mismatch`);
      addViolation(violations, report?.passed === true && computed.passed === true, `${location}: evaluation thresholds must pass`);
      addViolation(
        violations,
        report?.kind === computed.kind
          && report?.schemaVersion === computed.schemaVersion
          && report?.evidenceClass === "production-domain"
          && report?.datasetId === record.datasetId
          && report?.datasetVersion === record.datasetVersion
          && report?.providerId === record.providerId
          && report?.providerVersion === record.upstreamVersion,
        `${location}: evaluation report identity or classification mismatch`,
      );
      addViolation(
        violations,
        JSON.stringify(report?.metrics) === JSON.stringify(computed.metrics)
          && JSON.stringify(record.metrics) === JSON.stringify(computed.metrics),
        `${location}: recorded metrics must match deterministic recomputation`,
      );
    } catch (error) {
      violations.push(`${location}: referenced quality artifacts are invalid: ${error.message}`);
    }
  }
  return violations;
}

export async function validateKnowledgeEngineEvaluationWorkspace(workspaceRoot) {
  const violations = [];
  const specPath = path.join(workspaceRoot, "specs/knowledge-engine-evaluation.spec.json");
  try {
    const spec = JSON.parse(await readFile(specPath, "utf8"));
    addViolation(violations, spec?.schemaVersion === 1, "evaluation spec schemaVersion must be 1");
    addViolation(
      violations,
      spec?.kind === "sdkwork.knowledge-engine-evaluation.spec",
      "evaluation spec kind is invalid",
    );
    addViolation(violations, /^\d+\.\d+\.\d+$/u.test(spec?.contractVersion ?? ""), "evaluation contractVersion is invalid");
    addViolation(
      violations,
      Number.isInteger(spec?.productionPolicy?.minimumScoredQueries)
        && spec.productionPolicy.minimumScoredQueries >= 50,
      "production evaluation requires at least 50 scored queries",
    );
    addViolation(
      violations,
      Number.isInteger(spec?.productionPolicy?.minimumReviewers)
        && spec.productionPolicy.minimumReviewers >= 2,
      "production evaluation requires at least two reviewers",
    );
    addViolation(
      violations,
      Number.isInteger(spec?.productionPolicy?.maximumScoredQueries)
        && spec.productionPolicy.maximumScoredQueries >= spec.productionPolicy.minimumScoredQueries
        && spec.productionPolicy.maximumScoredQueries <= 5000,
      "production evaluation scored query count must be bounded at 5000",
    );
    addViolation(
      violations,
      Number.isInteger(spec?.productionPolicy?.maximumRejectionQueries)
        && spec.productionPolicy.maximumRejectionQueries >= spec.productionPolicy.minimumRejectionQueries
        && spec.productionPolicy.maximumRejectionQueries <= 500,
      "production evaluation rejection query count must be bounded at 500",
    );
    addViolation(
      violations,
      Number.isSafeInteger(spec?.productionPolicy?.maxArtifactBytes)
        && spec.productionPolicy.maxArtifactBytes > 0
        && spec.productionPolicy.maxArtifactBytes <= 33_554_432,
      "production evaluation artifacts must be bounded at 32 MiB",
    );
    addViolation(
      violations,
      spec?.productionPolicy?.thresholdBounds?.minRecallAtK >= 0.75
        && spec.productionPolicy.thresholdBounds.minCitationCorrectness >= 0.98
        && spec.productionPolicy.thresholdBounds.maxFailureRate <= 0.01
        && spec.productionPolicy.thresholdBounds.maxP95LatencyMs <= 2000,
      "production evaluation threshold bounds are weaker than the commercial baseline",
    );
    violations.push(...await validateQualityEvidenceSchema(spec, workspaceRoot));

    const dataset = JSON.parse(await readFile(path.join(workspaceRoot, spec.contractFixtureDataset), "utf8"));
    const results = JSON.parse(await readFile(path.join(workspaceRoot, spec.contractFixtureResults), "utf8"));
    addViolation(
      violations,
      dataset.classification === "contract-fixture",
      "the checked-in sample dataset must remain a contract fixture",
    );
    const report = evaluateRetrieval(dataset, results);
    addViolation(violations, report.passed === true, "the evaluator contract fixture must pass");
    addViolation(
      violations,
      report.evidenceClass === "contract-fixture",
      "contract evaluation output must not claim production evidence",
    );

    const template = JSON.parse(await readFile(path.join(workspaceRoot, spec.productionEvidenceTemplate), "utf8"));
    const templateViolations = await validateProductionQualityEvidenceRecord(
      template,
      {
        providerId: template.providerId,
        upstreamVersion: template.upstreamVersion,
        adapterCommit: template.adapterCommit,
      },
      spec.productionPolicy,
      workspaceRoot,
      "qualityTemplate",
    );
    addViolation(
      violations,
      templateViolations.some((violation) => violation.includes("kind is invalid"))
        && templateViolations.some((violation) => violation.includes("status must be passed"))
        && templateViolations.some((violation) => violation.includes("datasetClassification must be production-domain")),
      "quality evidence template must remain non-certifiable",
    );
  } catch (error) {
    violations.push(`knowledge engine evaluation workspace is invalid: ${error.message}`);
  }
  return violations;
}
