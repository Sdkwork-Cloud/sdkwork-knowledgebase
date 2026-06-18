use crate::ports::{
    knowledge_access_control::{
        KnowledgeAccessCheckRequest, KnowledgeAccessControl, KnowledgeAccessControlError,
        KnowledgeAccessRole,
    },
    knowledge_browser_projection_store::{
        KnowledgeBrowserDocumentProjection, KnowledgeBrowserProjectionStore,
        KnowledgeBrowserProjectionStoreError, KnowledgeBrowserWikiPageProjection,
    },
    knowledge_drive_node_tree::{
        DriveNodeKind, GetKnowledgeDriveNodeRequest, KnowledgeDriveNodeSummary,
        KnowledgeDriveNodeTree, KnowledgeDriveNodeTreeError, ListKnowledgeDriveNodeChildrenRequest,
        ResolveKnowledgeDriveNodePathRequest,
    },
    knowledge_space_store::{KnowledgeSpaceStore, KnowledgeSpaceStoreError},
};
use sdkwork_knowledgebase_contract::browser::{
    KnowledgeBrowserNode, KnowledgeBrowserNodePermissions, KnowledgeBrowserNodeType,
    KnowledgeBrowserPage, KnowledgeBrowserView, ListKnowledgeBrowserRequest,
};
use std::collections::HashMap;
use thiserror::Error;

const DEFAULT_BROWSER_PAGE_SIZE: u32 = 50;
const MAX_BROWSER_PAGE_SIZE: u32 = 200;
const WIKI_VIEW_ROOT_PATH: &str = "wiki";
const OUTPUTS_VIEW_ROOT_PATH: &str = "output";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeBrowserAccessContext {
    pub tenant_id: u64,
    pub actor_id: String,
}

pub struct KnowledgeBrowserService<'a> {
    spaces: &'a dyn KnowledgeSpaceStore,
    drive_tree: &'a dyn KnowledgeDriveNodeTree,
    projections: &'a dyn KnowledgeBrowserProjectionStore,
    access_control: Option<&'a dyn KnowledgeAccessControl>,
}

impl<'a> KnowledgeBrowserService<'a> {
    pub fn new(
        spaces: &'a dyn KnowledgeSpaceStore,
        drive_tree: &'a dyn KnowledgeDriveNodeTree,
        projections: &'a dyn KnowledgeBrowserProjectionStore,
    ) -> Self {
        Self {
            spaces,
            drive_tree,
            projections,
            access_control: None,
        }
    }

    pub fn with_access_control(mut self, access_control: &'a dyn KnowledgeAccessControl) -> Self {
        self.access_control = Some(access_control);
        self
    }

