use async_trait::async_trait;
use sdkwork_knowledgebase_contract::browser::{
    KnowledgeBrowserNodeType, KnowledgeBrowserView, ListKnowledgeBrowserRequest,
};
use sdkwork_knowledgebase_contract::space::{KnowledgeSpace, KnowledgeSpaceStatus};
use sdkwork_knowledgebase_contract::wiki::WikiPagePublishState;
use sdkwork_knowledgebase_product::browser::KnowledgeBrowserService;
use sdkwork_knowledgebase_product::ports::knowledge_browser_projection_store::{
    KnowledgeBrowserDocumentProjection, KnowledgeBrowserProjectionStore,
    KnowledgeBrowserProjectionStoreError, KnowledgeBrowserWikiPageProjection,
};
use sdkwork_knowledgebase_product::ports::knowledge_drive_node_tree::{
    DriveNodeKind, KnowledgeDriveNodePage, KnowledgeDriveNodeSummary, KnowledgeDriveNodeTree,
    KnowledgeDriveNodeTreeError, ListKnowledgeDriveNodeChildrenRequest,
    ResolveKnowledgeDriveNodePathRequest,
};
use sdkwork_knowledgebase_product::ports::knowledge_space_store::{
    CreateKnowledgeSpaceRecord, KnowledgeSpaceStore, KnowledgeSpaceStoreError,
};
use std::collections::HashMap;
use std::sync::Mutex;

#[tokio::test]
async fn browser_lists_drive_children_and_batches_document_projection() {
    let spaces = MemorySpaceStore::bound("drv-kb-001");
    let drive_tree = RecordingDriveTree::with_nodes(vec![
        KnowledgeDriveNodeSummary {
            drive_node_id: "node-folder".to_string(),
            parent_drive_node_id: Some("root".to_string()),
            kind: DriveNodeKind::Folder,
            name: "Papers".to_string(),
            path: "/Files/Papers".to_string(),
            content_type: None,
            size_bytes: None,
            children_count: Some(3),
            updated_at: "2026-06-04T12:00:00Z".to_string(),
        },
        KnowledgeDriveNodeSummary {
            drive_node_id: "node-pdf".to_string(),
            parent_drive_node_id: Some("root".to_string()),
            kind: DriveNodeKind::File,
            name: "LLM-Wiki Paper.pdf".to_string(),
            path: "/Files/Papers/LLM-Wiki Paper.pdf".to_string(),
            content_type: Some("application/pdf".to_string()),
            size_bytes: Some(42),
            children_count: None,
            updated_at: "2026-06-04T12:01:00Z".to_string(),
        },
    ]);
    let projections =
        RecordingProjectionStore::with_documents(vec![KnowledgeBrowserDocumentProjection {
            drive_node_id: "node-pdf".to_string(),
            document_id: 11,
            current_version_id: Some(7),
            ingest_state: "completed".to_string(),
            parse_state: "succeeded".to_string(),
            index_state: "indexed".to_string(),
            wiki_state: "candidate_ready".to_string(),
        }]);
    let service = KnowledgeBrowserService::new(&spaces, &drive_tree, &projections);

    let page = service
        .list(ListKnowledgeBrowserRequest {
            space_id: 1,
            parent_id: Some("root".to_string()),
            view: KnowledgeBrowserView::Files,
            cursor: Some("cursor-a".to_string()),
            page_size: Some(50),
        })
        .await
        .unwrap();

    assert_eq!(page.space_id, 1);
    assert_eq!(page.drive_space_id, "drv-kb-001");
    assert_eq!(page.parent_id.as_deref(), Some("root"));
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].node_type, KnowledgeBrowserNodeType::Folder);
    assert_eq!(page.items[0].drive_node_id.as_deref(), Some("node-folder"));
    assert_eq!(page.items[0].children_count, Some(3));
    assert_eq!(page.items[1].node_type, KnowledgeBrowserNodeType::Document);
    assert_eq!(page.items[1].document_id, Some(11));
    assert_eq!(page.items[1].document_version_id, Some(7));
    assert_eq!(page.items[1].ingest_state.as_deref(), Some("completed"));
    assert_eq!(page.next_cursor.as_deref(), Some("next-cursor"));
    assert_eq!(drive_tree.calls(), 1);
    assert_eq!(projections.calls(), 1);
    assert_eq!(
        projections.requested_drive_node_ids(),
        vec!["node-folder".to_string(), "node-pdf".to_string()]
    );
}

