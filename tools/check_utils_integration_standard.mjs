#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const failures = [];

function readText(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function assert(condition, message) {
  if (!condition) {
    failures.push(message);
  }
}

function walkFiles(relativeDir, predicate) {
  const absoluteDir = path.join(repoRoot, relativeDir);
  if (!fs.existsSync(absoluteDir)) {
    return [];
  }

  const results = [];
  const stack = [absoluteDir];
  while (stack.length > 0) {
    const current = stack.pop();
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      const absolutePath = path.join(current, entry.name);
      if (entry.isDirectory()) {
        if (entry.name === 'node_modules' || entry.name === 'dist' || entry.name === 'target') {
          continue;
        }
        stack.push(absolutePath);
        continue;
      }
      if (predicate(absolutePath)) {
        results.push(path.relative(repoRoot, absolutePath).replace(/\\/g, '/'));
      }
    }
  }
  return results;
}

const utilsRustCrates = [
  'crates/sdkwork-intelligence-knowledgebase-service/Cargo.toml',
  'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/Cargo.toml',
  'crates/sdkwork-knowledgebase-agent-provider/Cargo.toml',
  'crates/sdkwork-knowledgebase-drive/Cargo.toml',
  'crates/sdkwork-knowledgebase-memory/Cargo.toml',
  'crates/sdkwork-knowledgebase-test-support/Cargo.toml',
  'crates/sdkwork-router-knowledgebase-app-api/Cargo.toml',
];

for (const crateToml of utilsRustCrates) {
  const text = readText(crateToml);
  assert(
    text.includes('sdkwork-utils-rust'),
    `${crateToml} must depend on sdkwork-utils-rust`,
  );
}

const rustSources = walkFiles('crates', (filePath) => filePath.endsWith('.rs'));
for (const relativePath of rustSources) {
  const text = readText(relativePath);
  if (text.includes('.trim().is_empty()')) {
    failures.push(`${relativePath} must use sdkwork_utils_rust::is_blank instead of .trim().is_empty()`);
  }
  if (/fn\s+(checksum_sha256_hex|sha256_hex|hash_hex)\s*\(/u.test(text)) {
    failures.push(`${relativePath} must use sdkwork_utils_rust::sha256_hash instead of local SHA-256 helpers`);
  }
}

const pcWorkspace = readText('apps/sdkwork-knowledgebase-pc/pnpm-workspace.yaml');
assert(
  pcWorkspace.includes('sdkwork-utils/packages/sdkwork-utils-typescript'),
  'apps/sdkwork-knowledgebase-pc/pnpm-workspace.yaml must include @sdkwork/utils workspace package',
);

const pcPackageJson = JSON.parse(readText('apps/sdkwork-knowledgebase-pc/package.json'));
assert(
  pcPackageJson.dependencies?.['@sdkwork/utils'],
  'apps/sdkwork-knowledgebase-pc/package.json must declare @sdkwork/utils',
);

const pcCommonsPackageJson = JSON.parse(
  readText('apps/sdkwork-knowledgebase-pc/packages/sdkwork-knowledgebase-pc-commons/package.json'),
);
assert(
  pcCommonsPackageJson.dependencies?.['@sdkwork/utils'],
  'sdkwork-knowledgebase-pc-commons must declare @sdkwork/utils',
);

const tsSources = walkFiles('apps/sdkwork-knowledgebase-pc/packages', (filePath) =>
  /\.(ts|tsx)$/u.test(filePath),
);
const legacyBlankPatterns = [
  /\.trim\(\)\.isEmpty\(\)/u,
  /\.trim\(\)\.length\s*===\s*0/u,
  /\.trim\(\)\.length\s*>\s*0/u,
  /![\w.?()[\]'"-]+\.trim\(\)/u,
];

for (const relativePath of tsSources) {
  const text = readText(relativePath);
  if (
    text.includes('@sdkwork/utils')
    || text.includes('sdkwork-knowledgebase-pc-commons/stringUtils')
  ) {
    continue;
  }
  for (const pattern of legacyBlankPatterns) {
    if (pattern.test(text)) {
      failures.push(
        `${relativePath} must use @sdkwork/utils isBlank/trim instead of local .trim() blank checks`,
      );
      break;
    }
  }
}

if (failures.length > 0) {
  process.stderr.write(`Utils integration standard failed:\n${failures.map((f) => `- ${f}`).join('\n')}\n`);
  process.exit(1);
}

process.stdout.write('Utils integration standard check passed.\n');
