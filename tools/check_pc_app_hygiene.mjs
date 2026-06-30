import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const pcRoot = path.resolve(scriptDir, '../apps/sdkwork-knowledgebase-pc');
const packagesDir = path.join(pcRoot, 'packages');

const forbiddenRootPatterns = [
  /^update_.+\.(cjs|js)$/i,
  /^fix_.+\.(cjs|js)$/i,
  /^extract_.+\.(cjs|js)$/i,
  /^replace_.+\.(cjs|js)$/i,
  /^find_.+\.(cjs|js)$/i,
  /^repair_.+\.(cjs|js)$/i,
  /^rewrite_.+\.(cjs|js)$/i,
  /^inspect-.+\.(cjs|js)$/i,
];

const forbiddenE2ePatterns = [/^debug\..+\.spec\.ts$/i];

const requiredConfigExamples = [
  'config/browser/runtime-env.development.example.json',
  'config/browser/runtime-env.staging.example.json',
  'config/browser/runtime-env.production.example.json',
  'config/desktop/sdkwork-knowledgebase-pc.development.toml.example',
  'config/desktop/sdkwork-knowledgebase-pc.production.toml.example',
];

function listFiles(dir, relativePrefix = '') {
  const entries = [];
  for (const name of readdirSync(dir)) {
    if (name === 'node_modules' || name === 'dist' || name === '.git') {
      continue;
    }
    const full = path.join(dir, name);
    const relative = path.join(relativePrefix, name);
    const stat = statSync(full);
    if (stat.isDirectory()) {
      entries.push(...listFiles(full, relative));
      continue;
    }
    entries.push(relative);
  }
  return entries;
}

const forbiddenCapabilitySdkImportPatterns = [
  /^@sdkwork\/knowledgebase-app-sdk/u,
  /^@sdkwork\/drive-app-sdk/u,
  /^@sdkwork\/iam-app-sdk/u,
];

const capabilityPackageExclusions = new Set([
  'sdkwork-knowledgebase-pc-core',
  'sdkwork-knowledgebase-pc-commons',
]);

function isCapabilityPackage(name) {
  return !capabilityPackageExclusions.has(name);
}

function listTypeScriptFiles(dir, relativePrefix = '') {
  const entries = [];
  for (const name of readdirSync(dir)) {
    if (name === 'node_modules' || name === 'dist' || name === '.git') {
      continue;
    }
    const full = path.join(dir, name);
    const relative = path.join(relativePrefix, name);
    const stat = statSync(full);
    if (stat.isDirectory()) {
      entries.push(...listTypeScriptFiles(full, relative));
      continue;
    }
    if (/\.(?:ts|tsx)$/u.test(name) && !name.endsWith('.d.ts')) {
      entries.push({ full, relative });
    }
  }
  return entries;
}

function extractImportSpecifiers(sourceText) {
  const specifiers = [];
  const importRe = /import\s+(?:type\s+)?(?:\{[^}]*\}|[^"';\s]+)\s+from\s+["']([^"']+)["']/gu;
  let match = importRe.exec(sourceText);
  while (match) {
    specifiers.push(match[1]);
    match = importRe.exec(sourceText);
  }
  return specifiers;
}

const violations = [];

for (const file of listFiles(pcRoot)) {
  const base = path.basename(file);
  if (forbiddenRootPatterns.some((pattern) => pattern.test(base))) {
    violations.push(`forbidden ad-hoc script: apps/sdkwork-knowledgebase-pc/${file.replace(/\\/g, '/')}`);
  }
  if (file.replace(/\\/g, '/').startsWith('e2e/') && forbiddenE2ePatterns.some((pattern) => pattern.test(base))) {
    violations.push(`forbidden debug e2e flow: apps/sdkwork-knowledgebase-pc/${file.replace(/\\/g, '/')}`);
  }
}

for (const example of requiredConfigExamples) {
  const full = path.join(pcRoot, example);
  try {
    statSync(full);
  } catch {
    violations.push(`missing PC config example: apps/sdkwork-knowledgebase-pc/${example}`);
  }
}

if (statSync(packagesDir).isDirectory()) {
  for (const entry of readdirSync(packagesDir)) {
    const packageDir = path.join(packagesDir, entry);
    if (!statSync(packageDir).isDirectory() || !isCapabilityPackage(entry)) {
      continue;
    }
    const srcDir = path.join(packageDir, 'src');
    if (!existsSync(srcDir)) {
      continue;
    }
    for (const { full, relative } of listTypeScriptFiles(srcDir)) {
      const source = readFileSync(full, 'utf8');
      for (const specifier of extractImportSpecifiers(source)) {
        if (!forbiddenCapabilitySdkImportPatterns.some((pattern) => pattern.test(specifier))) {
          continue;
        }
        violations.push(
          `capability package must not import generated SDK module ${specifier}: apps/sdkwork-knowledgebase-pc/packages/${entry}/${relative.replace(/\\/g, '/')}`,
        );
      }
    }
  }
}

if (violations.length > 0) {
  console.error('PC app hygiene check failed:');
  for (const violation of violations) {
    console.error(`- ${violation}`);
  }
  process.exit(1);
}

console.log('PC app hygiene check passed');
