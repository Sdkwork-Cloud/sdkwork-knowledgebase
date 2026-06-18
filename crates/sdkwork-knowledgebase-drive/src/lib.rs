//! Drive adapter for SDKWork Knowledgebase.

mod access_control_adapter;
mod adapter;
mod bootstrap;
mod permission_adapter;

pub use access_control_adapter::KnowledgebaseKnowledgeAccessControlAdapter;
pub use adapter::{
    KnowledgebaseDriveNodeTreeAdapter, KnowledgebaseDriveSpaceProvisionerAdapter,
    KnowledgebaseDriveStorageAdapter, KnowledgebaseDriveWorkspaceAdapter,
};
pub use bootstrap::{connect_sqlite_drive_pool, sqlite_drive_health_check};
pub use permission_adapter::KnowledgebaseDrivePermissionAdapter;
