use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

const GITHUB_API_BASE: &str = "https://api.github.com";
const MAX_IMPORT_FILES: usize = 64;
const MAX_FILE_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedGitHubRepo {
    pub owner: String,
    pub repo: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubImportFile {
    pub path: String,
    pub use_private_api: bool,
}

#[derive(Debug, Error)]
pub enum GitHubApiError {
    #[error("invalid git import request: {0}")]
    InvalidRequest(String),
    #[error("github api request failed: {0}")]
    Upstream(String),
}

pub fn parse_github_repo_url(repo_url: &str) -> Result<ParsedGitHubRepo, GitHubApiError> {
    let mut trimmed = repo_url.trim();
    if let Some(stripped) = trimmed.strip_suffix(".git") {
        trimmed = stripped;
    }
    let rest = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .ok_or_else(|| {
            GitHubApiError::InvalidRequest(
                "only public HTTPS GitHub repository URLs are supported".to_string(),
            )
        })?;
    let host_and_path = rest.split_once('/').ok_or_else(|| {
        GitHubApiError::InvalidRequest(
            "only public HTTPS GitHub repository URLs are supported".to_string(),
        )
    })?;
    if !host_and_path.0.eq_ignore_ascii_case("github.com") {
        return Err(GitHubApiError::InvalidRequest(
            "only public HTTPS GitHub repository URLs are supported".to_string(),
        ));
    }
    let segments: Vec<&str> = host_and_path
        .1
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.len() != 2 {
        return Err(GitHubApiError::InvalidRequest(
            "only public HTTPS GitHub repository URLs are supported".to_string(),
        ));
    }
    Ok(ParsedGitHubRepo {
        owner: segments[0].to_string(),
        repo: segments[1].to_string(),
    })
}

pub fn normalize_branch(branch: Option<&str>) -> String {
    branch
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("main")
        .to_string()
}

pub fn build_public_raw_source_url(owner: &str, repo: &str, branch: &str, path: &str) -> String {
    let encoded_path = path
        .split('/')
        .map(|segment| urlencoding::encode(segment))
        .collect::<Vec<_>>()
        .join("/");
    format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        urlencoding::encode(owner),
        urlencoding::encode(repo),
        urlencoding::encode(branch),
        encoded_path
    )
}

pub fn is_importable_path(path: &str, size: Option<u64>) -> bool {
    if path.contains("..") || path.starts_with('/') {
        return false;
    }
    if let Some(size) = size {
        if size > MAX_FILE_BYTES as u64 {
            return false;
        }
    }
    let lower = path.to_ascii_lowercase();
    let Some(dot) = lower.rfind('.') else {
        return false;
    };
    matches!(
        &lower[dot..],
        ".md"
            | ".markdown"
            | ".txt"
            | ".json"
            | ".yaml"
            | ".yml"
            | ".toml"
            | ".csv"
            | ".ts"
            | ".tsx"
            | ".js"
            | ".jsx"
            | ".py"
            | ".java"
            | ".go"
            | ".rs"
            | ".html"
            | ".htm"
            | ".css"
            | ".xml"
    )
}

pub async fn list_importable_github_files(
    owner: &str,
    repo: &str,
    branch: &str,
    access_token: Option<&str>,
) -> Result<Vec<GitHubImportFile>, GitHubApiError> {
    let client = github_client()?;
    let branch_sha = resolve_branch_sha(&client, owner, repo, branch, access_token).await?;
    let url = format!(
        "{GITHUB_API_BASE}/repos/{}/{}/git/trees/{}?recursive=1",
        urlencoding::encode(owner),
        urlencoding::encode(repo),
        branch_sha
    );
    let response = client
        .get(url)
        .headers(github_headers(access_token)?)
        .send()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    if !response.status().is_success() {
        return Err(GitHubApiError::Upstream(format!(
            "failed to list repository tree: HTTP {}",
            response.status()
        )));
    }
    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    let use_private_api = access_token.is_some();
    let mut paths = payload
        .get("tree")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let entry_type = entry.get("type")?.as_str()?;
            if entry_type != "blob" {
                return None;
            }
            let path = entry.get("path")?.as_str()?.to_string();
            let size = entry.get("size").and_then(|value| value.as_u64());
            is_importable_path(&path, size).then_some(path)
        })
        .collect::<Vec<_>>();
    paths.sort();
    if paths.is_empty() {
        return Err(GitHubApiError::InvalidRequest(
            "no importable text or markdown files were found in the repository".to_string(),
        ));
    }
    Ok(paths
        .into_iter()
        .take(MAX_IMPORT_FILES)
        .map(|path| GitHubImportFile {
            path,
            use_private_api,
        })
        .collect())
}

