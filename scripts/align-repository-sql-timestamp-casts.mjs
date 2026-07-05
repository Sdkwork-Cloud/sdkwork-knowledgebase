#!/usr/bin/env node
/**
 * Add CAST($N AS TIMESTAMP) for updated_at/created_at assignments in repository-sqlx
 * when not already cast. Safe for SQLite and PostgreSQL Any pools.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const srcDir = path.join(
  root,
  "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src",
);

function alignTimestampCasts(content) {
  let next = content;
  next = next.replace(
    /(\bupdated_at\s*=\s*)(\$\d+)(?!\s+AS\s+TIMESTAMP)/g,
    "$1CAST($2 AS TIMESTAMP)",
  );
  next = next.replace(
    /(\bcreated_at\s*=\s*)(\$\d+)(?!\s+AS\s+TIMESTAMP)/g,
    "$1CAST($2 AS TIMESTAMP)",
  );
  return next;
}

function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(full);
      continue;
    }
    if (!entry.name.endsWith(".rs")) {
      continue;
    }
    const before = fs.readFileSync(full, "utf8");
    const after = alignTimestampCasts(before);
    if (after !== before) {
      fs.writeFileSync(full, after);
      console.log(`updated ${path.relative(root, full)}`);
    }
  }
}

walk(srcDir);
