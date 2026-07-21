import { readFile } from "node:fs/promises";
import path from "node:path";

import { validateProductionQualityEvidenceRecord } from "./quality_evaluation_evidence.mjs";
import {
  isCertificationArtifactReference,
  normalizeCertificationArtifactReference,
  readBoundedCertificationArtifact,
} from "./provider_certification_artifact.mjs";
import {
  loadOperationalEvidencePolicy,
  validateLoadSloEvidenceRecord,
  validateOperationalEvidenceSchemas,
  validateOutageRecoveryEvidenceRecord,
} from "./provider_operational_evidence.mjs";

export const LIVE_EVIDENCE_SCHEMA_PATH =
  "docs/releases/provider-certification/live-certification-evidence.schema.json";

export const LIVE_EVIDENCE_RECORD_FIELDS = Object.freeze([
  "providerId",
  "upstreamVersion",
  "adapterCommit",
  "contractSuiteVersion",
  "verifiedAt",
  "expiresAt",
  "environment",
  "workflowRunRef",
  "reviewedBy",
  "qualityEvaluationRef",
  "qualityEvaluationSha256",
  "contractReportRef",
  "contractReportSha256",
  "loadSloRef",
  "loadSloSha256",
  "outageRecoveryRef",
  "outageRecoverySha256",
  "licensingReviewRef",
  "licensingReviewSha256",
  "licensingApproval",
  "securityPrivacyReviewRef",
  "securityPrivacyReviewSha256",
  "securityPrivacyApproval",
]);

const LIVE_ARTIFACT_PAIRS = Object.freeze([
  ["qualityEvaluationRef", "qualityEvaluationSha256"],
  ["contractReportRef", "contractReportSha256"],
  ["loadSloRef", "loadSloSha256"],
  ["outageRecoveryRef", "outageRecoverySha256"],
  ["licensingReviewRef", "licensingReviewSha256"],
  ["securityPrivacyReviewRef", "securityPrivacyReviewSha256"],
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
    if (forbiddenKeyPattern.test(key)) {
      violations.push(`${location}.${key}: secret-bearing fields are forbidden`);
    }
    collectForbiddenKeys(nested, `${location}.${key}`, violations);
  }
}

function normalizeRepositoryPath(value) {
  return normalizeCertificationArtifactReference(value);
}

async function verifyArtifact(
  record,
  refField,
  digestField,
  maxArtifactBytes,
  workspaceRoot,
  location,
  violations,
) {
  const reference = normalizeRepositoryPath(record[refField]);
  addViolation(
    violations,
    isCertificationArtifactReference(reference),
    `${location}.${refField} must reference a certification artifact`,
  );
  addViolation(
    violations,
    sha256Pattern.test(record[digestField] ?? ""),
    `${location}.${digestField} must be a SHA-256 digest`,
  );
  if (!isCertificationArtifactReference(reference)) {
    return;
  }
  try {
    const { digest } = await readBoundedCertificationArtifact(
      reference,
      workspaceRoot,
      maxArtifactBytes,
    );
    addViolation(
      violations,
      record[digestField] === digest,
      `${location}.${digestField} does not match ${reference}`,
    );
  } catch (error) {
    violations.push(`${location}.${refField} is missing or invalid: ${reference}: ${error.message}`);
  }
}

export async function validateLiveEvidenceSchema(policy, workspaceRoot) {
  const violations = [];
  addViolation(
    violations,
    policy?.liveEvidenceSchema === LIVE_EVIDENCE_SCHEMA_PATH,
    `policy.liveEvidenceSchema must be ${LIVE_EVIDENCE_SCHEMA_PATH}`,
  );
  addViolation(
    violations,
    JSON.stringify(policy?.liveEvidenceRecordFields) === JSON.stringify(LIVE_EVIDENCE_RECORD_FIELDS),
    "policy.liveEvidenceRecordFields must match the canonical ordered fields",
  );
  try {
    const schema = JSON.parse(await readFile(path.join(workspaceRoot, LIVE_EVIDENCE_SCHEMA_PATH), "utf8"));
    const expectedRequired = ["schemaVersion", "kind", "status", ...LIVE_EVIDENCE_RECORD_FIELDS];
    addViolation(violations, schema?.type === "object", "live evidence schema must describe an object");
    addViolation(violations, schema?.additionalProperties === false, "live evidence schema must reject unknown fields");
    addViolation(
      violations,
      schema?.properties?.schemaVersion?.const === 1,
      "live evidence schemaVersion must be 1",
    );
    addViolation(
      violations,
      schema?.properties?.kind?.const === "sdkwork.knowledge-engine-live-certification-evidence",
      "live evidence schema kind is invalid",
    );
    addViolation(
      violations,
      JSON.stringify(schema?.required) === JSON.stringify(expectedRequired),
      "live evidence schema required fields must match the validator contract",
    );
  } catch {
    violations.push(`live evidence schema is missing or invalid: ${LIVE_EVIDENCE_SCHEMA_PATH}`);
  }
  violations.push(...await validateOperationalEvidenceSchemas(policy, workspaceRoot));
  return violations;
}

