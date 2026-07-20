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
  "crates/sdkwork-knowledgebase-contract/src/provider_binding.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_engine.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_provider_binding_store.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/provider_binding.rs",
  "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/provider_binding_store.rs",
  "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/tests/provider_binding_store.rs",
  "database/migrations/sqlite/202607200001_knowledge_engine_provider_binding.up.sql",
  "database/migrations/postgres/202607200001_knowledge_engine_provider_binding.up.sql",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/mod.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/okf_native.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/external_catalog.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/rag_native.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/rag/index_rebuild.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_index_store.rs",
  "crates/sdkwork-knowledgebase-engine-dify/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-dify/tests/dify_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-dify/tests/dify_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-dify/tests/dify_read_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-ragflow/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-ragflow/tests/ragflow_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-ragflow/tests/ragflow_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-ragflow/tests/ragflow_read_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-onyx/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-onyx/tests/onyx_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-onyx/tests/onyx_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-anythingllm/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-anythingllm/tests/anythingllm_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-anythingllm/tests/anythingllm_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-open-webui/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-open-webui/tests/open_webui_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-open-webui/tests/open_webui_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-flowise/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-flowise/tests/flowise_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-flowise/tests/flowise_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-chroma/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-chroma/tests/chroma_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-chroma/tests/chroma_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-qdrant/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-qdrant/tests/qdrant_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-qdrant/tests/qdrant_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-weaviate/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-weaviate/tests/weaviate_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-weaviate/tests/weaviate_adapter_http.rs",
  "crates/sdkwork-knowledgebase-engine-haystack/src/lib.rs",
  "crates/sdkwork-knowledgebase-engine-haystack/tests/haystack_stub_engine.rs",
  "crates/sdkwork-knowledgebase-engine-haystack/tests/haystack_adapter_http.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/tests/knowledge_engine_catalog.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/tests/knowledge_engine_native.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/space_resolver.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/tests/knowledge_engine_space_resolver.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/tests/knowledge_engine_registry.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/tests/knowledge_engine_kernel_bridge.rs",
  "crates/sdkwork-knowledgebase-contract/tests/knowledge_engine_contract.rs",
  "tools/evaluate_knowledge_engine_retrieval.mjs",
  "tools/evaluate_knowledge_engine_retrieval.test.mjs",
  "tests/fixtures/knowledge-engine-evaluation/v1/golden.json",
  "tests/fixtures/knowledge-engine-evaluation/v1/sample-results.json",
];

for (const relativePath of requiredPaths) {
  try {
    await readFile(path.join(root, relativePath), "utf8");
  } catch {
    violations.push(`missing required SPI artifact: ${relativePath}`);
  }
}

const runtimeSource = await readFile(
  path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/runtime.rs"),
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
  !runtimeSource.includes("load_runtime_external_adapter_engines(\n                    source_store")
    && !runtimeSource.includes("load_runtime_external_adapter_engines(source_store"),
  "runtime adapter registration must not receive a source store for Provider selection",
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
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("RagflowKnowledgeEngine"),
  "runtime must wire RAGFlow adapter crate alongside Dify",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("OnyxKnowledgeEngine"),
  "runtime must wire Onyx adapter crate alongside Dify and RAGFlow",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("AnythingLlmKnowledgeEngine"),
  "runtime must wire AnythingLLM adapter crate alongside other external adapters",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("OpenWebuiKnowledgeEngine"),
  "runtime must wire Open WebUI adapter crate alongside other external adapters",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("FlowiseKnowledgeEngine"),
  "runtime must wire Flowise adapter crate alongside other external adapters",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("ChromaKnowledgeEngine"),
  "runtime must wire Chroma adapter crate alongside other external adapters",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("QdrantKnowledgeEngine"),
  "runtime must wire Qdrant adapter crate alongside other external adapters",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("WeaviateKnowledgeEngine"),
  "runtime must wire Weaviate adapter crate alongside other external adapters",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/knowledge_engine_adapters.rs"),
    "utf8",
  )).includes("HaystackKnowledgeEngine"),
  "runtime must wire Haystack adapter crate alongside other external adapters",
);

const hostedBackendSource = await readFile(
  path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/hosted_backend.rs"),
  "utf8",
);

