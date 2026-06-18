//! Registry and validation for configurable knowledge agent runtime implementations.

use sdkwork_knowledgebase_contract::{
    default_agent_implementation_id, known_agent_implementation_ids,
    KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID, RIG_AGENT_IMPLEMENTATION_ID,
};

pub const CONTRACT_MODEL_PROVIDER_ID: &str = "provider.model.knowledgebase-contract";

pub fn validate_registered_agent_implementation(agent_implementation_id: &str) -> Result<(), String> {
    let trimmed = agent_implementation_id.trim();
    if trimmed.is_empty() {
        return Err("agent_implementation_id must not be empty".to_string());
    }
    if !trimmed.starts_with("plugin.") {
        return Err(format!(
            "agent_implementation_id must use plugin.* namespace: {trimmed}"
        ));
    }
    if !known_agent_implementation_ids()
        .iter()
        .any(|known| *known == trimmed)
    {
        return Err(format!(
            "unsupported agent_implementation_id: {trimmed}; known implementations: {}",
            known_agent_implementation_ids().join(", ")
        ));
    }
    Ok(())
}

pub fn resolve_model_provider_for_implementation(
    agent_implementation_id: &str,
    model_provider_id: &str,
) -> String {
    if agent_implementation_id == KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID {
        return CONTRACT_MODEL_PROVIDER_ID.to_string();
    }
    resolve_rig_model_provider_id(model_provider_id)
}

pub fn resolve_rig_model_provider_id(model_provider_id: &str) -> String {
    if model_provider_id == CONTRACT_MODEL_PROVIDER_ID {
        return model_provider_id.to_string();
    }
    if crate::claw_router::is_rig_model_provider(model_provider_id) {
        return crate::claw_router::RIG_MODEL_PROVIDER_ID.to_string();
    }
    model_provider_id.to_string()
}

pub fn default_profile_agent_implementation_id() -> String {
    default_agent_implementation_id()
}

pub fn is_rig_agent_implementation(agent_implementation_id: &str) -> bool {
    agent_implementation_id == RIG_AGENT_IMPLEMENTATION_ID
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_implementation_forces_contract_model_provider() {
        assert_eq!(
            resolve_model_provider_for_implementation(
                KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID,
                "provider.model.openai"
            ),
            CONTRACT_MODEL_PROVIDER_ID
        );
    }

    #[test]
    fn rejects_unknown_implementation() {
        assert!(validate_registered_agent_implementation("plugin.intelligence.unknown").is_err());
    }
}
