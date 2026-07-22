use axum::{http::StatusCode, response::Response};
use sdkwork_utils_rust::SdkWorkPageData;
use serde::Serialize;

pub fn success_json<T: Serialize>(value: T) -> Response {
    sdkwork_knowledgebase_observability::request_correlation::success_json_response(
        StatusCode::OK,
        value,
    )
}

pub fn success_list_json<T: Serialize>(value: SdkWorkPageData<T>) -> Response {
    sdkwork_knowledgebase_observability::request_correlation::success_list_json_response(
        StatusCode::OK,
        value,
    )
}
