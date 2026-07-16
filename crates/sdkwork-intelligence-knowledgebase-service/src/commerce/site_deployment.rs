use sdkwork_knowledgebase_contract::site_deployment::{
    KnowledgeSiteDeploymentPreview, KnowledgeSiteDeploymentRequest, KnowledgeSiteDeploymentResult,
};
use sdkwork_utils_rust::is_blank;
use thiserror::Error;

use crate::imports::KnowledgeDocumentMarkdownReader;
use crate::ports::commerce_store::{
    deployment_preview, deployment_result, validate_site_deployment_request,
    CreateSiteDeploymentRecord, KnowledgeSiteDeploymentStore, KnowledgeSiteDeploymentStoreError,
    KnowledgeSitePublisher, KnowledgeSitePublisherError, PublishKnowledgeSiteRequest,
};
use crate::ports::knowledge_document_store::KnowledgeDocumentStore;
use crate::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, PutKnowledgeObjectRequest,
};

const MAX_DEPLOYMENT_DOCUMENTS: usize = 64;
const MAX_DOCUMENT_BYTES: usize = 512 * 1024;
const MAX_DEPLOYMENT_TOTAL_BYTES: usize = 8 * 1024 * 1024;
const MAX_SITE_LOGO_DATA_URL_BYTES: usize = 512 * 1024;

#[derive(Debug, Error)]
pub enum KnowledgeSiteDeploymentServiceError {
    #[error("invalid site deployment request: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Store(#[from] KnowledgeSiteDeploymentStoreError),
    #[error(transparent)]
    DocumentStore(#[from] crate::ports::knowledge_document_store::KnowledgeDocumentStoreError),
    #[error("document markdown read failed: {0}")]
    DocumentContent(String),
    #[error("drive storage failed: {0}")]
    Storage(String),
    #[error("site deployment publisher is not configured")]
    PublisherUnavailable,
    #[error(transparent)]
    Publisher(#[from] KnowledgeSitePublisherError),
}

pub struct KnowledgeSiteDeploymentService<'a> {
    documents: &'a dyn KnowledgeDocumentStore,
    markdown: &'a dyn KnowledgeDocumentMarkdownReader,
    deployments: &'a dyn KnowledgeSiteDeploymentStore,
    drive: &'a dyn KnowledgeDriveStorage,
    publisher: Option<&'a dyn KnowledgeSitePublisher>,
}

impl<'a> KnowledgeSiteDeploymentService<'a> {
    pub fn new(
        documents: &'a dyn KnowledgeDocumentStore,
        markdown: &'a dyn KnowledgeDocumentMarkdownReader,
        deployments: &'a dyn KnowledgeSiteDeploymentStore,
        drive: &'a dyn KnowledgeDriveStorage,
        publisher: Option<&'a dyn KnowledgeSitePublisher>,
    ) -> Self {
        Self {
            documents,
            markdown,
            deployments,
            drive,
            publisher,
        }
    }