pub async fn fetch_github_file_content(
    owner: &str,
    repo: &str,
    branch: &str,
    path: &str,
    access_token: Option<&str>,
) -> Result<String, GitHubApiError> {
    let client = github_client()?;
    let encoded_path = path
        .split('/')
        .map(|segment| urlencoding::encode(segment))
        .collect::<Vec<_>>()
        .join("/");
    let url = format!(
        "{GITHUB_API_BASE}/repos/{}/{}/contents/{}?ref={}",
        urlencoding::encode(owner),
        urlencoding::encode(repo),
        encoded_path,
        urlencoding::encode(branch)
    );
    let mut headers = github_headers(access_token)?;
    headers.insert(ACCEPT, "application/vnd.github.raw".parse().unwrap());
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    if !response.status().is_success() {
        return Err(GitHubApiError::Upstream(format!(
            "failed to fetch \"{path}\": HTTP {}",
            response.status()
        )));
    }
    let text = response
        .text()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    if text.len() > MAX_FILE_BYTES {
        return Err(GitHubApiError::InvalidRequest(format!(
            "file \"{path}\" exceeds the {MAX_FILE_BYTES} byte import limit"
        )));
    }
    if is_blank(Some(text.as_str())) {
        return Err(GitHubApiError::InvalidRequest(format!(
            "file \"{path}\" is empty"
        )));
    }
    Ok(text)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubCommitFile {
    pub path: String,
    pub content: String,
}

pub async fn create_github_commit(
    owner: &str,
    repo: &str,
    branch: &str,
    access_token: Option<&str>,
    message: &str,
    files: &[GitHubCommitFile],
) -> Result<String, GitHubApiError> {
    if files.is_empty() {
        return Err(GitHubApiError::InvalidRequest(
            "at least one file is required to create a git commit".to_string(),
        ));
    }
    let token = access_token
        .filter(|value| !is_blank(Some(value)))
        .ok_or_else(|| {
            GitHubApiError::InvalidRequest(
                "git_access_token is required to push commits to GitHub".to_string(),
            )
        })?;

    let client = github_client()?;
    let branch_ref = format!("heads/{branch}");
    let ref_url = format!(
        "{GITHUB_API_BASE}/repos/{}/{}/git/refs/{}",
        urlencoding::encode(owner),
        urlencoding::encode(repo),
        branch_ref
    );
    let ref_response = client
        .get(&ref_url)
        .headers(github_headers(Some(token))?)
        .send()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    if !ref_response.status().is_success() {
        return Err(GitHubApiError::Upstream(format!(
            "failed to resolve git ref \"{branch}\": HTTP {}",
            ref_response.status()
        )));
    }
    let ref_payload: serde_json::Value = ref_response
        .json()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    let parent_commit_sha = ref_payload
        .get("object")
        .and_then(|value| value.get("sha"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            GitHubApiError::Upstream("failed to resolve parent commit SHA".to_string())
        })?;

    let commit_url = format!(
        "{GITHUB_API_BASE}/repos/{}/{}/git/commits/{}",
        urlencoding::encode(owner),
        urlencoding::encode(repo),
        parent_commit_sha
    );
    let commit_response = client
        .get(&commit_url)
        .headers(github_headers(Some(token))?)
        .send()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    if !commit_response.status().is_success() {
        return Err(GitHubApiError::Upstream(format!(
            "failed to resolve parent commit tree: HTTP {}",
            commit_response.status()
        )));
    }
    let commit_payload: serde_json::Value = commit_response
        .json()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    let base_tree_sha = commit_payload
        .get("tree")
        .and_then(|value| value.get("sha"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            GitHubApiError::Upstream("failed to resolve parent commit tree SHA".to_string())
        })?;

    let mut tree_entries = Vec::with_capacity(files.len());
    for file in files {
        if file.content.len() > MAX_FILE_BYTES {
            return Err(GitHubApiError::InvalidRequest(format!(
                "file \"{}\" exceeds the {MAX_FILE_BYTES} byte sync limit",
                file.path
            )));
        }
        let blob_url = format!(
            "{GITHUB_API_BASE}/repos/{}/{}/git/blobs",
            urlencoding::encode(owner),
            urlencoding::encode(repo)
        );
        let blob_payload = serde_json::json!({
            "content": file.content,
            "encoding": "utf-8",
        });
        let blob_response = client
            .post(blob_url)
            .headers(github_headers(Some(token))?)
            .header(CONTENT_TYPE, "application/json")
            .json(&blob_payload)
            .send()
            .await
            .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
        if !blob_response.status().is_success() {
            return Err(GitHubApiError::Upstream(format!(
                "failed to create git blob for \"{}\": HTTP {}",
                file.path,
                blob_response.status()
            )));
        }
        let blob_body: serde_json::Value = blob_response
            .json()
            .await
            .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
        let blob_sha = blob_body
            .get("sha")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                GitHubApiError::Upstream(format!(
                    "failed to resolve git blob SHA for \"{}\"",
                    file.path
                ))
            })?;
        tree_entries.push(serde_json::json!({
            "path": file.path,
            "mode": "100644",
            "type": "blob",
            "sha": blob_sha,
        }));
    }

    let tree_url = format!(
        "{GITHUB_API_BASE}/repos/{}/{}/git/trees",
        urlencoding::encode(owner),
        urlencoding::encode(repo)
    );
    let tree_payload = serde_json::json!({
        "base_tree": base_tree_sha,
        "tree": tree_entries,
    });
    let tree_response = client
        .post(tree_url)
        .headers(github_headers(Some(token))?)
        .header(CONTENT_TYPE, "application/json")
        .json(&tree_payload)
        .send()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    if !tree_response.status().is_success() {
        return Err(GitHubApiError::Upstream(format!(
            "failed to create git tree: HTTP {}",
            tree_response.status()
        )));
    }
    let tree_body: serde_json::Value = tree_response
        .json()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    let tree_sha = tree_body
        .get("sha")
        .and_then(|value| value.as_str())
        .ok_or_else(|| GitHubApiError::Upstream("failed to resolve git tree SHA".to_string()))?;

    let new_commit_url = format!(
        "{GITHUB_API_BASE}/repos/{}/{}/git/commits",
        urlencoding::encode(owner),
        urlencoding::encode(repo)
    );
    let new_commit_payload = serde_json::json!({
        "message": message,
        "tree": tree_sha,
        "parents": [parent_commit_sha],
    });
    let new_commit_response = client
        .post(new_commit_url)
        .headers(github_headers(Some(token))?)
        .header(CONTENT_TYPE, "application/json")
        .json(&new_commit_payload)
        .send()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    if !new_commit_response.status().is_success() {
        return Err(GitHubApiError::Upstream(format!(
            "failed to create git commit: HTTP {}",
            new_commit_response.status()
        )));
    }
    let new_commit_body: serde_json::Value = new_commit_response
        .json()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    let commit_sha = new_commit_body
        .get("sha")
        .and_then(|value| value.as_str())
        .ok_or_else(|| GitHubApiError::Upstream("failed to resolve new commit SHA".to_string()))?;

    let update_ref_response = client
        .patch(&ref_url)
        .headers(github_headers(Some(token))?)
        .header(CONTENT_TYPE, "application/json")
        .json(&serde_json::json!({ "sha": commit_sha }))
        .send()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    if !update_ref_response.status().is_success() {
        return Err(GitHubApiError::Upstream(format!(
            "failed to update git ref \"{branch}\": HTTP {}",
            update_ref_response.status()
        )));
    }

    Ok(commit_sha.to_string())
}

