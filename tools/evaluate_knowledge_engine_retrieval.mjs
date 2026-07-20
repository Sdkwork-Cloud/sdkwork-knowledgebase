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

export function evaluateRetrieval(dataset, results) {
  if (!dataset.datasetId || !dataset.version || !Array.isArray(dataset.queries)) {
    throw new Error("dataset must declare datasetId, version, and queries");
  }
  if (
    results.datasetId !== dataset.datasetId
    || results.datasetVersion !== dataset.version
  ) {
    throw new Error("result dataset identity/version does not match the golden dataset");
  }

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
    if (!run) {
      throw new Error(`missing result run for query ${query.id}`);
    }
    if (query.expectRejection) {
      emptyQueryCases += 1;
      if (run.rejected === true) emptyQueryPasses += 1;
      continue;
    }

    if (run.failed === true) failedRuns += 1;
    if (Number.isFinite(run.latencyMs)) latencies.push(run.latencyMs);

    const topK = query.topK ?? dataset.defaultTopK ?? 5;
    const relevant = new Set(query.relevantDocumentIds ?? []);
    if (relevant.size === 0) {
      throw new Error(`query ${query.id} must declare relevantDocumentIds`);
    }
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
    datasetId: dataset.datasetId,
    datasetVersion: dataset.version,
    providerId: results.providerId,
    providerVersion: results.providerVersion ?? null,
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
    const report = evaluateRetrieval(dataset, results);
    console.log(JSON.stringify(report, null, 2));
    if (!report.passed) process.exit(1);
  } catch (error) {
    fail(`knowledge engine retrieval evaluation failed: ${error.message}`);
  }
}

