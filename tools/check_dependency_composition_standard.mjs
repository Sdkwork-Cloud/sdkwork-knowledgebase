#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  extractApplicationCode,
  findCorePackages,
  validateBootstrapCompositionImports,
  validateCapabilitySdkImportBoundary,
  validateCoreCompositionLayout,
  validateManifestAlignment,
  validateManifestSchema,
  validateSdkInventoryComposition,
} from '../../sdkwork-specs/tools/lib/dependency-composition.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const appRoot = path.join(repoRoot, 'apps/sdkwork-knowledgebase-pc');
const failures = [];

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(repoRoot, relativePath), 'utf8'));
}

const relRoot = 'apps/sdkwork-knowledgebase-pc';
const manifestPath = path.join(appRoot, 'specs/dependency.composition.json');

if (!fs.existsSync(manifestPath)) {
  failures.push(`${relRoot}: missing specs/dependency.composition.json`);
} else {
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
  failures.push(...validateManifestSchema(manifest, `${relRoot}/specs/dependency.composition.json`));
  failures.push(...validateManifestAlignment(manifest, appRoot, `${relRoot}/specs/dependency.composition.json`));

  const applicationCode = extractApplicationCode('sdkwork-knowledgebase-pc');
  if (applicationCode && manifest.applicationCode !== applicationCode) {
    failures.push(
      `${relRoot}: applicationCode ${manifest.applicationCode} does not match root name ${applicationCode}`,
    );
  }
  if (manifest.clientArchitecture !== 'pc') {
    failures.push(`${relRoot}: clientArchitecture must be pc`);
  }
}

const componentSpecPath = path.join(appRoot, 'specs/component.spec.json');
if (!fs.existsSync(componentSpecPath)) {
  failures.push(`${relRoot}: missing specs/component.spec.json`);
} else {
  const componentSpec = JSON.parse(fs.readFileSync(componentSpecPath, 'utf8'));
  if (componentSpec.contracts?.dependencyComposition !== 'specs/dependency.composition.json') {
    failures.push(
      `${relRoot}: specs/component.spec.json must set contracts.dependencyComposition to specs/dependency.composition.json`,
    );
  }
}

if (!fs.existsSync(path.join(appRoot, 'sdkwork.app.config.json'))) {
  failures.push(`${relRoot}: missing sdkwork.app.config.json`);
}

const architecture = { id: 'pc', suffix: '-pc', coreRole: 'pc-core', corePattern: /-pc-core$/ };
const applicationCode = extractApplicationCode('sdkwork-knowledgebase-pc');
const cores = findCorePackages(appRoot, 'sdkwork-knowledgebase-pc', applicationCode, architecture);
for (const core of cores) {
  failures.push(...validateCoreCompositionLayout(core, `${relRoot}/packages/${core.packageName}`));
  failures.push(...validateSdkInventoryComposition(core, `${relRoot}/packages/${core.packageName}`));
}

failures.push(...validateBootstrapCompositionImports(appRoot, cores, relRoot));
failures.push(...validateCapabilitySdkImportBoundary(appRoot, relRoot));

if (failures.length > 0) {
  process.stderr.write(`Dependency composition standard failed:\n${failures.map((f) => `- ${f}`).join('\n')}\n`);
  process.exit(1);
}

process.stdout.write('Dependency composition standard check passed.\n');
