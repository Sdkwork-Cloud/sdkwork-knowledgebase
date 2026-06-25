#!/usr/bin/env node
/**
 * Plan or apply git submodule pins for external knowledge engine upstream repos.
 *
 * Usage:
 *   node tools/sync_external_knowledge_engine_submodules.mjs --check
 *   node tools/sync_external_knowledge_engine_submodules.mjs --apply
 */
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const catalogPath = path.join(root, "external/knowledge-engines/catalog.manifest.json");
const checkOnly = process.argv.includes("--check");
const apply = process.argv.includes("--apply");

if (!checkOnly && !apply) {
  console.error("Pass --check or --apply");
  process.exit(1);
}

const catalog = JSON.parse(await readFile(catalogPath, "utf8"));
const planned = [];

for (const entry of catalog.vendors) {
  const vendor = JSON.parse(
    await readFile(path.join(root, entry.manifestPath), "utf8"),
  );
  planned.push({
    vendorId: vendor.vendorId,
    path: vendor.upstream.submodulePath.replaceAll("\\", "/"),
    url: `${vendor.upstream.repositoryUrl}.git`,
    branch: vendor.upstream.defaultBranch,
  });
}

let gitmodules = "";
try {
  gitmodules = await readFile(path.join(root, ".gitmodules"), "utf8");
} catch {
  gitmodules = "";
}

const missing = planned.filter((item) => !gitmodules.includes(`path = ${item.path}`));

if (checkOnly) {
  if (missing.length === 0) {
    console.log(`All ${planned.length} upstream submodule entries are registered in .gitmodules.`);
    process.exit(0);
  }
  console.log("Missing .gitmodules entries (catalog-only mode is OK until adapter work starts):");
  for (const item of missing) {
    console.log(`  - ${item.vendorId}: git submodule add -b ${item.branch} ${item.url} ${item.path}`);
  }
  process.exit(0);
}

for (const item of missing) {
  const command = `git submodule add -b ${item.branch} ${item.url} ${item.path}`;
  console.log(`Running: ${command}`);
  execSync(command, { cwd: root, stdio: "inherit" });
}

console.log(`Registered ${missing.length} submodule(s).`);
