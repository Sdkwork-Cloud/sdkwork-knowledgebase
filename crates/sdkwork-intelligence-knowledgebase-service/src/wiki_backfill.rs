use crate::{
    ports::knowledge_wiki_persistence::{
        ListWikiPublicationBackfillCandidatesRequest, ProvisionWikiPublicationRequest,
        WikiPersistenceError, WikiPersistenceScope, WikiPublicationBackfillCandidate,
        WikiPublicationBackfillStore, WikiPublicationStore,
    },
    wiki_initialization::{
        InitializeKnowledgeWikiRequest, KnowledgeWikiInitializationError,
        KnowledgeWikiInitializationService,
    },
};
use thiserror::Error;

pub const MAX_WIKI_BACKFILL_PAGE_SIZE: u32 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunWikiPublicationBackfillRequest {
    pub scope: WikiPersistenceScope,
    pub after_space_id: Option<u64>,
    pub page_size: u32,
    pub actor_id: u64,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WikiPublicationBackfillDisposition {
    Planned,
    Initialized,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicationBackfillOutcome {
    pub space_id: u64,
    pub disposition: WikiPublicationBackfillDisposition,
    pub failure_code: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiPublicationBackfillPageResult {
    pub outcomes: Vec<WikiPublicationBackfillOutcome>,
    pub next_after_space_id: Option<u64>,
    pub stopped_on_failure: bool,
}

pub struct KnowledgeWikiBackfillService<'a> {
    backfill_store: &'a dyn WikiPublicationBackfillStore,
    publication_store: &'a dyn WikiPublicationStore,
    initializer: &'a KnowledgeWikiInitializationService<'a>,
}

impl<'a> KnowledgeWikiBackfillService<'a> {
    pub fn new(
        backfill_store: &'a dyn WikiPublicationBackfillStore,
        publication_store: &'a dyn WikiPublicationStore,
        initializer: &'a KnowledgeWikiInitializationService<'a>,
    ) -> Self {
        Self {
            backfill_store,
            publication_store,
            initializer,
        }
    }

    pub async fn run_page(
        &self,
        request: RunWikiPublicationBackfillRequest,
    ) -> Result<WikiPublicationBackfillPageResult, KnowledgeWikiBackfillError> {
        validate_request(request)?;
        let page = self
            .backfill_store
            .list_backfill_candidates(ListWikiPublicationBackfillCandidatesRequest {
                scope: request.scope,
                after_space_id: request.after_space_id,
                limit: request.page_size,
            })
            .await?;

        let mut outcomes = Vec::with_capacity(page.candidates.len());
        let mut resume_after_space_id = request.after_space_id;
        for candidate in page.candidates {
            if request.dry_run {
                resume_after_space_id = Some(candidate.space_id);
                outcomes.push(outcome(
                    candidate.space_id,
                    WikiPublicationBackfillDisposition::Planned,
                    None,
                ));
                continue;
            }

            match self.initialize_candidate(request, &candidate).await {
                Ok(()) => {
                    resume_after_space_id = Some(candidate.space_id);
                    outcomes.push(outcome(
                        candidate.space_id,
                        WikiPublicationBackfillDisposition::Initialized,
                        None,
                    ));
                }
                Err(error) => {
                    tracing::warn!(
                        knowledge_space_id = candidate.space_id,
                        error = %error,
                        "Wiki publication backfill stopped before advancing its resume cursor"
                    );
                    outcomes.push(outcome(
                        candidate.space_id,
                        WikiPublicationBackfillDisposition::Failed,
                        Some(error.code()),
                    ));
                    return Ok(WikiPublicationBackfillPageResult {
                        outcomes,
                        next_after_space_id: resume_after_space_id,
                        stopped_on_failure: true,
                    });
                }
            }
        }

        Ok(WikiPublicationBackfillPageResult {
            outcomes,
            next_after_space_id: page.next_after_space_id,
            stopped_on_failure: false,
        })
    }

    async fn initialize_candidate(
        &self,
        request: RunWikiPublicationBackfillRequest,
        candidate: &WikiPublicationBackfillCandidate,
    ) -> Result<(), KnowledgeWikiBackfillItemError> {
        self.publication_store
            .provision_publication(ProvisionWikiPublicationRequest {
                scope: request.scope,
                space_id: candidate.space_id,
                drive_space_uuid: candidate.drive_space_uuid.clone(),
                title: candidate.title.clone(),
                actor_id: request.actor_id,
            })
            .await?;
        self.initializer
            .initialize(InitializeKnowledgeWikiRequest {
                scope: request.scope,
                space_id: candidate.space_id,
                knowledgebase_uuid: candidate.knowledgebase_uuid.clone(),
                drive_space_uuid: candidate.drive_space_uuid.clone(),
                actor_id: request.actor_id,
            })
            .await?;
        Ok(())
    }
}

fn validate_request(
    request: RunWikiPublicationBackfillRequest,
) -> Result<(), KnowledgeWikiBackfillError> {
    if request.scope.tenant_id == 0 || request.actor_id == 0 {
        return Err(KnowledgeWikiBackfillError::InvalidRequest(
            "tenant_id and actor_id must be greater than zero".to_string(),
        ));
    }
    if request.page_size == 0 || request.page_size > MAX_WIKI_BACKFILL_PAGE_SIZE {
        return Err(KnowledgeWikiBackfillError::InvalidRequest(format!(
            "page_size must be between 1 and {MAX_WIKI_BACKFILL_PAGE_SIZE}"
        )));
    }
    Ok(())
}

fn outcome(
    space_id: u64,
    disposition: WikiPublicationBackfillDisposition,
    failure_code: Option<&'static str>,
) -> WikiPublicationBackfillOutcome {
    WikiPublicationBackfillOutcome {
        space_id,
        disposition,
        failure_code,
    }
}

#[derive(Debug, Error)]
enum KnowledgeWikiBackfillItemError {
    #[error(transparent)]
    Persistence(#[from] WikiPersistenceError),
    #[error(transparent)]
    Initialization(#[from] KnowledgeWikiInitializationError),
}

impl KnowledgeWikiBackfillItemError {
    fn code(&self) -> &'static str {
        match self {
            Self::Persistence(_) => "wiki_persistence_failed",
            Self::Initialization(_) => "wiki_initialization_failed",
        }
    }
}

#[derive(Debug, Error)]
pub enum KnowledgeWikiBackfillError {
    #[error("Wiki backfill request is invalid: {0}")]
    InvalidRequest(String),
    #[error(transparent)]
    Persistence(#[from] WikiPersistenceError),
}
