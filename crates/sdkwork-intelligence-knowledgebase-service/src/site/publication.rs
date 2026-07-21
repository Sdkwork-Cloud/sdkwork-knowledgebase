use pulldown_cmark::{html, CowStr, Event, Options, Parser, Tag};
use sdkwork_knowledgebase_contract::{
    KnowledgeSite, KnowledgeSiteHostBindingState, KnowledgeSiteHostBindingType,
    KnowledgeSitePublicationResult, KnowledgeSiteRelease,
    KnowledgeSiteReleaseList, RollbackKnowledgeSiteReleaseRequest, UpsertKnowledgeSiteRequest,
};
use sdkwork_utils_rust::{is_blank, sha256_hash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ports::{
    knowledge_drive_storage::{
        HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeStorageError,
    },
    knowledge_okf_concept_store::{KnowledgeOkfConceptStore, KnowledgeOkfConceptStoreError},
    knowledge_site_artifact_store::{
        KnowledgeSiteArtifactStore, KnowledgeSiteArtifactStoreError,
        WriteKnowledgeSiteArtifactRequest,
    },
    knowledge_site_store::{
        CompleteKnowledgeSiteReleaseRecord, CreateKnowledgeSiteHostBindingRecord,
        CreateKnowledgeSiteReleaseRecord, KnowledgeSiteStore, KnowledgeSiteStoreError,
        UpsertKnowledgeSiteRecord,
    },
    knowledge_space_store::{KnowledgeSpaceStore, KnowledgeSpaceStoreError},
};

const SITE_MANIFEST_SCHEMA_VERSION: u32 = 1;
const CONCEPT_PAGE_SIZE: u32 = 200;
const MAX_SITE_PAGES: usize = 10_000;
const MAX_CONCEPT_MARKDOWN_BYTES: u64 = 4 * 1024 * 1024;
const MAX_TOTAL_MARKDOWN_BYTES: usize = 128 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum KnowledgeSitePublicationServiceError {
    #[error("invalid site publication request: {0}")]
    InvalidRequest(String),
    #[error("site publication resource not found")]
    NotFound,
    #[error("site publication version conflict")]
    VersionConflict,
    #[error("site publication storage failed: {0}")]
    Storage(String),
    #[error("site publication internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSiteReleaseManifest {
    pub schema_version: u32,
    pub site_id: String,
    pub release_id: String,
    pub source_content_hash: String,
    pub homepage_path: String,
    pub pages: Vec<KnowledgeSiteReleaseManifestEntry>,
    pub search_index: KnowledgeSiteReleaseManifestEntry,
    pub sitemap: KnowledgeSiteReleaseManifestEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeSiteReleaseManifestEntry {
    pub public_path: String,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub content_type: String,
    pub content_length: u64,
    pub checksum_sha256_hex: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchIndex<'a> {
    schema_version: u32,
    release_id: String,
    pages: Vec<SearchIndexPage<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchIndexPage<'a> {
    path: &'a str,
    title: &'a str,
    description: &'a str,
    tags: &'a [String],
}

struct PublishedPage {
    concept_id: String,
    title: String,
    description: String,
    tags: Vec<String>,
    public_path: String,
    markdown: String,
}

pub struct KnowledgeSitePublicationService<'a> {
    tenant_id: u64,
    organization_id: u64,
    operator_id: &'a str,
    sites: &'a dyn KnowledgeSiteStore,
    spaces: &'a dyn KnowledgeSpaceStore,
    concepts: &'a dyn KnowledgeOkfConceptStore,
    knowledge_storage: &'a dyn KnowledgeDriveStorage,
    artifacts: &'a dyn KnowledgeSiteArtifactStore,
}

impl<'a> KnowledgeSitePublicationService<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tenant_id: u64,
        organization_id: u64,
        operator_id: &'a str,
        sites: &'a dyn KnowledgeSiteStore,
        spaces: &'a dyn KnowledgeSpaceStore,
        concepts: &'a dyn KnowledgeOkfConceptStore,
        knowledge_storage: &'a dyn KnowledgeDriveStorage,
        artifacts: &'a dyn KnowledgeSiteArtifactStore,
    ) -> Self {
        Self {
            tenant_id,
            organization_id,
            operator_id,
            sites,
            spaces,
            concepts,
            knowledge_storage,
            artifacts,
        }
    }

    pub async fn upsert_site(
        &self,
        request: UpsertKnowledgeSiteRequest,
    ) -> Result<KnowledgeSite, KnowledgeSitePublicationServiceError> {
        self.spaces
            .get_space(request.space_id)
            .await
            .map_err(map_space_error)?;
        let site = self
            .sites
            .upsert_site(UpsertKnowledgeSiteRecord {
                space_id: request.space_id,
                title: request.title,
                visibility: request.visibility,
                homepage_concept_id: request.homepage_concept_id,
                theme_id: request.theme_id,
                publish_mode: request.publish_mode,
                expected_version: request.expected_version,
            })
            .await
            .map_err(map_site_error)?;
        if site.version == 0 {
            self.ensure_system_host_binding(&site).await
        } else {
            Ok(site)
        }
    }

    pub async fn retrieve_site(
        &self,
        space_id: u64,
    ) -> Result<KnowledgeSite, KnowledgeSitePublicationServiceError> {
        self.sites
            .get_site_by_space(space_id)
            .await
            .map_err(map_site_error)
    }

    pub async fn publish(
        &self,
        site_id: u64,
        expected_site_version: u64,
        standalone_public_base_url: &str,
    ) -> Result<KnowledgeSitePublicationResult, KnowledgeSitePublicationServiceError> {
        let site = self.sites.get_site(site_id).await.map_err(map_site_error)?;
        if site.version != expected_site_version {
            return Err(KnowledgeSitePublicationServiceError::VersionConflict);
        }
        let space = self
            .spaces
            .get_space(site.space_id)
            .await
            .map_err(map_space_error)?;
        let pages = self.load_published_pages(&space.uuid, site.space_id).await?;
        if pages.is_empty() {
            return Err(KnowledgeSitePublicationServiceError::InvalidRequest(
                "at least one published OKF concept is required".to_string(),
            ));
        }
        let source_content_hash = source_content_hash(&site, &pages)?;
        let release = self
            .sites
            .create_release(CreateKnowledgeSiteReleaseRecord {
                site_id,
                source_content_hash: source_content_hash.clone(),
                previous_release_id: site.current_release_id,
            })
            .await
            .map_err(map_site_error)?;
        if release.lifecycle_state
            == sdkwork_knowledgebase_contract::KnowledgeSiteReleaseState::Ready
        {
            let site = self
                .sites
                .activate_release(site.id, release.id, expected_site_version)
                .await
                .map_err(map_site_error)?;
            return Ok(KnowledgeSitePublicationResult {
                public_url: standalone_site_url(standalone_public_base_url, site.space_id)?,
                site,
                release,
            });
        }

        let build_result = self.build_release(&site, &release, &pages).await;
        let ready_release = match build_result {
            Ok(ready_release) => ready_release,
            Err(error) => {
                let _ = self
                    .sites
                    .fail_release(release.id, "site_release_build_failed".to_string())
                    .await;
                return Err(error);
            }
        };
        let site = self
            .sites
            .activate_release(site.id, ready_release.id, expected_site_version)
            .await
            .map_err(map_site_error)?;
        Ok(KnowledgeSitePublicationResult {
            public_url: standalone_site_url(standalone_public_base_url, site.space_id)?,
            site,
            release: ready_release,
        })
    }

    pub async fn list_releases(
        &self,
        site_id: u64,
        cursor: Option<u64>,
        page_size: u32,
    ) -> Result<(KnowledgeSiteReleaseList, bool), KnowledgeSitePublicationServiceError> {
        let (items, next_cursor, has_more) = self
            .sites
            .list_releases_page(site_id, cursor, page_size)
            .await
            .map_err(map_site_error)?;
        Ok((
            KnowledgeSiteReleaseList {
                items,
                next_cursor: next_cursor.map(|value| value.to_string()),
            },
            has_more,
        ))
    }

    pub async fn rollback(
        &self,
        site_id: u64,
        request: RollbackKnowledgeSiteReleaseRequest,
    ) -> Result<KnowledgeSite, KnowledgeSitePublicationServiceError> {
        self.sites
            .activate_release(site_id, request.release_id, request.expected_site_version)
            .await
            .map_err(map_site_error)
    }

    async fn ensure_system_host_binding(
        &self,
        site: &KnowledgeSite,
    ) -> Result<KnowledgeSite, KnowledgeSitePublicationServiceError> {
        let host = format!("{}.kb.sdkwork.com", site.space_id);
        self.sites
            .create_host_binding(CreateKnowledgeSiteHostBindingRecord {
                site_id: site.id,
                binding_type: KnowledgeSiteHostBindingType::SystemId,
                normalized_host: host,
                canonical: true,
                lifecycle_state: KnowledgeSiteHostBindingState::Active,
                web_server_site_id: None,
                web_server_domain_id: None,
                web_server_deployment_id: None,
                expected_site_version: site.version,
            })
            .await
            .map_err(map_site_error)?;
        self.sites.get_site(site.id).await.map_err(map_site_error)
    }

    async fn load_published_pages(
        &self,
        space_uuid: &str,
        space_id: u64,
    ) -> Result<Vec<PublishedPage>, KnowledgeSitePublicationServiceError> {
        let mut cursor = None;
        let mut pages = Vec::new();
        let mut total_bytes = 0usize;
        loop {
            let (summaries, next_cursor, has_more) = self
                .concepts
                .list_concept_summaries_page(space_id, cursor, CONCEPT_PAGE_SIZE)
                .await
                .map_err(map_concept_error)?;
            for summary in summaries {
                validate_concept_public_path(&summary.concept_id)?;
                let object_ref = self
                    .knowledge_storage
                    .head_object(
                        HeadKnowledgeObjectRequest::managed_artifact(
                            summary.logical_path.clone(),
                            "okf_concept_published",
                        )
                        .with_space_uuid(space_uuid),
                    )
                    .await
                    .map_err(map_knowledge_storage_error)?;
                let markdown = self
                    .knowledge_storage
                    .get_object_text_bounded(&object_ref, MAX_CONCEPT_MARKDOWN_BYTES)
                    .await
                    .map_err(map_knowledge_storage_error)?;
                total_bytes = total_bytes.checked_add(markdown.len()).ok_or_else(|| {
                    KnowledgeSitePublicationServiceError::InvalidRequest(
                        "published concept size overflow".to_string(),
                    )
                })?;
                if total_bytes > MAX_TOTAL_MARKDOWN_BYTES {
                    return Err(KnowledgeSitePublicationServiceError::InvalidRequest(format!(
                        "published concepts exceed the {MAX_TOTAL_MARKDOWN_BYTES} byte site limit"
                    )));
                }
                pages.push(PublishedPage {
                    public_path: format!("{}/", summary.concept_id),
                    concept_id: summary.concept_id,
                    title: summary.title,
                    description: summary.description,
                    tags: summary.tags,
                    markdown,
                });
                if pages.len() > MAX_SITE_PAGES {
                    return Err(KnowledgeSitePublicationServiceError::InvalidRequest(format!(
                        "site exceeds the {MAX_SITE_PAGES} page limit"
                    )));
                }
            }
            if !has_more {
                break;
            }
            cursor = next_cursor;
            if cursor.is_none() {
                return Err(KnowledgeSitePublicationServiceError::Internal(
                    "published concept pagination returned has_more without a cursor".to_string(),
                ));
            }
        }
        pages.sort_by(|left, right| left.concept_id.cmp(&right.concept_id));
        Ok(pages)
    }

    async fn build_release(
        &self,
        site: &KnowledgeSite,
        release: &KnowledgeSiteRelease,
        pages: &[PublishedPage],
    ) -> Result<KnowledgeSiteRelease, KnowledgeSitePublicationServiceError> {
        let homepage_path = resolve_homepage_path(site, pages)?;
        let mut manifest_pages = Vec::with_capacity(pages.len());
        for page in pages {
            let html = render_page(site, page, pages, &homepage_path);
            let artifact = self
                .artifacts
                .write_artifact(self.artifact_request(
                    site,
                    release,
                    &page.public_path,
                    &format!("site-{}-{}.html", release.id, sha256_hash(page.public_path.as_bytes())),
                    "text/html; charset=utf-8",
                    html.into_bytes(),
                ))
                .await
                .map_err(map_artifact_error)?;
            manifest_pages.push(manifest_entry(page.public_path.clone(), artifact));
        }

        let search = SearchIndex {
            schema_version: SITE_MANIFEST_SCHEMA_VERSION,
            release_id: release.id.to_string(),
            pages: pages
                .iter()
                .map(|page| SearchIndexPage {
                    path: &page.public_path,
                    title: &page.title,
                    description: &page.description,
                    tags: &page.tags,
                })
                .collect(),
        };
        let search_bytes = serde_json::to_vec(&search).map_err(internal_error)?;
        let search_artifact = self
            .artifacts
            .write_artifact(self.artifact_request(
                site,
                release,
                "assets/search-index.json",
                &format!("site-{}-search-index.json", release.id),
                "application/json; charset=utf-8",
                search_bytes,
            ))
            .await
            .map_err(map_artifact_error)?;

        let sitemap = render_sitemap(site.space_id, pages);
        let sitemap_artifact = self
            .artifacts
            .write_artifact(self.artifact_request(
                site,
                release,
                "sitemap.xml",
                &format!("site-{}-sitemap.xml", release.id),
                "application/xml; charset=utf-8",
                sitemap.into_bytes(),
            ))
            .await
            .map_err(map_artifact_error)?;

        let manifest = KnowledgeSiteReleaseManifest {
            schema_version: SITE_MANIFEST_SCHEMA_VERSION,
            site_id: site.id.to_string(),
            release_id: release.id.to_string(),
            source_content_hash: release.source_content_hash.clone(),
            homepage_path,
            pages: manifest_pages,
            search_index: manifest_entry("assets/search-index.json".to_string(), search_artifact),
            sitemap: manifest_entry("sitemap.xml".to_string(), sitemap_artifact),
        };
        let manifest_bytes = serde_json::to_vec(&manifest).map_err(internal_error)?;
        let manifest_checksum = sha256_hash(&manifest_bytes);
        let manifest_artifact = self
            .artifacts
            .write_artifact(self.artifact_request(
                site,
                release,
                "manifest.json",
                &format!("site-{}-manifest.json", release.id),
                "application/json; charset=utf-8",
                manifest_bytes,
            ))
            .await
            .map_err(map_artifact_error)?;
        if manifest_artifact.checksum_sha256_hex != manifest_checksum {
            return Err(KnowledgeSitePublicationServiceError::Storage(
                "manifest checksum changed during Drive upload".to_string(),
            ));
        }
        self.sites
            .complete_release(CompleteKnowledgeSiteReleaseRecord {
                release_id: release.id,
                manifest_drive_uri: manifest_artifact.drive_uri,
                manifest_drive_space_id: manifest_artifact.drive_space_id,
                manifest_drive_node_id: manifest_artifact.drive_node_id,
                manifest_checksum_sha256_hex: manifest_checksum,
                page_count: u32::try_from(pages.len()).map_err(|_| {
                    KnowledgeSitePublicationServiceError::Internal(
                        "page count exceeds unsigned int32 range".to_string(),
                    )
                })?,
                asset_count: 2,
            })
            .await
            .map_err(map_site_error)
    }

    fn artifact_request(
        &self,
        site: &KnowledgeSite,
        release: &KnowledgeSiteRelease,
        public_path: &str,
        file_name: &str,
        content_type: &str,
        body: Vec<u8>,
    ) -> WriteKnowledgeSiteArtifactRequest {
        WriteKnowledgeSiteArtifactRequest {
            tenant_id: self.tenant_id,
            organization_id: self.organization_id,
            operator_id: self.operator_id.to_string(),
            site_id: site.id,
            release_id: release.id,
            public_path: public_path.to_string(),
            file_name: file_name.to_string(),
            content_type: content_type.to_string(),
            body,
        }
    }
}

