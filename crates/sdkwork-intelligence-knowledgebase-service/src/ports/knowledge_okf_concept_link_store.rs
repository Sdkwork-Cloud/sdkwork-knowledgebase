use async_trait::async_trait;
use thiserror::Error;

/// Hard safety bound for in-service aggregation sweeps over the OKF link
/// graph. Large spaces page through the store; the aggregate is bounded so a
/// pathological space cannot exhaust process memory.
pub const MAX_OKF_LINK_SCAN_ROWS: usize = 50_000;

/// Per-page OKF link-edge fetch limit used by aggregation sweeps.
pub const OKF_LINK_SCAN_PAGE_SIZE: u32 = 1_000;

#[async_trait]
pub trait KnowledgeOkfConceptLinkStore: Send + Sync {
    async fn replace_outbound_links(
        &self,
        record: ReplaceKnowledgeOkfConceptLinksRecord,
    ) -> Result<(), KnowledgeOkfConceptLinkStoreError>;

    async fn list_inbound_concept_ids(
        &self,
        space_id: u64,
        to_concept_id: &str,
    ) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError>;

    /// Paged distinct inbound link targets for a space (keyset on
    /// `to_concept_id`). Aggregation sweeps page through this instead of
    /// loading an unbounded target set.
    async fn list_inbound_link_targets_page(
        &self,
        space_id: u64,
        after_concept_id: Option<&str>,
        limit: u32,
    ) -> Result<InboundLinkTargetsPage, KnowledgeOkfConceptLinkStoreError>;

    /// Paged active link edges for a space (keyset on
    /// `(from_concept_id, to_concept_id, anchor_text)`). Aggregation sweeps
    /// page through this instead of silently truncating at a fixed row cap.
    async fn list_active_link_edges_page(
        &self,
        space_id: u64,
        after: Option<LinkEdgeCursor>,
        limit: u32,
    ) -> Result<LinkEdgePage, KnowledgeOkfConceptLinkStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeOkfConceptLinkEdge {
    pub from_concept_id: String,
    pub to_concept_id: String,
    pub anchor_text: String,
}

/// Keyset continuation for [`KnowledgeOkfConceptLinkStore::list_active_link_edges_page`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkEdgeCursor {
    pub from_concept_id: String,
    pub to_concept_id: String,
    pub anchor_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkEdgePage {
    pub edges: Vec<KnowledgeOkfConceptLinkEdge>,
    pub next_cursor: Option<LinkEdgeCursor>,
    pub has_more: bool,
}

/// One page of distinct inbound link targets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundLinkTargetsPage {
    pub targets: Vec<String>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Aggregates the complete active link-edge graph for a space by paging through
/// the store. Bounded by [`MAX_OKF_LINK_SCAN_ROWS`]; larger graphs fail closed
/// instead of being silently truncated.
pub async fn list_all_active_link_edges(
    store: &dyn KnowledgeOkfConceptLinkStore,
    space_id: u64,
) -> Result<Vec<KnowledgeOkfConceptLinkEdge>, KnowledgeOkfConceptLinkStoreError> {
    let mut edges = Vec::new();
    let mut cursor = None;
    loop {
        let page = store
            .list_active_link_edges_page(space_id, cursor.clone(), OKF_LINK_SCAN_PAGE_SIZE)
            .await?;
        if edges.len().saturating_add(page.edges.len()) > MAX_OKF_LINK_SCAN_ROWS {
            return Err(KnowledgeOkfConceptLinkStoreError::Internal(format!(
                "OKF link graph exceeds the {MAX_OKF_LINK_SCAN_ROWS} row safety bound for space {space_id}"
            )));
        }
        edges.extend(page.edges);
        if !page.has_more {
            break;
        }
        cursor = page.next_cursor;
    }
    Ok(edges)
}

/// Aggregates the distinct inbound link-target set for a space by paging
/// through the store. Bounded by [`MAX_OKF_LINK_SCAN_ROWS`].
pub async fn list_all_inbound_link_targets(
    store: &dyn KnowledgeOkfConceptLinkStore,
    space_id: u64,
) -> Result<Vec<String>, KnowledgeOkfConceptLinkStoreError> {
    let mut targets = Vec::new();
    let mut cursor = None;
    loop {
        let page = store
            .list_inbound_link_targets_page(space_id, cursor.as_deref(), OKF_LINK_SCAN_PAGE_SIZE)
            .await?;
        if targets.len().saturating_add(page.targets.len()) > MAX_OKF_LINK_SCAN_ROWS {
            return Err(KnowledgeOkfConceptLinkStoreError::Internal(format!(
                "OKF inbound link targets exceed the {MAX_OKF_LINK_SCAN_ROWS} row safety bound for space {space_id}"
            )));
        }
        targets.extend(page.targets);
        if !page.has_more {
            break;
        }
        cursor = page.next_cursor;
    }
    Ok(targets)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceKnowledgeOkfConceptLinksRecord {
    pub space_id: u64,
    pub from_concept_id: String,
    pub links: Vec<KnowledgeOkfConceptLinkRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeOkfConceptLinkRecord {
    pub to_concept_id: String,
    pub anchor_text: String,
}

#[derive(Debug, Error)]
pub enum KnowledgeOkfConceptLinkStoreError {
    #[error("internal knowledge okf concept link store error: {0}")]
    Internal(String),
}
