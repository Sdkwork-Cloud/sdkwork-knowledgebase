//! Internal RPC adapter for the IM-owned group knowledge-space lifecycle.
//!
//! The crate deliberately contains only protocol adaptation. Runtime composition belongs to the
//! sibling `*-rpc-bin` process crate, while lifecycle policy stays in the Knowledgebase service.

#![forbid(unsafe_code)]

mod context;
mod error;
mod mapper;
mod service;

pub mod runtime;

pub use runtime::GroupKnowledgeSpaceLifecycleRuntime;
pub use service::GroupKnowledgeSpaceLifecycleRpcService;
