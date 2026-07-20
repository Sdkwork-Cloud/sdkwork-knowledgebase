import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import { evaluateRetrieval } from "./evaluate_knowledge_engine_retrieval.mjs";

const dataset = JSON.parse(
  await readFile(
    new URL("../tests/fixtures/knowledge-engine-evaluation/v1/golden.json", import.meta.url),
    "utf8",
  ),
);
const sampleResults = JSON.parse(
  await readFile(
    new URL(
      "../tests/fixtures/knowledge-engine-evaluation/v1/sample-results.json",
      import.meta.url,
    ),
    "utf8",
  ),
);

test("retrieval evaluation passes results meeting every threshold", () => {
  const report = evaluateRetrieval(dataset, sampleResults);

  assert.equal(report.passed, true);
  assert.equal(report.metrics.recallAtK, 1);
  assert.equal(report.metrics.citationCorrectness, 1);
  assert.equal(report.metrics.emptyQueryPassRate, 1);
});

test("retrieval evaluation fails poor quality, latency, and empty-query behavior", () => {
  const results = structuredClone(sampleResults);
  results.runs[0].hits = [];
  results.runs[0].failed = true;
  results.runs[0].latencyMs = 3000;
  results.runs[2].rejected = false;

  const report = evaluateRetrieval(dataset, results);

  assert.equal(report.passed, false);
  assert.ok(report.failures.some((failure) => failure.startsWith("recallAtK=")));
  assert.ok(report.failures.some((failure) => failure.startsWith("failureRate=")));
  assert.ok(report.failures.some((failure) => failure.startsWith("p95LatencyMs=")));
  assert.ok(report.failures.some((failure) => failure.startsWith("emptyQueryPassRate=")));
});

test("retrieval evaluation rejects result files for another dataset version", () => {
  const results = structuredClone(sampleResults);
  results.datasetVersion = "2.0.0";

  assert.throws(
    () => evaluateRetrieval(dataset, results),
    /identity\/version does not match/,
  );
});

test("retrieval evaluation rejects duplicate and unknown result runs", () => {
  const duplicate = structuredClone(sampleResults);
  duplicate.runs.push(structuredClone(duplicate.runs[0]));
  assert.throws(() => evaluateRetrieval(dataset, duplicate), /duplicate result run/);

  const unknown = structuredClone(sampleResults);
  unknown.runs.push({ queryId: "unknown", failed: false, latencyMs: 1, hits: [] });
  assert.throws(() => evaluateRetrieval(dataset, unknown), /unknown result query/);
});

test("retrieval evaluation rejects invalid schemas and negative latency", () => {
  const wrongKind = structuredClone(sampleResults);
  wrongKind.kind = "sdkwork.knowledge-engine-retrieval-results-template";
  assert.throws(() => evaluateRetrieval(dataset, wrongKind), /results kind is invalid/);

  const negativeLatency = structuredClone(sampleResults);
  negativeLatency.runs[0].latencyMs = -1;
  assert.throws(() => evaluateRetrieval(dataset, negativeLatency), /latencyMs must be non-negative/);
});