    pub async fn create_deployment(
        &self,
        tenant_id: u64,
        request: KnowledgeSiteDeploymentRequest,
        drive_space_id: Option<&str>,
    ) -> Result<KnowledgeSiteDeploymentResult, KnowledgeSiteDeploymentServiceError> {
        validate_site_deployment_request(&request)?;
        validate_site_deployment_content(&request)?;
        let publisher = self
            .publisher
            .ok_or(KnowledgeSiteDeploymentServiceError::PublisherUnavailable)?;

        let documents = self
            .documents
            .list_documents_for_space(request.space_id, MAX_DEPLOYMENT_DOCUMENTS as u32)
            .await?;

        let mut sections = Vec::new();
        let mut total_document_bytes = 0usize;
        for document in documents {
            let markdown = self
                .markdown
                .read_document_markdown(document.id)
                .await
                .map_err(KnowledgeSiteDeploymentServiceError::DocumentContent)?;
            if is_blank(Some(markdown.as_str())) {
                continue;
            }
            if markdown.len() > MAX_DOCUMENT_BYTES {
                continue;
            }
            total_document_bytes = total_document_bytes
                .checked_add(markdown.len())
                .ok_or_else(|| {
                    KnowledgeSiteDeploymentServiceError::InvalidRequest(
                        "site deployment content size overflow".to_string(),
                    )
                })?;
            if total_document_bytes > MAX_DEPLOYMENT_TOTAL_BYTES {
                return Err(KnowledgeSiteDeploymentServiceError::InvalidRequest(
                    format!(
                        "site deployment content exceeds {} bytes",
                        MAX_DEPLOYMENT_TOTAL_BYTES
                    ),
                ));
            }
            sections.push((document.title.clone(), markdown));
            if sections.len() >= MAX_DEPLOYMENT_DOCUMENTS {
                break;
            }
        }

        if sections.is_empty() {
            return Err(KnowledgeSiteDeploymentServiceError::InvalidRequest(
                "no publishable document content was found in the knowledge space".to_string(),
            ));
        }

        let site_name = request
            .site_name
            .as_deref()
            .filter(|value| !is_blank(Some(value)))
            .unwrap_or("Knowledge Base");
        let platform = request.platform.trim().to_ascii_lowercase();
        let html =
            render_static_site_html(site_name, request.site_logo_data_url.as_deref(), &sections);
        let slug = build_site_slug(site_name, request.space_id);
        let preview_object_key = format!(
            "site-deployments/{}/{}-{}.html",
            request.space_id, slug, platform
        );

        let preview_object_ref = self
            .drive
            .put_object(
                PutKnowledgeObjectRequest {
                    logical_path: preview_object_key.clone(),
                    object_role: "site_deployment_preview".to_string(),
                    content_type: "text/html; charset=utf-8".to_string(),
                    body: html.into_bytes(),
                    checksum_sha256_hex: None,
                    space_uuid: None,
                }
                .with_drive_space_id(drive_space_id),
            )
            .await
            .map_err(|error| KnowledgeSiteDeploymentServiceError::Storage(error.to_string()))?;

        let published = match publisher
            .publish_site(PublishKnowledgeSiteRequest {
                tenant_id,
                space_id: request.space_id,
                platform: platform.clone(),
                site_name: site_name.to_string(),
                custom_domain: request.custom_domain.clone(),
                preview_object_key: preview_object_key.clone(),
            })
            .await
        {
            Ok(published) => published,
            Err(error) => {
                if let Err(compensation_error) = self.drive.delete_object(&preview_object_ref).await
                {
                    return Err(KnowledgeSiteDeploymentServiceError::Storage(format!(
                        "site publisher failed ({error}); preview cleanup also failed ({compensation_error})"
                    )));
                }
                return Err(KnowledgeSiteDeploymentServiceError::Publisher(error));
            }
        };
        if let Err(error) = validate_published_url(&published.public_url) {
            if let Err(compensation_error) = self.drive.delete_object(&preview_object_ref).await {
                return Err(KnowledgeSiteDeploymentServiceError::Storage(format!(
                    "site publisher returned an invalid URL ({error}); preview cleanup also failed ({compensation_error})"
                )));
            }
            return Err(error);
        }

        let record = match self
            .deployments
            .create_deployment(CreateSiteDeploymentRecord {
                tenant_id,
                space_id: request.space_id,
                platform,
                site_name: Some(site_name.to_string()),
                custom_domain: request.custom_domain.clone(),
                site_logo_data_url: request.site_logo_data_url.clone(),
                deployed_url: published.public_url,
                preview_object_key,
            })
            .await
        {
            Ok(record) => record,
            Err(error) => {
                if let Err(compensation_error) = self.drive.delete_object(&preview_object_ref).await
                {
                    return Err(KnowledgeSiteDeploymentServiceError::Storage(format!(
                        "site deployment persistence failed ({error}); preview cleanup also failed ({compensation_error})"
                    )));
                }
                return Err(KnowledgeSiteDeploymentServiceError::Store(error));
            }
        };

        Ok(deployment_result(&record))
    }

