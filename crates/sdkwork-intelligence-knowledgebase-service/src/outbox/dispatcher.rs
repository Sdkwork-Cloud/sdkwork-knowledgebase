use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use sdkwork_utils_rust::is_blank;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::ports::knowledge_outbox_dispatcher::{
    KnowledgeOutboxDispatchError, KnowledgeOutboxDispatcher,
};
use crate::ports::knowledge_outbox_store::PendingOutboxEvent;

const OUTBOX_WEBHOOK_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OUTBOX_WEBHOOK_URL";
const OUTBOX_WEBHOOK_SECRET_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OUTBOX_WEBHOOK_SECRET";
const OUTBOX_WEBHOOK_TIMEOUT_SECS_ENV: &str = "SDKWORK_KNOWLEDGEBASE_OUTBOX_WEBHOOK_TIMEOUT_SECS";

pub struct LoggingKnowledgeOutboxDispatcher;

pub struct FailClosedKnowledgeOutboxDispatcher;

#[async_trait]
impl KnowledgeOutboxDispatcher for FailClosedKnowledgeOutboxDispatcher {
    async fn dispatch(
        &self,
        _tenant_id: u64,
        _event: &PendingOutboxEvent,
    ) -> Result<(), KnowledgeOutboxDispatchError> {
        Err(KnowledgeOutboxDispatchError::Internal(
            "SDKWORK_KNOWLEDGEBASE_OUTBOX_WEBHOOK_URL must be configured outside development"
                .to_string(),
        ))
    }
}

#[async_trait]
impl KnowledgeOutboxDispatcher for LoggingKnowledgeOutboxDispatcher {
    async fn dispatch(
        &self,
        tenant_id: u64,
        event: &PendingOutboxEvent,
    ) -> Result<(), KnowledgeOutboxDispatchError> {
        tracing::info!(
            tenant_id,
            event_id = event.id,
            event_type = %event.event_type,
            aggregate_type = %event.aggregate_type,
            aggregate_id = event.aggregate_id,
            "delivered knowledgebase outbox event to logging dispatcher"
        );
        Ok(())
    }
}

pub struct WebhookKnowledgeOutboxDispatcher {
    client: Client,
    webhook_url: String,
    webhook_secret: Option<String>,
}

impl WebhookKnowledgeOutboxDispatcher {
    pub fn from_env() -> Result<Self, KnowledgeOutboxDispatchError> {
        let webhook_url = std::env::var(OUTBOX_WEBHOOK_ENV).map_err(|_| {
            KnowledgeOutboxDispatchError::Internal(format!(
                "{OUTBOX_WEBHOOK_ENV} is not configured"
            ))
        })?;
        if is_blank(Some(webhook_url.as_str())) {
            return Err(KnowledgeOutboxDispatchError::Internal(format!(
                "{OUTBOX_WEBHOOK_ENV} must not be blank"
            )));
        }

        let webhook_secret = std::env::var(OUTBOX_WEBHOOK_SECRET_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let timeout_secs = std::env::var(OUTBOX_WEBHOOK_TIMEOUT_SECS_ENV)
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(10)
            .clamp(1, 120);

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|error| KnowledgeOutboxDispatchError::Internal(error.to_string()))?;

        Ok(Self {
            client,
            webhook_url,
            webhook_secret,
        })
    }
}

#[async_trait]
impl KnowledgeOutboxDispatcher for WebhookKnowledgeOutboxDispatcher {
    async fn dispatch(
        &self,
        tenant_id: u64,
        event: &PendingOutboxEvent,
    ) -> Result<(), KnowledgeOutboxDispatchError> {
        let event_time = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
        let body = serde_json::json!({
            "specversion": "1.0",
            "id": format!("knowledgebase-outbox-{tenant_id}-{}", event.id),
            "time": event_time,
            "tenantId": tenant_id,
            "eventId": event.id,
            "eventType": event.event_type,
            "aggregateType": event.aggregate_type,
            "aggregateId": event.aggregate_id,
            "payload": serde_json::from_str::<serde_json::Value>(&event.payload_json)
                .unwrap_or_else(|_| serde_json::Value::String(event.payload_json.clone())),
        });
        let body_bytes = serde_json::to_vec(&body)
            .map_err(|error| KnowledgeOutboxDispatchError::Internal(error.to_string()))?;

        let mut request = self
            .client
            .post(&self.webhook_url)
            .header("content-type", "application/json");
        if let Some(secret) = &self.webhook_secret {
            request = request.header(
                "x-sdkwork-outbox-signature",
                sign_webhook_payload(secret, &body_bytes),
            );
        }

        let response = request
            .body(body_bytes)
            .send()
            .await
            .map_err(|error| KnowledgeOutboxDispatchError::DeliveryFailed(error.to_string()))?;

        if !response.status().is_success() {
            return Err(KnowledgeOutboxDispatchError::DeliveryFailed(format!(
                "webhook returned HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }
}

fn sign_webhook_payload(secret: &str, body: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts arbitrary key length");
    mac.update(body);
    let signature = mac.finalize().into_bytes();
    format!(
        "sha256={}",
        signature
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

pub fn knowledge_outbox_dispatcher_from_env() -> Arc<dyn KnowledgeOutboxDispatcher> {
    match WebhookKnowledgeOutboxDispatcher::from_env() {
        Ok(dispatcher) => Arc::new(dispatcher),
        Err(error) => {
            let is_development = std::env::var("SDKWORK_KNOWLEDGEBASE_ENVIRONMENT")
                .ok()
                .map(|value| value.eq_ignore_ascii_case("development"))
                .unwrap_or(false);
            if !is_development {
                tracing::error!(
                    error = %error,
                    "outbox webhook dispatcher is required outside development"
                );
                return Arc::new(FailClosedKnowledgeOutboxDispatcher);
            }
            tracing::debug!(
                error = %error,
                "outbox webhook dispatcher unavailable; using logging dispatcher for development"
            );
            Arc::new(LoggingKnowledgeOutboxDispatcher)
        }
    }
}