async fn resolve_branch_sha(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
    branch: &str,
    access_token: Option<&str>,
) -> Result<String, GitHubApiError> {
    let url = format!(
        "{GITHUB_API_BASE}/repos/{}/{}/branches/{}",
        urlencoding::encode(owner),
        urlencoding::encode(repo),
        urlencoding::encode(branch)
    );
    let response = client
        .get(url)
        .headers(github_headers(access_token)?)
        .send()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    if !response.status().is_success() {
        let detail = if response.status() == reqwest::StatusCode::NOT_FOUND {
            format!("branch \"{branch}\" was not found")
        } else {
            format!("GitHub API returned HTTP {}", response.status())
        };
        return Err(GitHubApiError::Upstream(format!(
            "failed to resolve git branch: {detail}"
        )));
    }
    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))?;
    payload
        .get("commit")
        .and_then(|value| value.get("sha"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            GitHubApiError::Upstream("failed to resolve git branch commit SHA".to_string())
        })
}

fn github_client() -> Result<reqwest::Client, GitHubApiError> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|error| GitHubApiError::Upstream(error.to_string()))
}

fn github_headers(
    access_token: Option<&str>,
) -> Result<reqwest::header::HeaderMap, GitHubApiError> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(ACCEPT, "application/vnd.github+json".parse().unwrap());
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
    headers.insert(
        USER_AGENT,
        "SDKWork-Knowledgebase-GitImport/1.0".parse().unwrap(),
    );
    if let Some(token) = access_token.filter(|value| !is_blank(Some(value))) {
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", token.trim()).parse().map_err(
                |error: reqwest::header::InvalidHeaderValue| {
                    GitHubApiError::InvalidRequest(error.to_string())
                },
            )?,
        );
    }
    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_repo_url_accepts_https_repo_urls() {
        let parsed = parse_github_repo_url("https://github.com/octocat/Hello-World.git").unwrap();
        assert_eq!(parsed.owner, "octocat");
        assert_eq!(parsed.repo, "Hello-World");
    }
}