assert(
  hostedBackendSource.includes("knowledge_engine_registry"),
  "retrieve_provider_health must report registered knowledge engines",
);
assert(
  hostedBackendSource.includes("KnowledgeEngineCapability::Health")
    && !hostedBackendSource.includes("if !descriptor.native"),
  "provider health must check every registered engine with the health capability",
);

assert(
  runtimeSource.includes("KnowledgeEngineSpaceResolver"),
  "KnowledgebaseRuntime must expose KnowledgeEngineSpaceResolver",
);
assert(
  runtimeSource.includes("SqlxKnowledgeEngineProviderBindingStore")
    && runtimeSource.includes("provider_binding_store"),
  "KnowledgebaseRuntime must wire the explicit Provider binding store",
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
  portsSource.includes("fn bind_provider"),
  "KnowledgeEngine SPI v2 must bind external implementations to persisted Provider bindings",
);

assert(
  portsSource.includes("trait KnowledgeEngineRegistrar"),
  "knowledge engine ports must declare KnowledgeEngineRegistrar (spec registry.register)",
);
assert(
  portsSource.includes("Result<(), KnowledgeEngineError>")
    && portsSource.includes("duplicate knowledge engine registration")
    && portsSource.includes("contains_key(&id)"),
  "knowledge engine registration must reject duplicate implementation ids without replacement",
);

const externalEngineSpec = spec.engineKinds.find((engine) => engine.id === "external");
assert(
  externalEngineSpec
    && externalEngineSpec.nativeCapabilities.includes("health")
    && externalEngineSpec.nativeCapabilities.includes("search")
    && externalEngineSpec.nativeCapabilities.includes("read")
    && !externalEngineSpec.nativeCapabilities.includes("list"),
  "external engine baseline capabilities must not imply document listing",
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
  !sourceContractSource.includes("dataset_id_from_connector_metadata_json")
    && !sourceContractSource.includes("workspace_slug_from_connector_metadata_json"),
  "KnowledgeSource contract must not expose source metadata as Provider configuration authority",
);

assert(
  (await readFile(
    path.join(root, "crates/sdkwork-knowledgebase-contract/src/knowledge_engine.rs"),
    "utf8",
  )).includes("parse_compound_document_ref"),
  "contract must expose shared compound external document ref parsing",
);
assert(
  (await readFile(
    path.join(root, "crates/sdkwork-knowledgebase-contract/src/knowledge_engine.rs"),
    "utf8",
  )).includes("enum KnowledgeEngineCapability"),
  "knowledge engine descriptors must publish machine-readable capabilities",
);

