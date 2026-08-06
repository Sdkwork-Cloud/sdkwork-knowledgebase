#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordSearchBackend {
    PostgresTsVector,
}

pub fn keyword_search_backend_for_database_url(_database_url: &str) -> KeywordSearchBackend {
    // 服务端权威持久化仅支持 PostgreSQL（DATABASE_SPEC：authoritative-server）
    KeywordSearchBackend::PostgresTsVector
}
