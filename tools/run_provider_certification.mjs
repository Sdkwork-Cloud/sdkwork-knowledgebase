#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  buildCertificationExecutionPlan,
  loadProviderCertification,
  validateProviderCertification,
} from "./provider_certification.mjs";

const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const manifestPath = path.join(
  workspaceRoot,
  "external/knowledge-engines/provider-certification.manifest.json",
);
const execute = process.argv.includes("--execute");
const selectedVendorIds = [];
for (let index = 0; index < process.argv.length; index += 1) {
  if (process.argv[index] === "--provider" && process.argv[index + 1]) {
    selectedVendorIds.push(process.argv[index + 1]);
    index += 1;
  }
}

const manifest = await loadProviderCertification(manifestPath);
const violations = await validateProviderCertification(manifest, workspaceRoot);
if (violations.length > 0) {
  console.error(`Provider certification violations:\n${violations.join("\n")}`);
  process.exit(1);
}

const plan = buildCertificationExecutionPlan(manifest, selectedVendorIds);
if (execute) {
  for (const command of plan) {
    console.log(`Running Provider contract certification: ${command.vendorId}`);
    const result = spawnSync(command.program, command.args, {
      cwd: workspaceRoot,
      stdio: "inherit",
      shell: false,
      env: process.env,
    });
    if (result.error) {
      console.error(`Provider contract certification could not start for ${command.vendorId}: ${result.error.message}`);
      process.exit(1);
    }
    if (result.status !== 0) {
      console.error(JSON.stringify({
        ok: false,
        vendorId: command.vendorId,
        exitCode: result.status,
        signal: result.signal,
      }, null, 2));
      process.exit(result.status ?? 1);
    }
  }
}

console.log(JSON.stringify({
  ok: true,
  mode: execute ? "execute" : "check",
  contractSuiteVersion: manifest.policy.contractSuiteVersion,
  providerCount: plan.length,
  providers: plan.map((command) => command.vendorId),
  liveCertifiedCount: (manifest.providers ?? []).filter(
    (provider) => provider.liveCertification?.status === "certified",
  ).length,
}, null, 2));
