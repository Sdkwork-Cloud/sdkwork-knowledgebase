mod api_markdown_ingest_pipeline;
mod idempotency;
mod job_worker;
mod markdown_index;
mod payload_limits;
mod post_ingest_embed;
mod service;
mod web_link_fetch;

pub use api_markdown_ingest_pipeline::{
    ApiMarkdownIngestPipeline, ApiMarkdownIngestPipelineError, ApiMarkdownIngestPipelineResult,
    ExistingMarkdownIngestJobParams,
};
pub use job_worker::{
    IngestionJobWorkerBatchResult, KnowledgeIngestionJobWorkerService,
    KnowledgeIngestionJobWorkerServiceError,
};
pub use markdown_index::{
    split_markdown_chunks, KnowledgeApiMarkdownIndexService, KnowledgeApiMarkdownIndexServiceError,
    MarkdownIndexResult, PreparedMarkdownIndex,
};
pub use payload_limits::{
    validate_markdown_payload, PayloadLimitError, GIT_IMPORT_CONCURRENCY, MAX_MARKDOWN_CHUNKS,
    MAX_MARKDOWN_CHUNK_CHARS, MAX_MARKDOWN_PAYLOAD_BYTES, OKF_IMPORT_CONCURRENCY,
};
pub use post_ingest_embed::{
    KnowledgePostIngestEmbeddingService, KnowledgePostIngestEmbeddingServiceError,
};
pub use service::{
    ingest_success_outbox_record, KnowledgeApiPayloadIngestService,
    KnowledgeApiPayloadIngestServiceError, KnowledgeIngestionService,
    KnowledgeIngestionServiceError,
};
pub use web_link_fetch::{fetch_web_link_markdown, validate_public_http_url, WebLinkFetchError};
