#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { validateKnowledgeEngineEvaluationWorkspace } from "./quality_evaluation_evidence.mjs";

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

const providerAdapterCrates = [
  "dify",
  "ragflow",
  "onyx",
  "anythingllm",
  "open-webui",
  "flowise",
  "chroma",
  "qdrant",
  "weaviate",
  "haystack",
];
const credentialAccessContextFields = [
  "tenant_id",
  "organization_id",
  "space_id",
  "binding_id",
  "credential_reference_id",
  "credential_reference_version",
  "implementation_id",
  "actor_id",
  "operation",
  "trace_id",
  "deadline_unix_ms",
];

const requiredPaths = [
  "specs/knowledge-engine-spi.spec.json",
  "specs/external-knowledge-engine-catalog.spec.json",
  "external/knowledge-engines/catalog.manifest.json",
  "crates/sdkwork-knowledgebase-contract/src/knowledge_engine.rs",
  "crates/sdkwork-knowledgebase-contract/src/provider_binding.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_engine.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_provider_binding_store.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_provider_credential_resolver.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/provider_binding.rs",
  "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/src/provider_binding_store.rs",
  "crates/sdkwork-intelligence-knowledgebase-repository-sqlx/tests/provider_binding_store.rs",
  "database/migrations/sqlite/202607200001_knowledge_engine_provider_binding.up.sql",
  "database/migrations/postgres/202607200001_knowledge_engine_provider_binding.up.sql",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/mod.rs",
  "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/execution_handle.rs",
  "crates/sdkwork-knowledgebase-provider-secret-adapter/src/lib.rs",
  "crates/sdkwork-knowledgebase-provider-secret-adapter/src/config.rs",
  "crates/sdkwork-knowledgebase-provider-secret-adapter/src/resolver.rs",
  "crates/sdkwork-knowledgebase-provider-secret-adapter/tests/provider_secret_adapter.rs",
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
  "tools/quality_evaluation_evidence.mjs",
  "specs/knowledge-engine-evaluation.spec.json",
  "docs/releases/provider-certification/quality-evaluation-evidence.schema.json",
  "docs/releases/provider-certification/quality-evaluation-evidence.template.json",
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

violations.push(...await validateKnowledgeEngineEvaluationWorkspace(root));

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
    && hostedBackendSource.includes("if !descriptor.native")
    && hostedBackendSource.includes("probe_active_bindings_health")
    && hostedBackendSource.includes("KnowledgeBackendRequestContext"),
  "provider health must probe native infrastructure directly and external Providers through authenticated active bindings",
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
const credentialPortSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_provider_credential_resolver.rs",
  ),
  "utf8",
);
const bindingStorePortSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/ports/knowledge_provider_binding_store.rs",
  ),
  "utf8",
);
const runtimeCredentialResolverSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-knowledgebase-provider-secret-adapter/src/resolver.rs",
  ),
  "utf8",
);
const runtimeCredentialResolverConfigSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-knowledgebase-provider-secret-adapter/src/config.rs",
  ),
  "utf8",
);
const runtimeCredentialResolverTestSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-knowledgebase-provider-secret-adapter/tests/provider_secret_adapter.rs",
  ),
  "utf8",
);
const providerBindingServiceSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/provider_binding.rs",
  ),
  "utf8",
);

