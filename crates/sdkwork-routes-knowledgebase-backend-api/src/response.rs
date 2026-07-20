use axum::{http::StatusCode, response::Response};
use sdkwork_utils_rust::{SdkWorkCommandData, SdkWorkPageData};
use serde::Serialize;

use crate::error::{BackendApiProblem, BackendApiResult};

pub(crate) fn ok_list_json<T>(
    result: BackendApiResult<SdkWorkPageData<T>>,
) -> Result<Response, BackendApiProblem>
where
    T: Serialize,
{
    result
        .map(|value| {
            sdkwork_knowledgebase_observability::request_correlation::success_list_json_response(
                StatusCode::OK,
                value,
            )
        })
        .map_err(BackendApiProblem::from)
}

pub(crate) fn ok_json<T>(result: BackendApiResult<T>) -> Result<Response, BackendApiProblem>
where
    T: Serialize,
{
    result
        .map(|value| {
            sdkwork_knowledgebase_observability::request_correlation::success_json_response(
                StatusCode::OK,
                value,
            )
        })
        .map_err(BackendApiProblem::from)
}

pub(crate) fn created_json<T>(result: BackendApiResult<T>) -> Result<Response, BackendApiProblem>
where
    T: Serialize,
{
    result
        .map(|value| {
            sdkwork_knowledgebase_observability::request_correlation::success_json_response(
                StatusCode::CREATED,
                value,
            )
        })
        .map_err(BackendApiProblem::from)
}

pub(crate) fn command_json(
    result: BackendApiResult<SdkWorkCommandData>,
) -> Result<Response, BackendApiProblem> {
    result
        .map(|value| {
            sdkwork_knowledgebase_observability::request_correlation::success_command_json_response(
                StatusCode::OK,
                value,
            )
        })
        .map_err(BackendApiProblem::from)
}