    pub async fn retrieve_preview(
        &self,
        tenant_id: u64,
        deployment_id: u64,
        drive_space_id: Option<&str>,
    ) -> Result<KnowledgeSiteDeploymentPreview, KnowledgeSiteDeploymentServiceError> {
        let record = self
            .deployments
            .get_deployment(tenant_id, deployment_id)
            .await?;
        let object_ref = self
            .drive
            .head_object(
                HeadKnowledgeObjectRequest::managed_artifact(
                    record.preview_object_key.clone(),
                    "site_deployment_preview",
                )
                .with_drive_space_id(drive_space_id),
            )
            .await
            .map_err(|error| KnowledgeSiteDeploymentServiceError::Storage(error.to_string()))?;
        let html = self
            .drive
            .get_object_text(&object_ref)
            .await
            .map_err(|error| KnowledgeSiteDeploymentServiceError::Storage(error.to_string()))?;
        Ok(deployment_preview(html, record.id))
    }
}

fn validate_published_url(url: &str) -> Result<(), KnowledgeSiteDeploymentServiceError> {
    let authority = url.trim().strip_prefix("https://").unwrap_or_default();
    if authority.is_empty()
        || authority.starts_with('/')
        || authority.chars().any(char::is_whitespace)
    {
        return Err(KnowledgeSiteDeploymentServiceError::Publisher(
            KnowledgeSitePublisherError::InvalidRequest(
                "publisher must return an absolute HTTPS public URL".to_string(),
            ),
        ));
    }
    Ok(())
}

fn validate_site_deployment_content(
    request: &KnowledgeSiteDeploymentRequest,
) -> Result<(), KnowledgeSiteDeploymentServiceError> {
    let platform = request.platform.trim();
    if platform.len() > 64
        || !platform
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(KnowledgeSiteDeploymentServiceError::InvalidRequest(
            "platform must contain at most 64 ASCII letters, digits, hyphens, or underscores"
                .to_string(),
        ));
    }
    if request
        .site_name
        .as_deref()
        .is_some_and(|site_name| site_name.trim().chars().count() > 128)
    {
        return Err(KnowledgeSiteDeploymentServiceError::InvalidRequest(
            "site_name exceeds 128 characters".to_string(),
        ));
    }
    if let Some(logo) = request
        .site_logo_data_url
        .as_deref()
        .filter(|value| !is_blank(Some(value)))
    {
        let supported = [
            "data:image/png;base64,",
            "data:image/jpeg;base64,",
            "data:image/webp;base64,",
        ]
        .iter()
        .any(|prefix| logo.starts_with(prefix));
        if !supported || logo.len() > MAX_SITE_LOGO_DATA_URL_BYTES {
            return Err(KnowledgeSiteDeploymentServiceError::InvalidRequest(format!(
                "site_logo_data_url must be a PNG, JPEG, or WebP base64 data URL no larger than {} bytes",
                MAX_SITE_LOGO_DATA_URL_BYTES
            )));
        }
    }
    Ok(())
}

fn build_site_slug(site_name: &str, space_id: u64) -> String {
    let slug: String = site_name
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let collapsed = slug
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if collapsed.is_empty() {
        format!("kb-{space_id}")
    } else {
        collapsed.chars().take(48).collect()
    }
}

fn render_static_site_html(
    site_name: &str,
    logo_data_url: Option<&str>,
    sections: &[(String, String)],
) -> String {
    let escaped_site_name = html_escape(site_name);
    let logo_html = logo_data_url
        .filter(|value| !is_blank(Some(value)))
        .map(|url| {
            format!(
                r#"<img class="site-logo" src="{}" alt="logo" />"#,
                html_escape(url)
            )
        })
        .unwrap_or_default();
    let body = sections
        .iter()
        .map(|(title, markdown)| {
            format!(
                r#"<article class="doc"><h2>{}</h2><pre class="markdown">{}</pre></article>"#,
                html_escape(title),
                html_escape(markdown)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{escaped_site_name}</title>
  <style>
    body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; margin: 0; background: #f8fafc; color: #0f172a; }}
    header {{ background: #0f766e; color: white; padding: 24px 32px; display: flex; align-items: center; gap: 16px; }}
    .site-logo {{ width: 48px; height: 48px; border-radius: 12px; object-fit: cover; background: white; }}
    main {{ max-width: 920px; margin: 32px auto; padding: 0 16px 48px; }}
    .doc {{ background: white; border-radius: 16px; padding: 24px; margin-bottom: 20px; box-shadow: 0 8px 24px rgba(15,23,42,.06); }}
    .doc h2 {{ margin-top: 0; }}
    pre.markdown {{ white-space: pre-wrap; word-break: break-word; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 14px; line-height: 1.7; }}
  </style>
</head>
<body>
  <header>
    {logo_html}
    <h1>{escaped_site_name}</h1>
  </header>
  <main>
    {body}
  </main>
</body>
</html>"#
    )
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_site_slug_sanitizes_title() {
        assert_eq!(build_site_slug("Hello World!", 42), "hello-world");
    }

    #[test]
    fn static_site_html_escapes_site_name_and_document_content() {
        let html = render_static_site_html(
            "</title><script>alert(1)</script>",
            None,
            &[(
                "<img onerror=alert(1)>".to_string(),
                "<script>x</script>".to_string(),
            )],
        );

        assert!(!html.contains("<script>alert(1)</script>"));
        assert!(!html.contains("<script>x</script>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn site_deployment_rejects_path_injection_and_active_svg_logos() {
        let invalid_platform = KnowledgeSiteDeploymentRequest {
            space_id: 1,
            platform: "../public".to_string(),
            site_name: None,
            custom_domain: None,
            site_logo_data_url: None,
        };
        assert!(validate_site_deployment_content(&invalid_platform).is_err());

        let active_logo = KnowledgeSiteDeploymentRequest {
            space_id: 1,
            platform: "static".to_string(),
            site_name: None,
            custom_domain: None,
            site_logo_data_url: Some(
                "data:image/svg+xml;base64,PHN2ZyBvbmxvYWQ9YWxlcnQoMSk+".to_string(),
            ),
        };
        assert!(validate_site_deployment_content(&active_logo).is_err());
    }
}
