use axum::{
    body::{Body, Bytes},
    extract::{Path, Query, State},
    http::{header, header::HeaderName, HeaderMap, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use sdkwork_drive_contract::drive::events::{
    WEBHOOK_CHANNEL_ID_HEADER, WEBHOOK_EVENT_ID_HEADER, WEBHOOK_EVENT_RETRY_COUNT_HEADER,
    WEBHOOK_EVENT_SIGNATURE_HEADER, WEBHOOK_EVENT_TIMESTAMP_HEADER, WEBHOOK_IDEMPOTENCY_KEY_HEADER,
};
use sdkwork_intelligence_knowledgebase_service::{
    ports::knowledge_wiki_persistence::WikiPersistenceScope,
    wiki_public_provider::{
        ListWikiPublicNavigationPageRequest, ResolveWikiPublicRouteRequest,
        RetrieveWikiPublicContentRequest, RetrieveWikiPublicPublicationRequest,
        SearchWikiPublicPageRequest,
    },
};
use sdkwork_knowledgebase_contract::{
    parse_canonical_nonnegative_signed_i64, parse_canonical_positive_signed_i64,
};
use sdkwork_utils_rust::{PageInfo, PageMode, SdkWorkPageData};
use sdkwork_web_core::RequireInternalApi;

use crate::{
    dto::{
        DriveEventReceiptResponse, ResolveWikiRouteBody, WikiNavigationQuery,
        WikiPublicPageResponse, WikiPublicationResponse, WikiRouteResolutionResponse,
        WikiSearchQuery,
    },
    error::InternalApiProblem,
    response::{success_json, success_list_json},
    state::InternalApiState,
};

pub async fn receive_drive_event(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, InternalApiProblem> {
    if body.len() > state.max_body_bytes {
        return Err(InternalApiProblem::new(
            axum::http::StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            "Drive event body exceeds the configured limit",
        ));
    }
    let principal = context
        .principal
        .as_ref()
        .ok_or_else(InternalApiProblem::unauthorized)?;
    if principal.app_id() != state.drive_event_caller_app_id.as_ref() {
        return Err(InternalApiProblem::forbidden(
            "the internal caller is not authorized for Drive event delivery",
        ));
    }
    let payload_json = String::from_utf8(body.to_vec()).map_err(|_| {
        InternalApiProblem::new(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_parameter",
            "Drive event body must be UTF-8 JSON",
        )
    })?;
    let request = sdkwork_intelligence_knowledgebase_service::wiki_event_consumer::ReceiveKnowledgeWikiDriveWebhookRequest {
        channel_id: required_header(&headers, WEBHOOK_CHANNEL_ID_HEADER)?,
        event_id: required_header(&headers, WEBHOOK_EVENT_ID_HEADER)?,
        timestamp: required_header(&headers, WEBHOOK_EVENT_TIMESTAMP_HEADER)?,
        signature: required_header(&headers, WEBHOOK_EVENT_SIGNATURE_HEADER)?,
        retry_count: required_header(&headers, WEBHOOK_EVENT_RETRY_COUNT_HEADER)?,
        idempotency_key: required_header(&headers, WEBHOOK_IDEMPOTENCY_KEY_HEADER)?,
        payload_json,
    };
    let receipt = state
        .receiver
        .receive_drive_webhook(request)
        .await
        .map_err(InternalApiProblem::from)?;
    Ok(success_json(DriveEventReceiptResponse::from(receipt)))
}

pub async fn retrieve_wiki_publication(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path(publication_uuid): Path<String>,
) -> Result<Response, InternalApiProblem> {
    let scope = require_wiki_provider_scope(&state, &context)?;
    let publication = state
        .wiki_provider
        .retrieve_publication(RetrieveWikiPublicPublicationRequest {
            scope,
            publication_uuid,
        })
        .await?;
    Ok(success_json(WikiPublicationResponse::from(publication)))
}

pub async fn resolve_wiki_route(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path(publication_uuid): Path<String>,
    Json(body): Json<ResolveWikiRouteBody>,
) -> Result<Response, InternalApiProblem> {
    let scope = require_wiki_provider_scope(&state, &context)?;
    let resolution = state
        .wiki_provider
        .resolve_route(ResolveWikiPublicRouteRequest {
            scope,
            publication_uuid,
            route: body.route,
            locale: body.locale,
        })
        .await?;
    Ok(success_json(WikiRouteResolutionResponse::from(resolution)))
}

pub async fn retrieve_wiki_content(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path((publication_uuid, content_handle)): Path<(String, String)>,
) -> Result<Response, InternalApiProblem> {
    let scope = require_wiki_provider_scope(&state, &context)?;
    let content = state
        .wiki_provider
        .retrieve_content(RetrieveWikiPublicContentRequest {
            scope,
            publication_uuid,
            content_handle,
        })
        .await?;
    let content_length = content.bytes.len();
    let etag = format!(
        "\"{}-v{}\"",
        content.content_sha256, content.page_public_version
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            safe_header_value(&content.media_type)?,
        )
        .header(header::CONTENT_LENGTH, content_length.to_string())
        .header(header::ETAG, safe_header_value(&etag)?)
        .header(
            header::CACHE_CONTROL,
            "private, max-age=31536000, immutable",
        )
        .header("x-content-type-options", "nosniff")
        .header(
            "x-sdkwork-wiki-page-public-version",
            content.page_public_version.to_string(),
        )
        .body(Body::from(content.bytes))
        .map_err(|_| {
            InternalApiProblem::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "failed to build Wiki content response",
            )
        })
}

pub async fn list_wiki_navigation(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path(publication_uuid): Path<String>,
    Query(query): Query<WikiNavigationQuery>,
) -> Result<Response, InternalApiProblem> {
    let scope = require_wiki_provider_scope(&state, &context)?;
    let page = state
        .wiki_provider
        .list_navigation(ListWikiPublicNavigationPageRequest {
            scope,
            publication_uuid,
            locale: query.locale,
            cursor: query.cursor,
            page_size: query.page_size,
        })
        .await?;
    Ok(public_page_response(page))
}

pub async fn search_wiki_pages(
    State(state): State<InternalApiState>,
    RequireInternalApi(context): RequireInternalApi,
    Path(publication_uuid): Path<String>,
    Query(query): Query<WikiSearchQuery>,
) -> Result<Response, InternalApiProblem> {
    let scope = require_wiki_provider_scope(&state, &context)?;
    let page = state
        .wiki_provider
        .search_pages(SearchWikiPublicPageRequest {
            scope,
            publication_uuid,
            query: query.q,
            locale: query.locale,
            cursor: query.cursor,
            page_size: query.page_size,
        })
        .await?;
    Ok(public_page_response(page))
}

fn public_page_response(
    page: sdkwork_intelligence_knowledgebase_service::wiki_public_provider::WikiPublicPageList,
) -> Response {
    let has_more = page.next_cursor.is_some();
    success_list_json(SdkWorkPageData {
        items: page
            .items
            .into_iter()
            .map(WikiPublicPageResponse::from)
            .collect(),
        page_info: PageInfo {
            mode: PageMode::Cursor,
            page: None,
            page_size: Some(page.page_size as i32),
            total_items: None,
            total_pages: None,
            next_cursor: page.next_cursor,
            has_more: Some(has_more),
        },
    })
}

fn require_wiki_provider_scope(
    state: &InternalApiState,
    context: &sdkwork_web_core::WebRequestContext,
) -> Result<WikiPersistenceScope, InternalApiProblem> {
    let principal = context
        .principal
        .as_ref()
        .ok_or_else(InternalApiProblem::unauthorized)?;
    if principal.app_id() != state.wiki_provider_caller_app_id.as_ref() {
        return Err(InternalApiProblem::forbidden(
            "the internal caller is not authorized for Wiki provider access",
        ));
    }
    let tenant_id = parse_canonical_positive_signed_i64(principal.tenant_id()).map_err(|_| {
        InternalApiProblem::new(
            StatusCode::UNAUTHORIZED,
            "authentication_required",
            "internal principal tenant scope is invalid",
        )
    })?;
    let organization_id =
        parse_canonical_nonnegative_signed_i64(principal.organization_id().unwrap_or("0"))
            .map_err(|_| {
                InternalApiProblem::new(
                    StatusCode::UNAUTHORIZED,
                    "authentication_required",
                    "internal principal organization scope is invalid",
                )
            })?;
    Ok(WikiPersistenceScope {
        tenant_id,
        organization_id,
    })
}

fn safe_header_value(value: &str) -> Result<HeaderValue, InternalApiProblem> {
    HeaderValue::from_str(value).map_err(|_| {
        InternalApiProblem::new(
            StatusCode::BAD_GATEWAY,
            "wiki_public_content_integrity_failed",
            "Wiki provider returned invalid response metadata",
        )
    })
}

fn required_header(headers: &HeaderMap, name: &str) -> Result<String, InternalApiProblem> {
    let name = HeaderName::from_bytes(name.as_bytes()).map_err(|_| {
        InternalApiProblem::new(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "internal webhook header configuration is invalid",
        )
    })?;
    let mut values = headers.get_all(name).iter();
    let value = values.next().ok_or_else(|| {
        InternalApiProblem::new(
            axum::http::StatusCode::BAD_REQUEST,
            "missing_required_field",
            "required Drive event header is missing",
        )
    })?;
    if values.next().is_some() {
        return Err(InternalApiProblem::new(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_parameter",
            "a Drive event header must appear at most once",
        ));
    }
    let value = value.to_str().map_err(|_| {
        InternalApiProblem::new(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_parameter",
            "a Drive event header must be valid ASCII",
        )
    })?;
    if value.is_empty() || value.len() > 4096 {
        return Err(InternalApiProblem::new(
            axum::http::StatusCode::BAD_REQUEST,
            "invalid_parameter",
            "a Drive event header is outside its length limit",
        ));
    }
    Ok(value.to_owned())
}
