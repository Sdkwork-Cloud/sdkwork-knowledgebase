#!/usr/bin/env node
import { readFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const specPath = "specs/external-knowledge-engine-catalog.spec.json";
const catalogPath = "external/knowledge-engines/catalog.manifest.json";

const vendorIdPattern = /^[a-z0-9][a-z0-9_-]*$/;
const implementationIdPattern =
  /^engine\.knowledge\.external\.[a-z0-9][a-z0-9_-]*$/;
const agentProviderIdPattern =
  /^provider\.knowledge\.external\.[a-z0-9][a-z0-9_-]*$/;

const violations = [];

function assert(condition, message) {
  if (!condition) {
    violations.push(message);
  }
}

const spec = JSON.parse(await readFile(path.join(root, specPath), "utf8"));
const catalog = JSON.parse(await readFile(path.join(root, catalogPath), "utf8"));

assert(
  catalog.kind === "sdkwork.external-knowledge-engine-catalog",
  `${catalogPath} must declare kind sdkwork.external-knowledge-engine-catalog`,
);

const categories = new Set(spec.categories);
const tiers = new Set(spec.integrationTiers);
const seenVendorIds = new Set();
const seenImplementationIds = new Set();

for (const entry of catalog.vendors ?? []) {
  const vendorManifestRel = entry.manifestPath;
  const vendorManifestAbs = path.join(root, vendorManifestRel);
  let vendor;
  try {
    vendor = JSON.parse(await readFile(vendorManifestAbs, "utf8"));
  } catch {
    violations.push(`missing vendor manifest: ${vendorManifestRel}`);
    continue;
  }

  for (const field of spec.requiredVendorFields) {
    if (vendor[field] === undefined) {
      violations.push(`${vendorManifestRel}: missing required field ${field}`);
    }
  }

  assert(vendorIdPattern.test(vendor.vendorId), `${vendorManifestRel}: invalid vendorId`);
  assert(
    vendor.vendorId === entry.vendorId,
    `${vendorManifestRel}: vendorId must match catalog entry ${entry.vendorId}`,
  );
  assert(
    vendor.implementationId === entry.implementationId,
    `${vendorManifestRel}: implementationId must match catalog entry`,
  );
  assert(
    implementationIdPattern.test(vendor.implementationId),
    `${vendorManifestRel}: invalid implementationId`,
  );
  assert(
    agentProviderIdPattern.test(vendor.agentProviderId),
    `${vendorManifestRel}: invalid agentProviderId`,
  );
  assert(categories.has(vendor.category), `${vendorManifestRel}: unknown category`);
  assert(tiers.has(vendor.integrationTier), `${vendorManifestRel}: unknown integrationTier`);

  if (vendor.integrationTier === "adapter") {
    const adapterCrate = vendor.adapterCrate
      ?? spec.adapterCratePattern.replace("{vendorId}", vendor.vendorId);
    assert(
      adapterCrate === spec.adapterCratePattern.replace("{vendorId}", vendor.vendorId),
      `${vendorManifestRel}: adapterCrate must follow ${spec.adapterCratePattern}`,
    );
    try {
      await access(path.join(root, adapterCrate, "Cargo.toml"));
    } catch {
      violations.push(
        `${vendorManifestRel}: integrationTier adapter requires adapter crate at ${adapterCrate}`,
      );
    }
  }

  for (const field of spec.requiredUpstreamFields) {
    if (!vendor.upstream?.[field]) {
      violations.push(`${vendorManifestRel}: upstream.${field} is required`);
    }
  }

  assert(
    vendor.upstream.submodulePath.startsWith(spec.upstreamSubmoduleRoot),
    `${vendorManifestRel}: upstream.submodulePath must start with ${spec.upstreamSubmoduleRoot}`,
  );
  assert(
    vendor.upstream.submodulePath.endsWith(`/${vendor.vendorId}`),
    `${vendorManifestRel}: upstream.submodulePath must end with /${vendor.vendorId}`,
  );

  for (const field of spec.requiredSpiMappingFields) {
    if (typeof vendor.spiMapping?.[field] !== "boolean") {
      violations.push(`${vendorManifestRel}: spiMapping.${field} must be boolean`);
    }
  }

  assert(!seenVendorIds.has(vendor.vendorId), `duplicate vendorId ${vendor.vendorId}`);
  seenVendorIds.add(vendor.vendorId);
  assert(
    !seenImplementationIds.has(vendor.implementationId),
    `duplicate implementationId ${vendor.implementationId}`,
  );
  seenImplementationIds.add(vendor.implementationId);
}

const externalCatalogSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/external_catalog.rs",
  ),
  "utf8",
);
const registryModSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/mod.rs",
  ),
  "utf8",
);

assert(
  registryModSource.includes("load_external_engines_from_catalog"),
  "knowledge_engine/mod.rs must register catalog external engines via load_external_engines_from_catalog",
);

for (const entry of catalog.vendors ?? []) {
  assert(
    externalCatalogSource.includes(`"${entry.vendorId}"`),
    `external_catalog.rs must embed vendor manifest loader for ${entry.vendorId}`,
  );
}

if (violations.length > 0) {
  console.error("External knowledge engine catalog violations:\n" + violations.join("\n"));
  process.exit(1);
}

console.log(
  `External knowledge engine catalog check passed (${catalog.vendors.length} vendors).`,
);
