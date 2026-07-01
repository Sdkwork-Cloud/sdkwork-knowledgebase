#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const targets = [
  'apis/backend-api/knowledgebase-backend-api.openapi.json',
  'sdks/sdkwork-knowledgebase-backend-sdk/openapi/knowledgebase-backend-api.openapi.json',
];

for (const relativePath of targets) {
  const filePath = path.join(repoRoot, relativePath);
  if (!fs.existsSync(filePath)) continue;
  const source = fs.readFileSync(filePath, 'utf8');
  const count = (source.match(/knowledge\.admin/g) ?? []).length;
  const updated = source.replaceAll(
    '"x-sdkwork-permission": "knowledge.admin"',
    '"x-sdkwork-permission": "knowledge.platform.manage"',
  );
  fs.writeFileSync(filePath, updated);
  console.log(`${relativePath}: replaced ${count} knowledge.admin permission declarations`);
}
