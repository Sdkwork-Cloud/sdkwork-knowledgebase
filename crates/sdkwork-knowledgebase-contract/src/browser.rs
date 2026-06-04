use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeBrowserView {
    Files,
    Wiki,
    Outputs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeBrowserNodeType {
    Folder,
    Document,
    WikiPage,
    Candidate,
    Answer,
    Report,
    VirtualFolder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListKnowledgeBrowserRequest {
    pub space_id: u64,
    pub parent_id: Option<String>,
    pub view: KnowledgeBrowserView,
    pub cursor: Option<String>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeBrowserPage {
    pub space_id: u64,
    pub drive_space_id: String,
    pub parent_id: Option<String>,
    pub view: KnowledgeBrowserView,
    pub page_size: u32,
    pub items: Vec<KnowledgeBrowserNode>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeBrowserNode {
    pub id: String,
    pub node_type: KnowledgeBrowserNodeType,
    pub name: String,
    pub parent_id: Option<String>,
    pub path: String,
    pub drive_space_id: Option<String>,
    pub drive_node_id: Option<String>,
    pub document_id: Option<u64>,
    pub document_version_id: Option<u64>,
    pub wiki_page_id: Option<u64>,
    pub wiki_revision_id: Option<u64>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub ingest_state: Option<String>,
    pub parse_state: Option<String>,
    pub index_state: Option<String>,
    pub wiki_state: Option<String>,
    pub children_count: Option<u64>,
    pub updated_at: String,
    pub permissions: KnowledgeBrowserNodePermissions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeBrowserNodePermissions {
    pub can_read: bool,
    pub can_upload: bool,
    pub can_rename: bool,
    pub can_move: bool,
    pub can_delete: bool,
    pub can_review: bool,
    pub can_publish: bool,
}

impl KnowledgeBrowserNodePermissions {
    pub const fn read_only() -> Self {
        Self {
            can_read: true,
            can_upload: false,
            can_rename: false,
            can_move: false,
            can_delete: false,
            can_review: false,
            can_publish: false,
        }
    }

    pub const fn file_manager() -> Self {
        Self {
            can_read: true,
            can_upload: true,
            can_rename: true,
            can_move: true,
            can_delete: true,
            can_review: false,
            can_publish: false,
        }
    }
}
