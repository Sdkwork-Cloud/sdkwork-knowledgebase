import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const toolDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(toolDir, '..');
const inventoryPath = path.join(
  repoRoot,
  'crates/sdkwork-intelligence-knowledgebase-repository-sqlx/specs/organization-scope-inventory.json',
);
const inventory = JSON.parse(readFileSync(inventoryPath, 'utf8'));
const sourceRoot = path.join(repoRoot, inventory.sourceRoot);
const migration = readFileSync(path.join(repoRoot, inventory.postgresMigration), 'utf8');
const tables = new Set(inventory.organizationOwnedTables);

assert.equal(inventory.schemaVersion, 1);
assert.equal(new Set(inventory.organizationOwnedTables).size, inventory.organizationOwnedTables.length);
for (const sessionKey of inventory.sessionKeys) {
  assert.match(migration, new RegExp(sessionKey.replaceAll('.', '\\.')));
}
for (const table of tables) {
  assert.match(migration, new RegExp(`'${table}'`), `${table} is missing from organization RLS`);
}

function listRustFiles(root) {
  return readdirSync(root).flatMap((entry) => {
    const absolute = path.join(root, entry);
    if (statSync(absolute).isDirectory()) {
      return listRustFiles(absolute);
    }
    return absolute.endsWith('.rs') ? [absolute] : [];
  });
}

function sqlLiterals(source) {
  const literals = [];
  for (const pattern of [/r(#+)"([\s\S]*?)"\1/g, /"((?:\\.|[^"\\])*)"/g]) {
    for (const match of source.matchAll(pattern)) {
      literals.push({
        text: match[2] ?? match[1],
        offset: match.index ?? 0,
      });
    }
  }
  return literals;
}

const violations = new Set();
for (const file of listRustFiles(sourceRoot)) {
  const source = readFileSync(file, 'utf8');
  for (const literal of sqlLiterals(source)) {
    const sql = literal.text.toLowerCase();
    if (!/\b(select|insert\s+into|update|delete\s+from)\b/.test(sql)) {
      continue;
    }
    if (!sql.includes('tenant_id') || sql.includes('organization_id')) {
      continue;
    }
    const touchedTables = [...tables].filter((table) =>
      new RegExp(`\\b${table}\\b`).test(sql),
    );
    if (touchedTables.length === 0) {
      continue;
    }
    const line = source.slice(0, literal.offset).split('\n').length;
    violations.add(
      `${path.relative(repoRoot, file).replaceAll('\\', '/')}:${line} `
      + `uses ${touchedTables.join(',')} with tenant_id but without organization_id`,
    );
  }
}

const violationList = [...violations];
if (process.argv.includes('--summary') && violationList.length > 0) {
  const counts = new Map();
  for (const violation of violationList) {
    const file = violation.slice(0, violation.indexOf(':'));
    counts.set(file, (counts.get(file) ?? 0) + 1);
  }
  for (const [file, count] of [...counts].sort()) {
    console.error(`${file}: ${count}`);
  }
  console.error(`total: ${violationList.length}`);
  process.exitCode = 1;
} else {
assert.deepEqual(
  violationList,
  [],
  `organization scope SQL violations:\n${violationList.join('\n')}`,
);
}
if (violationList.length > 0) {
  process.exit();
}
console.log(
  `organization scope SQL check passed (${tables.size} tables, ${listRustFiles(sourceRoot).length} Rust files)`,
);
