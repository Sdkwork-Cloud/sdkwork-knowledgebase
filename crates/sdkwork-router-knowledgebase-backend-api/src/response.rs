use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::error::{BackendApiProblem, BackendApiResult};

pub(crate) fn ok_json<T>(result: BackendApiResult<T>) -> Result<Response, BackendApiProblem>
where
    T: Serialize,
{
    result
        .map(|value| Json(value).into_response())
        .map_err(BackendApiProblem::from)
}

pub(crate) fn created_json<T>(result: BackendApiResult<T>) -> Result<Response, BackendApiProblem>
where
    T: Serialize,
{
    result
        .map(|value| (StatusCode::CREATED, Json(value)).into_response())
        .map_err(BackendApiProblem::from)
}