assert(
  !(await readFile(
    path.join(root, "crates/sdkwork-knowledgebase-engine-ragflow/src/lib.rs"),
    "utf8",
  )).includes("use search hits for now"),
  "RAGFlow adapter must implement read_document instead of leaving unsupported stub text",
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
assert(
  spaceResolverSource.includes("get_active_binding_for_space")
    && spaceResolverSource.includes("KnowledgeEngineProviderScope"),
  "external space resolution must use the scoped active Provider binding",
);
for (const forbiddenInference of [
  "KnowledgeSourceStore",
  "implementation_id_from_provider",
  "list_sources_for_space",
]) {
  assert(
    !spaceResolverSource.includes(forbiddenInference),
    `external space resolution must not infer Provider selection through ${forbiddenInference}`,
  );
}

const sqliteProviderMigration = await readFile(
  path.join(
    root,
    "database/migrations/sqlite/202607200001_knowledge_engine_provider_binding.up.sql",
  ),
  "utf8",
);
const postgresProviderMigration = await readFile(
  path.join(
    root,
    "database/migrations/postgres/202607200001_knowledge_engine_provider_binding.up.sql",
  ),
  "utf8",
);
for (const table of [
  "kb_provider_credential_reference",
  "kb_provider_binding",
  "kb_provider_migration_operation",
]) {
  assert(
    sqliteProviderMigration.includes(table) && postgresProviderMigration.includes(table),
    `Provider persistence must materialize ${table} for SQLite and PostgreSQL`,
  );
  assert(
    postgresProviderMigration.includes("ENABLE ROW LEVEL SECURITY")
      && postgresProviderMigration.includes("tenant_isolation"),
    `PostgreSQL Provider persistence must enable tenant RLS for ${table}`,
  );
}

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
    "crates/sdkwork-routes-knowledgebase-app-api/src/agent_chat_runtime.rs",
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
    "crates/sdkwork-routes-knowledgebase-app-api/tests/hosted_runtime_routes.rs",
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
  hostedRoutesSource.includes("hosted_external_agent_chat_succeeds_with_configured_anythingllm_adapter"),
  "hosted runtime must include configured AnythingLLM adapter agent chat success E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_succeeds_with_configured_open_webui_adapter"),
  "hosted runtime must include configured Open WebUI adapter agent chat success E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_succeeds_with_configured_flowise_adapter"),
  "hosted runtime must include configured Flowise adapter agent chat success E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_succeeds_with_configured_chroma_adapter"),
  "hosted runtime must include configured Chroma adapter agent chat success E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_succeeds_with_configured_qdrant_adapter"),
  "hosted runtime must include configured Qdrant adapter agent chat success E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_succeeds_with_configured_weaviate_adapter"),
  "hosted runtime must include configured Weaviate adapter agent chat success E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_succeeds_with_configured_haystack_adapter"),
  "hosted runtime must include configured Haystack adapter agent chat success E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_read_resolves_configured_open_webui_citation_document"),
  "hosted runtime must include configured Open WebUI external read citation E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_read_resolves_configured_flowise_citation_document"),
  "hosted runtime must include configured Flowise external read citation E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_read_resolves_configured_ragflow_citation_document"),
  "hosted runtime must include configured RAGFlow external read citation E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_read_resolves_configured_qdrant_citation_document"),
  "hosted runtime must include configured Qdrant external read citation E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_read_resolves_configured_chroma_citation_document"),
  "hosted runtime must include configured Chroma external read citation E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_read_resolves_configured_weaviate_citation_document"),
  "hosted runtime must include configured Weaviate external read citation E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_read_resolves_configured_haystack_citation_document"),
  "hosted runtime must include configured Haystack external read citation E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_read_resolves_configured_dify_citation_document"),
  "hosted runtime must include configured external read citation document E2E",
);
assert(
  hostedRoutesSource.includes("hosted_external_agent_chat_rejects_unconfigured_external_adapter"),
  "hosted runtime must include unconfigured external adapter rejection E2E",
);
assert(
  hostedRoutesSource.includes("hosted_okf_agent_chat_succeeds_with_published_concept_citations"),
  "hosted runtime must include OKF native agent chat success E2E",
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
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/runtime.rs"),
    "utf8",
  )).includes("read_knowledge_engine_document_for_space"),
  "hosted runtime must expose knowledge engine read bridge for external citation E2E",
);

assert(
  (await readFile(
    path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/hosted_support.rs"),
    "utf8",
  )).includes("rebuild_okf_index"),
  "hosted OKF index rebuild must route through knowledge engine SPI",
);

const hostedSupportSource = await readFile(
  path.join(root, "crates/sdkwork-routes-knowledgebase-app-api/src/hosted_support.rs"),
  "utf8",
);
assert(
  hostedSupportSource.includes("resolve_okf_bundle_engine_for_space"),
  "hosted OKF operations must resolve native bundle engine per space",
);
assert(
  hostedSupportSource.includes("lint_okf_bundle_report"),
  "hosted OKF lint must route through knowledge engine SPI",
);
assert(
  hostedSupportSource.includes("import_okf_bundle_for_actor")
    || hostedSupportSource.includes("import_okf_bundle_files"),
  "hosted OKF import must route through knowledge engine registry",
);
assert(
  hostedSupportSource.includes("OkfBundleWorkflowEngine"),
  "OKF compile/eval workflows must accept knowledge engine SPI ops",
);

assert(
  runtimeSource.includes("ensure_bindings_support_rag_retrieval"),
  "hosted runtime must reject RAG retrievals for non-RAG knowledge modes",
);

assert(
  portsSource.includes("lint_bundle_report"),
  "OkfBundleEngine must expose lint_bundle_report for hosted lint workflows",
);

if (violations.length > 0) {
  console.error("Knowledge Engine SPI standard violations:");
  for (const violation of violations) {
    console.error(`  - ${violation}`);
  }
  process.exit(1);
}

console.log("Knowledge Engine SPI standard check passed.");
