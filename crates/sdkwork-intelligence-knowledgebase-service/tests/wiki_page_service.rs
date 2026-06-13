use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_object_ref_store::{
    CreateKnowledgeDriveObjectRefRecord, KnowledgeDriveObjectRefStore,
    KnowledgeDriveObjectRefStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeObjectRef, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_drive_workspace::{
    EnsureKnowledgeDriveNodesRequest, KnowledgeDriveWorkspace, KnowledgeDriveWorkspaceError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_file_entry_store::{
    CreateKnowledgeWikiFileEntryRecord, KnowledgeWikiFileEntryStore,
    KnowledgeWikiFileEntryStoreError,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_wiki_page_store::{
    AppendKnowledgeWikiLogEntryRecord, CreateKnowledgeWikiPageRevisionRecord,
    KnowledgeWikiPageProjection, KnowledgeWikiPageStore, KnowledgeWikiPageStoreError,
    MarkKnowledgeWikiCurrentRevisionRecord, UpsertKnowledgeWikiPageRecord,
};
use sdkwork_intelligence_knowledgebase_service::wiki::KnowledgeWikiPageService;
use sdkwork_knowledgebase_contract::wiki::{
    PublishKnowledgeWikiPageRequest, WikiLogEntry, WikiLogEventType, WikiPagePublishState,
    WikiPageSummary, WikiPageType, WikiRevisionReviewState,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;

#[tokio::test]
async fn wiki_page_service_publishes_page_and_rebuilds_standard_files() {
    let drive = MemoryDrive::default();
    let object_refs = MemoryObjectRefStore::default();
    let wiki_pages = MemoryWikiPageStore::default();
    let file_entries = MemoryWikiFileEntryStore::default();
    let workspace = MemoryDriveWorkspace::default();
    let service = KnowledgeWikiPageService::new(&drive, &object_refs, &wiki_pages)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);

    let publication = service
        .publish_page(
            PublishKnowledgeWikiPageRequest {
                space_id: 7,
                slug: "entity-name".to_string(),
                title: "Entity Name".to_string(),
                page_type: WikiPageType::Entity,
                summary: "Entity summary.".to_string(),
                markdown: "# Entity Name\n\nA durable synthesis.".to_string(),
                source_count: 2,
                tags: vec!["entity".to_string()],
                actor: "system".to_string(),
            },
            Some("drv-kb-001"),
        )
        .await
        .unwrap();

    assert_eq!(
        publication.current_file_path,
        "wiki/pages/entities/entity-name/current.md"
    );
    assert_eq!(
        publication.revision_file_path,
        "wiki/pages/entities/entity-name/revisions/r1.md"
    );
    assert_eq!(
        publication.page.publish_state,
        WikiPagePublishState::Published
    );
    assert_eq!(
        publication.revision.review_state,
        WikiRevisionReviewState::Approved
    );

    assert_eq!(
        drive.body_at("wiki/pages/entities/entity-name/current.md"),
        Some("# Entity Name\n\nA durable synthesis.".to_string())
    );
    let current_ref = object_refs.ref_by_path("wiki/pages/entities/entity-name/current.md");
    assert!(current_ref.is_some());
    let revision_ref = object_refs.ref_by_path("wiki/pages/entities/entity-name/revisions/r1.md");
    assert!(revision_ref.is_some());

    assert!(file_entries.paths().contains(&"wiki/index.md".to_string()));
    assert!(file_entries.paths().contains(&"wiki/log.md".to_string()));
    assert!(workspace
        .paths()
        .contains(&"wiki/pages/entities/entity-name/current.md".to_string()));
    assert!(workspace
        .paths()
        .contains(&"wiki/pages/entities/entity-name/revisions/r1.md".to_string()));
    assert!(workspace.paths().contains(&"wiki/index.md".to_string()));
    assert!(workspace.paths().contains(&"wiki/log.md".to_string()));

    let index_ref = file_entries.object_key_for("wiki/index.md").unwrap();
    let index_content = drive.body_at(&index_ref).unwrap();
    assert!(index_content.contains("[[entity-name|Entity Name]]"));

    let log_ref = file_entries.object_key_for("wiki/log.md").unwrap();
    let log_content = drive.body_at(&log_ref).unwrap();
    assert!(log_content.contains("publish | Published Entity Name"));
}

#[tokio::test]
async fn wiki_page_service_requires_drive_space_before_publishing_when_workspace_enabled() {
    let drive = MemoryDrive::default();
    let object_refs = MemoryObjectRefStore::default();
    let wiki_pages = MemoryWikiPageStore::default();
    let file_entries = MemoryWikiFileEntryStore::default();
    let workspace = MemoryDriveWorkspace::default();
    let service = KnowledgeWikiPageService::new(&drive, &object_refs, &wiki_pages)
        .with_file_entry_store(&file_entries)
        .with_drive_workspace(&workspace);

    let error = service
        .publish_page(
            PublishKnowledgeWikiPageRequest {
                space_id: 7,
                slug: "entity-name".to_string(),
                title: "Entity Name".to_string(),
                page_type: WikiPageType::Entity,
                summary: "Entity summary.".to_string(),
                markdown: "# Entity Name\n\nA durable synthesis.".to_string(),
                source_count: 2,
                tags: vec!["entity".to_string()],
                actor: "system".to_string(),
            },
            None,
        )
        .await
        .unwrap_err();

    assert!(error.to_string().contains("drive_space_id is required"));
    assert_eq!(drive.object_count(), 0);
    assert_eq!(object_refs.ref_count(), 0);
    assert_eq!(wiki_pages.page_count(), 0);
    assert_eq!(wiki_pages.revision_count(), 0);
    assert_eq!(wiki_pages.log_count(), 0);
    assert!(file_entries.paths().is_empty());
    assert!(workspace.paths().is_empty());
}

#[derive(Default)]
struct MemoryDrive {
    objects: Mutex<HashMap<String, (KnowledgeObjectRef, Vec<u8>)>>,
}

impl MemoryDrive {
    fn object_count(&self) -> usize {
        self.objects.lock().unwrap().len()
    }

    fn body_at(&self, logical_path: &str) -> Option<String> {
        self.objects
            .lock()
            .unwrap()
            .get(logical_path)
            .and_then(|(_, body)| String::from_utf8(body.clone()).ok())
    }
}

#[async_trait]
impl KnowledgeDriveStorage for MemoryDrive {
    async fn put_object(
        &self,
        request: PutKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        let checksum = request
            .checksum_sha256_hex
            .clone()
            .unwrap_or_else(|| checksum_sha256_hex(&request.body));
        let object_ref = KnowledgeObjectRef {
            storage_provider_id: "provider-kb".to_string(),
            bucket: "knowledgebase-test".to_string(),
            object_key: request.logical_path.clone(),
            logical_path: request.logical_path.clone(),
            object_role: request.object_role,
            content_type: request.content_type,
            size_bytes: request.body.len() as u64,
            checksum_sha256_hex: Some(checksum),
            etag: None,
            version_id: Some("v1".to_string()),
        };
        self.objects
            .lock()
            .unwrap()
            .insert(request.logical_path, (object_ref.clone(), request.body));
        Ok(object_ref)
    }

    async fn head_object(
        &self,
        request: HeadKnowledgeObjectRequest,
    ) -> Result<KnowledgeObjectRef, KnowledgeStorageError> {
        self.objects
            .lock()
            .unwrap()
            .get(&request.object_key)
            .map(|(object_ref, _)| object_ref.clone())
            .ok_or_else(|| KnowledgeStorageError::NotFound(request.object_key))
    }

    async fn get_object_text(
        &self,
        object_ref: &KnowledgeObjectRef,
    ) -> Result<String, KnowledgeStorageError> {
        self.body_at(&object_ref.object_key)
            .ok_or_else(|| KnowledgeStorageError::NotFound(object_ref.object_key.clone()))
    }
}

fn checksum_sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

#[derive(Default)]
struct MemoryObjectRefStore {
    next_id: Mutex<u64>,
    refs: Mutex<Vec<sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef>>,
}

impl MemoryObjectRefStore {
    fn ref_count(&self) -> usize {
        self.refs.lock().unwrap().len()
    }

    fn ref_by_path(
        &self,
        logical_path: &str,
    ) -> Option<sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef> {
        self.refs
            .lock()
            .unwrap()
            .iter()
            .find(|object_ref| object_ref.logical_path.as_deref() == Some(logical_path))
            .cloned()
    }
}

#[async_trait]
impl KnowledgeDriveObjectRefStore for MemoryObjectRefStore {
    async fn create_object_ref(
        &self,
        record: CreateKnowledgeDriveObjectRefRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef,
        KnowledgeDriveObjectRefStoreError,
    > {
        let mut next_id = self.next_id.lock().unwrap();
        *next_id += 1;
        let object_ref = sdkwork_knowledgebase_contract::KnowledgeDriveObjectRef {
            id: *next_id,
            space_id: record.space_id,
            drive_space_id: record.drive_space_id,
            drive_node_id: record.drive_node_id,
            logical_path: record.logical_path,
            drive_provider_kind: record.drive_provider_kind,
            drive_storage_provider_id: record.drive_storage_provider_id,
            drive_bucket: record.drive_bucket,
            drive_object_key: record.drive_object_key,
            drive_object_version: record.drive_object_version,
            drive_etag: record.drive_etag,
            content_type: record.content_type,
            size_bytes: record.size_bytes,
            checksum_sha256_hex: record.checksum_sha256_hex,
            object_role: record.object_role,
            access_mode: record.access_mode,
        };
        self.refs.lock().unwrap().push(object_ref.clone());
        Ok(object_ref)
    }
}

#[derive(Default)]
struct MemoryWikiPageStore {
    next_page_id: Mutex<u64>,
    next_revision_id: Mutex<u64>,
    pages: Mutex<Vec<sdkwork_knowledgebase_contract::wiki::KnowledgeWikiPage>>,
    revisions: Mutex<Vec<sdkwork_knowledgebase_contract::wiki::KnowledgeWikiPageRevision>>,
    logs: Mutex<Vec<WikiLogEntry>>,
}

impl MemoryWikiPageStore {
    fn page_count(&self) -> usize {
        self.pages.lock().unwrap().len()
    }

    fn revision_count(&self) -> usize {
        self.revisions.lock().unwrap().len()
    }

    fn log_count(&self) -> usize {
        self.logs.lock().unwrap().len()
    }
}

#[async_trait]
impl KnowledgeWikiPageStore for MemoryWikiPageStore {
    async fn upsert_page(
        &self,
        record: UpsertKnowledgeWikiPageRecord,
    ) -> Result<sdkwork_knowledgebase_contract::wiki::KnowledgeWikiPage, KnowledgeWikiPageStoreError>
    {
        let mut pages = self.pages.lock().unwrap();
        if let Some(page) = pages
            .iter_mut()
            .find(|page| page.space_id == record.space_id && page.slug == record.slug)
        {
            page.title = record.title;
            page.page_type = record.page_type;
            page.logical_path = record.logical_path;
            page.summary = record.summary;
            page.source_count = record.source_count;
            page.tags = record.tags;
            page.publish_state = record.publish_state;
            return Ok(page.clone());
        }
        let mut next_page_id = self.next_page_id.lock().unwrap();
        *next_page_id += 1;
        let page = sdkwork_knowledgebase_contract::wiki::KnowledgeWikiPage {
            id: *next_page_id,
            space_id: record.space_id,
            slug: record.slug,
            title: record.title,
            page_type: record.page_type,
            logical_path: record.logical_path,
            summary: record.summary,
            source_count: record.source_count,
            tags: record.tags,
            current_revision_id: None,
            publish_state: record.publish_state,
            updated_at: "2026-06-04T12:00:00Z".to_string(),
        };
        pages.push(page.clone());
        Ok(page)
    }

    async fn create_revision(
        &self,
        record: CreateKnowledgeWikiPageRevisionRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::wiki::KnowledgeWikiPageRevision,
        KnowledgeWikiPageStoreError,
    > {
        let mut next_revision_id = self.next_revision_id.lock().unwrap();
        *next_revision_id += 1;
        let revision = sdkwork_knowledgebase_contract::wiki::KnowledgeWikiPageRevision {
            id: *next_revision_id,
            page_id: record.page_id,
            revision_no: record.revision_no,
            markdown_object_ref_id: record.markdown_object_ref_id,
            content_hash: record.content_hash,
            review_state: record.review_state,
            created_at: "2026-06-04T12:00:00Z".to_string(),
        };
        self.revisions.lock().unwrap().push(revision.clone());
        Ok(revision)
    }

    async fn next_revision_no(&self, page_id: u64) -> Result<u64, KnowledgeWikiPageStoreError> {
        let revisions = self.revisions.lock().unwrap();
        let max_revision = revisions
            .iter()
            .filter(|revision| revision.page_id == page_id)
            .map(|revision| revision.revision_no)
            .max()
            .unwrap_or(0);
        Ok(max_revision + 1)
    }

    async fn mark_current_revision(
        &self,
        record: MarkKnowledgeWikiCurrentRevisionRecord,
    ) -> Result<sdkwork_knowledgebase_contract::wiki::KnowledgeWikiPage, KnowledgeWikiPageStoreError>
    {
        let mut pages = self.pages.lock().unwrap();
        let page = pages
            .iter_mut()
            .find(|page| page.id == record.page_id)
            .ok_or_else(|| KnowledgeWikiPageStoreError::Internal("missing page".to_string()))?;
        page.current_revision_id = Some(record.revision_id);
        page.publish_state = record.publish_state;
        Ok(page.clone())
    }

    async fn list_page_summaries(
        &self,
        space_id: u64,
    ) -> Result<Vec<WikiPageSummary>, KnowledgeWikiPageStoreError> {
        Ok(self
            .pages
            .lock()
            .unwrap()
            .iter()
            .filter(|page| page.space_id == space_id)
            .map(|page| WikiPageSummary {
                title: page.title.clone(),
                slug: page.slug.clone(),
                page_type: page.page_type,
                logical_path: page.logical_path.clone(),
                summary: page.summary.clone(),
                source_count: page.source_count,
                updated_at: page.updated_at.clone(),
                tags: page.tags.clone(),
            })
            .collect())
    }

    async fn append_log_entry(
        &self,
        record: AppendKnowledgeWikiLogEntryRecord,
    ) -> Result<WikiLogEntry, KnowledgeWikiPageStoreError> {
        let entry = WikiLogEntry {
            occurred_at: record.event_time,
            event_type: WikiLogEventType::Publish,
            title: record.title,
            actor: record.actor,
            affected_pages: record.affected_pages,
            audit_event_id: record.audit_event_id,
            warnings: record.warnings,
        };
        self.logs.lock().unwrap().push(entry.clone());
        Ok(entry)
    }

    async fn list_log_entries(
        &self,
        _space_id: u64,
    ) -> Result<Vec<WikiLogEntry>, KnowledgeWikiPageStoreError> {
        Ok(self.logs.lock().unwrap().clone())
    }

    async fn batch_page_projections_by_paths(
        &self,
        _space_id: u64,
        _logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeWikiPageProjection>, KnowledgeWikiPageStoreError> {
        Ok(vec![])
    }
}

#[derive(Default)]
struct MemoryWikiFileEntryStore {
    entries: Mutex<Vec<CreateKnowledgeWikiFileEntryRecord>>,
}

impl MemoryWikiFileEntryStore {
    fn paths(&self) -> Vec<String> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .map(|entry| entry.logical_path.clone())
            .collect()
    }

    fn object_key_for(&self, logical_path: &str) -> Option<String> {
        self.entries
            .lock()
            .unwrap()
            .iter()
            .find(|entry| entry.logical_path == logical_path)
            .map(|entry| entry.drive_object_key.clone())
    }
}

#[async_trait]
impl KnowledgeWikiFileEntryStore for MemoryWikiFileEntryStore {
    async fn create_file_entry(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::wiki_file::KnowledgeWikiFileEntry,
        KnowledgeWikiFileEntryStoreError,
    > {
        self.upsert_file_entry(record).await
    }

    async fn upsert_file_entry(
        &self,
        record: CreateKnowledgeWikiFileEntryRecord,
    ) -> Result<
        sdkwork_knowledgebase_contract::wiki_file::KnowledgeWikiFileEntry,
        KnowledgeWikiFileEntryStoreError,
    > {
        self.entries.lock().unwrap().push(record.clone());
        Ok(
            sdkwork_knowledgebase_contract::wiki_file::KnowledgeWikiFileEntry {
                id: self.entries.lock().unwrap().len() as u64,
                space_id: record.space_id,
                logical_path: record.logical_path,
                entry_type: record.entry_type,
                artifact_role: record.artifact_role,
                drive_bucket: record.drive_bucket,
                drive_object_key: record.drive_object_key,
                checksum_sha256_hex: record.checksum_sha256_hex,
            },
        )
    }
}

#[derive(Default)]
struct MemoryDriveWorkspace {
    paths: Mutex<Vec<String>>,
}

impl MemoryDriveWorkspace {
    fn paths(&self) -> Vec<String> {
        self.paths.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeDriveWorkspace for MemoryDriveWorkspace {
    async fn ensure_nodes(
        &self,
        request: EnsureKnowledgeDriveNodesRequest,
    ) -> Result<(), KnowledgeDriveWorkspaceError> {
        for node in request.nodes {
            self.paths.lock().unwrap().push(node.logical_path);
        }
        Ok(())
    }
}
