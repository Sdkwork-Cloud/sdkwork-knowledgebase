//! Drive adapter for SDKWork Knowledgebase.

mod access_control_adapter;
mod adapter;
mod bootstrap;
mod embedded_event_relay;
mod embedded_wiki_source_adapter;
mod internal_sdk_adapter;
mod permission_adapter;
mod wiki_scope_adapter;

pub use access_control_adapter::KnowledgebaseKnowledgeAccessControlAdapter;
pub use adapter::{
    KnowledgebaseDriveNodeTreeAdapter, KnowledgebaseDriveSpaceProvisionerAdapter,
    KnowledgebaseDriveStorageAdapter, KnowledgebaseDriveWorkspaceAdapter,
};
pub use bootstrap::{connect_knowledgebase_drive_pool, knowledgebase_drive_health_check};
pub use embedded_event_relay::{
    embedded_knowledgebase_raw_channel_id, KnowledgebaseDriveEmbeddedEventRelay,
};
pub use embedded_wiki_source_adapter::KnowledgebaseDriveEmbeddedWikiSourceAdapter;
pub use internal_sdk_adapter::{
    KnowledgebaseDriveEventDeliveryConfig, KnowledgebaseDriveInternalSdkAdapter,
};
pub use permission_adapter::KnowledgebaseDrivePermissionAdapter;
pub use wiki_scope_adapter::KnowledgebaseDriveRootScopeAdapter;
