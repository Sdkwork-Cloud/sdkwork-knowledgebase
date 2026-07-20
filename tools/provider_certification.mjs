#!/usr/bin/env node
import { access, readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import path from "node:path";

import {
  validateLiveCertificationEvidenceRecord,
  validateLiveEvidenceSchema,
} from "./provider_live_certification.mjs";

export const REQUIRED_CONTRACT_DIMENSIONS = Object.freeze([
  "capability",
  "authentication",
  "error_mapping",
  "resilience",
  "isolation",
  "health",
]);

const vendorIdPattern = /^[a-z0-9][a-z0-9_-]*$/u;
const semverPattern = /^\d+\.\d+\.\d+$/u;
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
      violations.push(`${location}.${key}: secret-bearing fields are forbidden in certification evidence`);
    }
    collectForbiddenKeys(nested, `${location}.${key}`, violations);
  }
}

function validateCommand(command, vendorId, violations) {
  const location = `providers.${vendorId}.contractCertification.command`;
  addViolation(violations, command?.program === "cargo", `${location}.program must be cargo`);
  addViolation(violations, Array.isArray(command?.args), `${location}.args must be an array`);
  const args = command?.args ?? [];
  addViolation(
    violations,
    args.length === 4
      && args[0] === "test"
      && args[1] === "-p"
      && args[2] === `sdkwork-knowledgebase-engine-${vendorId}`
      && args[3] === "--all-targets",
    `${location} must execute the complete owned adapter crate with --all-targets`,
  );
  for (const argument of args) {
    addViolation(
      violations,
      typeof argument === "string" && argument.length > 0 && !/[;&|`$<>\r\n]/u.test(argument),
      `${location} contains an unsafe argument`,
    );
  }
}

export async function validateProviderCertification(manifest, workspaceRoot) {
  const violations = [];
  addViolation(violations, manifest?.schemaVersion === 2, "schemaVersion must be 2");
  addViolation(
    violations,
    manifest?.kind === "sdkwork.knowledge-engine-provider-certification",
    "kind must be sdkwork.knowledge-engine-provider-certification",
  );
  addViolation(violations, manifest?.owner === "sdkwork-knowledgebase", "owner must be sdkwork-knowledgebase");
  addViolation(
    violations,
    semverPattern.test(manifest?.policy?.contractSuiteVersion ?? ""),
    "policy.contractSuiteVersion must be semantic version",
  );
  addViolation(
    violations,
    JSON.stringify(manifest?.policy?.requiredContractDimensions) === JSON.stringify(REQUIRED_CONTRACT_DIMENSIONS),
    "policy.requiredContractDimensions must match the canonical ordered dimensions",
  );
  addViolation(
    violations,
    Number.isInteger(manifest?.policy?.liveEvidenceMaxAgeDays)
      && manifest.policy.liveEvidenceMaxAgeDays >= 1
      && manifest.policy.liveEvidenceMaxAgeDays <= 365,
    "policy.liveEvidenceMaxAgeDays must be between 1 and 365",
  );
  violations.push(...await validateLiveEvidenceSchema(manifest?.policy, workspaceRoot));
  collectForbiddenKeys(manifest, "manifest", violations);

  const seen = new Set();
  for (const provider of manifest?.providers ?? []) {
    const location = `providers.${provider?.vendorId ?? "unknown"}`;
    addViolation(violations, vendorIdPattern.test(provider?.vendorId ?? ""), `${location}.vendorId is invalid`);
    addViolation(violations, !seen.has(provider.vendorId), `${location}.vendorId is duplicated`);
    seen.add(provider.vendorId);

    if (provider.contractGate === "not-applicable") {
      addViolation(
        violations,
        provider.contractCertification?.status === "not-applicable"
          && provider.liveCertification?.status === "not-applicable",
        `${location}: non-executable providers must be not-applicable`,
      );
      continue;
    }

    addViolation(violations, provider.contractGate === "required", `${location}.contractGate must be required`);
    const contract = provider.contractCertification;
    addViolation(violations, contract?.status === "passed", `${location}.contractCertification.status must be passed`);
    addViolation(
      violations,
      contract?.suiteVersion === manifest.policy.contractSuiteVersion,
      `${location}.contractCertification.suiteVersion must match policy`,
    );
    addViolation(
      violations,
      /^\d{4}-\d{2}-\d{2}$/u.test(contract?.verifiedAt ?? ""),
      `${location}.contractCertification.verifiedAt must be an ISO date`,
    );
    addViolation(
      violations,
      sha256Pattern.test(contract?.sourceFingerprint ?? ""),
      `${location}.contractCertification.sourceFingerprint must be a SHA-256 digest`,
    );
    validateCommand(contract?.command, provider.vendorId, violations);

    const evidence = contract?.evidence ?? {};
    const evidenceReferences = new Set();
    for (const dimension of REQUIRED_CONTRACT_DIMENSIONS) {
      const refs = evidence[dimension];
      addViolation(
        violations,
        Array.isArray(refs) && refs.length > 0,
        `${location}.contractCertification.evidence.${dimension} must contain evidence references`,
      );
      for (const reference of refs ?? []) {
        const normalized = typeof reference === "string" ? reference.replaceAll("\\", "/") : "";
        addViolation(
          violations,
          normalized.startsWith("crates/") && !normalized.includes(".."),
          `${location}.contractCertification.evidence.${dimension} contains an invalid path`,
        );
        if (normalized.startsWith("crates/") && !normalized.includes("..")) {
          evidenceReferences.add(normalized);
          try {
            await access(path.join(workspaceRoot, normalized));
          } catch {
            violations.push(`${location}.contractCertification.evidence.${dimension}: missing ${normalized}`);
          }
        }
      }
    }
    addViolation(
      violations,
      Object.keys(evidence).every((key) => REQUIRED_CONTRACT_DIMENSIONS.includes(key)),
      `${location}.contractCertification.evidence contains an unknown dimension`,
    );
    if (evidenceReferences.size > 0) {
      const fingerprint = await computeEvidenceFingerprint(workspaceRoot, [...evidenceReferences]);
      addViolation(
        violations,
        contract?.sourceFingerprint === fingerprint,
        `${location}.contractCertification.sourceFingerprint does not match its evidence sources`,
      );
    }
    addViolation(
      violations,
      ["pending", "certified"].includes(provider.liveCertification?.status),
      `${location}.liveCertification.status must be pending or certified`,
    );
    if (provider.liveCertification?.status === "certified") {
      const live = provider.liveCertification;
      for (const field of manifest.policy.productionEvidence ?? []) {
        addViolation(violations, Boolean(live[field]), `${location}.liveCertification.${field} is required`);
      }
      addViolation(
        violations,
        typeof live.upstreamVersion === "string"
          && live.upstreamVersion.length > 0
          && !/latest|[*xX]/u.test(live.upstreamVersion),
        `${location}.liveCertification.upstreamVersion must be pinned`,
      );
      addViolation(
        violations,
        sha256Pattern.test(live.evidenceSha256 ?? ""),
        `${location}.liveCertification.evidenceSha256 must be a SHA-256 digest`,
      );
      const evidenceRef = typeof live.evidenceRef === "string" ? live.evidenceRef.replaceAll("\\", "/") : "";
      addViolation(
        violations,
        evidenceRef.startsWith("docs/releases/provider-certification/") && !evidenceRef.includes(".."),
        `${location}.liveCertification.evidenceRef must be a repository certification record`,
      );
      if (evidenceRef.startsWith("docs/releases/provider-certification/") && !evidenceRef.includes("..")) {
        try {
          const evidenceBytes = await readFile(path.join(workspaceRoot, evidenceRef));
          const evidenceDigest = createHash("sha256").update(evidenceBytes).digest("hex");
          addViolation(
            violations,
            live.evidenceSha256 === evidenceDigest,
            `${location}.liveCertification.evidenceSha256 does not match the evidence record`,
          );
          let evidenceRecord;
          try {
            evidenceRecord = JSON.parse(evidenceBytes.toString("utf8"));
          } catch {
            violations.push(`${location}.liveCertification.evidenceRef must contain valid JSON`);
          }
          if (evidenceRecord) {
            violations.push(...await validateLiveCertificationEvidenceRecord(
              evidenceRecord,
              {
                providerId: provider.vendorId,
                upstreamVersion: live.upstreamVersion,
                contractSuiteVersion: manifest.policy.contractSuiteVersion,
                verifiedAt: live.verifiedAt,
                licensingApproval: live.licensingApproval,
                securityPrivacyApproval: live.securityPrivacyApproval,
              },
              manifest.policy,
              workspaceRoot,
              `${location}.liveCertification.evidence`,
            ));
          }
        } catch {
          violations.push(`${location}.liveCertification.evidenceRef is missing: ${evidenceRef}`);
        }
      }
      addViolation(
        violations,
        live.licensingApproval === "approved",
        `${location}.liveCertification.licensingApproval must be approved`,
      );
      addViolation(
        violations,
        live.securityPrivacyApproval === "approved",
        `${location}.liveCertification.securityPrivacyApproval must be approved`,
      );
      const verifiedAt = Date.parse(`${live.verifiedAt}T00:00:00Z`);
      const ageDays = (Date.now() - verifiedAt) / 86_400_000;
      addViolation(
        violations,
        Number.isFinite(verifiedAt)
          && ageDays >= 0
          && ageDays <= manifest.policy.liveEvidenceMaxAgeDays,
        `${location}.liveCertification.verifiedAt is missing, future-dated, or stale`,
      );
    }
  }
  return violations;
}

export async function computeEvidenceFingerprint(workspaceRoot, references) {
  const hash = createHash("sha256");
  for (const reference of [...new Set(references)].sort()) {
    hash.update(`${reference}\0`, "utf8");
    hash.update(await readFile(path.join(workspaceRoot, reference)));
    hash.update("\0", "utf8");
  }
  return hash.digest("hex");
}

export function buildCertificationExecutionPlan(manifest, selectedVendorIds = []) {
  const selected = new Set(selectedVendorIds);
  const known = new Set((manifest.providers ?? []).map((provider) => provider.vendorId));
  const unknown = [...selected].filter((vendorId) => !known.has(vendorId));
  if (unknown.length > 0) {
    throw new Error(`unknown Provider certification target(s): ${unknown.join(", ")}`);
  }
  return (manifest.providers ?? [])
    .filter((provider) => provider.contractCertification?.status === "passed")
    .filter((provider) => selected.size === 0 || selected.has(provider.vendorId))
    .map((provider) => ({
      vendorId: provider.vendorId,
      program: provider.contractCertification.command.program,
      args: [...provider.contractCertification.command.args],
    }));
}

export async function loadProviderCertification(manifestPath) {
  return JSON.parse(await readFile(manifestPath, "utf8"));
}
