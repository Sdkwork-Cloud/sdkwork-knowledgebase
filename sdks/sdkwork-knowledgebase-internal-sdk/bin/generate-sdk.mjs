#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const familyRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const generator = path.resolve(familyRoot, "tools", "knowledgebase_sdk_generate.mjs");
const result = spawnSync(process.execPath, [generator, "--family", "sdkwork-knowledgebase-internal-sdk", ...process.argv.slice(2)], {
  cwd: familyRoot,
  stdio: "inherit",
});

if (result.error) {
  throw result.error;
}
process.exit(result.status ?? 1);
