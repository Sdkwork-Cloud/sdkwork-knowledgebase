//! Secure credential resolution for external Knowledgebase Provider bindings.

mod config;
mod resolver;

pub use config::{
    KnowledgebaseProviderCredentialEnvironment, KnowledgebaseProviderCredentialResolverConfig,
    KnowledgebaseProviderCredentialResolverConfigurationError,
    KNOWLEDGEBASE_PROVIDER_SECRET_ENV_PREFIX,
};
pub use resolver::KnowledgebaseProviderCredentialResolver;
