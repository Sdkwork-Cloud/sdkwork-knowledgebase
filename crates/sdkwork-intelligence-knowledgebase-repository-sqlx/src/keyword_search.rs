use crate::db::is_postgres_database_url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordSearchBackend {
    SqliteFts5,
    PostgresTsVector,
}

pub fn keyword_search_backend_for_database_url(database_url: &str) -> KeywordSearchBackend {
    if is_postgres_database_url(database_url) {
        KeywordSearchBackend::PostgresTsVector
    } else {
        KeywordSearchBackend::SqliteFts5
    }
}
