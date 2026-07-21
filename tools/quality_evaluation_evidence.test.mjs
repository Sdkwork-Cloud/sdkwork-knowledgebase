import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { evaluateRetrieval } from "./evaluate_knowledge_engine_retrieval.mjs";
import {
  validateKnowledgeEngineEvaluationWorkspace,
  validateProductionQualityEvidenceRecord,
} from "./quality_evaluation_evidence.mjs";

const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const evaluationSpec = JSON.parse(await readFile(
  path.join(workspaceRoot, "specs/knowledge-engine-evaluation.spec.json"),
  "utf8",
));

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function isoDateFromNow(days) {
  return new Date(Date.now() + days * 86_400_000).toISOString().slice(0, 10);
}

function productionFixture() {
  const scoredQueries = Array.from({ length: 50 }, (_, index) => ({
    id: `scored-${index + 1}`,
    query: `Reviewed production-domain question ${index + 1}`,
    relevantDocumentIds: [`document-${index + 1}`],
    expectedCitationDocumentIds: [`document-${index + 1}`],
  }));
  const rejectionQueries = Array.from({ length: 3 }, (_, index) => ({
    id: `rejection-${index + 1}`,
    query: "",
    expectRejection: true,
  }));
  const dataset = {
    schemaVersion: 1,
    kind: "sdkwork.knowledge-engine-retrieval-golden-dataset",
    classification: "production-domain",
    datasetId: "test-only-production-domain",
    version: "1.0.0",
    defaultTopK: 5,
    thresholds: {
      minRecallAtK: 0.9,
      minMrr: 0.9,
      minNdcgAtK: 0.9,
      minCitationCorrectness: 1,
      minEmptyQueryPassRate: 1,
      maxFailureRate: 0,
      maxP95LatencyMs: 800,
    },
    queries: [...scoredQueries, ...rejectionQueries],
  };
  const results = {
    schemaVersion: 1,
    kind: "sdkwork.knowledge-engine-retrieval-results",
    datasetId: dataset.datasetId,
    datasetVersion: dataset.version,
    providerId: "dify",
    providerVersion: "1.2.3",
    runs: [
      ...scoredQueries.map((query, index) => ({
        queryId: query.id,
        latencyMs: 20 + index,
        failed: false,
        hits: [{
          documentId: query.relevantDocumentIds[0],
          citationDocumentId: query.expectedCitationDocumentIds[0],
        }],
      })),
      ...rejectionQueries.map((query) => ({ queryId: query.id, rejected: true, hits: [] })),
    ],
  };
  return { dataset, results };
}

test("knowledge engine evaluation workspace keeps fixtures non-production", async () => {
  assert.deepEqual(await validateKnowledgeEngineEvaluationWorkspace(workspaceRoot), []);
});

test("production datasets cannot weaken commercial thresholds", () => {
  const { dataset, results } = productionFixture();
  dataset.thresholds.minRecallAtK = 0;
  assert.throws(
    () => evaluateRetrieval(dataset, results, evaluationSpec.productionPolicy),
    /threshold minRecallAtK must be at least/,
  );
});

test("production quality datasets and artifacts remain bounded", () => {
  const { dataset, results } = productionFixture();
  const boundedPolicy = structuredClone(evaluationSpec.productionPolicy);
  boundedPolicy.maximumScoredQueries = 49;
  assert.throws(
    () => evaluateRetrieval(dataset, results, boundedPolicy),
    /allows at most 49 scored queries/,
  );
});

test("production quality evidence verifies artifacts and deterministic metrics", async (context) => {
  const temporaryRoot = await mkdtemp(path.join(tmpdir(), "sdkwork-quality-evidence-"));
  context.after(() => rm(temporaryRoot, { recursive: true, force: true }));
  const artifactRoot = path.join(
    temporaryRoot,
    "docs/releases/provider-certification/artifacts/test-quality",
  );
  await mkdir(artifactRoot, { recursive: true });

  const { dataset, results } = productionFixture();
  const report = evaluateRetrieval(dataset, results, evaluationSpec.productionPolicy);
  const datasetBytes = Buffer.from(`${JSON.stringify(dataset, null, 2)}\n`);
  const resultsBytes = Buffer.from(`${JSON.stringify(results, null, 2)}\n`);
  const reportBytes = Buffer.from(`${JSON.stringify(report, null, 2)}\n`);
  const datasetRef = "docs/releases/provider-certification/artifacts/test-quality/dataset.json";
  const resultsRef = "docs/releases/provider-certification/artifacts/test-quality/results.json";
  const evaluationReportRef = "docs/releases/provider-certification/artifacts/test-quality/report.json";
  await Promise.all([
    writeFile(path.join(temporaryRoot, datasetRef), datasetBytes),
    writeFile(path.join(temporaryRoot, resultsRef), resultsBytes),
    writeFile(path.join(temporaryRoot, evaluationReportRef), reportBytes),
  ]);

  const record = {
    schemaVersion: 1,
    kind: "sdkwork.knowledge-engine-quality-evaluation-evidence",
    status: "passed",
    providerId: "dify",
    upstreamVersion: "1.2.3",
    adapterCommit: "a".repeat(40),
    datasetId: dataset.datasetId,
    datasetVersion: dataset.version,
    datasetClassification: dataset.classification,
    datasetRef,
    datasetSha256: sha256(datasetBytes),
    resultsRef,
    resultsSha256: sha256(resultsBytes),
    evaluationReportRef,
    evaluationReportSha256: sha256(reportBytes),
    environment: "release",
    workflowRunRef: "https://ci.example.test/runs/quality-1",
    verifiedAt: isoDateFromNow(0),
    expiresAt: isoDateFromNow(30),
    reviewedBy: ["quality-owner", "domain-owner"],
    metrics: report.metrics,
  };
  const expected = {
    providerId: record.providerId,
    upstreamVersion: record.upstreamVersion,
    adapterCommit: record.adapterCommit,
  };
  assert.deepEqual(
    await validateProductionQualityEvidenceRecord(
      record,
      expected,
      evaluationSpec.productionPolicy,
      temporaryRoot,
    ),
    [],
  );

  const byteBoundedPolicy = structuredClone(evaluationSpec.productionPolicy);
  byteBoundedPolicy.maxArtifactBytes = 1;
  const byteBoundedViolations = await validateProductionQualityEvidenceRecord(
    record,
    expected,
    byteBoundedPolicy,
    temporaryRoot,
  );
  assert.ok(byteBoundedViolations.some((violation) => violation.includes("1 byte limit")));

  const futureRecord = {
    ...record,
    verifiedAt: isoDateFromNow(1),
  };
  const futureViolations = await validateProductionQualityEvidenceRecord(
    futureRecord,
    expected,
    evaluationSpec.productionPolicy,
    temporaryRoot,
  );
  assert.ok(futureViolations.some((violation) => violation.includes("non-future")));

  record.metrics.recallAtK = 0;
  const violations = await validateProductionQualityEvidenceRecord(
    record,
    expected,
    evaluationSpec.productionPolicy,
    temporaryRoot,
  );
  assert.ok(violations.some((violation) => violation.includes("deterministic recomputation")));
});