assert(
  portsSource.includes("trait KnowledgeEngineSpaceRegistry"),
  "knowledge engine ports must declare KnowledgeEngineSpaceRegistry",
);
assert(
  /fn bind_provider\(\s*&self,\s*[^,]+,\s*[^:]+:\s*Option<KnowledgeEngineProviderCredential>,/s.test(portsSource),
  "KnowledgeEngine SPI v2 must bind persisted Provider bindings with a one-time resolved credential",
);
assert(
  credentialPortSource.includes("pub struct KnowledgeEngineProviderCredential")
    && credentialPortSource.includes("impl std::fmt::Debug for KnowledgeEngineProviderCredential")
    && credentialPortSource.includes("impl Drop for KnowledgeEngineProviderCredential")
    && credentialPortSource.includes("self.value.zeroize()")
    && /pub fn into_secret\([^)]*\) -> Zeroizing<String>/.test(credentialPortSource)
    && !credentialPortSource.includes("Serialize"),
  "resolved Provider credentials must be non-serializable, redacted, and zeroized on drop",
);
assert(
  bindingStorePortSource.includes("impl std::fmt::Debug for ResolvedKnowledgeEngineProviderCredential")
    && bindingStorePortSource.includes('.field("reference_locator", &"[REDACTED]")'),
  "resolved Provider credential references must redact their write-only locator",
);
assert(
  credentialPortSource.includes("pub struct KnowledgeEngineProviderCredentialAccessContext")
    && credentialPortSource.includes("pub tenant_id: u64")
    && credentialPortSource.includes("pub organization_id: u64")
    && credentialPortSource.includes("pub space_id: u64")
    && credentialPortSource.includes("pub binding_id: u64")
    && credentialPortSource.includes("pub credential_reference_id: u64")
    && credentialPortSource.includes("pub credential_reference_version: u64")
    && credentialPortSource.includes("pub implementation_id: String")
    && credentialPortSource.includes("pub actor_id: String")
    && credentialPortSource.includes("pub operation: KnowledgeEngineProviderOperation")
    && credentialPortSource.includes("pub trace_id: String")
    && credentialPortSource.includes("pub deadline_unix_ms: u64")
    && /fn validate_reference_locator\(\s*&self,\s*implementation_id: &str,\s*reference_locator: &str,/s.test(credentialPortSource)
    && /async fn resolve\(\s*&self,\s*context: &KnowledgeEngineProviderCredentialAccessContext,/s.test(credentialPortSource),
  "credential resolver port must require the complete immutable binding access context",
);
assert(
  runtimeCredentialResolverConfigSource.includes("SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRET_")
    && runtimeCredentialResolverSource.includes('strip_prefix("env://")')
    && runtimeCredentialResolverSource.includes('starts_with("file://")')
    && runtimeCredentialResolverSource.includes("secret://knowledgebase/provider/")
    && runtimeCredentialResolverSource.includes("tokio::fs::canonicalize")
    && runtimeCredentialResolverSource.includes("canonical_path.starts_with(&canonical_root)")
    && runtimeCredentialResolverSource.includes("metadata.is_file()")
    && runtimeCredentialResolverSource.includes("SecretProvider")
    && runtimeCredentialResolverSource.includes("spawn_blocking")
    && runtimeCredentialResolverSource.includes("Semaphore")
    && runtimeCredentialResolverSource.includes("concurrency.acquire_owned()")
    && runtimeCredentialResolverSource.includes("let _permit = permit")
    && runtimeCredentialResolverSource.includes("max_managed_resolution_duration")
    && runtimeCredentialResolverSource.includes("Zeroizing::new(value)")
    && runtimeCredentialResolverSource.includes("Zeroizing::new(Vec::with_capacity")
    && runtimeCredentialResolverSource.includes("credential_from_bounded_string")
    && runtimeCredentialResolverSource.includes("audit_record_id")
    && runtimeCredentialResolverSource.includes("provider_code(implementation_id)")
    && runtimeCredentialResolverSource.includes('security_event = "knowledge.provider_credential.access"'),
  "credential adapter must enforce namespaced local sources, canonical file containment, managed Secret Provider access, bounded results, audit evidence and sanitized telemetry",
);
assert(
  runtimeCredentialResolverTestSource.includes("local_environment_resolution_is_namespaced")
    && runtimeCredentialResolverTestSource.includes("credential_sources_are_bound_to_the_reference_implementation")
    && runtimeCredentialResolverTestSource.includes("local_file_resolution_rejects_symlink_escape")
    && runtimeCredentialResolverTestSource.includes("local_file_resolution_rejects_empty_non_utf8_and_oversized_values")
    && runtimeCredentialResolverTestSource.includes("staging_and_production_require_managed_sources")
    && runtimeCredentialResolverTestSource.includes("managed_concurrency_limit_is_bounded")
    && runtimeCredentialResolverTestSource.includes("managed_provider_errors_are_sanitized")
    && runtimeCredentialResolverTestSource.includes("managed_provider_result_size_is_bounded")
    && runtimeCredentialResolverTestSource.includes("managed_provider_access_wait_is_independently_bounded")
    && runtimeCredentialResolverTestSource.includes("timed_out_managed_calls_keep_the_bulkhead_permit_until_the_provider_returns")
    && runtimeCredentialResolverTestSource.includes("managed_provider_rotation_and_revocation_take_effect_without_cache"),
  "credential adapter must keep executable negative security, size, sanitization and no-cache rotation tests",
);
assert(
  providerBindingServiceSource.includes("probe_active_bindings_health")
    && providerBindingServiceSource.includes("buffer_unordered(MAX_CONCURRENT_PROVIDER_HEALTH_PROBES)")
    && providerBindingServiceSource.includes("remaining_deadline(context)")
    && providerBindingServiceSource.includes("KnowledgeEngineProviderBindingState::Active"),
  "aggregate external Provider health must be active-binding aware, deadline-bounded, and concurrency-bounded",
);
assert(
  spec.spiSurface.executionContextType === "KnowledgeEngineExecutionContext"
    && spec.spiSurface.resolvedHandleType === "KnowledgeEngineExecutionHandle"
    && spec.spiSurface.contextRequiredMethods?.includes("search")
    && spec.spiSurface.contextRequiredMethods?.includes("read_document")
    && spec.spiSurface.contextRequiredMethods?.includes("list_documents")
    && spec.spiSurface.externalContextRequiredMethods?.includes("sync_sources"),
  "machine SPI spec must declare the immutable execution context and binding-aware resolved handle",
);
assert(
  spec.credentialBoundary?.resolverPort === "KnowledgeEngineProviderCredentialResolver"
    && spec.credentialBoundary?.resolverComponent === "crates/sdkwork-knowledgebase-provider-secret-adapter"
    && spec.credentialBoundary?.resolvedSecretType === "KnowledgeEngineProviderCredential"
    && spec.credentialBoundary?.accessContextType === "KnowledgeEngineProviderCredentialAccessContext"
    && JSON.stringify(spec.credentialBoundary?.accessContextFields) === JSON.stringify(credentialAccessContextFields)
    && spec.credentialBoundary?.maxCredentialBytes === 65536
    && spec.credentialBoundary?.maxManagedResolutionMilliseconds === 5000
    && spec.credentialBoundary?.defaultMaxManagedConcurrency === 32
    && JSON.stringify(spec.credentialBoundary?.telemetryOutcomes) === JSON.stringify([
      "granted",
      "invalid_reference",
      "access_denied",
      "unavailable",
      "response_too_large",
      "internal",
    ])
    && spec.credentialBoundary?.managedProviderPort === "sdkwork_agent_kernel::SecretProvider"
    && spec.credentialBoundary?.supportedRuntimeSchemes?.some(
      (entry) => entry.scheme === "env://SDKWORK_KNOWLEDGEBASE_PROVIDER_SECRET_<PROVIDER_CODE>_*"
        && JSON.stringify(entry.environments) === JSON.stringify(["development", "test"]),
    )
    && spec.credentialBoundary?.supportedRuntimeSchemes?.some(
      (entry) => entry.scheme === "file://"
        && JSON.stringify(entry.environments) === JSON.stringify(["development", "test"]),
    )
    && spec.credentialBoundary?.supportedRuntimeSchemes?.some(
      (entry) => entry.scheme === "secret://knowledgebase/provider/..."
        && JSON.stringify(entry.environments) === JSON.stringify(["staging", "production"]),
    )
    && spec.credentialBoundary?.cachePolicy?.startsWith("No secret cache")
    && spec.credentialBoundary?.forbidden?.includes("startup credential reads in adapter config")
    && spec.credentialBoundary?.forbidden?.includes("credential file environment aliases")
    && spec.credentialBoundary?.forbidden?.includes("unrelated process environment variable lookup")
    && spec.credentialBoundary?.forbidden?.includes("credential locator for another Provider implementation")
    && spec.credentialBoundary?.forbidden?.includes("file resolution outside the approved canonical secret root")
    && spec.credentialBoundary?.forbidden?.includes("env:// or file:// in staging or production")
    && spec.credentialBoundary?.forbidden?.includes("unbounded managed SecretProvider concurrency")
    && spec.credentialBoundary?.forbidden?.includes("credential lookup before authenticated scope validation"),
  "machine SPI spec must declare the accepted binding-scoped credential boundary and its forbidden legacy paths",
);
assert(
  /async fn sync_sources\(\s*&self,\s*context: &KnowledgeEngineExecutionContext,/s.test(portsSource),
  "ExternalKnowledgeEngine sync_sources must require an explicit immutable execution context",
);
assert(
  /async fn search\(\s*&self,\s*context: &KnowledgeEngineExecutionContext,/s.test(portsSource)
    && /async fn read_document\(\s*&self,\s*context: &KnowledgeEngineExecutionContext,/s.test(portsSource)
    && /async fn list_documents\(\s*&self,\s*context: &KnowledgeEngineExecutionContext,/s.test(portsSource),
  "KnowledgeEngine search/read/list must require an explicit immutable execution context",
);
assert(
  /Result<crate::knowledge_engine::KnowledgeEngineExecutionHandle, KnowledgeEngineError>/.test(portsSource),
  "KnowledgeEngineSpaceRegistry must return KnowledgeEngineExecutionHandle",
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
const executionHandleSource = await readFile(
  path.join(
    root,
    "crates/sdkwork-intelligence-knowledgebase-service/src/knowledge_engine/execution_handle.rs",
  ),
  "utf8",
);
assert(
  spaceResolverSource.includes("resolve_for_space"),
  "KnowledgeEngineSpaceResolver must implement per-space engine resolution",
);
assert(
  spaceResolverSource.includes("Result<KnowledgeEngineExecutionHandle, KnowledgeEngineError>")
    && spaceResolverSource.includes("KnowledgeEngineExecutionHandle::external")
    && spaceResolverSource.includes("KnowledgeEngineExecutionHandle::native"),
  "space resolution must return a binding-aware execution handle for native and external engines",
);
assert(
  executionHandleSource.includes("fn scoped_context")
    && executionHandleSource.includes("permission_scope")
    && executionHandleSource.includes("allowed_space_ids")
    && executionHandleSource.includes("deadline_unix_ms")
    && executionHandleSource.includes("scoped.binding_id = Some(binding.id)"),
  "execution handle must validate scope and inject the resolved binding before engine execution",
);
const searchMethodStart = executionHandleSource.indexOf("pub async fn search(");
const searchMethodEnd = executionHandleSource.indexOf("pub async fn read_document(", searchMethodStart);
const searchMethodBody = executionHandleSource.slice(searchMethodStart, searchMethodEnd);
assert(
  searchMethodBody.indexOf("self.scoped_context(") >= 0
    && searchMethodBody.indexOf(".engine_for_operation(") > searchMethodBody.indexOf("self.scoped_context(")
    && searchMethodBody.indexOf("engine.search(&context, request).await")
      > searchMethodBody.indexOf(".engine_for_operation("),
  "search must validate authenticated scope before binding or executing a Provider",
);
const operationMethodStart = executionHandleSource.indexOf("async fn engine_for_operation(");
const operationMethodEnd = executionHandleSource.indexOf("fn scoped_context(", operationMethodStart);
const operationMethodBody = executionHandleSource.slice(operationMethodStart, operationMethodEnd);
assert(
  operationMethodBody.indexOf("capability_snapshot.contains") >= 0
    && operationMethodBody.indexOf(".resolve_credential_reference(")
      > operationMethodBody.indexOf("capability_snapshot.contains")
    && operationMethodBody.indexOf("KnowledgeEngineProviderCredentialAccessContext::for_binding")
      > operationMethodBody.indexOf(".resolve_credential_reference(")
    && operationMethodBody.indexOf(".bind_provider(binding, credential)")
      > operationMethodBody.indexOf("KnowledgeEngineProviderCredentialAccessContext::for_binding"),
  "execution handle must check the tested capability, resolve the credential reference, construct binding access context, resolve the secret, then bind the Provider",
);
assert(
  !spaceResolverSource.includes("bind_provider("),
  "space resolution must return an unbound handle and must not resolve Provider credentials before operation authorization",
);
assert(
  runtimeSource.includes("KnowledgebaseProviderCredentialResolver")
    && runtimeSource.includes("connect_with_provider_credential_resolver")
    && runtimeSource.includes("staging and production require an injected managed Knowledgebase Provider credential resolver")
    && runtimeSource.includes("credential_resolver"),
  "KnowledgebaseRuntime must inject the Provider credential resolver and fail closed on unmanaged production defaults",
);

for (const adapterCrate of providerAdapterCrates) {
  const adapterRoot = path.join(root, `crates/sdkwork-knowledgebase-engine-${adapterCrate}/src`);
  const adapterSource = await readFile(path.join(adapterRoot, "lib.rs"), "utf8");
  const configSource = await readFile(path.join(adapterRoot, "config.rs"), "utf8");
  assert(
    /fn bind_provider\(\s*&self,\s*[^,]+,\s*credential:\s*Option<KnowledgeEngineProviderCredential>,/s.test(adapterSource),
    `${adapterCrate} adapter must consume the one-time resolved Provider credential in bind_provider`,
  );
  assert(
    !configSource.includes("read_credential")
      && !configSource.includes("read_to_string")
      && !/std::env::var\([^)]*CREDENTIAL/.test(configSource),
    `${adapterCrate} adapter config must not read Provider credentials during startup`,
  );
  assert(
    configSource.includes("Zeroizing<String>")
      && !/#\[derive\([^\]]*Debug[^\]]*\)\]\s*pub struct \w+ConnectorConfig/s.test(configSource),
    `${adapterCrate} adapter config must zeroize held credentials and must not derive printable Debug`,
  );
  assert(
    !configSource.includes("_CREDENTIAL_FILE_ENV"),
    `${adapterCrate} adapter must not retain the retired credential-file environment alias`,
  );
}
const providerRuntimeSource = await readFile(
  path.join(root, "crates/sdkwork-knowledgebase-provider-runtime/src/runtime.rs"),
  "utf8",
);
assert(
  providerRuntimeSource.includes("pub fn from_knowledge_engine_request(")
    && providerRuntimeSource.includes("request_tenant_id != context.tenant_id")
    && providerRuntimeSource.includes("request_space_id != context.space_id"),
  "Provider Runtime must revalidate request tenant and space before outbound execution",
);
assert(
  runtimeSource.includes("fn knowledge_engine_execution_context(")
    && runtimeSource.includes("actor_id: actor_id.to_string()")
    && runtimeSource.includes("trace_id,")
    && runtimeSource.includes("deadline_unix_ms,")
    && runtimeSource.includes("binding_id: None"),
  "App API runtime must create bounded request-derived knowledge execution contexts and leave binding selection to the resolver",
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
  (hostedRoutesSource.match(/activate_provider_binding\(/g) ?? []).length >= 30
    && hostedRoutesSource.includes(".create_binding(")
    && hostedRoutesSource.includes(".begin_binding_test(")
    && hostedRoutesSource.includes(".record_binding_test_result(")
    && hostedRoutesSource.includes(".activate_binding(")
    && !hostedRoutesSource.includes("hosted_backend_resolves_external_space"),
  "hosted Provider fixtures must create tested active bindings explicitly and must not treat connector sources as resolution authority",
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
