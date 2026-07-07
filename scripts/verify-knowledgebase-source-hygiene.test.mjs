import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, it } from 'node:test';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');

const skippedDirectoryNames = new Set([
  '.git',
  '.next',
  '.turbo',
  'coverage',
  'dist',
  'generated',
  'node_modules',
  'target',
]);

const productionSourceRoots = ['apps', 'crates'];
const appSourceExtensions = new Set(['.js', '.jsx', '.ts', '.tsx']);
const rustSourceExtension = '.rs';

const forbiddenProductionPhrases = [
  {
    pattern: /\bnot implemented\b/i,
    reason: 'use a typed unsupported/unavailable result instead of incomplete implementation wording',
  },
  {
    pattern: /\bnot_implemented\b/,
    reason: 'use supported operation naming such as unsupported_operation',
  },
  {
    pattern: /\boperation_not_implemented\b/i,
    reason: 'API error codes must describe runtime unsupported/unavailable semantics',
  },
];

function listFiles(root) {
  const files = [];
  const absoluteRoot = path.join(repoRoot, root);

  function visit(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (entry.isDirectory()) {
        if (!skippedDirectoryNames.has(entry.name)) {
          visit(path.join(directory, entry.name));
        }
        continue;
      }
      files.push(path.join(directory, entry.name));
    }
  }

  visit(absoluteRoot);
  return files;
}

function countBraces(line) {
  let depth = 0;
  for (const char of line) {
    if (char === '{') {
      depth += 1;
    } else if (char === '}') {
      depth -= 1;
    }
  }
  return depth;
}

function stripRustCfgTestBlocks(source) {
  const lines = source.split(/\r?\n/);
  const stripped = [];
  let cfgTestPending = false;
  let skipping = false;
  let braceDepth = 0;

  for (const line of lines) {
    if (skipping) {
      braceDepth += countBraces(line);
      stripped.push('');
      if (braceDepth <= 0) {
        skipping = false;
      }
      continue;
    }

    if (/^\s*#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/.test(line)) {
      cfgTestPending = true;
      stripped.push('');
      continue;
    }

    if (cfgTestPending) {
      const depth = countBraces(line);
      stripped.push('');
      cfgTestPending = false;
      if (depth > 0) {
        skipping = true;
        braceDepth = depth;
      }
      continue;
    }

    stripped.push(line);
  }

  return stripped.join('\n');
}

function isProductionRustSource(filePath) {
  const relativePath = path.relative(repoRoot, filePath);
  return (
    relativePath.startsWith(`crates${path.sep}`) &&
    relativePath.includes(`${path.sep}src${path.sep}`) &&
    path.extname(filePath) === rustSourceExtension
  );
}

function isProductionAppSource(filePath) {
  const relativePath = path.relative(repoRoot, filePath);
  const extension = path.extname(filePath);
  const basename = path.basename(filePath);
  return (
    relativePath.startsWith(`apps${path.sep}`) &&
    relativePath.includes(`${path.sep}src${path.sep}`) &&
    appSourceExtensions.has(extension) &&
    !basename.includes('.test.') &&
    !basename.includes('.spec.')
  );
}

function productionSourceFiles() {
  return productionSourceRoots
    .flatMap((root) => listFiles(root))
    .filter((filePath) => isProductionRustSource(filePath) || isProductionAppSource(filePath));
}

function isAuthoredRustTestSource(filePath) {
  const relativePath = path.relative(repoRoot, filePath);
  return (
    relativePath.startsWith(`crates${path.sep}`) &&
    relativePath.includes(`${path.sep}tests${path.sep}`) &&
    path.extname(filePath) === rustSourceExtension
  );
}

function authoredRustTestFiles() {
  return listFiles('crates').filter((filePath) => isAuthoredRustTestSource(filePath));
}

describe('knowledgebase production source hygiene', () => {
  it('does not leave incomplete implementation wording in production source', () => {
    const violations = [];

    for (const filePath of productionSourceFiles()) {
      const relativePath = path.relative(repoRoot, filePath).replaceAll(path.sep, '/');
      const rawSource = readFileSync(filePath, 'utf8');
      const source = path.extname(filePath) === rustSourceExtension
        ? stripRustCfgTestBlocks(rawSource)
        : rawSource;

      source.split(/\r?\n/).forEach((line, index) => {
        for (const forbidden of forbiddenProductionPhrases) {
          if (forbidden.pattern.test(line)) {
            violations.push(`${relativePath}:${index + 1}: ${forbidden.reason}`);
          }
        }
      });
    }

    assert.deepEqual(violations, []);
  });

  it('does not leave incomplete implementation wording in authored Rust tests', () => {
    const violations = [];

    for (const filePath of authoredRustTestFiles()) {
      const relativePath = path.relative(repoRoot, filePath).replaceAll(path.sep, '/');
      const source = readFileSync(filePath, 'utf8');

      source.split(/\r?\n/).forEach((line, index) => {
        for (const forbidden of forbiddenProductionPhrases) {
          if (forbidden.pattern.test(line)) {
            violations.push(`${relativePath}:${index + 1}: ${forbidden.reason}`);
          }
        }
      });
    }

    assert.deepEqual(violations, []);
  });
});