fn source_content_hash(
    site: &KnowledgeSite,
    pages: &[PublishedPage],
) -> Result<String, KnowledgeSitePublicationServiceError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct HashInput<'a> {
        title: &'a str,
        visibility: &'a str,
        homepage_concept_id: &'a Option<String>,
        theme_id: &'a str,
        pages: Vec<HashPage<'a>>,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct HashPage<'a> {
        concept_id: &'a str,
        title: &'a str,
        description: &'a str,
        tags: &'a [String],
        markdown_sha256_hex: String,
    }
    let input = HashInput {
        title: &site.title,
        visibility: site.visibility.as_str(),
        homepage_concept_id: &site.homepage_concept_id,
        theme_id: &site.theme_id,
        pages: pages
            .iter()
            .map(|page| HashPage {
                concept_id: &page.concept_id,
                title: &page.title,
                description: &page.description,
                tags: &page.tags,
                markdown_sha256_hex: sha256_hash(page.markdown.as_bytes()),
            })
            .collect(),
    };
    serde_json::to_vec(&input)
        .map(|bytes| sha256_hash(&bytes))
        .map_err(internal_error)
}

fn resolve_homepage_path(
    site: &KnowledgeSite,
    pages: &[PublishedPage],
) -> Result<String, KnowledgeSitePublicationServiceError> {
    if let Some(homepage_concept_id) = site.homepage_concept_id.as_deref() {
        return pages
            .iter()
            .find(|page| page.concept_id == homepage_concept_id)
            .map(|page| page.public_path.clone())
            .ok_or_else(|| {
                KnowledgeSitePublicationServiceError::InvalidRequest(
                    "homepage_concept_id must reference a published concept".to_string(),
                )
            });
    }
    pages.first().map(|page| page.public_path.clone()).ok_or_else(|| {
        KnowledgeSitePublicationServiceError::InvalidRequest(
            "at least one published concept is required".to_string(),
        )
    })
}

