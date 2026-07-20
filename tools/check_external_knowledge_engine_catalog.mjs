#!/usr/bin/env node
import { readFile, access, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { validateProviderCertification } from "./provider_certification.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const specPath = "specs/external-knowledge-engine-catalog.spec.json";
const catalogPath = "external/knowledge-engines/catalog.manifest.json";
const certificationPath = "external/knowledge-engines/provider-certification.manifest.json";

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
const certification = JSON.parse(
  await readFile(path.join(root, certificationPath), "utf8"),
);
violations.push(...await validateProviderCertification(certification, root));

assert(
  catalog.kind === "sdkwork.external-knowledge-engine-catalog",
  `${catalogPath} must declare kind sdkwork.external-knowledge-engine-catalog`,
);
assert(
  certification.kind === "sdkwork.knowledge-engine-provider-certification",
  `${certificationPath} must declare kind sdkwork.knowledge-engine-provider-certification`,
);
assert(
  spec.certificationManifest === certificationPath,
  `${specPath}: certificationManifest must match the checked certification authority`,
);
assert(
  spec.liveCertificationEvidenceSchema === certification.policy?.liveEvidenceSchema,
  `${specPath}: liveCertificationEvidenceSchema must match the certification policy`,
);

const certificationsByVendor = new Map();
for (const provider of certification.providers ?? []) {
  assert(
    !certificationsByVendor.has(provider.vendorId),
    `${certificationPath}: duplicate vendorId ${provider.vendorId}`,
  );
  certificationsByVendor.set(provider.vendorId, provider);
}

const categories = new Set(spec.categories);
const tiers = new Set(spec.integrationTiers);
const seenVendorIds = new Set();
const seenImplementationIds = new Set();
const runtimeAdapterSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs",
  ),
  "utf8",
);

