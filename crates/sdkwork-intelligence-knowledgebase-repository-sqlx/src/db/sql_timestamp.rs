//! Cross-engine SQL timestamp bindings for sqlite and PostgreSQL pools.

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub fn utc_sql_timestamp_text() -> Result<String, String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| error.to_string())
}

pub fn sql_timestamp_expr(placeholder: &str) -> String {
    format!("CAST({placeholder} AS TIMESTAMP)")
}
