//! Structured audit log lines, Prometheus counters, and durable persistence hooks.

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use serde_json::{json, Value};

type AuditPersistenceFuture =
    Pin<Box<dyn Future<Output = Result<(), AuditPersistenceError>> + Send>>;
type AuditPersistenceHandler =
    Arc<dyn Fn(AuditPersistenceEvent) -> AuditPersistenceFuture + Send + Sync>;

static DOCUMENT_VISIBILITY_CHANGED_TOTAL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
static SPACE_MEMBER_GRANTED_TOTAL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
static SPACE_MEMBER_REVOKED_TOTAL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
static BACKEND_ADMIN_OPERATION_TOTAL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
static PROVIDER_MIGRATION_TRANSITION_TOTAL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

static AUDIT_PERSISTENCE: Mutex<Option<AuditPersistenceHandler>> = Mutex::new(None);

#[derive(Debug, Clone)]
pub struct AuditPersistenceEvent {
    pub event_type: String,
    pub actor_type: String,
    pub actor_id: String,
    pub resource_type: String,
    pub resource_id: Option<u64>,
    pub result: String,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuditPersistenceError {
    #[error("audit persistence is not configured")]
    Unavailable,
    #[error("audit persistence failed: {0}")]
    WriteFailed(String),
}

impl AuditPersistenceError {
    pub fn write_failed(detail: impl Into<String>) -> Self {
        Self::WriteFailed(detail.into())
    }
}

/// Installs a durable audit writer invoked for security-relevant mutations.
pub fn install_audit_persistence<F, Fut>(handler: F)
where
    F: Fn(AuditPersistenceEvent) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), AuditPersistenceError>> + Send + 'static,
{
    let handler: AuditPersistenceHandler = Arc::new(move |event| Box::pin(handler(event)));
    let mut slot = AUDIT_PERSISTENCE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if slot.replace(handler).is_some() {
        tracing::warn!("audit persistence handler already installed; replacing stale handler");
    }
}

async fn persist(event: AuditPersistenceEvent) -> Result<(), AuditPersistenceError> {
    let handler = AUDIT_PERSISTENCE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
        .cloned()
        .ok_or(AuditPersistenceError::Unavailable)?;
    handler(event).await
}

#[cfg(test)]
fn reset_audit_persistence_for_tests() {
    *AUDIT_PERSISTENCE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
}

#[cfg(test)]
fn reset_audit_counters_for_tests() {
    use std::sync::atomic::Ordering;

    DOCUMENT_VISIBILITY_CHANGED_TOTAL.store(0, Ordering::Relaxed);
    SPACE_MEMBER_GRANTED_TOTAL.store(0, Ordering::Relaxed);
    SPACE_MEMBER_REVOKED_TOTAL.store(0, Ordering::Relaxed);
    BACKEND_ADMIN_OPERATION_TOTAL.store(0, Ordering::Relaxed);
    PROVIDER_MIGRATION_TRANSITION_TOTAL.store(0, Ordering::Relaxed);
}

pub async fn record_document_visibility_changed(
    document_id: u64,
    space_id: u64,
    actor_id: u64,
    previous_visibility: &str,
    new_visibility: &str,
) -> Result<(), AuditPersistenceError> {
    use std::sync::atomic::Ordering;

    DOCUMENT_VISIBILITY_CHANGED_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        audit_event = "knowledge.document.visibility_changed",
        document_id,
        space_id,
        actor_id,
        previous_visibility,
        new_visibility,
        "document visibility updated"
    );
    persist(AuditPersistenceEvent {
        event_type: "knowledge.document.visibility_changed".to_string(),
        actor_type: "user".to_string(),
        actor_id: actor_id.to_string(),
        resource_type: "document".to_string(),
        resource_id: Some(document_id),
        result: "success".to_string(),
        payload: Some(json!({
            "space_id": space_id,
            "previous_visibility": previous_visibility,
            "new_visibility": new_visibility,
        })),
    })
    .await
}

pub async fn record_space_member_granted(
    space_id: u64,
    actor_id: u64,
    subject_type: &str,
    subject_id: &str,
    role: &str,
) -> Result<(), AuditPersistenceError> {
    use std::sync::atomic::Ordering;

    SPACE_MEMBER_GRANTED_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        audit_event = "knowledge.space.member_granted",
        space_id,
        actor_id,
        subject_type,
        subject_id,
        role,
        "knowledge space member granted"
    );
    persist(AuditPersistenceEvent {
        event_type: "knowledge.space.member_granted".to_string(),
        actor_type: "user".to_string(),
        actor_id: actor_id.to_string(),
        resource_type: "space".to_string(),
        resource_id: Some(space_id),
        result: "success".to_string(),
        payload: Some(json!({
            "subject_type": subject_type,
            "subject_id": subject_id,
            "role": role,
        })),
    })
    .await
}

