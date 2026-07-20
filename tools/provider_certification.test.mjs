import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  buildCertificationExecutionPlan,
  loadProviderCertification,
  validateProviderCertification,
} from "./provider_certification.mjs";
import { validateLiveCertificationEvidenceRecord } from "./provider_live_certification.mjs";

const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestPath = path.join(
  workspaceRoot,
  "external/knowledge-engines/provider-certification.manifest.json",
);

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
