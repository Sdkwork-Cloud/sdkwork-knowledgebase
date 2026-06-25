use async_trait::async_trait;
use sdkwork_knowledgebase_contract::git_sync::{KnowledgeGitSyncRequest, KnowledgeGitSyncResult};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

use super::github_api::{
    create_github_commit, normalize_branch, parse_github_repo_url, GitHubApiError, GitHubCommitFile,
};
use crate::ports::knowledge_document_store::KnowledgeDocumentStore;

const MAX_SYNC_FILES: usize = 64;
const SYNC_ROOT_PREFIX: &str = "sdkwork-knowledgebase";

#[derive(Debug, Error)]
pub enum KnowledgeGitSyncServiceError {
    #[error("invalid git sync request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    GitHub(#[from] GitHubApiError),
    #[error("document markdown read failed: {0}")]
    DocumentContent(String),
    #[error(transparent)]
    DocumentStore(#[from] crate::ports::knowledge_document_store::KnowledgeDocumentStoreError),
}

#[async_trait]
pub trait KnowledgeDocumentMarkdownReader: Send + Sync {
    async fn read_document_markdown(&self, document_id: u64) -> Result<String, String>;
}

pub struct KnowledgeGitSyncService<'a> {
    documents: &'a dyn KnowledgeDocumentStore,
    markdown: &'a dyn KnowledgeDocumentMarkdownReader,
}

impl<'a> KnowledgeGitSyncService<'a> {
    pub fn new(
        documents: &'a dyn KnowledgeDocumentStore,
        markdown: &'a dyn KnowledgeDocumentMarkdownReader,
    ) -> Self {
        Self {
            documents,
            markdown,
        }
    }

    pub async fn sync_repository(
        &self,
        request: KnowledgeGitSyncRequest,
    ) -> Result<KnowledgeGitSyncResult, KnowledgeGitSyncServiceError> {
        if request.space_id == 0 {
            return Err(KnowledgeGitSyncServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }
        if is_blank(Some(request.repo_url.as_str())) {
            return Err(KnowledgeGitSyncServiceError::InvalidRequest(
                "repo_url is required".to_string(),
            ));
        }
        if is_blank(Some(request.commit_message.as_str())) {
            return Err(KnowledgeGitSyncServiceError::InvalidRequest(
                "commit_message is required".to_string(),
            ));
        }
        if is_blank(Some(request.idempotency_key.as_str())) {
            return Err(KnowledgeGitSyncServiceError::InvalidRequest(
                "idempotency_key is required".to_string(),
            ));
        }

        let parsed = parse_github_repo_url(&request.repo_url)?;
        let branch = normalize_branch(request.branch.as_deref());
        let access_token = request
            .git_access_token
            .as_deref()
            .filter(|value| !is_blank(Some(value)));
        let documents = self
            .documents
            .list_documents_for_space(request.space_id, MAX_SYNC_FILES as u32)
            .await?;
        if documents.is_empty() {
            return Err(KnowledgeGitSyncServiceError::InvalidRequest(
                "no documents were found in the knowledge space to sync".to_string(),
            ));
        }

        let mut files = Vec::new();
        let mut used_paths = std::collections::HashSet::new();
        for document in documents {
            let markdown = self
                .markdown
                .read_document_markdown(document.id)
                .await
                .map_err(KnowledgeGitSyncServiceError::DocumentContent)?;
            if is_blank(Some(markdown.as_str())) {
                continue;
            }
            let path = unique_sync_path(&document.title, document.id, &mut used_paths);
            files.push(GitHubCommitFile {
                path,
                content: markdown,
            });
            if files.len() >= MAX_SYNC_FILES {
                break;
            }
        }

        if files.is_empty() {
            return Err(KnowledgeGitSyncServiceError::InvalidRequest(
                "no markdown document content was available to sync".to_string(),
            ));
        }

        let commit_sha = create_github_commit(
            &parsed.owner,
            &parsed.repo,
            &branch,
            access_token,
            request.commit_message.trim(),
            &files,
        )
        .await?;

        Ok(KnowledgeGitSyncResult {
            success: true,
            hash: commit_sha,
            synced_count: files.len() as u32,
        })
    }
}

fn unique_sync_path(
    title: &str,
    document_id: u64,
    used_paths: &mut std::collections::HashSet<String>,
) -> String {
    let base = sanitize_sync_filename(title);
    let mut candidate = format!("{SYNC_ROOT_PREFIX}/{base}.md");
    if used_paths.insert(candidate.clone()) {
        return candidate;
    }
    candidate = format!("{SYNC_ROOT_PREFIX}/{base}-{document_id}.md");
    if used_paths.insert(candidate.clone()) {
        return candidate;
    }
    format!("{SYNC_ROOT_PREFIX}/{base}-{document_id}-sync.md")
}

fn sanitize_sync_filename(title: &str) -> String {
    let trimmed = title.trim();
    let fallback = if trimmed.is_empty() {
        "document".to_string()
    } else {
        trimmed
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                    ch
                } else if ch.is_whitespace() {
                    '-'
                } else {
                    '_'
                }
            })
            .collect::<String>()
    };
    let collapsed = fallback
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if collapsed.is_empty() {
        "document".to_string()
    } else {
        collapsed.chars().take(96).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_sync_filename_replaces_unsafe_characters() {
        assert_eq!(
            sanitize_sync_filename("Hello World/README"),
            "Hello-World_README"
        );
    }

    #[test]
    fn unique_sync_path_appends_document_id_on_collision() {
        let mut used = std::collections::HashSet::new();
        let first = unique_sync_path("Guide", 1, &mut used);
        let second = unique_sync_path("Guide", 2, &mut used);
        assert_ne!(first, second);
        assert!(second.contains("2"));
    }
}
