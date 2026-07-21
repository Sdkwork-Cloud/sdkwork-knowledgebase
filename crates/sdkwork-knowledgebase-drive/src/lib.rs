//! Drive adapter for SDKWork Knowledgebase.

mod access_control_adapter;
mod adapter;
mod bootstrap;
mod permission_adapter;
mod site_artifact_adapter;

pub use access_control_adapter::KnowledgebaseKnowledgeAccessControlAdapter;
pub use adapter::{
    KnowledgebaseDriveNodeTreeAdapter, KnowledgebaseDriveSpaceProvisionerAdapter,
    KnowledgebaseDriveStorageAdapter, KnowledgebaseDriveWorkspaceAdapter,
};
pub use bootstrap::{connect_knowledgebase_drive_pool, knowledgebase_drive_health_check};
pub use permission_adapter::KnowledgebaseDrivePermissionAdapter;
pub use site_artifact_adapter::KnowledgebaseDriveSiteArtifactStore;
