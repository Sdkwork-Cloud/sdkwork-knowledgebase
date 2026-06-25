use super::github_api::{
    build_public_raw_source_url, fetch_github_file_content, list_importable_github_files,
    normalize_branch, parse_github_repo_url, GitHubApiError, GitHubImportFile,
};
use crate::ingest::{
    ApiMarkdownIngestPipeline, ApiMarkdownIngestPipelineError, GIT_IMPORT_CONCURRENCY,
};
use crate::ports::{
    knowledge_drive_storage::KnowledgeDriveStorage,
    knowledge_ingestion_job_store::IngestionJobStore,
    markdown_index_metadata_store::MarkdownIndexMetadataStore,
};
use sdkwork_knowledgebase_contract::{
    git_import::{KnowledgeGitImportRequest, KnowledgeGitImportResult},
    KnowledgeIngestRequest,
};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;
use tokio::task::JoinSet;
use tracing::warn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeGitImportRunResult {
    pub result: KnowledgeGitImportResult,
    pub document_version_ids: Vec<u64>,
}

pub struct KnowledgeGitImportService<'a> {
    pipeline: ApiMarkdownIngestPipeline<'a>,
}

#[derive(Debug, Error)]
pub enum KnowledgeGitImportServiceError {
    #[error("invalid git import request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    GitHub(#[from] GitHubApiError),
    #[error(transparent)]
    Pipeline(#[from] ApiMarkdownIngestPipelineError),
}

impl<'a> KnowledgeGitImportService<'a> {
    pub fn new(
        drive: &'a dyn KnowledgeDriveStorage,
        jobs: &'a dyn IngestionJobStore,
        markdown_metadata: &'a dyn MarkdownIndexMetadataStore,
    ) -> Self {
        Self {
            pipeline: ApiMarkdownIngestPipeline::new(drive, jobs, markdown_metadata),
        }
    }

    pub async fn import_repository(
        &self,
        request: KnowledgeGitImportRequest,
        drive_space_id: Option<&str>,
    ) -> Result<KnowledgeGitImportRunResult, KnowledgeGitImportServiceError> {
        if request.space_id == 0 {
            return Err(KnowledgeGitImportServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }
        if is_blank(Some(request.repo_url.as_str())) {
            return Err(KnowledgeGitImportServiceError::InvalidRequest(
                "repo_url is required".to_string(),
            ));
        }
        if is_blank(Some(request.idempotency_key.as_str())) {
            return Err(KnowledgeGitImportServiceError::InvalidRequest(
                "idempotency_key is required".to_string(),
            ));
        }

        let parsed = parse_github_repo_url(&request.repo_url)?;
        let branch = normalize_branch(request.branch.as_deref());
        let access_token = request
            .git_access_token
            .as_deref()
            .filter(|value| !is_blank(Some(*value)));
        let repo_key = format!("{}/{}@{}", parsed.owner, parsed.repo, branch);
        let files =
            list_importable_github_files(&parsed.owner, &parsed.repo, &branch, access_token)
                .await?;

        let mut imported_count = 0u32;
        let mut skipped_count = 0u32;
        let mut document_version_ids = Vec::new();
        let access_token = access_token.map(str::to_string);

        for batch in files.chunks(GIT_IMPORT_CONCURRENCY) {
            let mut join_set = JoinSet::new();
            for file in batch {
                let file = file.clone();
                let owner = parsed.owner.clone();
                let repo = parsed.repo.clone();
                let branch = branch.clone();
                let repo_key = repo_key.clone();
                let space_id = request.space_id;
                let token = access_token.clone();
                join_set.spawn(async move {
                    build_git_ingest_request(
                        space_id,
                        &repo_key,
                        &owner,
                        &repo,
                        &branch,
                        token.as_deref(),
                        file,
                    )
                    .await
                });
            }

            let mut prepared = Vec::new();
            while let Some(join_result) = join_set.join_next().await {
                match join_result {
                    Ok(Ok(ingest_request)) => prepared.push(ingest_request),
                    Ok(Err(error)) => {
                        warn!(?error, "skipped git import file during fetch");
                        skipped_count += 1;
                    }
                    Err(error) => {
                        warn!(?error, "skipped git import file due to task failure");
                        skipped_count += 1;
                    }
                }
            }

            for ingest_request in prepared {
                match self
                    .pipeline
                    .run(ingest_request, drive_space_id, "git-import")
                    .await
                {
                    Ok(pipeline_result) => {
                        imported_count += 1;
                        if let Some(document_version_id) = pipeline_result.document_version_id {
                            document_version_ids.push(document_version_id);
                        }
                    }
                    Err(error) => {
                        warn!(?error, "skipped git import file during ingest");
                        skipped_count += 1;
                    }
                }
            }
        }

        if imported_count == 0 {
            return Err(KnowledgeGitImportServiceError::InvalidRequest(
                "git import did not ingest any files; check repository access and file types"
                    .to_string(),
            ));
        }

        Ok(KnowledgeGitImportRunResult {
            result: KnowledgeGitImportResult {
                imported_count,
                skipped_count,
            },
            document_version_ids,
        })
    }
}

async fn build_git_ingest_request(
    space_id: u64,
    repo_key: &str,
    owner: &str,
    repo: &str,
    branch: &str,
    access_token: Option<&str>,
    file: GitHubImportFile,
) -> Result<KnowledgeIngestRequest, GitHubApiError> {
    let title = title_from_path(&file.path);
    let idempotency_key = build_file_idempotency_key(space_id, repo_key, &file.path);
    if file.use_private_api {
        let payload_markdown =
            fetch_github_file_content(owner, repo, branch, &file.path, access_token).await?;
        return Ok(KnowledgeIngestRequest {
            space_id,
            title,
            payload_markdown,
            source_url: None,
            idempotency_key,
        });
    }

    Ok(KnowledgeIngestRequest {
        space_id,
        title,
        payload_markdown: String::new(),
        source_url: Some(build_public_raw_source_url(owner, repo, branch, &file.path)),
        idempotency_key,
    })
}

fn title_from_path(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn build_file_idempotency_key(space_id: u64, repo_key: &str, path: &str) -> String {
    let raw = format!("git-import-{space_id}-{repo_key}-{path}");
    raw.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '-'
            }
        })
        .take(128)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_file_idempotency_key_sanitizes_and_truncates() {
        let key = build_file_idempotency_key(42, "octocat/Hello@main", "docs/read me.md");
        assert!(key.starts_with("git-import-42-"));
        assert!(!key.contains(' '));
    }
}
