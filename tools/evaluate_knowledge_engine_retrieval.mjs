#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

function argument(name) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

function fail(message) {
  console.error(message);
  process.exit(1);
}

function mean(values) {
  return values.length === 0
    ? 0
    : values.reduce((total, value) => total + value, 0) / values.length;
}

function dcg(relevance) {
  return relevance.reduce(
    (total, value, index) => total + value / Math.log2(index + 2),
    0,
  );
}

function percentile95(values) {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.max(0, Math.ceil(sorted.length * 0.95) - 1)];
}

function round(value) {
  return Number(value.toFixed(6));
}

function requireCondition(condition, message) {
  if (!condition) throw new Error(message);
}

function requireUnitInterval(value, location) {
  requireCondition(Number.isFinite(value) && value >= 0 && value <= 1, `${location} must be between 0 and 1`);
}

export function validateRetrievalDataset(dataset, productionPolicy = undefined) {
  requireCondition(dataset?.schemaVersion === 1, "dataset schemaVersion must be 1");
  requireCondition(
    dataset?.kind === "sdkwork.knowledge-engine-retrieval-golden-dataset",
    "dataset kind is invalid",
  );
  requireCondition(
    ["contract-fixture", "production-domain"].includes(dataset?.classification),
    "dataset classification must be contract-fixture or production-domain",
  );
  requireCondition(typeof dataset.datasetId === "string" && dataset.datasetId.length > 0, "datasetId is required");
  requireCondition(/^\d+\.\d+\.\d+$/u.test(dataset.version ?? ""), "dataset version must be semantic version");
  requireCondition(Array.isArray(dataset.queries) && dataset.queries.length > 0, "dataset queries are required");
  requireCondition(
    Number.isInteger(dataset.defaultTopK) && dataset.defaultTopK >= 1 && dataset.defaultTopK <= 200,
    "dataset defaultTopK must be between 1 and 200",
  );

  const thresholds = dataset.thresholds ?? {};
  for (const key of [
    "minRecallAtK",
    "minMrr",
    "minNdcgAtK",
    "minCitationCorrectness",
    "minEmptyQueryPassRate",
    "maxFailureRate",
  ]) {
    requireUnitInterval(thresholds[key], `dataset.thresholds.${key}`);
  }
  requireCondition(
    Number.isFinite(thresholds.maxP95LatencyMs) && thresholds.maxP95LatencyMs > 0,
    "dataset.thresholds.maxP95LatencyMs must be positive",
  );

  const queryIds = new Set();
  let scoredQueryCount = 0;
  let rejectionQueryCount = 0;
  for (const query of dataset.queries) {
    requireCondition(typeof query?.id === "string" && query.id.length > 0, "query id is required");
    requireCondition(!queryIds.has(query.id), `duplicate dataset query id ${query.id}`);
    queryIds.add(query.id);
    requireCondition(typeof query.query === "string", `query ${query.id} text must be a string`);
    if (query.expectRejection === true) {
      rejectionQueryCount += 1;
      continue;
    }
    scoredQueryCount += 1;
    requireCondition(
      Array.isArray(query.relevantDocumentIds) && query.relevantDocumentIds.length > 0,
      `query ${query.id} must declare relevantDocumentIds`,
    );
    requireCondition(
      new Set(query.relevantDocumentIds).size === query.relevantDocumentIds.length,
      `query ${query.id} contains duplicate relevantDocumentIds`,
    );
    requireCondition(
      query.relevantDocumentIds.every((documentId) => typeof documentId === "string" && documentId.length > 0),
      `query ${query.id} contains an invalid relevantDocumentId`,
    );
    if (query.topK !== undefined) {
      requireCondition(Number.isInteger(query.topK) && query.topK >= 1 && query.topK <= 200, `query ${query.id} topK is invalid`);
    }
  }

  if (dataset.classification === "production-domain") {
    requireCondition(productionPolicy, "production-domain dataset requires production policy");
    requireCondition(
      scoredQueryCount >= productionPolicy.minimumScoredQueries,
      `production-domain dataset requires at least ${productionPolicy.minimumScoredQueries} scored queries`,
    );
    requireCondition(
      rejectionQueryCount >= productionPolicy.minimumRejectionQueries,
      `production-domain dataset requires at least ${productionPolicy.minimumRejectionQueries} rejection queries`,
    );
    requireCondition(
      scoredQueryCount <= productionPolicy.maximumScoredQueries,
      `production-domain dataset allows at most ${productionPolicy.maximumScoredQueries} scored queries`,
    );
    requireCondition(
      rejectionQueryCount <= productionPolicy.maximumRejectionQueries,
      `production-domain dataset allows at most ${productionPolicy.maximumRejectionQueries} rejection queries`,
    );
    for (const key of [
      "minRecallAtK",
      "minMrr",
      "minNdcgAtK",
      "minCitationCorrectness",
      "minEmptyQueryPassRate",
    ]) {
      requireCondition(
        thresholds[key] >= productionPolicy.thresholdBounds[key],
        `production-domain threshold ${key} must be at least ${productionPolicy.thresholdBounds[key]}`,
      );
    }
    for (const key of ["maxFailureRate", "maxP95LatencyMs"]) {
      requireCondition(
        thresholds[key] <= productionPolicy.thresholdBounds[key],
        `production-domain threshold ${key} must not exceed ${productionPolicy.thresholdBounds[key]}`,
      );
    }
  }
  return { scoredQueryCount, rejectionQueryCount };
}

