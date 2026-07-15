use std::{
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use sdkwork_database_config::DatabaseEngine;
use sdkwork_drive_storage_local::LocalDriveObjectStore;
use sdkwork_intelligence_knowledgebase_repository_sqlx::{
    connect_knowledgebase_and_install_schema, is_postgres_database_url, knowledgebase_health_check,
    SqliteGroupKnowledgeSpaceBindingStore, SqliteKnowledgeOkfBundleFileStore,
    SqliteKnowledgeSpaceStore,
};
use sdkwork_intelligence_knowledgebase_rpc::GroupKnowledgeSpaceLifecycleRuntime;
use sdkwork_intelligence_knowledgebase_service::{
    group_space::{
        GroupKnowledgeSpaceOperation, KnowledgeGroupKnowledgeSpaceService,
        KnowledgeGroupKnowledgeSpaceServiceError,
    },
    okf::{OkfBundleFileRegistryService, OkfBundleInitializerService},
    ports::knowledge_group_space_binding_store::{
        GroupKnowledgeSpaceMembershipChange, GroupKnowledgeSpaceScope,
    },
};
use sdkwork_knowledgebase_contract::group_space::{
    ArchiveGroupKnowledgeSpaceRequest, EnsureGroupKnowledgeSpaceRequest,
    GroupKnowledgeSpaceBinding, SynchronizeGroupKnowledgeSpaceMembersRequest,
};
use sdkwork_knowledgebase_drive::{
    connect_knowledgebase_drive_pool, knowledgebase_drive_health_check,
    KnowledgebaseDriveSpaceProvisionerAdapter, KnowledgebaseDriveStorageAdapter,
    KnowledgebaseDriveWorkspaceAdapter, KnowledgebaseKnowledgeAccessControlAdapter,
};
use thiserror::Error;

const DRIVE_PROVIDER_ID: &str = "sdkwork-knowledgebase-local";
const DRIVE_BUCKET: &str = "knowledgebase";

/// Concrete, process-owned lifecycle runtime. It creates tenant/organization-scoped adapters for
/// each verified command, so no caller scope can leak through a long-lived mutable runtime.
#[derive(Clone)]
pub struct KnowledgebaseGroupKnowledgeSpaceLifecycleRuntime {
    pool: sqlx::AnyPool,
    drive_pool: sqlx::AnyPool,
    database_engine: DatabaseEngine,
    object_store: Arc<LocalDriveObjectStore>,
    operator_id: String,
}

impl KnowledgebaseGroupKnowledgeSpaceLifecycleRuntime {
    pub async fn connect(
        database_url: &str,
        drive_storage_root: PathBuf,
        operator_id: String,
    ) -> Result<Self, KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError> {
        verify_drive_storage_root(&drive_storage_root)?;
        let pool = connect_knowledgebase_and_install_schema(database_url)
            .await
            .map_err(|_| {
                KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable
            })?;
        let drive_pool = connect_knowledgebase_drive_pool(database_url)
            .await
            .map_err(|_| {
                KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable
            })?;
        knowledgebase_health_check(&pool).await.map_err(|_| {
            KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable
        })?;
        knowledgebase_drive_health_check(&drive_pool)
            .await
            .map_err(|_| {
                KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable
            })?;

        Ok(Self {
            pool,
            drive_pool,
            database_engine: if is_postgres_database_url(database_url) {
                DatabaseEngine::Postgres
            } else {
                DatabaseEngine::Sqlite
            },
            object_store: Arc::new(LocalDriveObjectStore::new(drive_storage_root)),
            operator_id,
        })
    }

    pub async fn readiness_check(
        &self,
    ) -> Result<(), KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError> {
        knowledgebase_health_check(&self.pool).await.map_err(|_| {
            KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable
        })?;
        knowledgebase_drive_health_check(&self.drive_pool)
            .await
            .map_err(|_| {
                KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable
            })
    }

    fn dependencies_for_scope(
        &self,
        scope: GroupKnowledgeSpaceScope,
    ) -> GroupKnowledgeSpaceLifecycleDependencies {
        let tenant_id = scope.tenant_id.to_string();
        GroupKnowledgeSpaceLifecycleDependencies {
            binding_store: Arc::new(
                SqliteGroupKnowledgeSpaceBindingStore::new(self.pool.clone())
                    .with_database_engine(self.database_engine),
            ),
            space_store: Arc::new(
                SqliteKnowledgeSpaceStore::new(
                    self.pool.clone(),
                    scope.tenant_id,
                    scope.organization_id,
                )
                .with_database_engine(self.database_engine),
            ),
            bundle_file_store: Arc::new(
                SqliteKnowledgeOkfBundleFileStore::new(self.pool.clone(), scope.tenant_id)
                    .with_database_engine(self.database_engine),
            ),
            drive_storage: Arc::new(KnowledgebaseDriveStorageAdapter::new(
                self.object_store.clone(),
                DRIVE_PROVIDER_ID,
                DRIVE_BUCKET,
                tenant_id.clone(),
            )),
            drive_space_provisioner: Arc::new(KnowledgebaseDriveSpaceProvisionerAdapter::new(
                self.drive_pool.clone(),
            )),
            drive_workspace: Arc::new(KnowledgebaseDriveWorkspaceAdapter::new(
                self.drive_pool.clone(),
                tenant_id,
                self.operator_id.clone(),
            )),
            access_control: Arc::new(KnowledgebaseKnowledgeAccessControlAdapter::new(
                self.drive_pool.clone(),
            )),
        }
    }

    async fn ensure_from_im(
        &self,
        scope: GroupKnowledgeSpaceScope,
        service_actor_id: &str,
        request: EnsureGroupKnowledgeSpaceRequest,
    ) -> Result<GroupKnowledgeSpaceOperation, KnowledgeGroupKnowledgeSpaceServiceError> {
        let dependencies = self.dependencies_for_scope(scope);
        let registry = OkfBundleFileRegistryService::new(dependencies.bundle_file_store.as_ref());
        let initializer = OkfBundleInitializerService::new(dependencies.drive_storage.as_ref())
            .with_registry(&registry)
            .with_drive_workspace(dependencies.drive_workspace.as_ref());
        let service = KnowledgeGroupKnowledgeSpaceService::new(
            dependencies.binding_store.as_ref(),
            dependencies.space_store.as_ref(),
            &initializer,
            dependencies.drive_space_provisioner.as_ref(),
            dependencies.access_control.as_ref(),
            self.operator_id.clone(),
        );
        service
            .ensure_from_im(scope, service_actor_id, request)
            .await
    }

    async fn synchronize_from_im(
        &self,
        scope: GroupKnowledgeSpaceScope,
        service_actor_id: &str,
        request: SynchronizeGroupKnowledgeSpaceMembersRequest,
    ) -> Result<GroupKnowledgeSpaceMembershipChange, KnowledgeGroupKnowledgeSpaceServiceError> {
        let dependencies = self.dependencies_for_scope(scope);
        let registry = OkfBundleFileRegistryService::new(dependencies.bundle_file_store.as_ref());
        let initializer = OkfBundleInitializerService::new(dependencies.drive_storage.as_ref())
            .with_registry(&registry)
            .with_drive_workspace(dependencies.drive_workspace.as_ref());
        let service = KnowledgeGroupKnowledgeSpaceService::new(
            dependencies.binding_store.as_ref(),
            dependencies.space_store.as_ref(),
            &initializer,
            dependencies.drive_space_provisioner.as_ref(),
            dependencies.access_control.as_ref(),
            self.operator_id.clone(),
        );
        service
            .synchronize_members_from_im(scope, service_actor_id, request)
            .await
    }

    async fn archive_from_im(
        &self,
        scope: GroupKnowledgeSpaceScope,
        service_actor_id: &str,
        archived_by: &str,
        request: ArchiveGroupKnowledgeSpaceRequest,
    ) -> Result<GroupKnowledgeSpaceBinding, KnowledgeGroupKnowledgeSpaceServiceError> {
        let dependencies = self.dependencies_for_scope(scope);
        let registry = OkfBundleFileRegistryService::new(dependencies.bundle_file_store.as_ref());
        let initializer = OkfBundleInitializerService::new(dependencies.drive_storage.as_ref())
            .with_registry(&registry)
            .with_drive_workspace(dependencies.drive_workspace.as_ref());
        let service = KnowledgeGroupKnowledgeSpaceService::new(
            dependencies.binding_store.as_ref(),
            dependencies.space_store.as_ref(),
            &initializer,
            dependencies.drive_space_provisioner.as_ref(),
            dependencies.access_control.as_ref(),
            self.operator_id.clone(),
        );
        service
            .archive_from_im(scope, service_actor_id, archived_by, request)
            .await
    }
}

#[async_trait]
impl GroupKnowledgeSpaceLifecycleRuntime for KnowledgebaseGroupKnowledgeSpaceLifecycleRuntime {
    async fn ensure_group_knowledge_space(
        &self,
        scope: GroupKnowledgeSpaceScope,
        actor_id: &str,
        request: EnsureGroupKnowledgeSpaceRequest,
    ) -> Result<GroupKnowledgeSpaceOperation, KnowledgeGroupKnowledgeSpaceServiceError> {
        self.ensure_from_im(scope, actor_id, request).await
    }

    async fn synchronize_group_knowledge_space_members(
        &self,
        scope: GroupKnowledgeSpaceScope,
        actor_id: &str,
        request: SynchronizeGroupKnowledgeSpaceMembersRequest,
    ) -> Result<GroupKnowledgeSpaceMembershipChange, KnowledgeGroupKnowledgeSpaceServiceError> {
        self.synchronize_from_im(scope, actor_id, request).await
    }

    async fn archive_group_knowledge_space(
        &self,
        scope: GroupKnowledgeSpaceScope,
        service_actor_id: &str,
        archived_by: &str,
        request: ArchiveGroupKnowledgeSpaceRequest,
    ) -> Result<GroupKnowledgeSpaceBinding, KnowledgeGroupKnowledgeSpaceServiceError> {
        self.archive_from_im(scope, service_actor_id, archived_by, request)
            .await
    }
}

struct GroupKnowledgeSpaceLifecycleDependencies {
    binding_store: Arc<SqliteGroupKnowledgeSpaceBindingStore>,
    space_store: Arc<SqliteKnowledgeSpaceStore>,
    bundle_file_store: Arc<SqliteKnowledgeOkfBundleFileStore>,
    drive_storage: Arc<KnowledgebaseDriveStorageAdapter>,
    drive_space_provisioner: Arc<KnowledgebaseDriveSpaceProvisionerAdapter>,
    drive_workspace: Arc<KnowledgebaseDriveWorkspaceAdapter>,
    access_control: Arc<KnowledgebaseKnowledgeAccessControlAdapter>,
}

#[derive(Debug, Error)]
pub enum KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError {
    #[error("Knowledgebase lifecycle runtime dependency is unavailable")]
    DependencyUnavailable,
}

/// Creates the configured root when needed and verifies that it is a directory usable for Drive
/// object writes before the RPC listener accepts lifecycle commands. The probe is unique, read
/// back, and removed immediately; no Knowledgebase data is created or modified.
fn verify_drive_storage_root(
    drive_storage_root: &Path,
) -> Result<(), KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError> {
    fs::create_dir_all(drive_storage_root).map_err(|_| {
        KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable
    })?;
    if !fs::metadata(drive_storage_root)
        .map_err(|_| KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable)?
        .is_dir()
    {
        return Err(KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable);
    }

    let probe_path = drive_storage_root.join(format!(
        ".sdkwork-knowledgebase-rpc-preflight-{}",
        sdkwork_utils_rust::uuid()
    ));
    let result = (|| -> std::io::Result<()> {
        let mut probe = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&probe_path)?;
        probe.write_all(b"sdkwork-knowledgebase-rpc-preflight")?;
        probe.sync_all()?;
        probe.seek(SeekFrom::Start(0))?;
        let mut contents = Vec::new();
        probe.read_to_end(&mut contents)?;
        if contents != b"sdkwork-knowledgebase-rpc-preflight" {
            return Err(std::io::Error::other(
                "drive storage probe readback mismatch",
            ));
        }
        drop(probe);
        fs::remove_file(&probe_path)
    })();

    if result.is_err() {
        let _ = fs::remove_file(&probe_path);
        return Err(KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_storage_preflight_creates_a_writable_root_without_leaving_a_probe() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let root = temporary.path().join("drive-storage");

        verify_drive_storage_root(&root).expect("writable drive root");

        assert!(root.is_dir());
        assert!(
            fs::read_dir(&root)
                .expect("read drive root")
                .next()
                .is_none(),
            "the readiness probe must not leave runtime data behind"
        );
    }

    #[test]
    fn drive_storage_preflight_rejects_a_file_instead_of_a_directory() {
        let temporary = tempfile::tempdir().expect("temporary directory");
        let root = temporary.path().join("not-a-directory");
        fs::write(&root, b"not a directory").expect("fixture file");

        assert!(matches!(
            verify_drive_storage_root(&root),
            Err(KnowledgebaseGroupKnowledgeSpaceLifecycleRuntimeError::DependencyUnavailable)
        ));
    }
}
