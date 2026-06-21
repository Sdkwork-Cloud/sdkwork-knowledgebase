#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packagesRoot = path.join(repoRoot, 'apps/sdkwork-knowledgebase-pc/packages');
const importLine =
  "import { isBlank, trim } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';\n";

function walkTsFiles(dir) {
  const results = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const absolutePath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'node_modules' || entry.name === 'dist') {
        continue;
      }
      results.push(...walkTsFiles(absolutePath));
      continue;
    }
    if (/\.(ts|tsx)$/u.test(entry.name)) {
      results.push(absolutePath);
    }
  }
  return results;
}

function needsMigration(text) {
  return (
    /\.trim\(\)/u.test(text)
    && !text.includes('@sdkwork/utils')
    && !text.includes('sdkwork-knowledgebase-pc-commons/stringUtils')
  );
}

function migrateText(text) {
  let next = text;

  if (!next.includes('sdkwork-knowledgebase-pc-commons/stringUtils')) {
    const importMatch = next.match(/^import .+?;\r?\n/m);
    if (importMatch) {
      const insertAt = importMatch.index + importMatch[0].length;
      next = `${next.slice(0, insertAt)}${importLine}${next.slice(insertAt)}`;
    } else {
      next = `${importLine}${next}`;
    }
  }

  next = next.replace(/!([\w$]+)\.trim\(\)/gu, 'isBlank($1)');
  next = next.replace(/!([\w$]+)\?\.trim\(\)/gu, 'isBlank($1)');
  next = next.replace(/([\w$]+(?:\.[\w$]+)*)\.trim\(\)\.length\s*>\s*0/gu, '!isBlank($1)');
  next = next.replace(/([\w$]+(?:\.[\w$]+)*)\.trim\(\)\.length\s*===\s*0/gu, 'isBlank($1)');
  next = next.replace(/if\s*\(\s*!([\w$]+(?:\.[\w$]+)*)\.trim\(\)\s*\|\|/gu, 'if (isBlank($1) ||');
  next = next.replace(/if\s*\(\s*!([\w$]+(?:\.[\w$]+)*)\.trim\(\)\s*&&/gu, 'if (isBlank($1) &&');

  return next;
}

let changed = 0;
for (const filePath of walkTsFiles(packagesRoot)) {
  const original = fs.readFileSync(filePath, 'utf8');
  if (!needsMigration(original)) {
    continue;
  }
  const migrated = migrateText(original);
  if (migrated !== original) {
    fs.writeFileSync(filePath, migrated);
    changed += 1;
    process.stdout.write(`${path.relative(repoRoot, filePath)}\n`);
  }
}

process.stdout.write(`Migrated ${changed} files.\n`);