    pub async fn list(
        &self,
        access: Option<KnowledgeBrowserAccessContext>,
        request: ListKnowledgeBrowserRequest,
    ) -> Result<KnowledgeBrowserPage, KnowledgeBrowserServiceError> {
        if request.space_id == 0 {
            return Err(KnowledgeBrowserServiceError::InvalidRequest(
                "space_id is required".to_string(),
            ));
        }

        let space = self.spaces.get_space(request.space_id).await?;
        if let Some(access_control) = self.access_control {
            let access = access.ok_or_else(|| {
                KnowledgeBrowserServiceError::InvalidRequest(
                    "authenticated browser access context is required".to_string(),
                )
            })?;
            let drive_space_id = space.drive_space_id.as_ref().ok_or_else(|| {
                KnowledgeBrowserServiceError::InvalidRequest(
                    "drive space is not bound for knowledge space".to_string(),
                )
            })?;
            let grant = access_control
                .check_space_access(KnowledgeAccessCheckRequest {
                    tenant_id: access.tenant_id.to_string(),
                    actor_id: access.actor_id,
                    drive_space_id: drive_space_id.clone(),
                    required_role: KnowledgeAccessRole::Reader,
                })
                .await
                .map_err(KnowledgeBrowserServiceError::AccessControl)?;
            if !grant.allowed {
                return Err(KnowledgeBrowserServiceError::AccessDenied(format!(
                    "actor does not have access to space {}",
                    request.space_id
                )));
            }
        }
        let drive_space_id = space.drive_space_id.ok_or_else(|| {
            KnowledgeBrowserServiceError::InvalidRequest(
                "drive space is not bound for knowledge space".to_string(),
            )
        })?;
        let page_size = normalize_page_size(request.page_size);

        let parent_drive_node_id = self
            .resolve_view_parent_id(&drive_space_id, request.view, request.parent_id)
            .await?;
        let drive_page = self
            .drive_tree
            .list_children(ListKnowledgeDriveNodeChildrenRequest {
                drive_space_id: drive_space_id.clone(),
                parent_drive_node_id: parent_drive_node_id.clone(),
                cursor: request.cursor,
                page_size,
            })
            .await?;

        let drive_node_ids = drive_page
            .nodes
            .iter()
            .map(|node| node.drive_node_id.clone())
            .collect::<Vec<_>>();
        let document_projection_by_node = self
            .projections
            .batch_document_projections(request.space_id, drive_node_ids)
            .await?
            .into_iter()
            .map(|projection| (projection.drive_node_id.clone(), projection))
            .collect::<HashMap<_, _>>();
        let wiki_projection_by_path = if request.view == KnowledgeBrowserView::Wiki {
            let logical_paths = drive_page
                .nodes
                .iter()
                .filter(|node| node.kind == DriveNodeKind::File)
                .map(|node| node.path.trim_start_matches('/').to_string())
                .collect::<Vec<_>>();
            self.projections
                .batch_wiki_page_projections(request.space_id, logical_paths)
                .await?
                .into_iter()
                .map(|projection| (projection.logical_path.clone(), projection))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        let items = drive_page
            .nodes
            .into_iter()
            .map(|node| {
                let projection = document_projection_by_node.get(&node.drive_node_id);
                let wiki_projection =
                    wiki_projection_by_path.get(node.path.trim_start_matches('/'));
                drive_node_to_browser_node(
                    &drive_space_id,
                    request.view,
                    node,
                    projection,
                    wiki_projection,
                )
            })
            .collect();

        Ok(KnowledgeBrowserPage {
            space_id: request.space_id,
            drive_space_id,
            parent_id: parent_drive_node_id,
            view: request.view,
            page_size,
            items,
            next_cursor: drive_page.next_cursor,
        })
    }

    async fn resolve_view_parent_id(
        &self,
        drive_space_id: &str,
        view: KnowledgeBrowserView,
        parent_id: Option<String>,
    ) -> Result<Option<String>, KnowledgeBrowserServiceError> {
        if view == KnowledgeBrowserView::Files {
            if let Some(parent_id) = parent_id {
                self.validate_folder_parent(drive_space_id, &parent_id, None)
                    .await?;
                return Ok(Some(parent_id));
            }
            return Ok(None);
        }

        let root_path = match view {
            KnowledgeBrowserView::Files => return Ok(None),
            KnowledgeBrowserView::Wiki => WIKI_VIEW_ROOT_PATH,
            KnowledgeBrowserView::Outputs => OUTPUTS_VIEW_ROOT_PATH,
        };

        if let Some(parent_id) = parent_id {
            self.validate_folder_parent(drive_space_id, &parent_id, Some((view, root_path)))
                .await?;
            return Ok(Some(parent_id));
        }

        let root = self
            .drive_tree
            .resolve_path(ResolveKnowledgeDriveNodePathRequest {
                drive_space_id: drive_space_id.to_string(),
                logical_path: root_path.to_string(),
            })
            .await?;

        root.map(|node| Some(node.drive_node_id)).ok_or_else(|| {
            KnowledgeBrowserServiceError::InvalidRequest(format!(
                "browser view root is missing in drive space: {root_path}"
            ))
        })
    }

    async fn validate_folder_parent(
        &self,
        drive_space_id: &str,
        parent_id: &str,
        root_boundary: Option<(KnowledgeBrowserView, &str)>,
    ) -> Result<(), KnowledgeBrowserServiceError> {
        let parent = self
            .drive_tree
            .get_node(GetKnowledgeDriveNodeRequest {
                drive_space_id: drive_space_id.to_string(),
                drive_node_id: parent_id.to_string(),
            })
            .await?
            .ok_or_else(|| {
                KnowledgeBrowserServiceError::InvalidRequest(format!(
                    "browser parent node is missing: {parent_id}"
                ))
            })?;
        if parent.kind != DriveNodeKind::Folder {
            return Err(KnowledgeBrowserServiceError::InvalidRequest(
                "browser parent node must be a folder".to_string(),
            ));
        }
        if let Some((view, root_path)) = root_boundary {
            if !path_is_within_root(&parent.path, root_path) {
                return Err(KnowledgeBrowserServiceError::InvalidRequest(format!(
                    "browser parent node is outside {} view root: {}",
                    view_name(view),
                    parent.path
                )));
            }
        }
        Ok(())
    }
}

fn path_is_within_root(path: &str, root_path: &str) -> bool {
    let path = path.trim_matches('/');
    path == root_path || path.starts_with(&format!("{root_path}/"))
}

fn view_name(view: KnowledgeBrowserView) -> &'static str {
    match view {
        KnowledgeBrowserView::Files => "files",
        KnowledgeBrowserView::Wiki => "wiki",
        KnowledgeBrowserView::Outputs => "outputs",
    }
}

