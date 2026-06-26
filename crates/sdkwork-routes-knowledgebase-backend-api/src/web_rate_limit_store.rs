use std::sync::Arc;

use sdkwork_knowledgebase_observability::is_production_like_environment;
use sdkwork_web_core::{memory_rate_limit_store, RateLimitStore};

use sdkwork_utils_rust::is_blank;

pub fn knowledgebase_rate_limit_store() -> Arc<dyn RateLimitStore> {
    if let Some(store) = knowledgebase_redis_rate_limit_store() {
        return store;
    }
    if is_production_like_environment() {
        eprintln!(
            "Redis rate limit store is required for production-like environments; configure SDKWORK_KNOWLEDGEBASE_REDIS_URL or SDKWORK_KNOWLEDGEBASE_REDIS_ENABLED"
        );
        std::process::exit(1);
    }
    memory_rate_limit_store()
}

fn knowledgebase_redis_rate_limit_store() -> Option<Arc<dyn RateLimitStore>> {
    let redis_url = knowledgebase_redis_url()?;
    match sdkwork_web_store_redis::shared_rate_limit_store(&redis_url, "sdkwork:knowledgebase") {
        Ok(store) => Some(store),
        Err(error) => {
            if is_production_like_environment() {
                eprintln!(
                    "[knowledgebase] Redis rate limit store unavailable in production-like environment: {error}"
                );
                std::process::exit(1);
            }
            eprintln!(
                "[knowledgebase] Redis rate limit store unavailable ({error}); falling back to in-memory store"
            );
            None
        }
    }
}

fn knowledgebase_redis_url() -> Option<String> {
    if let Ok(url) = std::env::var("SDKWORK_KNOWLEDGEBASE_REDIS_URL") {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let enabled = std::env::var("SDKWORK_KNOWLEDGEBASE_REDIS_ENABLED")
        .ok()
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);
    if !enabled {
        return None;
    }

    let host = std::env::var("SDKWORK_KNOWLEDGEBASE_REDIS_HOST")
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())))?;
    let port = std::env::var("SDKWORK_KNOWLEDGEBASE_REDIS_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(6379);
    let database = std::env::var("SDKWORK_KNOWLEDGEBASE_REDIS_DATABASE")
        .ok()
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(0);
    let scheme = if std::env::var("SDKWORK_KNOWLEDGEBASE_REDIS_TLS")
        .ok()
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
    {
        "rediss"
    } else {
        "redis"
    };

    Some(format!("{scheme}://{host}:{port}/{database}"))
}
