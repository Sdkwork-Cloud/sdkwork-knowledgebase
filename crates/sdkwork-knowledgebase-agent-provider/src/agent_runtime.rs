//! Builds sdkwork-agent-kernel runtimes for registered knowledge agent implementations.
//!
//! Register new implementations by:
//! 1. Adding a `plugin.intelligence.*` id to `known_agent_implementation_ids()` in contract.
//! 2. Extending `configure_agent_implementation()` and `chat_agent_manifest()` below.
//! 3. Persisting the id on `kb_agent_profile.agent_implementation_id` (default: Rig).

use crate::agent_implementation::{
    validate_registered_agent_implementation, CONTRACT_MODEL_PROVIDER_ID,
};
use crate::{
    ClawRouterChatModelProvider, KnowledgebaseRetrievalClient, OkfKnowledgeClient,
    OkfKnowledgeProvider, SdkworkKnowledgebaseProvider, SpaceEngineKnowledgeProvider,
    OKF_KNOWLEDGE_PROVIDER_ID, SDKWORK_KNOWLEDGEBASE_PROVIDER_ID,
};
use sdkwork_agent_kernel::{
    AgentManifest, AgentRuntime, KernelError, ModelProvider, ModelRequest, ModelResponse,
    PolicyDecision, PolicyProvider, PolicyRequest, ProviderHealth, ProviderManifest,
    RuntimeBuilder,
};
use sdkwork_agent_plugin_core::SdkworkKernelPlugin;
use sdkwork_agent_provider_rig::{ids as rig_ids, rig_agent_manifest, RigKernelPlugin};
use sdkwork_knowledgebase_contract::rag::KnowledgeAgentKnowledgeMode;
use sdkwork_knowledgebase_contract::{
    KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID, RIG_AGENT_IMPLEMENTATION_ID,
};
use std::sync::Arc;

use crate::knowledge_access::SpaceKnowledgeEngineClient;

pub const CLAW_ROUTER_OPEN_HTTP_URL_ENV: &str = crate::claw_router::CLAW_ROUTER_OPEN_HTTP_URL_ENV;

pub struct KnowledgeAgentRuntimeBuildRequest<R, W> {
    pub agent_implementation_id: String,
    pub model_provider_id: String,
    pub mode: KnowledgeAgentKnowledgeMode,
    pub retrieval_client: R,
    pub okf_client: W,
    pub tenant_id: u64,
    pub claw_router_client: Option<Arc<clawrouter_open_sdk::SdkworkAiClient>>,
    pub space_engine_client: Option<Arc<dyn SpaceKnowledgeEngineClient>>,
    pub external_knowledge_provider_ids: Vec<String>,
}

pub fn build_knowledge_agent_runtime<R, W>(
    request: KnowledgeAgentRuntimeBuildRequest<R, W>,
) -> Result<AgentRuntime, String>
where
    R: KnowledgebaseRetrievalClient + Send + Sync + 'static,
    W: OkfKnowledgeClient + Send + Sync + 'static,
{
    validate_registered_agent_implementation(&request.agent_implementation_id)?;

    let manifest = chat_agent_manifest(&request.agent_implementation_id)?;
    let mut builder = RuntimeBuilder::new("runtime.knowledgebase.chat", manifest);

    builder = configure_agent_implementation(
        builder,
        &request.agent_implementation_id,
        &request.model_provider_id,
        request.claw_router_client,
    )?;

    let _ = request.mode;

    builder = builder
        .register_knowledge_provider(
            SDKWORK_KNOWLEDGEBASE_PROVIDER_ID,
            "0.1.0",
            SdkworkKnowledgebaseProvider::new(request.retrieval_client, request.tenant_id),
        )
        .register_knowledge_provider(
            OKF_KNOWLEDGE_PROVIDER_ID,
            "0.1.0",
            OkfKnowledgeProvider::new(request.okf_client),
        );

    if !request.external_knowledge_provider_ids.is_empty() {
        let space_engine_client = request.space_engine_client.ok_or_else(|| {
            "external knowledge provider registration requires space engine client wiring"
                .to_string()
        })?;
        for provider_id in request.external_knowledge_provider_ids {
            builder = builder.register_knowledge_provider(
                provider_id.as_str(),
                "0.1.0",
                SpaceEngineKnowledgeProvider::new(
                    provider_id.clone(),
                    space_engine_client.clone(),
                    request.tenant_id,
                ),
            );
        }
    }

    builder
        .bootstrap()
        .map(|bootstrapped| bootstrapped.runtime)
        .map_err(|error| format!("agent runtime bootstrap failed: {error}"))
}

