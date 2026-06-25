//! Request correlation identifiers for logs, traces, and problem responses.

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use sdkwork_knowledgebase_contract::ProblemDetails;
use std::future::Future;

tokio::task_local! {
    static REQUEST_ID: String;
}

/// Run a request-scoped future with the active correlation id.
pub async fn request_id_scope<Fut>(request_id: String, future: Fut) -> Fut::Output
where
    Fut: Future,
{
    REQUEST_ID.scope(request_id, future).await
}

/// Returns the active request correlation id when called inside [`request_id_scope`].
pub fn current_request_id() -> Option<String> {
    REQUEST_ID.try_with(|request_id| request_id.clone()).ok()
}

/// Attach the active request id to problem details when available.
pub fn enrich_problem_trace_id(mut problem: ProblemDetails) -> ProblemDetails {
    if problem.trace_id.is_none() {
        problem.trace_id = current_request_id();
    }
    problem
}

/// Build a RFC 7807 problem response with correlation metadata.
pub fn problem_json_response(status: StatusCode, problem: ProblemDetails) -> Response {
    let problem = enrich_problem_trace_id(problem);
    let mut response = (status, Json(problem)).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn enrich_problem_trace_id_uses_scoped_request_id() {
        let problem = ProblemDetails {
            r#type: "about:blank".to_string(),
            title: "Bad Request".to_string(),
            status: 400,
            detail: Some("invalid".to_string()),
            instance: None,
            code: Some("invalid_request".to_string()),
            trace_id: None,
        };

        let enriched = request_id_scope("corr-123".to_string(), async {
            enrich_problem_trace_id(problem)
        })
        .await;

        assert_eq!(enriched.trace_id.as_deref(), Some("corr-123"));
    }
}
