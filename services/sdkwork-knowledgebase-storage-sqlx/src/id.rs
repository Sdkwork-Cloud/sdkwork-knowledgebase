use std::fmt;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SDKWORK_SNOWFLAKE_EPOCH_MILLIS: u64 = 1_735_689_600_000;
const NODE_ID_BITS: u8 = 10;
const SEQUENCE_BITS: u8 = 12;
const MAX_NODE_ID: u16 = (1 << NODE_ID_BITS) - 1;
const MAX_SEQUENCE: u16 = (1 << SEQUENCE_BITS) - 1;
const NODE_ID_SHIFT: u8 = SEQUENCE_BITS;
const TIMESTAMP_SHIFT: u8 = NODE_ID_BITS + SEQUENCE_BITS;

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
    node_id: u16,
    state: Mutex<SnowflakeState>,
}

impl SnowflakeKnowledgeIdGenerator {
    pub fn new(node_id: u16) -> Result<Self, KnowledgeIdGeneratorError> {
        if node_id > MAX_NODE_ID {
            return Err(KnowledgeIdGeneratorError::InvalidNodeId {
                node_id,
                max_node_id: MAX_NODE_ID,
            });
        }

        Ok(Self {
            node_id,
            state: Mutex::new(SnowflakeState {
                last_millis: 0,
                sequence: 0,
            }),
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
        let node_id = value.parse::<u16>().map_err(|_| {
            KnowledgeIdGeneratorError::Internal(
                "snowflake node id must be an integer between 0 and 1023".to_string(),
            )
        })?;
        Self::new(node_id)
    }

    pub fn node_id(&self) -> u16 {
        self.node_id
    }
}

impl KnowledgeIdGenerator for SnowflakeKnowledgeIdGenerator {
    fn next_id(&self) -> Result<u64, KnowledgeIdGeneratorError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| KnowledgeIdGeneratorError::Poisoned)?;
        let mut now_millis = current_epoch_millis()?;

        if now_millis < state.last_millis {
            return Err(KnowledgeIdGeneratorError::ClockRollback {
                last_millis: state.last_millis,
                now_millis,
            });
        }

        if now_millis == state.last_millis {
            if state.sequence == MAX_SEQUENCE {
                now_millis = wait_next_millis(state.last_millis)?;
                state.sequence = 0;
            } else {
                state.sequence += 1;
            }
        } else {
            state.sequence = 0;
        }

        state.last_millis = now_millis;
        Ok(
            ((now_millis - SDKWORK_SNOWFLAKE_EPOCH_MILLIS) << TIMESTAMP_SHIFT)
                | (u64::from(self.node_id) << NODE_ID_SHIFT)
                | u64::from(state.sequence),
        )
    }
}

#[derive(Debug)]
struct SnowflakeState {
    last_millis: u64,
    sequence: u16,
}

pub(crate) fn default_knowledge_id_generator() -> Arc<dyn KnowledgeIdGenerator> {
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

fn wait_next_millis(last_millis: u64) -> Result<u64, KnowledgeIdGeneratorError> {
    loop {
        let now_millis = current_epoch_millis()?;
        if now_millis > last_millis {
            return Ok(now_millis);
        }
        std::thread::sleep(Duration::from_micros(100));
    }
}

fn current_epoch_millis() -> Result<u64, KnowledgeIdGeneratorError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| {
        KnowledgeIdGeneratorError::ClockBeforeEpoch {
            now_millis: 0,
            epoch_millis: SDKWORK_SNOWFLAKE_EPOCH_MILLIS,
        }
    })?;
    let millis = duration.as_millis();
    let millis = u64::try_from(millis).map_err(|_| {
        KnowledgeIdGeneratorError::Internal("system clock millis exceeds u64 range".to_string())
    })?;
    if millis < SDKWORK_SNOWFLAKE_EPOCH_MILLIS {
        return Err(KnowledgeIdGeneratorError::ClockBeforeEpoch {
            now_millis: millis,
            epoch_millis: SDKWORK_SNOWFLAKE_EPOCH_MILLIS,
        });
    }
    Ok(millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_default_generator_config_returns_errors_without_panicking() {
        let generator = knowledge_id_generator_from_config(Some("not-a-node-id"));

        let error = generator.next_id().unwrap_err();

        assert!(error.to_string().contains("must be an integer"));
    }
}