fn render_page(
    site: &KnowledgeSite,
    page: &PublishedPage,
    pages: &[PublishedPage],
    homepage_path: &str,
) -> String {
    let body = render_safe_markdown(strip_front_matter(&page.markdown));
    let navigation = pages
        .iter()
        .map(|item| {
            format!(
                "<li><a href=\"{}\">{}</a></li>",
                relative_site_href(&page.public_path, &item.public_path),
                escape_html(&item.title)
            )
        })
        .collect::<String>();
    let canonical = format!(
        "https://{}.kb.sdkwork.com/{}",
        site.space_id, page.public_path
    );
    let home_href = relative_site_href(&page.public_path, homepage_path);
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"description\" content=\"{}\"><link rel=\"canonical\" href=\"{}\"><title>{} | {}</title><style>{}</style></head><body><header><a class=\"brand\" href=\"{}\">{}</a></header><div class=\"layout\"><nav aria-label=\"Knowledgebase\"><ul>{}</ul></nav><main><article>{}</article></main></div></body></html>",
        escape_html_attribute(&page.description),
        escape_html_attribute(&canonical),
        escape_html(&page.title),
        escape_html(&site.title),
        SITE_CSS,
        home_href,
        escape_html(&site.title),
        navigation,
        body
    )
}

fn render_safe_markdown(markdown: &str) -> String {
    let parser = Parser::new_ext(markdown, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH)
        .map(|event| match event {
            Event::Html(value) | Event::InlineHtml(value) => {
                Event::Text(CowStr::Boxed(value.to_string().into_boxed_str()))
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => Event::Start(Tag::Link {
                link_type,
                dest_url: CowStr::Boxed(safe_link_url(dest_url.as_ref()).into_boxed_str()),
                title,
                id,
            }),
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => Event::Start(Tag::Image {
                link_type,
                dest_url: CowStr::Boxed(safe_image_url(dest_url.as_ref()).into_boxed_str()),
                title,
                id,
            }),
            other => other,
        });
    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}

fn safe_link_url(value: &str) -> String {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("mailto:")
        || lower.starts_with('#')
        || is_safe_relative_url(trimmed)
    {
        trimmed.to_string()
    } else {
        "#".to_string()
    }
}

fn safe_image_url(value: &str) -> String {
    let trimmed = value.trim();
    if is_safe_relative_url(trimmed) {
        trimmed.to_string()
    } else {
        "#".to_string()
    }
}

fn is_safe_relative_url(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.starts_with('\\')
        && !value.contains(':')
        && !value
            .split(['/', '\\'])
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}

fn strip_front_matter(markdown: &str) -> &str {
    let Some(rest) = markdown.strip_prefix("---\n") else {
        return markdown;
    };
    rest.find("\n---\n")
        .map(|index| &rest[index + 5..])
        .unwrap_or(markdown)
}

fn validate_concept_public_path(
    concept_id: &str,
) -> Result<(), KnowledgeSitePublicationServiceError> {
    if concept_id.is_empty()
        || concept_id.len() > 512
        || concept_id.starts_with('/')
        || concept_id.ends_with('/')
        || concept_id.split('/').any(|segment| {
            segment.is_empty()
                || segment == "."
                || segment == ".."
                || !segment.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
                })
        })
    {
        return Err(KnowledgeSitePublicationServiceError::InvalidRequest(
            "published concept_id is not a safe public path".to_string(),
        ));
    }
    Ok(())
}

