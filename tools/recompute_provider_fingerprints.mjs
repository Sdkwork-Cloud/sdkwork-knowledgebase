#!/usr/bin/env node
// One-shot maintenance tool: recomputes provider contractCertification
// sourceFingerprint values from their evidence sources after source drift.
// Run: node tools/recompute_provider_fingerprints.mjs
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const manifestPath = path.join(
  root,
  'external/knowledge-engines/provider-certification.manifest.json',
);

async function computeFingerprint(references) {
  const hash = createHash('sha256');
  for (const reference of [...new Set(references)].sort()) {
    hash.update(`${reference}\0`, 'utf8');
    hash.update(await readFile(path.join(root, reference)));
    hash.update('\0', 'utf8');
  }
  return hash.digest('hex');
}

const manifest = JSON.parse(await readFile(manifestPath, 'utf8'));
const updated = [];
for (const provider of manifest.providers ?? []) {
  const certification = provider.contractCertification ?? {};
  const evidence = certification.evidence ?? {};
  const references = new Set();
  for (const dimension of Object.values(evidence)) {
    for (const entry of Array.isArray(dimension) ? dimension : []) {
      if (typeof entry === 'string' && entry.startsWith('crates/')) {
        references.add(entry.replaceAll('\\', '/'));
      }
    }
  }
  if (references.size === 0) {
    continue;
  }
  const fingerprint = await computeFingerprint(references);
  if (certification.sourceFingerprint !== fingerprint) {
    certification.sourceFingerprint = fingerprint;
    updated.push(provider.vendorId);
  }
}

await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
console.log(`recomputed sourceFingerprint for: ${updated.join(', ') || '(none)'}`);
