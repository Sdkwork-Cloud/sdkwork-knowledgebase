//! Process bootstrap for the Knowledgebase internal group lifecycle RPC surface.

#![forbid(unsafe_code)]

pub mod bootstrap;
pub mod config;
pub mod runtime;

pub use bootstrap::run_group_knowledge_space_lifecycle_rpc_from_env;