export async function validateLiveCertificationEvidenceRecord(
  record,
  { providerId, upstreamVersion, contractSuiteVersion, verifiedAt, licensingApproval, securityPrivacyApproval },
  policy,
  workspaceRoot,
  location,
) {
  const violations = [];
  collectForbiddenKeys(record, location, violations);
  addViolation(violations, record?.schemaVersion === 1, `${location}.schemaVersion must be 1`);
  addViolation(
    violations,
    record?.kind === "sdkwork.knowledge-engine-live-certification-evidence",
    `${location}.kind is invalid`,
  );
  addViolation(violations, record?.status === "certified", `${location}.status must be certified`);
  for (const field of LIVE_EVIDENCE_RECORD_FIELDS) {
    addViolation(
      violations,
      typeof record?.[field] === "string" && record[field].length > 0,
      `${location}.${field} is required`,
    );
  }
  addViolation(violations, record?.providerId === providerId, `${location}.providerId mismatch`);
  addViolation(violations, record?.upstreamVersion === upstreamVersion, `${location}.upstreamVersion mismatch`);
  addViolation(
    violations,
    record?.contractSuiteVersion === contractSuiteVersion,
    `${location}.contractSuiteVersion mismatch`,
  );
  addViolation(violations, record?.verifiedAt === verifiedAt, `${location}.verifiedAt mismatch`);
  addViolation(violations, record?.environment === "release", `${location}.environment must be release`);
  addViolation(
    violations,
    gitCommitPattern.test(record?.adapterCommit ?? ""),
    `${location}.adapterCommit must be a full Git commit`,
  );
  addViolation(
    violations,
    /^https:\/\//u.test(record?.workflowRunRef ?? ""),
    `${location}.workflowRunRef must be an HTTPS release workflow reference`,
  );
  addViolation(
    violations,
    record?.licensingApproval === "approved" && record.licensingApproval === licensingApproval,
    `${location}.licensingApproval must match an approved manifest gate`,
  );
  addViolation(
    violations,
    record?.securityPrivacyApproval === "approved"
      && record.securityPrivacyApproval === securityPrivacyApproval,
    `${location}.securityPrivacyApproval must match an approved manifest gate`,
  );

  const verifiedTime = datePattern.test(record?.verifiedAt ?? "")
    ? Date.parse(`${record.verifiedAt}T00:00:00Z`)
    : Number.NaN;
  const expiryTime = datePattern.test(record?.expiresAt ?? "")
    ? Date.parse(`${record.expiresAt}T00:00:00Z`)
    : Number.NaN;
  const maximumExpiry = verifiedTime + policy.liveEvidenceMaxAgeDays * 86_400_000;
  addViolation(
    violations,
    Number.isFinite(verifiedTime)
      && Number.isFinite(expiryTime)
      && verifiedTime <= Date.now()
      && expiryTime > verifiedTime
      && expiryTime <= maximumExpiry
      && Date.now() <= expiryTime + 86_399_999,
    `${location}.verifiedAt/expiresAt must be current, non-future, and within the policy evidence age`,
  );

  const operationalPolicy = await loadOperationalEvidencePolicy(policy, workspaceRoot, violations);
  await Promise.all(
    LIVE_ARTIFACT_PAIRS.map(([refField, digestField]) => verifyArtifact(
      record,
      refField,
      digestField,
      operationalPolicy.maxArtifactBytes,
      workspaceRoot,
      location,
      violations,
    )),
  );
  const qualityReference = normalizeRepositoryPath(record?.qualityEvaluationRef);
  if (isCertificationArtifactReference(qualityReference)) {
    try {
      const { bytes } = await readBoundedCertificationArtifact(
        qualityReference,
        workspaceRoot,
        operationalPolicy.maxArtifactBytes,
      );
      const qualityRecord = JSON.parse(bytes.toString("utf8"));
      const evaluationSpec = JSON.parse(await readFile(
        path.join(workspaceRoot, "specs/knowledge-engine-evaluation.spec.json"),
        "utf8",
      ));
      violations.push(...await validateProductionQualityEvidenceRecord(
        qualityRecord,
        {
          providerId,
          upstreamVersion,
          adapterCommit: record.adapterCommit,
        },
        evaluationSpec.productionPolicy,
        workspaceRoot,
        `${location}.qualityEvaluation`,
      ));
    } catch (error) {
      violations.push(`${location}.qualityEvaluationRef is not valid quality evidence: ${error.message}`);
    }
  }
  for (const [referenceField, validator, evidenceLocation] of [
    ["loadSloRef", validateLoadSloEvidenceRecord, "loadSlo"],
    ["outageRecoveryRef", validateOutageRecoveryEvidenceRecord, "outageRecovery"],
  ]) {
    const reference = normalizeRepositoryPath(record?.[referenceField]);
    if (!isCertificationArtifactReference(reference)) {
      continue;
    }
    try {
      const { bytes } = await readBoundedCertificationArtifact(
        reference,
        workspaceRoot,
        operationalPolicy.maxArtifactBytes,
      );
      const operationalRecord = JSON.parse(bytes.toString("utf8"));
      violations.push(...await validator(
        operationalRecord,
        {
          providerId,
          upstreamVersion,
          adapterCommit: record.adapterCommit,
          verifiedAt: record.verifiedAt,
        },
        operationalPolicy,
        workspaceRoot,
        `${location}.${evidenceLocation}`,
      ));
    } catch (error) {
      violations.push(`${location}.${referenceField} is not valid operational evidence: ${error.message}`);
    }
  }
  return violations;
}
