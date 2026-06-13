//! Agent provider adapter for SDKWork Knowledgebase.

pub mod client;
mod mapper;
pub mod provider;

pub use client::KnowledgebaseRetrievalClient;
pub use provider::{SdkworkKnowledgebaseProvider, SDKWORK_KNOWLEDGEBASE_PROVIDER_ID};
