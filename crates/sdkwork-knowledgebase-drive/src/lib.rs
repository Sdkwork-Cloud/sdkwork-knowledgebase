//! Drive adapter for SDKWork Knowledgebase.

mod adapter;
mod permission_adapter;

pub use adapter::{
    KnowledgebaseDriveNodeTreeAdapter, KnowledgebaseDriveSpaceProvisionerAdapter,
    KnowledgebaseDriveStorageAdapter, KnowledgebaseDriveWorkspaceAdapter,
};
pub use permission_adapter::KnowledgebaseDrivePermissionAdapter;