fn relative_site_href(from: &str, to: &str) -> String {
    let depth = from.trim_matches('/').split('/').count();
    format!("{}{}", "../".repeat(depth), to)
}

fn render_sitemap(space_id: u64, pages: &[PublishedPage]) -> String {
    let urls = pages
        .iter()
        .map(|page| {
            format!(
                "<url><loc>https://{}.kb.sdkwork.com/{}</loc></url>",
                space_id,
                escape_html(&page.public_path)
            )
        })
        .collect::<String>();
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{urls}</urlset>"
    )
}

fn manifest_entry(
    public_path: String,
    artifact: crate::ports::knowledge_site_artifact_store::KnowledgeSiteArtifactRef,
) -> KnowledgeSiteReleaseManifestEntry {
    KnowledgeSiteReleaseManifestEntry {
        public_path,
        drive_space_id: artifact.drive_space_id,
        drive_node_id: artifact.drive_node_id,
        content_type: artifact.content_type,
        content_length: artifact.content_length,
        checksum_sha256_hex: artifact.checksum_sha256_hex,
    }
}

fn standalone_site_url(
    public_base_url: &str,
    space_id: u64,
) -> Result<String, KnowledgeSitePublicationServiceError> {
    let base = public_base_url.trim().trim_end_matches('/');
    if is_blank(Some(base)) || !(base.starts_with("http://") || base.starts_with("https://")) {
        return Err(KnowledgeSitePublicationServiceError::InvalidRequest(
            "standalone public base URL must use http or https".to_string(),
        ));
    }
    Ok(format!("{base}/wiki/{space_id}/"))
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn escape_html_attribute(value: &str) -> String {
    escape_html(value).replace(['\r', '\n'], " ")
}

fn map_site_error(error: KnowledgeSiteStoreError) -> KnowledgeSitePublicationServiceError {
    match error {
        KnowledgeSiteStoreError::InvalidRequest(detail)
        | KnowledgeSiteStoreError::Conflict(detail) => {
            KnowledgeSitePublicationServiceError::InvalidRequest(detail)
        }
        KnowledgeSiteStoreError::NotFound => KnowledgeSitePublicationServiceError::NotFound,
        KnowledgeSiteStoreError::VersionConflict => {
            KnowledgeSitePublicationServiceError::VersionConflict
        }
        KnowledgeSiteStoreError::Internal(detail) => {
            KnowledgeSitePublicationServiceError::Internal(detail)
        }
    }
}

fn map_space_error(error: KnowledgeSpaceStoreError) -> KnowledgeSitePublicationServiceError {
    KnowledgeSitePublicationServiceError::Internal(error.to_string())
}

fn map_concept_error(
    error: KnowledgeOkfConceptStoreError,
) -> KnowledgeSitePublicationServiceError {
    KnowledgeSitePublicationServiceError::Internal(error.to_string())
}

fn map_knowledge_storage_error(
    error: KnowledgeStorageError,
) -> KnowledgeSitePublicationServiceError {
    match error {
        KnowledgeStorageError::NotFound(_) => KnowledgeSitePublicationServiceError::NotFound,
        KnowledgeStorageError::InvalidRequest(detail)
        | KnowledgeStorageError::IntegrityFailed(detail) => {
            KnowledgeSitePublicationServiceError::InvalidRequest(detail)
        }
        KnowledgeStorageError::Upstream(detail) | KnowledgeStorageError::Internal(detail) => {
            KnowledgeSitePublicationServiceError::Storage(detail)
        }
    }
}

fn map_artifact_error(
    error: KnowledgeSiteArtifactStoreError,
) -> KnowledgeSitePublicationServiceError {
    match error {
        KnowledgeSiteArtifactStoreError::InvalidRequest(detail) => {
            KnowledgeSitePublicationServiceError::InvalidRequest(detail)
        }
        KnowledgeSiteArtifactStoreError::NotFound => KnowledgeSitePublicationServiceError::NotFound,
        KnowledgeSiteArtifactStoreError::IntegrityFailed(detail)
        | KnowledgeSiteArtifactStoreError::Internal(detail) => {
            KnowledgeSitePublicationServiceError::Storage(detail)
        }
    }
}

fn internal_error(error: impl ToString) -> KnowledgeSitePublicationServiceError {
    KnowledgeSitePublicationServiceError::Internal(error.to_string())
}

const SITE_CSS: &str = "*{box-sizing:border-box}body{margin:0;color:#1f2933;background:#fff;font:16px/1.65 system-ui,sans-serif}header{height:56px;border-bottom:1px solid #d9e2ec;display:flex;align-items:center;padding:0 24px}.brand{font-weight:700;color:#102a43;text-decoration:none}.layout{display:grid;grid-template-columns:minmax(220px,280px) minmax(0,1fr);max-width:1280px;margin:0 auto}nav{padding:24px;border-right:1px solid #e6eaf0;min-height:calc(100vh - 56px)}nav ul{list-style:none;padding:0;margin:0}nav a{display:block;padding:6px 8px;color:#334e68;text-decoration:none}main{min-width:0;padding:40px clamp(24px,5vw,72px)}article{max-width:820px}pre{overflow:auto;background:#f4f6f8;padding:16px;border-radius:4px}code{font-family:ui-monospace,monospace}img{max-width:100%;height:auto}table{border-collapse:collapse;width:100%}th,td{border:1px solid #bcccdc;padding:8px;text-align:left}@media(max-width:760px){.layout{display:block}nav{min-height:auto;border-right:0;border-bottom:1px solid #e6eaf0}main{padding:24px}}";

#[cfg(test)]
mod tests {
    use super::{render_safe_markdown, safe_image_url, validate_concept_public_path};

    #[test]
    fn markdown_renderer_escapes_raw_html_and_rejects_active_urls() {
        let html = render_safe_markdown(
            "# Title\n\n<script>alert(1)</script>\n\n[x](javascript:alert(1)) ![x](data:image/svg+xml,x)",
        );
        assert!(!html.contains("<script>"));
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("data:image"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn concept_paths_reject_traversal_and_images_reject_remote_urls() {
        assert!(validate_concept_public_path("guides/getting-started").is_ok());
        assert!(validate_concept_public_path("../private").is_err());
        assert_eq!(safe_image_url("https://example.com/a.png"), "#");
    }
}