fn normalize_page_size(page_size: Option<u32>) -> u32 {
    page_size
        .unwrap_or(DEFAULT_BROWSER_PAGE_SIZE)
        .clamp(1, MAX_BROWSER_PAGE_SIZE)
}

fn drive_node_to_browser_node(
    drive_space_id: &str,
    view: KnowledgeBrowserView,
    node: KnowledgeDriveNodeSummary,
    projection: Option<&KnowledgeBrowserDocumentProjection>,
    wiki_projection: Option<&KnowledgeBrowserWikiPageProjection>,
) -> KnowledgeBrowserNode {
    let node_type = browser_node_type(view, node.kind);

    KnowledgeBrowserNode {
        id: node.drive_node_id.clone(),
        node_type,
        name: node.name,
        parent_id: node.parent_drive_node_id,
        path: node.path,
        drive_space_id: Some(drive_space_id.to_string()),
        drive_node_id: Some(node.drive_node_id),
        document_id: projection.map(|projection| projection.document_id),
        document_version_id: projection.and_then(|projection| projection.current_version_id),
        wiki_page_id: wiki_projection.map(|projection| projection.page_id),
        wiki_revision_id: wiki_projection.and_then(|projection| projection.current_revision_id),
        mime_type: node.content_type,
        size_bytes: node.size_bytes,
        ingest_state: projection.map(|projection| projection.ingest_state.clone()),
        parse_state: projection.map(|projection| projection.parse_state.clone()),
        index_state: projection.map(|projection| projection.index_state.clone()),
        wiki_state: wiki_projection
            .map(|projection| projection.publish_state.as_str().to_string())
            .or_else(|| projection.map(|projection| projection.wiki_state.clone())),
        children_count: node.children_count,
        updated_at: node.updated_at,
        permissions: match node.kind {
            DriveNodeKind::Folder => KnowledgeBrowserNodePermissions::file_manager(),
            DriveNodeKind::File => KnowledgeBrowserNodePermissions::read_only(),
        },
    }
}

fn browser_node_type(view: KnowledgeBrowserView, kind: DriveNodeKind) -> KnowledgeBrowserNodeType {
    match (view, kind) {
        (_, DriveNodeKind::Folder) => match view {
            KnowledgeBrowserView::Outputs => KnowledgeBrowserNodeType::VirtualFolder,
            KnowledgeBrowserView::Files | KnowledgeBrowserView::Wiki => {
                KnowledgeBrowserNodeType::Folder
            }
        },
        (KnowledgeBrowserView::Wiki, DriveNodeKind::File) => KnowledgeBrowserNodeType::WikiPage,
        (KnowledgeBrowserView::Outputs, DriveNodeKind::File) => KnowledgeBrowserNodeType::Report,
        (KnowledgeBrowserView::Files, DriveNodeKind::File) => KnowledgeBrowserNodeType::Document,
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeBrowserServiceError {
    #[error("invalid knowledge browser request: {0}")]
    InvalidRequest(String),
    #[error("knowledge browser access denied: {0}")]
    AccessDenied(String),
    #[error(transparent)]
    AccessControl(#[from] KnowledgeAccessControlError),
    #[error(transparent)]
    SpaceStore(#[from] KnowledgeSpaceStoreError),
    #[error(transparent)]
    DriveTree(#[from] KnowledgeDriveNodeTreeError),
    #[error(transparent)]
    ProjectionStore(#[from] KnowledgeBrowserProjectionStoreError),
}