for (const entry of catalog.vendors ?? []) {
  const vendorManifestRel = entry.manifestPath;
  const vendorManifestAbs = path.join(root, vendorManifestRel);
  let vendor;
  const providerCertification = certificationsByVendor.get(entry.vendorId);
  assert(
    providerCertification,
    `${certificationPath}: missing provider ${entry.vendorId}`,
  );
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
    vendor.integrationTier === entry.integrationTier,
    `${vendorManifestRel}: integrationTier must match catalog entry`,
  );
  assert(
    vendor.category === entry.category,
    `${vendorManifestRel}: category must match catalog entry`,
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

  if (["adapter", "production"].includes(vendor.integrationTier)) {
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

    const adapterCargoSource = await readFile(
      path.join(root, adapterCrate, "Cargo.toml"),
      "utf8",
    );
    const adapterClientSource = await readFile(
      path.join(root, adapterCrate, "src/client.rs"),
      "utf8",
    );
    assert(
      adapterCargoSource.includes(
        "sdkwork-knowledgebase-provider-runtime.workspace = true",
      ) && adapterCargoSource.includes("zeroize.workspace = true"),
      `${vendorManifestRel}: adapter must depend on the shared Provider Runtime and zeroizing secret storage`,
    );
    assert(
      adapterClientSource.includes("ProviderRuntime"),
      `${vendorManifestRel}: adapter client must execute through ProviderRuntime`,
    );
    assert(
      (adapterClientSource.match(/context:\s*&ProviderExecutionContext/g) ?? []).length >= 2,
      `${vendorManifestRel}: adapter search/read clients must require caller-provided ProviderExecutionContext`,
    );
    assert(
      !adapterClientSource.includes("for_implementation"),
      `${vendorManifestRel}: adapter client must not fabricate business execution context`,
    );
    const forbiddenDirectHttpPatterns = [
      ["reqwest::Client", "reqwest::Client"],
      ["RequestBuilder", "reqwest::RequestBuilder"],
      ["Client::new()", "Client::new()"],
      [".send()", "direct request send"],
    ];
    for (const [pattern, label] of forbiddenDirectHttpPatterns) {
      assert(
        !adapterClientSource.includes(pattern),
        `${vendorManifestRel}: adapter client must not use ${label}; use ProviderRuntime`,
      );
    }

    const adapterSource = await readFile(
      path.join(root, adapterCrate, "src/lib.rs"),
      "utf8",
    );
    const adapterConfigSource = await readFile(
      path.join(root, adapterCrate, "src/config.rs"),
      "utf8",
    );
    const forbiddenProviderAuthorityPatterns = [
      ["KnowledgeSourceStore", "source-store Provider authority"],
      ["from_runtime", "legacy runtime constructor"],
      ["resolve_connector_", "source-metadata Provider resolution"],
      ["connector_metadata", "connector metadata Provider configuration"],
    ];
    for (const [pattern, label] of forbiddenProviderAuthorityPatterns) {
      assert(
        !adapterSource.includes(pattern),
        `${vendorManifestRel}: adapter must not use ${label}; active Provider bindings own remote resources`,
      );
    }
    assert(
      (adapterSource.match(/from_knowledge_engine_request\(/g) ?? []).length >= 2
        && !adapterSource.includes("ProviderExecutionContext::from_knowledge_engine("),
      `${vendorManifestRel}: adapter search/read must validate request tenant and space through Provider Runtime`,
    );
    assert(
      /fn bind_provider\(\s*&self,\s*[^,]+,\s*credential:\s*Option<KnowledgeEngineProviderCredential>,/s.test(adapterSource)
        && (vendor.vendorId === "onyx"
          || adapterSource.includes("binding.remote_resource_id")),
      `${vendorManifestRel}: adapter must instantiate a binding-scoped remote resource with a one-time resolved credential`,
    );
    assert(
      !adapterConfigSource.includes("read_credential")
        && !adapterConfigSource.includes("read_to_string")
        && !/std::env::var\([^)]*CREDENTIAL/.test(adapterConfigSource)
        && !adapterConfigSource.includes("_CREDENTIAL_FILE_ENV"),
      `${vendorManifestRel}: adapter config must not read credentials at startup or expose credential-file environment aliases`,
    );
    assert(
      adapterConfigSource.includes("Zeroizing<String>")
        && !/#\[derive\([^\]]*Debug[^\]]*\)\]\s*pub struct \w+ConnectorConfig/s.test(adapterConfigSource),
      `${vendorManifestRel}: adapter config must zeroize held credentials and must not derive printable Debug`,
    );
    const adapterTestsDir = path.join(root, adapterCrate, "tests");
    let adapterTestSource = "";
    try {
      const testFiles = (await readdir(adapterTestsDir))
        .filter((file) => file.endsWith(".rs"));
      adapterTestSource = (
        await Promise.all(
          testFiles.map((file) => readFile(path.join(adapterTestsDir, file), "utf8")),
        )
      ).join("\n");
    } catch {
      violations.push(`${vendorManifestRel}: adapter tests directory is required`);
    }
    const runtimeCrateName = adapterCrate
      .split("/")
      .at(-1)
      .replaceAll("-", "_");
    assert(
      runtimeAdapterSource.includes(runtimeCrateName),
      `${vendorManifestRel}: adapter crate must be wired by knowledge_engine_adapters.rs`,
    );

    if (adapterSource.includes("descriptor_for_external_search_read")) {
      const expectedMapping = {
        search: true,
        read: true,
        list: false,
        health: true,
        ingest: false,
        syncSources: false,
      };
      for (const [capability, expected] of Object.entries(expectedMapping)) {
        assert(
          vendor.spiMapping?.[capability] === expected,
          `${vendorManifestRel}: spiMapping.${capability} must be ${expected} for the runtime descriptor`,
        );
      }
    } else {
      violations.push(
        `${vendorManifestRel}: adapter must publish runtime capabilities through an approved descriptor helper`,
      );
    }

    assert(
      providerCertification?.contractGate === "required",
      `${certificationPath}: ${vendor.vendorId} adapter contractGate must be required`,
    );
    assert(
      adapterTestSource.includes("health_maps_upstream_availability")
        && adapterTestSource.includes("KnowledgeEngineHealthStatus::Available")
        && adapterTestSource.includes("KnowledgeEngineHealthStatus::Degraded")
        && adapterTestSource.includes(
          ".expect(if upstream_status >= 500 { 3 } else { 1 })",
        ),
      `${vendorManifestRel}: adapter certification requires health success and degradation tests`,
    );
    assert(
      /async fn [a-z0-9_]*search[a-z0-9_]*\(/.test(adapterTestSource),
      `${vendorManifestRel}: adapter certification requires an executable search test`,
    );
    assert(
      /async fn [a-z0-9_]*read_document[a-z0-9_]*\(/.test(adapterTestSource),
      `${vendorManifestRel}: adapter certification requires an executable read_document test`,
    );
    const listMethodStart = adapterSource.indexOf("async fn list_documents");
    const listMethodEvidence = listMethodStart >= 0
      ? adapterSource.slice(listMethodStart, listMethodStart + 1_200)
      : "";
    assert(
      listMethodEvidence.includes("KnowledgeEngineError::Unsupported"),
      `${vendorManifestRel}: spiMapping.list=false requires explicit Unsupported method behavior`,
    );

    const liveStatus = providerCertification?.liveCertification?.status;
    assert(
      liveStatus === "pending" || liveStatus === "certified",
      `${certificationPath}: ${vendor.vendorId} liveCertification.status must be pending or certified`,
    );
    if (liveStatus === "certified" || vendor.integrationTier === "production") {
      for (const field of certification.policy?.productionEvidence ?? []) {
        assert(
          providerCertification?.liveCertification?.[field],
          `${certificationPath}: ${vendor.vendorId} production certification requires ${field}`,
        );
      }
    }
    if (vendor.integrationTier === "production") {
      assert(
        liveStatus === "certified",
        `${certificationPath}: ${vendor.vendorId} production tier requires certified live evidence`,
      );
    }
  } else {
    for (const field of spec.requiredSpiMappingFields) {
      assert(
        vendor.spiMapping?.[field] === false,
        `${vendorManifestRel}: non-adapter tier must not advertise executable spiMapping.${field}`,
      );
    }
    assert(
      providerCertification?.contractGate === "not-applicable"
        && providerCertification?.liveCertification?.status === "not-applicable",
      `${certificationPath}: non-executable ${vendor.vendorId} must be not-applicable`,
    );
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

for (const vendorId of certificationsByVendor.keys()) {
  assert(
    seenVendorIds.has(vendorId),
    `${certificationPath}: unknown provider ${vendorId}`,
  );
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