export function validateRetrievalResults(dataset, results) {
  requireCondition(results?.schemaVersion === 1, "results schemaVersion must be 1");
  requireCondition(
    results?.kind === "sdkwork.knowledge-engine-retrieval-results",
    "results kind is invalid",
  );
  requireCondition(
    results.datasetId === dataset.datasetId && results.datasetVersion === dataset.version,
    "result dataset identity/version does not match the golden dataset",
  );
  requireCondition(typeof results.providerId === "string" && results.providerId.length > 0, "results providerId is required");
  requireCondition(typeof results.providerVersion === "string" && results.providerVersion.length > 0, "results providerVersion is required");
  requireCondition(Array.isArray(results.runs), "results runs must be an array");

  const queriesById = new Map(dataset.queries.map((query) => [query.id, query]));
  const runIds = new Set();
  for (const run of results.runs) {
    requireCondition(typeof run?.queryId === "string" && run.queryId.length > 0, "result queryId is required");
    requireCondition(queriesById.has(run.queryId), `unknown result query ${run.queryId}`);
    requireCondition(!runIds.has(run.queryId), `duplicate result run for query ${run.queryId}`);
    runIds.add(run.queryId);
    const query = queriesById.get(run.queryId);
    requireCondition(Array.isArray(run.hits), `result run ${run.queryId} hits must be an array`);
    if (query.expectRejection === true) {
      requireCondition(typeof run.rejected === "boolean", `result run ${run.queryId} rejected must be boolean`);
      continue;
    }
    requireCondition(typeof run.failed === "boolean", `result run ${run.queryId} failed must be boolean`);
    requireCondition(
      Number.isFinite(run.latencyMs) && run.latencyMs >= 0,
      `result run ${run.queryId} latencyMs must be non-negative`,
    );
    const hitDocumentIds = new Set();
    for (const hit of run.hits) {
      requireCondition(
        typeof hit?.documentId === "string" && hit.documentId.length > 0,
        `result run ${run.queryId} contains an invalid documentId`,
      );
      requireCondition(
        !hitDocumentIds.has(hit.documentId),
        `result run ${run.queryId} contains duplicate documentId ${hit.documentId}`,
      );
      hitDocumentIds.add(hit.documentId);
      requireCondition(
        hit.citationDocumentId === undefined
          || (typeof hit.citationDocumentId === "string" && hit.citationDocumentId.length > 0),
        `result run ${run.queryId} contains an invalid citationDocumentId`,
      );
    }
  }
  for (const query of dataset.queries) {
    requireCondition(runIds.has(query.id), `missing result run for query ${query.id}`);
  }
}

