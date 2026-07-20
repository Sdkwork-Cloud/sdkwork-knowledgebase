use serde::{de, de::Visitor, Deserialize, Deserializer, Serializer};
use std::fmt;

pub const MAX_SIGNED_I64_AS_U64: u64 = i64::MAX as u64;

/// Parses an unsigned decimal integer only when its text representation is canonical.
///
/// Cross-service resource scopes are signed `BIGINT` values on the wire. Accepting convenient
/// spellings such as `01`, `+1`, or whitespace would let two textual payloads describe the same
/// durable resource, weakening idempotency fingerprints and signed caller-context comparisons.
pub fn parse_canonical_u64(value: &str) -> Result<u64, CanonicalIntegerError> {
    if value.is_empty() {
        return Err(CanonicalIntegerError::Empty);
    }
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CanonicalIntegerError::NonDecimal);
    }
    if value.len() > 1 && value.starts_with('0') {
        return Err(CanonicalIntegerError::LeadingZero);
    }
    value
        .parse::<u64>()
        .map_err(|_| CanonicalIntegerError::OutOfRange)
}

/// Parses a positive, canonical decimal value that fits the signed SQL `BIGINT` range.
pub fn parse_canonical_positive_signed_i64(value: &str) -> Result<u64, CanonicalIntegerError> {
    let parsed = parse_canonical_u64(value)?;
    if parsed == 0 {
        return Err(CanonicalIntegerError::NotPositive);
    }
    if parsed > MAX_SIGNED_I64_AS_U64 {
        return Err(CanonicalIntegerError::OutOfRange);
    }
    Ok(parsed)
}

/// Parses a nonnegative, canonical decimal value that fits the signed SQL `BIGINT` range.
pub fn parse_canonical_nonnegative_signed_i64(value: &str) -> Result<u64, CanonicalIntegerError> {
    let parsed = parse_canonical_u64(value)?;
    if parsed > MAX_SIGNED_I64_AS_U64 {
        return Err(CanonicalIntegerError::OutOfRange);
    }
    Ok(parsed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalIntegerError {
    Empty,
    NonDecimal,
    LeadingZero,
    NotPositive,
    OutOfRange,
}

impl fmt::Display for CanonicalIntegerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "value must not be empty",
            Self::NonDecimal => "value must contain canonical decimal digits only",
            Self::LeadingZero => "value must not contain leading zeroes",
            Self::NotPositive => "value must be positive",
            Self::OutOfRange => "value must fit a signed 64-bit integer",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CanonicalIntegerError {}

pub fn serialize_u64_as_string<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize_u64_from_string_or_number<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(U64StringOrNumberVisitor)
}

/// Decodes a canonical positive signed BIGINT while preserving the existing Rust `u64` model
/// used by legacy contract consumers. Lifecycle scope and resource identifiers use this at every
/// wire boundary so `u64` does not silently widen PostgreSQL/SQLite BIGINT identity.
pub fn deserialize_positive_i64_as_u64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = deserialize_u64_from_string_or_number(deserializer)?;
    if value == 0 || value > MAX_SIGNED_I64_AS_U64 {
        return Err(de::Error::custom(
            "value must be a positive signed 64-bit integer",
        ));
    }
    Ok(value)
}

/// Decodes a nonnegative signed BIGINT for monotonic counters such as membership epochs and
/// upstream link generations.
pub fn deserialize_nonnegative_i64_as_u64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = deserialize_u64_from_string_or_number(deserializer)?;
    if value > MAX_SIGNED_I64_AS_U64 {
        return Err(de::Error::custom("value must fit a signed 64-bit integer"));
    }
    Ok(value)
}

pub fn serialize_option_u64_as_string<S>(
    value: &Option<u64>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serializer.serialize_some(&value.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_option_u64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<U64StringOrNumber>::deserialize(deserializer).map(|value| value.map(|value| value.0))
}

pub fn deserialize_option_positive_i64_as_u64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<U64StringOrNumber>::deserialize(deserializer).and_then(|value| {
        value
            .map(|value| {
                if value.0 == 0 || value.0 > MAX_SIGNED_I64_AS_U64 {
                    Err(de::Error::custom(
                        "value must be a positive signed 64-bit integer",
                    ))
                } else {
                    Ok(value.0)
                }
            })
            .transpose()
    })
}

struct U64StringOrNumber(u64);

impl<'de> Deserialize<'de> for U64StringOrNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_u64_from_string_or_number(deserializer).map(Self)
    }
}

struct U64StringOrNumberVisitor;

impl<'de> Visitor<'de> for U64StringOrNumberVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a u64 encoded as a JSON string or number")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u64::try_from(value).map_err(|_| E::custom("u64 value must not be negative"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse_canonical_u64(value).map_err(E::custom)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }
}
