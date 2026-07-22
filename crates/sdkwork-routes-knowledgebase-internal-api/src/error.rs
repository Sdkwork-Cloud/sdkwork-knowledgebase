use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sdkwork_intelligence_knowledgebase_service::wiki_event_consumer::KnowledgeWikiDriveEventConsumerError;
use sdkwork_intelligence_knowledgebase_service::wiki_public_provider::KnowledgeWikiPublicProviderError;
use sdkwork_knowledgebase_contract::ProblemDetails;

#[derive(Debug)]
pub struct InternalApiProblem {
    status: StatusCode,
    problem: Box<ProblemDetails>,
}

impl InternalApiProblem {
    pub fn new(status: StatusCode, code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            status,
            problem: Box::new(ProblemDetails::pending_trace(status, code, detail)),
        }
    }

    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "authentication_required",
            "authenticated internal principal is required",
        )
    }

    pub fn forbidden(detail: &'static str) -> Self {
        Self::new(StatusCode::FORBIDDEN, "permission_required", detail)
    }
}

impl From<KnowledgeWikiDriveEventConsumerError> for InternalApiProblem {
    fn from(error: KnowledgeWikiDriveEventConsumerError) -> Self {
        match error {
            KnowledgeWikiDriveEventConsumerError::InvalidRequest(detail)
            | KnowledgeWikiDriveEventConsumerError::InvalidEvent(detail) => {
                Self::new(StatusCode::BAD_REQUEST, "invalid_parameter", detail)
            }
            KnowledgeWikiDriveEventConsumerError::Integrity(_) => Self::new(
                StatusCode::FORBIDDEN,
                "drive_event_integrity_failed",
                "signed Drive event verification failed",
            ),
            KnowledgeWikiDriveEventConsumerError::Persistence(_) => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "wiki_event_persistence_failed",
                "An internal error occurred. Please try again later.",
            ),
            KnowledgeWikiDriveEventConsumerError::Drive(_) => Self::new(
                StatusCode::BAD_GATEWAY,
                "wiki_event_drive_unavailable",
                "Drive source resolution is temporarily unavailable.",
            ),
        }
    }
}

impl From<KnowledgeWikiPublicProviderError> for InternalApiProblem {
    fn from(error: KnowledgeWikiPublicProviderError) -> Self {
        match error {
            KnowledgeWikiPublicProviderError::InvalidRequest(detail) => {
                Self::new(StatusCode::BAD_REQUEST, "invalid_parameter", detail)
            }
            KnowledgeWikiPublicProviderError::NotFoundOrNotPublic => Self::new(
                StatusCode::NOT_FOUND,
                "wiki_not_found_or_not_public",
                "The requested Wiki resource was not found.",
            ),
            KnowledgeWikiPublicProviderError::ContentUnavailable => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "wiki_public_content_unavailable",
                "The requested Wiki representation is temporarily unavailable.",
            ),
            KnowledgeWikiPublicProviderError::IntegrityFailed => Self::new(
                StatusCode::BAD_GATEWAY,
                "wiki_public_content_integrity_failed",
                "The Wiki content provider failed integrity validation.",
            ),
            KnowledgeWikiPublicProviderError::TemporarilyUnavailable => Self::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "wiki_public_provider_unavailable",
                "The Wiki provider is temporarily unavailable.",
            ),
        }
    }
}

impl IntoResponse for InternalApiProblem {
    fn into_response(self) -> Response {
        sdkwork_knowledgebase_observability::request_correlation::problem_json_response(
            self.status,
            *self.problem,
        )
    }
}