export function evaluateRetrieval(dataset, results, productionPolicy = undefined) {
  validateRetrievalDataset(dataset, productionPolicy);
  validateRetrievalResults(dataset, results);

  const runsByQuery = new Map(
    (results.runs ?? []).map((run) => [run.queryId, run]),
  );
  const recall = [];
  const reciprocalRanks = [];
  const normalizedDcg = [];
  const latencies = [];
  let failedRuns = 0;
  let citationCount = 0;
  let correctCitationCount = 0;
  let emptyQueryCases = 0;
  let emptyQueryPasses = 0;

  for (const query of dataset.queries) {
    const run = runsByQuery.get(query.id);
    if (query.expectRejection) {
      emptyQueryCases += 1;
      if (run.rejected === true) emptyQueryPasses += 1;
      continue;
    }

    if (run.failed === true) failedRuns += 1;
    if (Number.isFinite(run.latencyMs)) latencies.push(run.latencyMs);

    const topK = query.topK ?? dataset.defaultTopK ?? 5;
    const relevant = new Set(query.relevantDocumentIds ?? []);
    const hits = (run.hits ?? []).slice(0, topK);
    const relevance = hits.map((hit) => Number(relevant.has(hit.documentId)));
    const matched = new Set(
      hits.filter((hit) => relevant.has(hit.documentId)).map((hit) => hit.documentId),
    );
    recall.push(matched.size / relevant.size);

    const firstRelevant = relevance.findIndex((value) => value === 1);
    reciprocalRanks.push(firstRelevant < 0 ? 0 : 1 / (firstRelevant + 1));
    const ideal = Array.from(
      { length: Math.min(relevant.size, topK) },
      () => 1,
    );
    const idealDcg = dcg(ideal);
    normalizedDcg.push(idealDcg === 0 ? 0 : dcg(relevance) / idealDcg);

    const expectedCitations = new Set(
      query.expectedCitationDocumentIds ?? query.relevantDocumentIds,
    );
    for (const hit of hits) {
      if (!hit.citationDocumentId) continue;
      citationCount += 1;
      if (expectedCitations.has(hit.citationDocumentId)) {
        correctCitationCount += 1;
      }
    }
  }

  const scoredQueryCount = recall.length;
  const metrics = {
    recallAtK: round(mean(recall)),
    mrr: round(mean(reciprocalRanks)),
    ndcgAtK: round(mean(normalizedDcg)),
    citationCorrectness: round(
      citationCount === 0 ? 0 : correctCitationCount / citationCount,
    ),
    failureRate: round(scoredQueryCount === 0 ? 0 : failedRuns / scoredQueryCount),
    p95LatencyMs: percentile95(latencies),
    emptyQueryPassRate: round(
      emptyQueryCases === 0 ? 1 : emptyQueryPasses / emptyQueryCases,
    ),
    scoredQueryCount,
  };

  const thresholds = dataset.thresholds ?? {};
  const failures = [];
  for (const key of [
    "recallAtK",
    "mrr",
    "ndcgAtK",
    "citationCorrectness",
    "emptyQueryPassRate",
  ]) {
    const threshold = thresholds[`min${key[0].toUpperCase()}${key.slice(1)}`];
    if (threshold !== undefined && metrics[key] < threshold) {
      failures.push(`${key}=${metrics[key]} is below ${threshold}`);
    }
  }
  if (
    thresholds.maxFailureRate !== undefined
    && metrics.failureRate > thresholds.maxFailureRate
  ) {
    failures.push(
      `failureRate=${metrics.failureRate} exceeds ${thresholds.maxFailureRate}`,
    );
  }
  if (
    thresholds.maxP95LatencyMs !== undefined
    && metrics.p95LatencyMs > thresholds.maxP95LatencyMs
  ) {
    failures.push(
      `p95LatencyMs=${metrics.p95LatencyMs} exceeds ${thresholds.maxP95LatencyMs}`,
    );
  }

  return {
    kind: "sdkwork.knowledge-engine-retrieval-evaluation",
    schemaVersion: 1,
    evidenceClass: dataset.classification,
    datasetId: dataset.datasetId,
    datasetVersion: dataset.version,
    providerId: results.providerId,
    providerVersion: results.providerVersion,
    metrics,
    thresholds,
    passed: failures.length === 0,
    failures,
  };
}

if (process.argv[1] && import.meta.url === new URL(`file://${path.resolve(process.argv[1])}`).href) {
  const datasetPath = argument("--dataset");
  const resultsPath = argument("--results");
  if (!datasetPath || !resultsPath) {
    fail("usage: evaluate_knowledge_engine_retrieval.mjs --dataset <golden.json> --results <results.json>");
  }
  try {
    const dataset = JSON.parse(await readFile(path.resolve(datasetPath), "utf8"));
    const results = JSON.parse(await readFile(path.resolve(resultsPath), "utf8"));
    const productionPolicy = dataset.classification === "production-domain"
      ? JSON.parse(await readFile(
        new URL("../specs/knowledge-engine-evaluation.spec.json", import.meta.url),
        "utf8",
      )).productionPolicy
      : undefined;
    const report = evaluateRetrieval(dataset, results, productionPolicy);
    console.log(JSON.stringify(report, null, 2));
    if (!report.passed) process.exit(1);
  } catch (error) {
    fail(`knowledge engine retrieval evaluation failed: ${error.message}`);
  }
}