pub async fn record_space_member_revoked(
    space_id: u64,
    actor_id: u64,
    subject_type: &str,
    subject_id: &str,
) -> Result<(), AuditPersistenceError> {
    use std::sync::atomic::Ordering;

    SPACE_MEMBER_REVOKED_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        audit_event = "knowledge.space.member_revoked",
        space_id,
        actor_id,
        subject_type,
        subject_id,
        "knowledge space member revoked"
    );
    persist(AuditPersistenceEvent {
        event_type: "knowledge.space.member_revoked".to_string(),
        actor_type: "user".to_string(),
        actor_id: actor_id.to_string(),
        resource_type: "space".to_string(),
        resource_id: Some(space_id),
        result: "success".to_string(),
        payload: Some(json!({
            "subject_type": subject_type,
            "subject_id": subject_id,
        })),
    })
    .await
}

pub async fn record_backend_admin_operation(
    operation: &str,
    tenant_id: u64,
    operator_id: u64,
) -> Result<(), AuditPersistenceError> {
    record_backend_admin_resource_operation(
        operation,
        tenant_id,
        operator_id,
        BackendAdminResourceAudit {
            resource_type: "backend_operation".to_string(),
            resource_id: None,
            space_id: None,
            expected_version: None,
            result_version: None,
            result_status: None,
        },
    )
    .await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendAdminResourceAudit {
    pub resource_type: String,
    pub resource_id: Option<u64>,
    pub space_id: Option<u64>,
    pub expected_version: Option<u64>,
    pub result_version: Option<u64>,
    pub result_status: Option<String>,
}

pub async fn record_backend_admin_resource_operation(
    operation: &str,
    tenant_id: u64,
    operator_id: u64,
    resource: BackendAdminResourceAudit,
) -> Result<(), AuditPersistenceError> {
    use std::sync::atomic::Ordering;

    BACKEND_ADMIN_OPERATION_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        audit_event = "knowledge.backend.admin_operation",
        operation,
        tenant_id,
        operator_id,
        resource_type = %resource.resource_type,
        resource_id = ?resource.resource_id,
        space_id = ?resource.space_id,
        expected_version = ?resource.expected_version,
        result_version = ?resource.result_version,
        result_status = ?resource.result_status,
        "backend admin operation executed"
    );
    persist(AuditPersistenceEvent {
        event_type: "knowledge.backend.admin_operation".to_string(),
        actor_type: "user".to_string(),
        actor_id: operator_id.to_string(),
        resource_type: resource.resource_type,
        resource_id: resource.resource_id,
        result: "success".to_string(),
        payload: Some(json!({
            "operation": operation,
            "tenant_id": tenant_id,
            "space_id": resource.space_id,
            "expected_version": resource.expected_version,
            "result_version": resource.result_version,
            "result_status": resource.result_status,
        })),
    })
    .await
}

pub async fn record_provider_migration_transition(
    tenant_id: u64,
    worker_id: &str,
    operation_id: u64,
    space_id: u64,
    previous_state: &str,
    result_state: &str,
    result_version: u64,
) -> Result<(), AuditPersistenceError> {
    use std::sync::atomic::Ordering;

    PROVIDER_MIGRATION_TRANSITION_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        audit_event = "knowledge.provider_migration.transition",
        tenant_id,
        worker_id,
        operation_id,
        space_id,
        previous_state,
        result_state,
        result_version,
        "Provider migration phase transitioned"
    );
    persist(AuditPersistenceEvent {
        event_type: "knowledge.provider_migration.transition".to_string(),
        actor_type: "service".to_string(),
        actor_id: worker_id.to_string(),
        resource_type: "provider_migration_operation".to_string(),
        resource_id: Some(operation_id),
        result: if result_state == "failed" {
            "failure".to_string()
        } else {
            "success".to_string()
        },
        payload: Some(json!({
            "tenant_id": tenant_id,
            "space_id": space_id,
            "previous_state": previous_state,
            "result_state": result_state,
            "result_version": result_version,
        })),
    })
    .await
}

