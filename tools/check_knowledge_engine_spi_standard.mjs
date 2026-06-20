#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const violations = [];

function assert(condition, message) {
  if (!condition) {
    violations.push(message);
  }
}

const spec = JSON.parse(
  await readFile(path.join(root, "specs/knowledge-engine-spi.spec.json"), "utf8"),
);

const requiredPaths = [
  "specs/knowledge-engine-spi.spec.json",
  "specs/external-knowledge-engine-catalog.spec.json",
  "external/knowledge-engines/catalog.manifest.json",
  "crates/sdkwork-knowledgebase-contract/src/knowledge_engine.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_engine.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/mod.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/okf_native.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/external_catalog.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/rag_native.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/rag/index_rebuild.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_index_store.rs",
  "crates/sdkwork-knowledgebase-engine-dify/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-dify/tests/dify_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-dify/tests/dify_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-ragflow/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-ragflow/tests/ragflow_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-onyx/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-onyx/tests/onyx_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-onyx/tests/onyx_adapter_http.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/tests/knowledge_engine_catalog.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/tests/knowledge_engine_native.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/space_resolver.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/tests/knowledge_engine_space_resolver.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/tests/knowledge_engine_kernel_bridge.rs",
  "crates/sdkwork-knowledgebase-contract/tests/knowledge_engine_contract.rs",
];

for (const relativePath of requiredPaths) {
  try {
    await readFile(path.join(root, relativePath), "utf8");
  } catch {
    violations.push(`missing required SPI artifact: ${relativePath}`);
  }
}

const runtimeSource = await readFile(
  path.join(root, "crates/sdkwork-router-knowledgebase-app-api/src/runtime.rs"),
  "utf8",
);

assert(
  runtimeSource.includes("build_default_registry"),
  "KnowledgebaseRuntime must build default knowledge engine registry",
);
assert(
  runtimeSource.includes("load_runtime_external_adapter_engines"),
  "KnowledgebaseRuntime must wire approved external adapter crates",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/mod.rs"),
    "utf8",
  )).includes("deps.external_engines"),
  "build_default_registry must register runtime adapter engines before catalog stubs",
);

const registryModSource = await readFile(
  path.join(root, "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/mod.rs"),
  "utf8",
);
assert(
  registryModSource.includes("load_external_engines_from_catalog"),
  "build_default_registry must register catalog external engines",
);
const registryBuildFnStart = registryModSource.indexOf("pub fn build_default_registry");
const registryBuildFnEnd = registryModSource.indexOf("tenant_id: deps.tenant_id,", registryBuildFnStart);
const registryBuildBody = registryModSource.slice(registryBuildFnStart, registryBuildFnEnd);
assert(
  registryBuildBody.includes("deps.external_engines")
    && registryBuildBody.includes("load_external_engines_from_catalog")
    && registryBuildBody.indexOf("deps.external_engines")
      < registryBuildBody.indexOf("load_external_engines_from_catalog"),
  "adapter crate registration must precede catalog stub registration inside build_default_registry",
);

assert(
  (await readFile(
    path.join(root, "crates/sdkwork-router-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("RagflowKnowledgeEngine"),
  "runtime must wire RAGFlow adapter crate alongside Dify",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-router-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("OnyxKnowledgeEngine"),
  "runtime must wire Onyx adapter crate alongside Dify and RAGFlow",
);

const hostedBackendSource = await readFile(
  path.join(root, "crates/sdkwork-router-knowledgebase-app-api/src/hosted_backend.rs"),
  "utf8",
);

assert(
  hostedBackendSource.includes("knowledge_engine_registry"),
  "retrieve_provider_health must report registered knowledge engines",
);

assert(
  runtimeSource.includes("KnowledgeEngineSpaceResolver"),
  "KnowledgebaseRuntime must expose KnowledgeEngineSpaceResolver",
);

const portsSource = await readFile(
  path.join(root, "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_engine.rs"),
  "utf8",
);

assert(
  portsSource.includes("trait KnowledgeEngineSpaceRegistry"),
  "knowledge engine ports must declare KnowledgeEngineSpaceRegistry",
);

assert(
  portsSource.includes("trait KnowledgeEngineRegistrar"),
  "knowledge engine ports must declare KnowledgeEngineRegistrar (spec registry.register)",
);

const externalCatalogSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/external_catalog.rs",
  ),
  "utf8",
);
assert(
  externalCatalogSource.includes('integration_tier == "adapter"'),
  "catalog loader must skip adapter-tier vendors (runtime adapter crates own registration)",
);

assert(
  (await readFile(
    path.join(
      root,
      "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/external_connector.rs",
    ),
    "utf8",
  )).includes("resolve_connector_dataset_id_for_space"),
  "external adapter connector resolution must be shared in service layer",
);

for (const traitName of ["OkfBundleEngine", "RagKnowledgeEngine", "ExternalKnowledgeEngine"]) {
  assert(
    portsSource.includes(`trait ${traitName}`),
    `knowledge engine ports must declare ${traitName}`,
  );
}

const sourceContractSource = await readFile(
  path.join(root, "crates/sdkwork-knowledgebase-contract/src/source.rs"),
  "utf8",
);
assert(
  sourceContractSource.includes("dataset_id_from_connector_metadata_json"),
  "KnowledgeSource contract must expose shared connector metadata parsing",
);

for (const implementationId of spec.registry.builtInImplementations) {
  assert(
    runtimeSource.includes(implementationId)
      || hostedBackendSource.includes(implementationId)
      || (await readFile(
        path.join(
          root,
          "crates/sdkwork-knowledgebase-contract/src/knowledge_engine.rs",
        ),
        "utf8",
      )).includes(implementationId),
    `built-in implementation must be declared: ${implementationId}`,
  );
}

const spaceResolverSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/space_resolver.rs",
  ),
  "utf8",
);
assert(
  spaceResolverSource.includes("resolve_for_space"),
  "KnowledgeEngineSpaceResolver must implement per-space engine resolution",
);

const ragNativeSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/rag_native.rs",
  ),
  "utf8",
);
assert(
  ragNativeSource.includes("async fn rebuild_index"),
  "RagNativeKnowledgeEngine must implement rebuild_index SPI extension",
);

const agentChatSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/agent_chat.rs",
  ),
  "utf8",
);
assert(
  agentChatSource.includes("validate_bindings_support_mode"),
  "agent chat must validate bindings against knowledge mode before fetch",
);
assert(
  agentChatSource.includes("spawn_blocking"),
  "agent chat must invoke sync kernel runtime off the async worker thread",
);

const mapperSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-knowledgebase-agent-provider/src/mapper.rs",
  ),
  "utf8",
);
assert(
  mapperSource.includes("scoped_knowledge_document_ref"),
  "RAG search results must expose scoped document refs for read alignment",
);

const externalProviderSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-knowledgebase-agent-provider/src/external_space_engine_provider.rs",
  ),
  "utf8",
);
assert(
  externalProviderSource.includes("SpaceEngineKnowledgeProvider"),
  "agent provider must expose external space engine knowledge provider bridge",
);
assert(
  externalProviderSource.includes("block_on_async"),
  "external knowledge provider must bridge async space engine client without current-thread runtime panics",
);
assert(
  externalProviderSource.includes("read_space_document"),
  "external knowledge provider must read via space engine client",
);

const agentChatRuntimeSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-router-knowledgebase-app-api/src/agent_chat_runtime.rs",
  ),
  "utf8",
);
assert(
  agentChatRuntimeSource.includes("read_space_document"),
  "runtime space engine client must implement external read_document SPI bridge",
);

const agentRuntimeSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-knowledgebase-agent-provider/src/agent_runtime.rs",
  ),
  "utf8",
);
assert(
  agentRuntimeSource.includes("external_knowledge_provider_ids"),
  "agent runtime must register resolved external knowledge providers",
);
assert(
  agentChatSource.includes("external_knowledge_provider_ids"),
  "agent chat must pass resolved external knowledge provider ids into runtime build",
);

const hostedRoutesSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-router-knowledgebase-app-api/tests/hosted_runtime_routes.rs",
  ),
  "utf8",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_succeeds_with_configured_dify_adapter"),
  "hosted runtime must include configured external agent chat success E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_succeeds_with_configured_ragflow_adapter"),
  "hosted runtime must include configured RAGFlow adapter agent chat success E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_rejects_unconfigured_external_adapter"),
  "hosted runtime must include unconfigured external adapter rejection E2E",
);

assert(
  (await readFile(
    path.join(root, "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/mod.rs"),
    "utf8",
  )).includes("DefaultKnowledgeEngineRegistry"),
  "build_default_registry must return DefaultKnowledgeEngineRegistry with typed native engines",
);

assert(
  hostedBackendSource.includes("knowledge_engines()")
    || hostedBackendSource.includes("rebuild_rag_index"),
  "hosted backend RAG index operations must route through knowledge engine registry",
);

assert(
  (await readFile(
    path.join(root, "crates/sdkwork-router-knowledgebase-app-api/src/hosted_support.rs"),
    "utf8",
  )).includes("rebuild_okf_index"),
  "hosted OKF index rebuild must route through knowledge engine SPI",
);

if (violations.length > 0) {
  console.error("Knowledge Engine SPI standard violations:");
  for (const violation of violations) {
    console.error(`  - ${violation}`);
  }
  process.exit(1);
}

console.log("Knowledge Engine SPI standard check passed.");