#[tokio::test]
async fn browser_caps_page_size_to_prevent_unbounded_directory_scans() {
    let spaces = MemorySpaceStore::bound("drv-kb-001");
    let drive_tree = RecordingDriveTree::with_nodes(vec![]);
    let projections = RecordingProjectionStore::default();
    let service = KnowledgeBrowserService::new(&spaces, &drive_tree, &projections);

    let page = service
        .list(ListKnowledgeBrowserRequest {
            space_id: 1,
            parent_id: None,
            view: KnowledgeBrowserView::Files,
            cursor: None,
            page_size: Some(10_000),
        })
        .await
        .unwrap();

    assert_eq!(page.page_size, 200);
    assert_eq!(drive_tree.last_page_size(), Some(200));
}

#[tokio::test]
async fn browser_wiki_root_lists_children_under_llm_wiki_drive_folder() {
    let spaces = MemorySpaceStore::bound("drv-kb-001");
    let drive_tree = RecordingDriveTree::with_nodes(vec![KnowledgeDriveNodeSummary {
        drive_node_id: "node-index".to_string(),
        parent_drive_node_id: Some("node-wiki-root".to_string()),
        kind: DriveNodeKind::File,
        name: "index.md".to_string(),
        path: "wiki/index.md".to_string(),
        content_type: Some("text/markdown; charset=utf-8".to_string()),
        size_bytes: Some(128),
        children_count: None,
        updated_at: "2026-06-04T12:02:00Z".to_string(),
    }])
    .with_resolved_path("wiki", Some("node-wiki-root"))
    .expect_parent_id(Some("node-wiki-root"));
    let projections =
        RecordingProjectionStore::with_wiki_pages(vec![KnowledgeBrowserWikiPageProjection {
            logical_path: "wiki/index.md".to_string(),
            page_id: 21,
            current_revision_id: Some(34),
            publish_state: WikiPagePublishState::Published,
        }]);
    let service = KnowledgeBrowserService::new(&spaces, &drive_tree, &projections);

    let page = service
        .list(ListKnowledgeBrowserRequest {
            space_id: 1,
            parent_id: None,
            view: KnowledgeBrowserView::Wiki,
            cursor: None,
            page_size: Some(50),
        })
        .await
        .unwrap();

    assert_eq!(page.view, KnowledgeBrowserView::Wiki);
    assert_eq!(page.parent_id.as_deref(), Some("node-wiki-root"));
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].node_type, KnowledgeBrowserNodeType::WikiPage);
    assert_eq!(page.items[0].name, "index.md");
    assert_eq!(page.items[0].wiki_page_id, Some(21));
    assert_eq!(page.items[0].wiki_revision_id, Some(34));
    assert_eq!(page.items[0].wiki_state.as_deref(), Some("published"));
    assert_eq!(drive_tree.resolved_paths(), vec!["wiki".to_string()]);
    assert_eq!(drive_tree.calls(), 1);
    assert_eq!(projections.calls(), 1);
    assert_eq!(projections.wiki_calls(), 1);
}

#[tokio::test]
async fn browser_outputs_root_lists_children_under_standard_output_drive_folder() {
    let spaces = MemorySpaceStore::bound("drv-kb-001");
    let drive_tree = RecordingDriveTree::with_nodes(vec![KnowledgeDriveNodeSummary {
        drive_node_id: "node-answer".to_string(),
        parent_drive_node_id: Some("node-output-root".to_string()),
        kind: DriveNodeKind::Folder,
        name: "answers".to_string(),
        path: "output/answers".to_string(),
        content_type: None,
        size_bytes: None,
        children_count: Some(12),
        updated_at: "2026-06-04T12:03:00Z".to_string(),
    }])
    .with_resolved_path("output", Some("node-output-root"))
    .expect_parent_id(Some("node-output-root"));
    let projections = RecordingProjectionStore::default();
    let service = KnowledgeBrowserService::new(&spaces, &drive_tree, &projections);

    let page = service
        .list(ListKnowledgeBrowserRequest {
            space_id: 1,
            parent_id: None,
            view: KnowledgeBrowserView::Outputs,
            cursor: None,
            page_size: Some(50),
        })
        .await
        .unwrap();

    assert_eq!(page.view, KnowledgeBrowserView::Outputs);
    assert_eq!(page.parent_id.as_deref(), Some("node-output-root"));
    assert_eq!(page.items.len(), 1);
    assert_eq!(
        page.items[0].node_type,
        KnowledgeBrowserNodeType::VirtualFolder
    );
    assert_eq!(page.items[0].name, "answers");
    assert_eq!(drive_tree.resolved_paths(), vec!["output".to_string()]);
    assert_eq!(drive_tree.calls(), 1);
}

