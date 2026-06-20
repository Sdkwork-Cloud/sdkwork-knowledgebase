#!/usr/bin/env node
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const forbiddenPatterns = [
  /\bllm_wiki\b/,
  /\bLlmWiki\b/,
  /\bWikiPageType\b/,
  /\bkb_wiki_/,
  /\bwiki\.pages\b/,
  /\bwiki_schema\.yaml\b/,
  /\bKnowledgeWiki\b/,
  /\bprovider\.knowledge\.llm-wiki\b/,
  /\bupsert_page\b/,
  /\bget_page_by_id\b/,
  /\blist_okf_pages\b/,
];

const skipPathParts = [
  "target",
  "node_modules",
  "dist",
  "external",
  ".git",
  "2026-06-01-knowledgebase-backend-design.md",
  "2026-06-01-knowledgebase-backend-phase1-implementation.md",
  "okf-knowledge-bundle.spec.json",
  "2026-06-19-okf-knowledge-bundle-design.md",
  "migrate_openapi_wiki_to_okf.mjs",
];

const allowedExtensions = new Set([
  ".rs",
  ".ts",
  ".tsx",
  ".json",
  ".md",
  ".yaml",
  ".yml",
  ".sql",
  ".mjs",
  ".ps1",
]);

async function walk(dir, out = []) {
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    const rel = path.relative(root, full).replaceAll("\\", "/");
    if (skipPathParts.some((part) => rel.includes(part))) {
      continue;
    }
    if (entry.isDirectory()) {
      await walk(full, out);
      continue;
    }
    if (!allowedExtensions.has(path.extname(entry.name))) {
      continue;
    }
    out.push(full);
  }
  return out;
}

const requiredOkfServiceModules = [
  "concept_service.rs",
  "document.rs",
  "validator.rs",
  "index_renderer.rs",
  "log_renderer.rs",
  "bundle_linter.rs",
  "importer.rs",
  "exporter.rs",
  "initializer.rs",
  "storage.rs",
  "linter.rs",
];

const requiredOkfStorageSymbols = [
  "read_managed_object_bytes",
  "get_object_bytes",
  "export_manifest.yaml",
  "validate_concept_bundle_relative_path",
  "extract_index_linked_concept_ids",
  "kb_okf_candidate",
  "SqliteKnowledgeOkfCandidateStore",
  "stage_concept_candidate",
];

const requiredOkfMigrationTables = ["kb_okf_concept", "kb_okf_concept_link", "kb_okf_candidate"];

async function assertRequiredOkfModules() {
  const okfDir = path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/okf",
  );
  const entries = await readdir(okfDir);
  for (const moduleName of requiredOkfServiceModules) {
    if (!entries.includes(moduleName)) {
      violations.push(`missing okf service module: okf/${moduleName}`);
    }
  }
}

async function assertRequiredOkfStorageSymbols() {
  const files = [
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/storage.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_drive_storage.rs",
    ),
    path.join(root, "crates/sdkwork-knowledgebase-drive/src/adapter.rs"),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/exporter.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/validator.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/linter.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/okf_candidate_store.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/concept_service.rs",
    ),
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/okf/importer.rs",
    ),
  ];
  const combined = (
    await Promise.all(files.map((file) => readFile(file, "utf8")))
  ).join("\n");
  for (const symbol of requiredOkfStorageSymbols) {
    if (!combined.includes(symbol)) {
      violations.push(`missing okf storage/export symbol: ${symbol}`);
    }
  }
}

async function assertRequiredOkfMigrations() {
  const migrationsDir = path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/migrations",
  );
  const migrationRoots = await readdir(migrationsDir, { withFileTypes: true });
  let migrationSql = "";
  for (const entry of migrationRoots) {
    if (!entry.isDirectory()) {
      continue;
    }
    const files = await readdir(path.join(migrationsDir, entry.name));
    for (const file of files) {
      if (file.endsWith(".sql")) {
        migrationSql += await readFile(
          path.join(migrationsDir, entry.name, file),
          "utf8",
        );
      }
    }
  }
  for (const tableName of requiredOkfMigrationTables) {
    if (!migrationSql.includes(tableName)) {
      violations.push(`missing okf migration table: ${tableName}`);
    }
  }
}

const violations = [];
await assertRequiredOkfModules();
await assertRequiredOkfStorageSymbols();
await assertRequiredOkfMigrations();

for (const file of await walk(root)) {
  const content = await readFile(file, "utf8");
  for (const pattern of forbiddenPatterns) {
    if (pattern.test(content)) {
      violations.push(`${path.relative(root, file)}: ${pattern}`);
    }
  }
}

if (violations.length > 0) {
  console.error("OKF legacy violations found:\n" + violations.join("\n"));
  process.exit(1);
}

console.log("OKF knowledge bundle standard check passed.");