pub fn render_audit_prometheus_metrics() -> String {
    use std::sync::atomic::Ordering;

    format!(
        "# HELP knowledge_audit_document_visibility_changed_total Document visibility audit events.\n\
         # TYPE knowledge_audit_document_visibility_changed_total counter\n\
         knowledge_audit_document_visibility_changed_total {}\n\
         # HELP knowledge_audit_space_member_granted_total Knowledge space member grant audit events.\n\
         # TYPE knowledge_audit_space_member_granted_total counter\n\
         knowledge_audit_space_member_granted_total {}\n\
         # HELP knowledge_audit_space_member_revoked_total Knowledge space member revoke audit events.\n\
         # TYPE knowledge_audit_space_member_revoked_total counter\n\
         knowledge_audit_space_member_revoked_total {}\n\
         # HELP knowledge_audit_backend_admin_operation_total Backend admin mutation audit events.\n\
         # TYPE knowledge_audit_backend_admin_operation_total counter\n\
         knowledge_audit_backend_admin_operation_total {}\n\
         # HELP knowledge_audit_provider_migration_transition_total Provider migration transition audit events.\n\
         # TYPE knowledge_audit_provider_migration_transition_total counter\n\
         knowledge_audit_provider_migration_transition_total {}\n",
        DOCUMENT_VISIBILITY_CHANGED_TOTAL.load(Ordering::Relaxed),
        SPACE_MEMBER_GRANTED_TOTAL.load(Ordering::Relaxed),
        SPACE_MEMBER_REVOKED_TOTAL.load(Ordering::Relaxed),
        BACKEND_ADMIN_OPERATION_TOTAL.load(Ordering::Relaxed),
        PROVIDER_MIGRATION_TRANSITION_TOTAL.load(Ordering::Relaxed),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn audit_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn block_on_audit<F>(future: F) -> Result<(), AuditPersistenceError>
    where
        F: Future<Output = Result<(), AuditPersistenceError>>,
    {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
            .block_on(future)
    }

    #[test]
    fn audit_metrics_export_prometheus_lines() {
        let _guard = audit_test_lock();
        reset_audit_persistence_for_tests();
        reset_audit_counters_for_tests();
        let _ = block_on_audit(record_document_visibility_changed(
            1, 2, 3, "space", "public",
        ));
        let _ = block_on_audit(record_space_member_granted(2, 3, "user", "alice", "writer"));
        let _ = block_on_audit(record_space_member_revoked(2, 3, "user", "alice"));

        let body = render_audit_prometheus_metrics();
        assert!(body.contains("knowledge_audit_document_visibility_changed_total 1"));
        assert!(body.contains("knowledge_audit_space_member_granted_total 1"));
        assert!(body.contains("knowledge_audit_space_member_revoked_total 1"));
    }

    #[test]
    fn backend_admin_audit_metrics_export_prometheus_lines() {
        let _guard = audit_test_lock();
        reset_audit_persistence_for_tests();
        reset_audit_counters_for_tests();
        let _ = block_on_audit(record_backend_admin_operation(
            "sources.create",
            100_001,
            99,
        ));
        let body = render_audit_prometheus_metrics();
        assert!(body.contains("knowledge_audit_backend_admin_operation_total 1"));
    }

    #[test]
    fn install_audit_persistence_invokes_handler() {
        let _guard = audit_test_lock();
        reset_audit_persistence_for_tests();
        reset_audit_counters_for_tests();
        let captured = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = Arc::clone(&captured);
        install_audit_persistence(move |event| {
            let sink = Arc::clone(&sink);
            async move {
                sink.lock().expect("lock").push(event.event_type);
                Ok(())
            }
        });
        block_on_audit(record_backend_admin_operation("sources.list", 1, 2))
            .expect("audit persistence");
        let events = captured.lock().expect("lock");
        assert_eq!(
            events.last().map(String::as_str),
            Some("knowledge.backend.admin_operation")
        );
        reset_audit_persistence_for_tests();
    }

    #[test]
    fn backend_admin_resource_audit_persists_only_whitelisted_provider_metadata() {
        let _guard = audit_test_lock();
        reset_audit_persistence_for_tests();
        reset_audit_counters_for_tests();
        let captured = Arc::new(Mutex::new(Vec::<AuditPersistenceEvent>::new()));
        let sink = Arc::clone(&captured);
        install_audit_persistence(move |event| {
            let sink = Arc::clone(&sink);
            async move {
                sink.lock().expect("lock").push(event);
                Ok(())
            }
        });

        block_on_audit(record_backend_admin_resource_operation(
            "spaces.providerBindings.activate",
            100_001,
            99,
            BackendAdminResourceAudit {
                resource_type: "provider_binding".to_string(),
                resource_id: Some(42),
                space_id: Some(7),
                expected_version: Some(3),
                result_version: None,
                result_status: Some("active".to_string()),
            },
        ))
        .expect("audit persistence");

        let events = captured.lock().expect("lock");
        let event = events.last().expect("provider audit event");
        assert_eq!(event.resource_type, "provider_binding");
        assert_eq!(event.resource_id, Some(42));
        assert_eq!(event.actor_id, "99");
        assert_eq!(
            event.payload,
            Some(json!({
                "operation": "spaces.providerBindings.activate",
                "tenant_id": 100_001,
                "space_id": 7,
                "expected_version": 3,
                "result_version": null,
                "result_status": "active",
            }))
        );
        let serialized_event = format!("{event:?}");
        assert!(!serialized_event.contains("referenceLocator"));
        assert!(!serialized_event.contains("referenceFingerprint"));
        assert!(!serialized_event.contains("remoteResourceId"));
        assert!(!serialized_event.contains("env://"));
        reset_audit_persistence_for_tests();
    }

    #[test]
    fn provider_migration_audit_persists_only_transition_metadata() {
        let _guard = audit_test_lock();
        reset_audit_persistence_for_tests();
        reset_audit_counters_for_tests();
        let captured = Arc::new(Mutex::new(Vec::<AuditPersistenceEvent>::new()));
        let sink = Arc::clone(&captured);
        install_audit_persistence(move |event| {
            let sink = Arc::clone(&sink);
            async move {
                sink.lock().expect("lock").push(event);
                Ok(())
            }
        });

        block_on_audit(record_provider_migration_transition(
            100_001,
            "provider-worker-1",
            91,
            7,
            "validating",
            "cutover",
            8,
        ))
        .expect("Provider migration audit persistence");

        let events = captured.lock().expect("lock");
        let event = events.last().expect("Provider migration audit event");
        assert_eq!(event.actor_type, "service");
        assert_eq!(event.actor_id, "provider-worker-1");
        assert_eq!(event.resource_type, "provider_migration_operation");
        assert_eq!(event.resource_id, Some(91));
        assert_eq!(
            event.payload,
            Some(json!({
                "tenant_id": 100_001,
                "space_id": 7,
                "previous_state": "validating",
                "result_state": "cutover",
                "result_version": 8,
            }))
        );
        let serialized_event = format!("{event:?}");
        for forbidden in [
            "checkpoint",
            "claim_token",
            "remoteResourceId",
            "credential",
        ] {
            assert!(!serialized_event.contains(forbidden));
        }
        assert!(render_audit_prometheus_metrics()
            .contains("knowledge_audit_provider_migration_transition_total 1"));
        reset_audit_persistence_for_tests();
    }

    #[test]
    fn audit_persistence_waits_for_completion_before_returning() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let _guard = audit_test_lock();
        reset_audit_persistence_for_tests();
        reset_audit_counters_for_tests();
        let completed = Arc::new(AtomicBool::new(false));
        let completed_by_handler = Arc::clone(&completed);
        install_audit_persistence(move |_event| {
            let completed = Arc::clone(&completed_by_handler);
            async move {
                tokio::task::yield_now().await;
                completed.store(true, Ordering::SeqCst);
                Ok(())
            }
        });

        block_on_audit(record_backend_admin_operation(
            "sources.create",
            100_001,
            99,
        ))
        .expect("audit persistence");

        assert!(completed.load(Ordering::SeqCst));
        reset_audit_persistence_for_tests();
    }

    #[test]
    fn audit_persistence_returns_database_failure_to_caller() {
        let _guard = audit_test_lock();
        reset_audit_persistence_for_tests();
        reset_audit_counters_for_tests();
        install_audit_persistence(|_event| async {
            Err(AuditPersistenceError::write_failed("database unavailable"))
        });

        let error = block_on_audit(record_backend_admin_operation(
            "sources.create",
            100_001,
            99,
        ))
        .expect_err("persistence failure must be observable");

        assert!(matches!(
            error,
            AuditPersistenceError::WriteFailed(ref detail)
                if detail == "database unavailable"
        ));
        reset_audit_persistence_for_tests();
    }
}
