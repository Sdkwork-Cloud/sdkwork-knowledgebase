//! Configurable knowledge agent runtime implementation identifiers.

/// Default production agent runtime: Rig kernel plugin (`sdkwork-agent-plugin-rig`).
pub const RIG_AGENT_IMPLEMENTATION_ID: &str = "plugin.intelligence.rig";

/// Offline / contract-test agent runtime without external LLM dependencies.
pub const KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID: &str =
    "plugin.intelligence.knowledgebase-contract";

pub fn default_agent_implementation_id() -> String {
    RIG_AGENT_IMPLEMENTATION_ID.to_string()
}

/// Resolve implementation from chat request override, then profile, then Rig default.
pub fn resolve_agent_implementation_id(
    request_override: Option<&str>,
    profile_value: &str,
) -> String {
    request_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            let trimmed = profile_value.trim();
            if trimmed.is_empty() {
                default_agent_implementation_id()
            } else {
                trimmed.to_string()
            }
        })
}

pub fn known_agent_implementation_ids() -> &'static [&'static str] {
    &[
        RIG_AGENT_IMPLEMENTATION_ID,
        KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_prefers_request_override() {
        assert_eq!(
            resolve_agent_implementation_id(
                Some(KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID),
                RIG_AGENT_IMPLEMENTATION_ID
            ),
            KNOWLEDGEBASE_CONTRACT_AGENT_IMPLEMENTATION_ID
        );
    }

    #[test]
    fn resolve_falls_back_to_profile_then_default() {
        assert_eq!(
            resolve_agent_implementation_id(None, RIG_AGENT_IMPLEMENTATION_ID),
            RIG_AGENT_IMPLEMENTATION_ID
        );
        assert_eq!(
            resolve_agent_implementation_id(None, ""),
            default_agent_implementation_id()
        );
    }
}