#[tokio::test]
async fn browser_rejects_spaces_without_drive_space_binding() {
    let spaces = MemorySpaceStore::unbound();
    let drive_tree = RecordingDriveTree::with_nodes(vec![]);
    let projections = RecordingProjectionStore::default();
    let service = KnowledgeBrowserService::new(&spaces, &drive_tree, &projections);

    let error = service
        .list(ListKnowledgeBrowserRequest {
            space_id: 1,
            parent_id: None,
            view: KnowledgeBrowserView::Files,
            cursor: None,
            page_size: None,
        })
        .await
        .unwrap_err();

    assert!(error.to_string().contains("drive space is not bound"));
    assert_eq!(drive_tree.calls(), 0);
    assert_eq!(projections.calls(), 0);
}

struct MemorySpaceStore {
    space: Mutex<KnowledgeSpace>,
}

impl MemorySpaceStore {
    fn bound(drive_space_id: &str) -> Self {
        Self {
            space: Mutex::new(space(Some(drive_space_id.to_string()))),
        }
    }

    fn unbound() -> Self {
        Self {
            space: Mutex::new(space(None)),
        }
    }
}

#[async_trait]
impl KnowledgeSpaceStore for MemorySpaceStore {
    async fn create_space(
        &self,
        _record: CreateKnowledgeSpaceRecord,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        Ok(self.space.lock().unwrap().clone())
    }

    async fn get_space(&self, _space_id: u64) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        Ok(self.space.lock().unwrap().clone())
    }

    async fn mark_drive_space_bound(
        &self,
        _space_id: u64,
        drive_space_id: String,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        let mut space = self.space.lock().unwrap();
        space.drive_space_id = Some(drive_space_id);
        Ok(space.clone())
    }

    async fn mark_llm_wiki_initialized(
        &self,
        _space_id: u64,
    ) -> Result<KnowledgeSpace, KnowledgeSpaceStoreError> {
        let mut space = self.space.lock().unwrap();
        space.llm_wiki_initialized = true;
        Ok(space.clone())
    }
}

#[derive(Default)]
struct RecordingDriveTree {
    nodes: Vec<KnowledgeDriveNodeSummary>,
    resolved_paths: Mutex<Vec<String>>,
    path_bindings: HashMap<String, Option<String>>,
    expected_parent_id: Option<Option<String>>,
    calls: Mutex<u32>,
    last_page_size: Mutex<Option<u32>>,
}

impl RecordingDriveTree {
    fn with_nodes(nodes: Vec<KnowledgeDriveNodeSummary>) -> Self {
        Self {
            nodes,
            resolved_paths: Mutex::new(vec![]),
            path_bindings: HashMap::new(),
            expected_parent_id: None,
            calls: Mutex::new(0),
            last_page_size: Mutex::new(None),
        }
    }

    fn with_resolved_path(mut self, logical_path: &str, drive_node_id: Option<&str>) -> Self {
        self.path_bindings.insert(
            logical_path.to_string(),
            drive_node_id.map(std::string::ToString::to_string),
        );
        self
    }

    fn expect_parent_id(mut self, parent_id: Option<&str>) -> Self {
        self.expected_parent_id = Some(parent_id.map(std::string::ToString::to_string));
        self
    }

    fn calls(&self) -> u32 {
        *self.calls.lock().unwrap()
    }

    fn last_page_size(&self) -> Option<u32> {
        *self.last_page_size.lock().unwrap()
    }

    fn resolved_paths(&self) -> Vec<String> {
        self.resolved_paths.lock().unwrap().clone()
    }
}

