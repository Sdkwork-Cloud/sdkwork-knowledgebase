//! Structured audit log lines, Prometheus counters, and durable persistence hooks.

use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

type AuditPersistenceHandler = Arc<dyn Fn(AuditPersistenceEvent) + Send + Sync>;

static DOCUMENT_VISIBILITY_CHANGED_TOTAL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
static SPACE_MEMBER_GRANTED_TOTAL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
static SPACE_MEMBER_REVOKED_TOTAL: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
static BACKEND_ADMIN_OPERATION_TOTAL: std::sync::atomic::AtomicU64 =
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

/// Installs a durable audit writer invoked for security-relevant mutations.
pub fn install_audit_persistence(handler: AuditPersistenceHandler) {
    let mut slot = AUDIT_PERSISTENCE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if slot.is_some() {
        tracing::warn!("audit persistence handler already installed; ignoring duplicate install");
        return;
    }
    *slot = Some(handler);
}

fn persist(event: AuditPersistenceEvent) {
    if let Some(handler) = AUDIT_PERSISTENCE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
    {
        handler(event);
    }
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
}

pub fn record_document_visibility_changed(
    document_id: u64,
    space_id: u64,
    actor_id: u64,
    previous_visibility: &str,
    new_visibility: &str,
) {
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
    });
}

pub fn record_space_member_granted(
    space_id: u64,
    actor_id: u64,
    subject_type: &str,
    subject_id: &str,
    role: &str,
) {
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
    });
}

pub fn record_space_member_revoked(
    space_id: u64,
    actor_id: u64,
    subject_type: &str,
    subject_id: &str,
) {
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
    });
}

pub fn record_backend_admin_operation(operation: &str, tenant_id: u64, operator_id: u64) {
    use std::sync::atomic::Ordering;

    BACKEND_ADMIN_OPERATION_TOTAL.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        audit_event = "knowledge.backend.admin_operation",
        operation,
        tenant_id,
        operator_id,
        "backend admin operation executed"
    );
    persist(AuditPersistenceEvent {
        event_type: "knowledge.backend.admin_operation".to_string(),
        actor_type: "user".to_string(),
        actor_id: operator_id.to_string(),
        resource_type: "backend_operation".to_string(),
        resource_id: None,
        result: "success".to_string(),
        payload: Some(json!({
            "operation": operation,
            "tenant_id": tenant_id,
        })),
    });
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
         knowledge_audit_backend_admin_operation_total {}\n",
        DOCUMENT_VISIBILITY_CHANGED_TOTAL.load(Ordering::Relaxed),
        SPACE_MEMBER_GRANTED_TOTAL.load(Ordering::Relaxed),
        SPACE_MEMBER_REVOKED_TOTAL.load(Ordering::Relaxed),
        BACKEND_ADMIN_OPERATION_TOTAL.load(Ordering::Relaxed),
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

    #[test]
    fn audit_metrics_export_prometheus_lines() {
        let _guard = audit_test_lock();
        reset_audit_persistence_for_tests();
        reset_audit_counters_for_tests();
        record_document_visibility_changed(1, 2, 3, "space", "public");
        record_space_member_granted(2, 3, "user", "alice", "writer");
        record_space_member_revoked(2, 3, "user", "alice");

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
        record_backend_admin_operation("sources.create", 100_001, 99);
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
        install_audit_persistence(Arc::new(move |event| {
            sink.lock().expect("lock").push(event.event_type);
        }));
        record_backend_admin_operation("sources.list", 1, 2);
        let events = captured.lock().expect("lock");
        assert_eq!(
            events.last().map(String::as_str),
            Some("knowledge.backend.admin_operation")
        );
        reset_audit_persistence_for_tests();
    }
}