fn configure_agent_implementation(
    builder: RuntimeBuilder,
    agent_implementation_id: &str,
    model_provider_id: &str,
    claw_router_client: Option<Arc<clawrouter_open_sdk::SdkworkAiClient>>,
) -> Result<RuntimeBuilder, String> {
    match agent_implementation_id {
        KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID => {
            if model_provider_id != CONTRACT_MODEL_PROVIDER_ID {
                return Err(format!(
                    "agent implementation {KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID} requires model provider {CONTRACT_MODEL_PROVIDER_ID}"
                ));
            }
            Ok(builder
                .register_model_provider(
                    CONTRACT_MODEL_PROVIDER_ID,
                    "0.1.0",
                    ContractKnowledgeChatModelProvider,
                )
                .register_policy_provider("provider.policy.allow", "0.1.0", AllowPolicyProvider))
        }
        RIG_AGENT_IMPLEMENTATION_ID => {
            let client = claw_router_client.ok_or_else(|| {
                format!(
                    "Rig agent implementation requires claw-router SdkworkAiClient ({CLAW_ROUTER_OPEN_HTTP_URL_ENV})"
                )
            })?;
            Ok(builder
                .register_model_provider(
                    rig_ids::MODEL_PROVIDER_ID,
                    "0.1.0",
                    ClawRouterChatModelProvider::for_rig(client),
                )
                .pipe(|configured| RigKernelPlugin::fail_closed().configure_runtime(configured)))
        }
        other => Err(format!(
            "no runtime builder registered for agent_implementation_id: {other}"
        )),
    }
}

fn chat_agent_manifest(agent_implementation_id: &str) -> Result<AgentManifest, String> {
    match agent_implementation_id {
        KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID => Ok(knowledgebase_chat_agent_manifest()),
        RIG_AGENT_IMPLEMENTATION_ID => Ok(rig_agent_manifest()),
        other => Err(format!(
            "no agent manifest registered for agent_implementation_id: {other}"
        )),
    }
}

fn knowledgebase_chat_agent_manifest() -> AgentManifest {
    AgentManifest::from_json(
        r#"
{
  "schema_version": "0.1.0",
  "manifest_type": "agent",
  "agent_id": "agent.knowledgebase.chat",
  "name": "knowledgebase-chat",
  "display_name": "Knowledgebase Chat",
  "description": "Knowledge-backed chat agent for SDKWork Knowledgebase.",
  "version": "0.1.0",
  "domain": "intelligence",
  "required_capabilities": [
    { "capability_id": "model.chat", "min_version": "0.1.0" },
    { "capability_id": "policy.evaluate", "min_version": "0.1.0" }
  ],
  "optional_capabilities": [
    { "capability_id": "knowledge.search", "min_version": "0.1.0" }
  ],
  "event_families": ["agent.model.*", "agent.knowledge.*"],
  "owner": { "name": "sdkwork-platform" },
  "status": "candidate"
}
"#,
    )
    .expect("knowledgebase chat manifest parses")
}

trait RuntimeBuilderPipe {
    fn pipe(self, configure: impl FnOnce(Self) -> Self) -> Self
    where
        Self: Sized;
}

impl RuntimeBuilderPipe for RuntimeBuilder {
    fn pipe(self, configure: impl FnOnce(Self) -> Self) -> Self {
        configure(self)
    }
}

#[derive(Debug, Clone)]
struct AllowPolicyProvider;

impl PolicyProvider for AllowPolicyProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            "provider.policy.allow",
            "policy",
            "allow-policy",
            "0.1.0",
            vec!["policy.evaluate".to_string()],
        )
    }

    fn evaluate(&self, request: PolicyRequest) -> Result<PolicyDecision, KernelError> {
        Ok(PolicyDecision::allow(
            format!("decision.{}", request.policy_request_id),
            request.policy_request_id,
            "provider.policy.allow",
        ))
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }
}

#[derive(Debug, Clone)]
struct ContractKnowledgeChatModelProvider;

impl ModelProvider for ContractKnowledgeChatModelProvider {
    fn provider_manifest(&self) -> ProviderManifest {
        ProviderManifest::new(
            CONTRACT_MODEL_PROVIDER_ID,
            "model",
            "knowledgebase-contract",
            "0.1.0",
            vec!["model.chat".to_string()],
        )
    }

    fn health(&self) -> ProviderHealth {
        ProviderHealth::available()
    }

    fn invoke(&self, request: ModelRequest) -> Result<ModelResponse, KernelError> {
        let context_titles = request
            .context_frames
            .iter()
            .filter_map(|frame| frame.metadata_value("sdkwork.knowledge.title"))
            .collect::<Vec<_>>();

        let answer = if context_titles.is_empty() {
            format!(
                "No knowledge context was attached for this question: {}",
                request.messages.join(" ")
            )
        } else {
            format!(
                "Based on {} knowledge source(s) [{}]: {}",
                context_titles.len(),
                context_titles.join(", "),
                request.messages.join(" ")
            )
        };

        Ok(ModelResponse::text(
            request.model_request_id,
            CONTRACT_MODEL_PROVIDER_ID,
            answer,
        ))
    }
}