#[async_trait]
impl KnowledgeDriveNodeTree for RecordingDriveTree {
    async fn resolve_path(
        &self,
        request: ResolveKnowledgeDriveNodePathRequest,
    ) -> Result<Option<KnowledgeDriveNodeSummary>, KnowledgeDriveNodeTreeError> {
        assert_eq!(request.drive_space_id, "drv-kb-001");
        self.resolved_paths
            .lock()
            .unwrap()
            .push(request.logical_path.clone());

        Ok(self
            .path_bindings
            .get(&request.logical_path)
            .and_then(|drive_node_id| {
                drive_node_id
                    .as_ref()
                    .map(|drive_node_id| KnowledgeDriveNodeSummary {
                        drive_node_id: drive_node_id.clone(),
                        parent_drive_node_id: None,
                        kind: DriveNodeKind::Folder,
                        name: request.logical_path.clone(),
                        path: request.logical_path.clone(),
                        content_type: None,
                        size_bytes: None,
                        children_count: Some(self.nodes.len() as u64),
                        updated_at: "2026-06-04T12:00:00Z".to_string(),
                    })
            }))
    }

    async fn list_children(
        &self,
        request: ListKnowledgeDriveNodeChildrenRequest,
    ) -> Result<KnowledgeDriveNodePage, KnowledgeDriveNodeTreeError> {
        *self.calls.lock().unwrap() += 1;
        *self.last_page_size.lock().unwrap() = Some(request.page_size);
        assert_eq!(request.drive_space_id, "drv-kb-001");
        if let Some(expected_parent_id) = &self.expected_parent_id {
            assert_eq!(&request.parent_drive_node_id, expected_parent_id);
        }
        Ok(KnowledgeDriveNodePage {
            nodes: self.nodes.clone(),
            next_cursor: Some("next-cursor".to_string()),
        })
    }
}

#[derive(Default)]
struct RecordingProjectionStore {
    documents: Vec<KnowledgeBrowserDocumentProjection>,
    wiki_pages: Vec<KnowledgeBrowserWikiPageProjection>,
    calls: Mutex<u32>,
    wiki_calls: Mutex<u32>,
    requested_drive_node_ids: Mutex<Vec<String>>,
    requested_logical_paths: Mutex<Vec<String>>,
}

impl RecordingProjectionStore {
    fn with_documents(documents: Vec<KnowledgeBrowserDocumentProjection>) -> Self {
        Self {
            documents,
            wiki_pages: vec![],
            calls: Mutex::new(0),
            wiki_calls: Mutex::new(0),
            requested_drive_node_ids: Mutex::new(vec![]),
            requested_logical_paths: Mutex::new(vec![]),
        }
    }

    fn with_wiki_pages(wiki_pages: Vec<KnowledgeBrowserWikiPageProjection>) -> Self {
        Self {
            documents: vec![],
            wiki_pages,
            calls: Mutex::new(0),
            wiki_calls: Mutex::new(0),
            requested_drive_node_ids: Mutex::new(vec![]),
            requested_logical_paths: Mutex::new(vec![]),
        }
    }

    fn calls(&self) -> u32 {
        *self.calls.lock().unwrap()
    }

    fn requested_drive_node_ids(&self) -> Vec<String> {
        self.requested_drive_node_ids.lock().unwrap().clone()
    }

    fn wiki_calls(&self) -> u32 {
        *self.wiki_calls.lock().unwrap()
    }
}

#[async_trait]
impl KnowledgeBrowserProjectionStore for RecordingProjectionStore {
    async fn batch_document_projections(
        &self,
        _space_id: u64,
        drive_node_ids: Vec<String>,
    ) -> Result<Vec<KnowledgeBrowserDocumentProjection>, KnowledgeBrowserProjectionStoreError> {
        *self.calls.lock().unwrap() += 1;
        *self.requested_drive_node_ids.lock().unwrap() = drive_node_ids;
        Ok(self.documents.clone())
    }

    async fn batch_wiki_page_projections(
        &self,
        _space_id: u64,
        logical_paths: Vec<String>,
    ) -> Result<Vec<KnowledgeBrowserWikiPageProjection>, KnowledgeBrowserProjectionStoreError> {
        *self.wiki_calls.lock().unwrap() += 1;
        *self.requested_logical_paths.lock().unwrap() = logical_paths;
        Ok(self.wiki_pages.clone())
    }
}

fn space(drive_space_id: Option<String>) -> KnowledgeSpace {
    KnowledgeSpace {
        id: 1,
        uuid: "kb-001".to_string(),
        name: "Research Space".to_string(),
        description: None,
        drive_space_id,
        status: KnowledgeSpaceStatus::Active,
        llm_wiki_initialized: false,
    }
}
