//! Test-only request-context fixtures.
//!
//! This module is only compiled in debug builds (`#[cfg(debug_assertions)]`).
//! Production/runtime assembly must authenticate through `sdkwork-iam` and
//! `sdkwork-iam-web-adapter` instead of injecting context with these helpers.

use axum::{
    extract::Request,
    middleware::{self, Next},
    Extension, Router,
};
use sdkwork_routes_knowledgebase_backend_api::{
    permission::KNOWLEDGE_ADMIN_PERMISSION, KnowledgeBackendRequestContext,
};
use sdkwork_routes_knowledgebase_open_api::KnowledgeOpenApiRequestContext;

use crate::KnowledgeAppRequestContext;

pub fn with_dev_app_auth(router: Router, tenant_id: u64, actor_id: Option<u64>) -> Router {
    router.layer(middleware::from_fn(
        move |mut request: Request, next: Next| {
            let actor_id = actor_id;
            async move {
                if request
                    .extensions()
                    .get::<KnowledgeAppRequestContext>()
                    .is_none()
                {
                    request.extensions_mut().insert(KnowledgeAppRequestContext {
                        tenant_id,
                        actor_id,
                        organization_id: None,
                        session_id: None,
                    });
                }
                next.run(request).await
            }
        },
    ))
}

pub fn with_dev_backend_auth(router: Router, tenant_id: u64, operator_id: Option<u64>) -> Router {
    router.layer(middleware::from_fn(
        move |mut request: Request, next: Next| {
            let operator_id = operator_id;
            async move {
                if request
                    .extensions()
                    .get::<KnowledgeBackendRequestContext>()
                    .is_none()
                {
                    request
                        .extensions_mut()
                        .insert(KnowledgeBackendRequestContext {
                            tenant_id,
                            operator_id,
                            organization_id: None,
                            permission_scope: vec![KNOWLEDGE_ADMIN_PERMISSION.to_string()],
                        });
                }
                next.run(request).await
            }
        },
    ))
}

pub fn with_dev_open_auth(router: Router, tenant_id: u64, actor_id: Option<u64>) -> Router {
    router.layer(middleware::from_fn(
        move |mut request: Request, next: Next| {
            let actor_id = actor_id;
            async move {
                if request
                    .extensions()
                    .get::<KnowledgeOpenApiRequestContext>()
                    .is_none()
                {
                    request
                        .extensions_mut()
                        .insert(KnowledgeOpenApiRequestContext {
                            api_key_id: "dev-local".to_string(),
                            tenant_id,
                            actor_id,
                            organization_id: None,
                        });
                }
                next.run(request).await
            }
        },
    ))
}

pub fn inject_dev_app_context(
    tenant_id: u64,
    actor_id: Option<u64>,
) -> Extension<KnowledgeAppRequestContext> {
    Extension(KnowledgeAppRequestContext {
        tenant_id,
        actor_id,
        organization_id: None,
        session_id: None,
    })
}
