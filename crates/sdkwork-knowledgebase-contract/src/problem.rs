use sdkwork_utils_rust::SdkWorkResultCode;
use serde::{Deserialize, Serialize};

/// RFC 9457 `application/problem+json` body (`API_SPEC.md` §15.2).
///
/// `code` is a numeric platform error code per `API_SPEC.md` §15.3.
/// `traceId` is always present and echoes the server-owned request correlation id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub r#type: String,
    pub title: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    pub code: i32,
    pub trace_id: String,
}

impl ProblemDetails {
    /// Build a `ProblemDetails` from an HTTP status code and a domain-specific
    /// string code (used for logging). The wire `code` field is the numeric
    /// platform error code derived from the HTTP status.
    pub fn from_status(
        status: http::StatusCode,
        domain_code: impl Into<String>,
        detail: impl Into<String>,
        trace_id: impl Into<String>,
    ) -> Self {
        let domain_code = domain_code.into();
        let result_code = platform_code_for_status(status, &domain_code);
        let title = status
            .canonical_reason()
            .unwrap_or("HTTP Error")
            .to_string();
        let detail_text = detail.into();
        let client_detail = if status.is_server_error() {
            None
        } else if detail_text.is_empty() {
            None
        } else {
            Some(detail_text)
        };
        Self {
            r#type: format!("https://docs.sdkwork.com/problems/{}", result_code.as_i32()),
            title,
            status: status.as_u16(),
            detail: client_detail,
            instance: None,
            code: result_code.as_i32(),
            trace_id: trace_id.into(),
        }
    }

    /// Create a placeholder with an empty trace id; the observability layer
    /// will enrich it with the active request correlation id.
    pub fn pending_trace(
        status: http::StatusCode,
        domain_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::from_status(status, domain_code, detail, String::new())
    }
}

/// Map an HTTP status code (and optional domain code hint) to the canonical
/// numeric `SdkWorkResultCode` per `API_SPEC.md` §15.3.
fn platform_code_for_status(status: http::StatusCode, domain_code: &str) -> SdkWorkResultCode {
    use http::StatusCode;
    match status {
        StatusCode::BAD_REQUEST => {
            if domain_code.contains("validation") || domain_code.contains("invalid") {
                SdkWorkResultCode::ValidationError
            } else if domain_code.contains("malformed") {
                SdkWorkResultCode::MalformedRequest
            } else if domain_code.contains("missing") {
                SdkWorkResultCode::MissingRequiredField
            } else {
                SdkWorkResultCode::InvalidParameter
            }
        }
        StatusCode::UNAUTHORIZED => {
            if domain_code.contains("expired") {
                SdkWorkResultCode::TokenExpired
            } else if domain_code.contains("revoked") {
                SdkWorkResultCode::SessionRevoked
            } else {
                SdkWorkResultCode::AuthenticationRequired
            }
        }
        StatusCode::FORBIDDEN => {
            if domain_code.contains("tenant") {
                SdkWorkResultCode::TenantAccessDenied
            } else if domain_code.contains("organization") {
                SdkWorkResultCode::OrganizationAccessDenied
            } else if domain_code.contains("scope") {
                SdkWorkResultCode::InsufficientScope
            } else {
                SdkWorkResultCode::PermissionRequired
            }
        }
        StatusCode::NOT_FOUND => SdkWorkResultCode::NotFound,
        StatusCode::METHOD_NOT_ALLOWED => SdkWorkResultCode::MethodNotAllowed,
        StatusCode::REQUEST_TIMEOUT => SdkWorkResultCode::RequestTimeout,
        StatusCode::CONFLICT => SdkWorkResultCode::Conflict,
        StatusCode::GONE => SdkWorkResultCode::Gone,
        StatusCode::PRECONDITION_FAILED => SdkWorkResultCode::PreconditionFailed,
        StatusCode::PAYLOAD_TOO_LARGE => SdkWorkResultCode::PayloadTooLarge,
        StatusCode::UNSUPPORTED_MEDIA_TYPE => SdkWorkResultCode::UnsupportedMediaType,
        StatusCode::UNPROCESSABLE_ENTITY => SdkWorkResultCode::UnprocessableEntity,
        StatusCode::LOCKED => SdkWorkResultCode::Locked,
        StatusCode::PRECONDITION_REQUIRED => SdkWorkResultCode::PreconditionRequired,
        StatusCode::TOO_MANY_REQUESTS => {
            if domain_code.contains("quota") {
                SdkWorkResultCode::QuotaExceeded
            } else {
                SdkWorkResultCode::RateLimitExceeded
            }
        }
        StatusCode::INTERNAL_SERVER_ERROR => SdkWorkResultCode::InternalError,
        StatusCode::BAD_GATEWAY => SdkWorkResultCode::BadGateway,
        StatusCode::SERVICE_UNAVAILABLE => SdkWorkResultCode::ServiceUnavailable,
        StatusCode::GATEWAY_TIMEOUT => SdkWorkResultCode::GatewayTimeout,
        StatusCode::NOT_IMPLEMENTED => SdkWorkResultCode::InternalError,
        _ => SdkWorkResultCode::InternalError,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::StatusCode;

    #[test]
    fn problem_details_uses_numeric_code() {
        let problem = ProblemDetails::from_status(
            StatusCode::NOT_FOUND,
            "knowledge_space_not_found",
            "space was not found",
            "trace-123",
        );
        assert_eq!(problem.code, 40401);
        assert_eq!(problem.status, 404);
        assert_eq!(problem.trace_id, "trace-123");
        assert!(problem.detail.is_some());
    }

    #[test]
    fn problem_details_server_error_has_no_detail() {
        let problem = ProblemDetails::from_status(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "sensitive internal detail",
            "trace-456",
        );
        assert_eq!(problem.code, 50001);
        assert!(problem.detail.is_none());
    }

    #[test]
    fn problem_details_validation_error_maps_to_40001() {
        let problem = ProblemDetails::from_status(
            StatusCode::BAD_REQUEST,
            "invalid_knowledge_retrieval_request",
            "query is required",
            "trace-789",
        );
        assert_eq!(problem.code, 40001);
    }

    #[test]
    fn problem_details_conflict_maps_to_40901() {
        let problem = ProblemDetails::from_status(
            StatusCode::CONFLICT,
            "knowledge_space_conflict",
            "space already exists",
            "trace-conflict",
        );
        assert_eq!(problem.code, 40901);
    }

    #[test]
    fn problem_details_quota_exceeded_maps_to_60002() {
        let problem = ProblemDetails::from_status(
            StatusCode::TOO_MANY_REQUESTS,
            "knowledge_tenant_quota_exceeded",
            "document quota exceeded",
            "trace-quota",
        );
        assert_eq!(problem.code, 60002);
    }
}
