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

const violations = [];
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
