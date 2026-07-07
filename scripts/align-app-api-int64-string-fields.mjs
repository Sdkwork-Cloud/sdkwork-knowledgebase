#!/usr/bin/env node
/**
 * Align app-api OpenAPI uint64 fields to SDKWork int64-string wire format (API_SPEC / SDK_SPEC).
 */
import { readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const checkOnly = process.argv.includes('--check');

const targets = [
  path.join(repoRoot, 'apis/app-api/knowledgebase-app-api.openapi.json'),
  path.join(
    repoRoot,
    'sdks/sdkwork-knowledgebase-app-sdk/openapi/knowledgebase-app-api.openapi.json',
  ),
];

function isUint64IntegerSchema(schema) {
  return (
    schema
    && typeof schema === 'object'
    && (
      schema.type === 'integer'
      || (Array.isArray(schema.type) && schema.type.includes('integer'))
    )
    && (schema.format === 'uint64' || schema.format === 'int64')
  );
}

function toInt64StringSchema(schema) {
  const type = Array.isArray(schema.type)
    ? schema.type.map((item) => (item === 'integer' ? 'string' : item))
    : 'string';
  const next = {
    type,
    format: schema.format,
    pattern: '^[0-9]+$',
    'x-sdkwork-int64-string': true,
  };
  if (schema.description) {
    next.description = schema.description;
  }
  return next;
}

function walk(node, stats) {
  if (!node || typeof node !== 'object') {
    return;
  }
  if (Array.isArray(node)) {
    for (const item of node) {
      walk(item, stats);
    }
    return;
  }

  if (isUint64IntegerSchema(node)) {
    for (const key of ['minimum', 'maximum', 'exclusiveMinimum', 'exclusiveMaximum', 'multipleOf']) {
      delete node[key];
    }
    Object.assign(node, toInt64StringSchema(node));
    stats.converted += 1;
    return;
  }

  for (const value of Object.values(node)) {
    walk(value, stats);
  }
}

let totalConverted = 0;
const pendingChanges = [];
for (const target of targets) {
  const before = readFileSync(target, 'utf8');
  const openapi = JSON.parse(before);
  const stats = { converted: 0 };
  walk(openapi, stats);
  const after = `${JSON.stringify(openapi, null, 2)}\n`;
  if (before !== after) {
    if (checkOnly) {
      pendingChanges.push(path.relative(repoRoot, target).replaceAll('\\', '/'));
    } else {
      writeFileSync(target, after, 'utf8');
    }
  }
  totalConverted += stats.converted;
  console.log(`aligned ${target}: converted ${stats.converted} uint64 integer schema(s)`);
}

if (checkOnly && pendingChanges.length > 0) {
  console.error(
    JSON.stringify(
      {
        ok: false,
        pendingChanges,
      },
      null,
      2,
    ),
  );
  process.exit(1);
}

console.log(`total converted: ${totalConverted}`);
