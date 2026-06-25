use std::{
    fmt,
    sync::{Arc, OnceLock},
};

use sdkwork_id_core::{max_snowflake_node_id, SnowflakeIdError, SnowflakeIdGenerator};

static DEFAULT_ID_GENERATOR: OnceLock<Arc<dyn KnowledgeIdGenerator>> = OnceLock::new();

pub trait KnowledgeIdGenerator: fmt::Debug + Send + Sync {
    fn next_id(&self) -> Result<u64, KnowledgeIdGeneratorError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeIdGeneratorError {
    InvalidNodeId { node_id: u16, max_node_id: u16 },
    ClockBeforeEpoch { now_millis: u64, epoch_millis: u64 },
    ClockRollback { last_millis: u64, now_millis: u64 },
    Poisoned,
    Internal(String),
}

impl fmt::Display for KnowledgeIdGeneratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNodeId {
                node_id,
                max_node_id,
            } => write!(
                formatter,
                "snowflake node id {node_id} exceeds max node id {max_node_id}"
            ),
            Self::ClockBeforeEpoch {
                now_millis,
                epoch_millis,
            } => write!(
                formatter,
                "system clock {now_millis} is before snowflake epoch {epoch_millis}"
            ),
            Self::ClockRollback {
                last_millis,
                now_millis,
            } => write!(
                formatter,
                "system clock moved backwards from {last_millis} to {now_millis}"
            ),
            Self::Poisoned => write!(formatter, "snowflake id generator state lock poisoned"),
            Self::Internal(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for KnowledgeIdGeneratorError {}

#[derive(Debug)]
struct FailedKnowledgeIdGenerator {
    error: KnowledgeIdGeneratorError,
}

impl KnowledgeIdGenerator for FailedKnowledgeIdGenerator {
    fn next_id(&self) -> Result<u64, KnowledgeIdGeneratorError> {
        Err(self.error.clone())
    }
}

#[derive(Debug)]
pub struct SnowflakeKnowledgeIdGenerator {
    inner: SnowflakeIdGenerator,
}

impl SnowflakeKnowledgeIdGenerator {
    pub fn new(node_id: u16) -> Result<Self, KnowledgeIdGeneratorError> {
        Ok(Self {
            inner: SnowflakeIdGenerator::new(node_id).map_err(map_snowflake_error)?,
        })
    }

    pub fn from_node_id_config(value: Option<&str>) -> Result<Self, KnowledgeIdGeneratorError> {
        let Some(value) = value else {
            return Self::new(0);
        };
        let value = value.trim();
        if value.is_empty() {
            return Err(KnowledgeIdGeneratorError::Internal(
                "snowflake node id is required when configured".to_string(),
            ));
        }
        let node_id = parse_or_hash_node_id(value)?;
        Self::new(node_id)
    }

    pub fn node_id(&self) -> u16 {
        self.inner.node_id()
    }

    pub fn epoch_millis(&self) -> u64 {
        self.inner.epoch_millis()
    }
}

impl KnowledgeIdGenerator for SnowflakeKnowledgeIdGenerator {
    fn next_id(&self) -> Result<u64, KnowledgeIdGeneratorError> {
        let id = self.inner.generate().map_err(map_snowflake_error)?;
        u64::try_from(id).map_err(|_| {
            KnowledgeIdGeneratorError::Internal("snowflake id is negative".to_string())
        })
    }
}

pub fn default_knowledge_id_generator() -> Arc<dyn KnowledgeIdGenerator> {
    DEFAULT_ID_GENERATOR
        .get_or_init(|| {
            knowledge_id_generator_from_config(
                std::env::var("SDKWORK_KNOWLEDGEBASE_SNOWFLAKE_NODE_ID")
                    .ok()
                    .as_deref(),
            )
        })
        .clone()
}

fn knowledge_id_generator_from_config(value: Option<&str>) -> Arc<dyn KnowledgeIdGenerator> {
    match SnowflakeKnowledgeIdGenerator::from_node_id_config(value) {
        Ok(generator) => Arc::new(generator),
        Err(error) => Arc::new(FailedKnowledgeIdGenerator { error }),
    }
}

pub(crate) fn next_i64_id(
    generator: &Arc<dyn KnowledgeIdGenerator>,
) -> Result<i64, KnowledgeIdGeneratorError> {
    let id = generator.next_id()?;
    i64::try_from(id).map_err(|_| {
        KnowledgeIdGeneratorError::Internal("snowflake id exceeds signed int64 range".to_string())
    })
}

/// Parses a numeric node id or deterministically hashes orchestration identifiers
/// (for example Kubernetes pod names) into the valid Snowflake node id range.
fn parse_or_hash_node_id(value: &str) -> Result<u16, KnowledgeIdGeneratorError> {
    if let Ok(node_id) = value.parse::<u16>() {
        return Ok(node_id);
    }

    Ok(hash_identifier_to_node_id(value))
}

fn hash_identifier_to_node_id(identifier: &str) -> u16 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in identifier.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash as u16) & max_snowflake_node_id()
}

fn map_snowflake_error(error: SnowflakeIdError) -> KnowledgeIdGeneratorError {
    match error {
        SnowflakeIdError::InvalidNodeId {
            node_id,
            max_node_id,
        } => KnowledgeIdGeneratorError::InvalidNodeId {
            node_id,
            max_node_id,
        },
        SnowflakeIdError::ClockBeforeEpoch {
            now_millis,
            epoch_millis,
        } => KnowledgeIdGeneratorError::ClockBeforeEpoch {
            now_millis,
            epoch_millis,
        },
        SnowflakeIdError::ClockMovedBackwards {
            last_millis,
            now_millis,
        } => KnowledgeIdGeneratorError::ClockRollback {
            last_millis,
            now_millis,
        },
        SnowflakeIdError::StatePoisoned => KnowledgeIdGeneratorError::Poisoned,
        SnowflakeIdError::SequenceExhausted { millis } => KnowledgeIdGeneratorError::Internal(
            format!("snowflake sequence exhausted at millis {millis}"),
        ),
        SnowflakeIdError::TimestampOverflow {
            delta_millis,
            max_delta_millis,
        } => KnowledgeIdGeneratorError::Internal(format!(
            "snowflake timestamp delta {delta_millis} exceeds max {max_delta_millis}"
        )),
        SnowflakeIdError::SystemTime(message) => KnowledgeIdGeneratorError::Internal(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_default_generator_config_returns_errors_without_panicking() {
        let generator = knowledge_id_generator_from_config(Some(""));

        let error = generator.next_id().unwrap_err();

        assert!(error.to_string().contains("required when configured"));
    }

    #[test]
    fn orchestration_identifier_hashes_to_stable_node_id() {
        let first = SnowflakeKnowledgeIdGenerator::from_node_id_config(Some(
            "sdkwork-knowledgebase-app-api-7f4d9c8b5-xk2jp",
        ))
        .expect("pod name should hash to a valid node id");
        let second = SnowflakeKnowledgeIdGenerator::from_node_id_config(Some(
            "sdkwork-knowledgebase-app-api-7f4d9c8b5-xk2jp",
        ))
        .expect("pod name should hash to a valid node id");

        assert_eq!(first.node_id(), second.node_id());
        assert!(first.node_id() <= max_snowflake_node_id());
    }
}
