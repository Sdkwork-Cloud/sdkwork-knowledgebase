use async_trait::async_trait;
use sdkwork_intelligence_knowledgebase_service::ports::{
    knowledge_wiki_persistence::{
        WikiIndexState, WikiPagePublicationState, WikiPersistenceError, WikiPersistenceScope,
        WikiPublication, WikiPublicationStatus, WikiSourceProjection, WikiSourceState,
        WikiVisibility,
    },
    knowledge_wiki_publication_lifecycle::{
        ChangeWikiPageVisibilityRequest, ChangeWikiPublicationStatusRequest,
        PublishWikiPageRequest, UnpublishWikiPageRequest, WikiLifecycleAuditContext,
        WikiLifecycleDisposition, WikiPageLifecycleResult, WikiPublicationLifecycleAction,
        WikiPublicationLifecycleResult, WikiPublicationLifecycleStore,
    },
};
use sdkwork_utils_rust::uuid;
use serde_json::json;
use sqlx::{Any, Transaction};

use super::{
    projection::{projection_from_row, PROJECTION_COLUMNS},
    publication::{publication_from_row, PUBLICATION_COLUMNS},
    require_id, require_text, sql_error, to_i64, validate_scope, SqlxWikiPersistenceStore,
};

const PROVIDER_CHANGED_EVENT: &str = "knowledgebase.wiki.provider.changed.v1";
const ROUTE_CHANGED_EVENT: &str = "knowledgebase.wiki.route.changed.v1";
const ROUTE_REVOKED_EVENT: &str = "knowledgebase.wiki.route.revoked.v1";
const NAVIGATION_CHANGED_EVENT: &str = "knowledgebase.wiki.navigation.changed.v1";
const SEARCH_CHANGED_EVENT: &str = "knowledgebase.wiki.search.changed.v1";
const PUBLICATION_ACTIVATED_AUDIT_EVENT: &str = "knowledge.wiki.publication.activated";
const PUBLICATION_PAUSED_AUDIT_EVENT: &str = "knowledge.wiki.publication.paused";
const SOURCE_FILE_PUBLISHED_AUDIT_EVENT: &str = "knowledge.wiki.source_file.published";
const SOURCE_FILE_UNPUBLISHED_AUDIT_EVENT: &str = "knowledge.wiki.source_file.unpublished";
const SOURCE_FILE_VISIBILITY_AUDIT_EVENT: &str = "knowledge.wiki.source_file.visibility_changed";

#[async_trait]
impl WikiPublicationLifecycleStore for SqlxWikiPersistenceStore {
    async fn change_publication_status(
        &self,
        request: ChangeWikiPublicationStatusRequest,
    ) -> Result<WikiPublicationLifecycleResult, WikiPersistenceError> {
        validate_scope(request.scope)?;
        require_id("space_id", request.space_id)?;
        let actor_id = require_id("actor_id", request.actor_id)?;
        validate_audit_context(&request.audit)?;
        let mut transaction = self.pool.begin().await.map_err(sql_error)?;
        let current =
            load_publication_for_space(self, &mut transaction, request.scope, request.space_id)
                .await?;
        ensure_expected_version(
            "wiki_publication",
            current.id,
            current.version,
            request.expected_version,
        )?;

        let target = match request.action {
            WikiPublicationLifecycleAction::Activate => WikiPublicationStatus::Active,
            WikiPublicationLifecycleAction::Pause => WikiPublicationStatus::Paused,
        };
        let now = super::now()?;
        if current.wiki_status == target {
            append_lifecycle_audit(
                self,
                &mut transaction,
                &current,
                None,
                publication_audit_event(request.action),
                request.action.as_operation(),
                request.actor_id,
                &request.audit,
                WikiLifecycleDisposition::Unchanged,
                &now,
            )
            .await?;
            transaction.commit().await.map_err(sql_error)?;
            return Ok(WikiPublicationLifecycleResult {
                publication: current,
                disposition: WikiLifecycleDisposition::Unchanged,
            });
        }
        validate_publication_transition(&current, request.action)?;

        let updated_at = self.dialect.sql_timestamp_expr("$7");
        let activated_at = self.dialect.sql_timestamp_expr("$8");
        let paused_at = self.dialect.sql_timestamp_expr("$9");
        let query = format!(
            r#"
            UPDATE kb_site_publication
            SET wiki_status = $4,
                provider_generation = provider_generation + 1,
                activated_at = CASE WHEN $4 = 'ACTIVE' THEN {activated_at} ELSE activated_at END,
                paused_at = CASE WHEN $4 = 'PAUSED' THEN {paused_at} ELSE NULL END,
                last_error_code = NULL,
                updated_by = $5,
                updated_at = {updated_at},
                version = version + 1
            WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
              AND version = $6 AND status = 1
            RETURNING {PUBLICATION_COLUMNS}
            "#,
        );
        let updated = sqlx::query(&query)
            .bind(to_i64("tenant_id", request.scope.tenant_id)?)
            .bind(to_i64("organization_id", request.scope.organization_id)?)
            .bind(to_i64("site_publication_id", current.id)?)
            .bind(target.as_str())
            .bind(actor_id)
            .bind(to_i64("expected_version", request.expected_version)?)
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .fetch_optional(&mut *transaction)
            .await
            .map_err(sql_error)?
            .ok_or(WikiPersistenceError::StaleVersion {
                resource: "wiki_publication",
                id: current.id,
                expected: request.expected_version,
            })?;
        let publication = publication_from_row(&updated)?;
        append_lifecycle_event(
            self,
            &mut transaction,
            LifecycleEvent {
                publication: &publication,
                page: None,
                event_type: PROVIDER_CHANGED_EVENT,
                operation: request.action.as_operation(),
                route: None,
                page_public_version: None,
                now: &now,
            },
        )
        .await?;
        append_lifecycle_audit(
            self,
            &mut transaction,
            &publication,
            None,
            publication_audit_event(request.action),
            request.action.as_operation(),
            request.actor_id,
            &request.audit,
            WikiLifecycleDisposition::Changed,
            &now,
        )
        .await?;
        transaction.commit().await.map_err(sql_error)?;
        Ok(WikiPublicationLifecycleResult {
            publication,
            disposition: WikiLifecycleDisposition::Changed,
        })
    }

    async fn publish_page(
        &self,
        request: PublishWikiPageRequest,
    ) -> Result<WikiPageLifecycleResult, WikiPersistenceError> {
        validate_page_request(
            request.scope,
            request.space_id,
            &request.source_file_uuid,
            request.actor_id,
            &request.audit,
        )?;
        if request.visibility == WikiVisibility::Private {
            return Err(WikiPersistenceError::InvalidRequest(
                "publishing requires PUBLIC or UNLISTED visibility".to_string(),
            ));
        }
        apply_page_command(
            self,
            PageCommandRequest {
                scope: request.scope,
                space_id: request.space_id,
                source_file_uuid: &request.source_file_uuid,
                expected_publication_version: request.expected_publication_version,
                expected_page_version: request.expected_page_version,
                actor_id: request.actor_id,
                command: PageCommand::Publish(request.visibility),
                audit: &request.audit,
            },
        )
        .await
    }

    async fn unpublish_page(
        &self,
        request: UnpublishWikiPageRequest,
    ) -> Result<WikiPageLifecycleResult, WikiPersistenceError> {
        validate_page_request(
            request.scope,
            request.space_id,
            &request.source_file_uuid,
            request.actor_id,
            &request.audit,
        )?;
        apply_page_command(
            self,
            PageCommandRequest {
                scope: request.scope,
                space_id: request.space_id,
                source_file_uuid: &request.source_file_uuid,
                expected_publication_version: request.expected_publication_version,
                expected_page_version: request.expected_page_version,
                actor_id: request.actor_id,
                command: PageCommand::Unpublish,
                audit: &request.audit,
            },
        )
        .await
    }

    async fn change_page_visibility(
        &self,
        request: ChangeWikiPageVisibilityRequest,
    ) -> Result<WikiPageLifecycleResult, WikiPersistenceError> {
        validate_page_request(
            request.scope,
            request.space_id,
            &request.source_file_uuid,
            request.actor_id,
            &request.audit,
        )?;
        apply_page_command(
            self,
            PageCommandRequest {
                scope: request.scope,
                space_id: request.space_id,
                source_file_uuid: &request.source_file_uuid,
                expected_publication_version: request.expected_publication_version,
                expected_page_version: request.expected_page_version,
                actor_id: request.actor_id,
                command: PageCommand::ChangeVisibility(request.visibility),
                audit: &request.audit,
            },
        )
        .await
    }
}

#[derive(Debug, Clone, Copy)]
enum PageCommand {
    Publish(WikiVisibility),
    Unpublish,
    ChangeVisibility(WikiVisibility),
}

impl PageCommand {
    const fn as_operation(self) -> &'static str {
        match self {
            Self::Publish(_) => "PUBLISH",
            Self::Unpublish => "UNPUBLISH",
            Self::ChangeVisibility(_) => "VISIBILITY_CHANGE",
        }
    }

    const fn audit_event(self) -> &'static str {
        match self {
            Self::Publish(_) => SOURCE_FILE_PUBLISHED_AUDIT_EVENT,
            Self::Unpublish => SOURCE_FILE_UNPUBLISHED_AUDIT_EVENT,
            Self::ChangeVisibility(_) => SOURCE_FILE_VISIBILITY_AUDIT_EVENT,
        }
    }
}

const fn publication_audit_event(action: WikiPublicationLifecycleAction) -> &'static str {
    match action {
        WikiPublicationLifecycleAction::Activate => PUBLICATION_ACTIVATED_AUDIT_EVENT,
        WikiPublicationLifecycleAction::Pause => PUBLICATION_PAUSED_AUDIT_EVENT,
    }
}

struct PageCommandRequest<'a> {
    scope: WikiPersistenceScope,
    space_id: u64,
    source_file_uuid: &'a str,
    expected_publication_version: u64,
    expected_page_version: u64,
    actor_id: u64,
    command: PageCommand,
    audit: &'a WikiLifecycleAuditContext,
}

async fn apply_page_command(
    store: &SqlxWikiPersistenceStore,
    request: PageCommandRequest<'_>,
) -> Result<WikiPageLifecycleResult, WikiPersistenceError> {
    let mut transaction = store.pool.begin().await.map_err(sql_error)?;
    let mut publication =
        load_publication_for_space(store, &mut transaction, request.scope, request.space_id)
            .await?;
    ensure_expected_version(
        "wiki_publication",
        publication.id,
        publication.version,
        request.expected_publication_version,
    )?;
    let current = load_page(
        &mut transaction,
        request.scope,
        publication.id,
        request.source_file_uuid,
    )
    .await?;
    ensure_expected_version(
        "wiki_source_file",
        current.id,
        current.version,
        request.expected_page_version,
    )?;

    if page_command_is_unchanged(&current, request.command) {
        let now = super::now()?;
        append_lifecycle_audit(
            store,
            &mut transaction,
            &publication,
            Some(&current),
            request.command.audit_event(),
            request.command.as_operation(),
            request.actor_id,
            request.audit,
            WikiLifecycleDisposition::Unchanged,
            &now,
        )
        .await?;
        transaction.commit().await.map_err(sql_error)?;
        return Ok(WikiPageLifecycleResult {
            publication,
            page: current,
            disposition: WikiLifecycleDisposition::Unchanged,
        });
    }
    if matches!(request.command, PageCommand::Publish(_)) {
        validate_publish_eligibility(&publication, &current)?;
    }

    let old_public = current.publication_state == WikiPagePublicationState::Published;
    let old_visibility = current.visibility;
    let old_route = current.canonical_route.clone();
    let now = super::now()?;
    let page = update_page(
        store,
        &mut transaction,
        &current,
        request.command,
        request.actor_id,
        request.expected_page_version,
        &now,
    )
    .await?;
    let new_public = page.publication_state == WikiPagePublicationState::Published;
    if !old_public && !new_public {
        append_lifecycle_audit(
            store,
            &mut transaction,
            &publication,
            Some(&page),
            request.command.audit_event(),
            request.command.as_operation(),
            request.actor_id,
            request.audit,
            WikiLifecycleDisposition::Changed,
            &now,
        )
        .await?;
        transaction.commit().await.map_err(sql_error)?;
        return Ok(WikiPageLifecycleResult {
            publication,
            page,
            disposition: WikiLifecycleDisposition::Changed,
        });
    }
    let navigation_or_search_changed = (old_public && old_visibility == WikiVisibility::Public)
        || (new_public && page.visibility == WikiVisibility::Public);
    if navigation_or_search_changed {
        publication = advance_navigation_and_search_generations(
            store,
            &mut transaction,
            &publication,
            request.actor_id,
            request.expected_publication_version,
            &now,
        )
        .await?;
    }

    let (route_event, operation) = page_event(request.command, old_public, new_public);
    let route = if route_event == ROUTE_REVOKED_EVENT {
        old_route.as_deref()
    } else {
        page.canonical_route.as_deref()
    };
    append_lifecycle_event(
        store,
        &mut transaction,
        LifecycleEvent {
            publication: &publication,
            page: Some(&page),
            event_type: route_event,
            operation,
            route,
            page_public_version: Some(page.page_public_version),
            now: &now,
        },
    )
    .await?;
    if navigation_or_search_changed {
        append_lifecycle_event(
            store,
            &mut transaction,
            LifecycleEvent {
                publication: &publication,
                page: Some(&page),
                event_type: NAVIGATION_CHANGED_EVENT,
                operation,
                route,
                page_public_version: Some(page.page_public_version),
                now: &now,
            },
        )
        .await?;
        append_lifecycle_event(
            store,
            &mut transaction,
            LifecycleEvent {
                publication: &publication,
                page: Some(&page),
                event_type: SEARCH_CHANGED_EVENT,
                operation,
                route,
                page_public_version: Some(page.page_public_version),
                now: &now,
            },
        )
        .await?;
    }
    append_lifecycle_audit(
        store,
        &mut transaction,
        &publication,
        Some(&page),
        request.command.audit_event(),
        request.command.as_operation(),
        request.actor_id,
        request.audit,
        WikiLifecycleDisposition::Changed,
        &now,
    )
    .await?;
    transaction.commit().await.map_err(sql_error)?;
    Ok(WikiPageLifecycleResult {
        publication,
        page,
        disposition: WikiLifecycleDisposition::Changed,
    })
}

fn validate_publication_transition(
    publication: &WikiPublication,
    action: WikiPublicationLifecycleAction,
) -> Result<(), WikiPersistenceError> {
    match action {
        WikiPublicationLifecycleAction::Activate => {
            if publication.source_root_node_uuid.is_none()
                || publication.source_scope_uuid.is_none()
            {
                return Err(WikiPersistenceError::Conflict(
                    "Wiki publication must be bound to sources/raw before activation".to_string(),
                ));
            }
            if !matches!(
                publication.wiki_status,
                WikiPublicationStatus::Ready | WikiPublicationStatus::Paused
            ) {
                return Err(WikiPersistenceError::Conflict(format!(
                    "Wiki publication cannot activate from {}",
                    publication.wiki_status.as_str()
                )));
            }
        }
        WikiPublicationLifecycleAction::Pause => {
            if !matches!(
                publication.wiki_status,
                WikiPublicationStatus::Active | WikiPublicationStatus::Degraded
            ) {
                return Err(WikiPersistenceError::Conflict(format!(
                    "Wiki publication cannot pause from {}",
                    publication.wiki_status.as_str()
                )));
            }
        }
    }
    Ok(())
}

fn validate_publish_eligibility(
    publication: &WikiPublication,
    page: &WikiSourceProjection,
) -> Result<(), WikiPersistenceError> {
    if !matches!(
        publication.wiki_status,
        WikiPublicationStatus::Ready
            | WikiPublicationStatus::Active
            | WikiPublicationStatus::Paused
    ) {
        return Err(WikiPersistenceError::Conflict(format!(
            "Wiki page cannot publish while publication is {}",
            publication.wiki_status.as_str()
        )));
    }
    if page.source_state != WikiSourceState::Ready {
        return Err(WikiPersistenceError::Conflict(
            "Wiki page source must be READY before publication".to_string(),
        ));
    }
    if page.canonical_route.as_deref().is_none_or(str::is_empty) {
        return Err(WikiPersistenceError::Conflict(
            "Wiki page must have a canonical route before publication".to_string(),
        ));
    }
    if page.index_state == WikiIndexState::Error {
        return Err(WikiPersistenceError::Conflict(
            "Wiki page with a failed index projection cannot be published".to_string(),
        ));
    }
    Ok(())
}

fn page_command_is_unchanged(page: &WikiSourceProjection, command: PageCommand) -> bool {
    match command {
        PageCommand::Publish(visibility) => {
            page.publication_state == WikiPagePublicationState::Published
                && page.visibility == visibility
                && page.public_drive_version_uuid.as_deref()
                    == Some(page.drive_version_uuid.as_str())
        }
        PageCommand::Unpublish => page.publication_state != WikiPagePublicationState::Published,
        PageCommand::ChangeVisibility(visibility) => {
            if visibility == WikiVisibility::Private {
                page.publication_state != WikiPagePublicationState::Published
                    && page.visibility == WikiVisibility::Private
            } else {
                page.visibility == visibility
            }
        }
    }
}

async fn update_page(
    store: &SqlxWikiPersistenceStore,
    transaction: &mut Transaction<'_, Any>,
    current: &WikiSourceProjection,
    command: PageCommand,
    actor_id: u64,
    expected_page_version: u64,
    now: &str,
) -> Result<WikiSourceProjection, WikiPersistenceError> {
    let updated_at = store.dialect.sql_timestamp_expr("$8");
    let published_at = store.dialect.sql_timestamp_expr("$9");
    let unpublished_at = store.dialect.sql_timestamp_expr("$10");
    let (publication_state, visibility, pin_current_version, advance_public_version) = match command
    {
        PageCommand::Publish(visibility) => {
            (WikiPagePublicationState::Published, visibility, true, true)
        }
        PageCommand::Unpublish => (
            WikiPagePublicationState::Unpublished,
            WikiVisibility::Private,
            false,
            true,
        ),
        PageCommand::ChangeVisibility(WikiVisibility::Private)
            if current.publication_state == WikiPagePublicationState::Published =>
        {
            (
                WikiPagePublicationState::Unpublished,
                WikiVisibility::Private,
                false,
                true,
            )
        }
        PageCommand::ChangeVisibility(visibility) => (
            current.publication_state,
            visibility,
            current.publication_state == WikiPagePublicationState::Published,
            current.publication_state == WikiPagePublicationState::Published,
        ),
    };
    let query = format!(
        r#"
        UPDATE kb_source_file_projection
        SET publication_state = $5,
            visibility = $6,
            public_drive_version_uuid = CASE WHEN $11 THEN drive_version_uuid ELSE NULL END,
            page_public_version = page_public_version + CASE WHEN $12 THEN 1 ELSE 0 END,
            published_at = CASE WHEN $5 = 'PUBLISHED' THEN {published_at} ELSE published_at END,
            unpublished_at = CASE WHEN $5 = 'UNPUBLISHED' THEN {unpublished_at} ELSE NULL END,
            scheduled_publish_at = NULL,
            updated_by = $7,
            updated_at = {updated_at},
            version = version + 1
        WHERE tenant_id = $1 AND organization_id = $2
          AND site_publication_id = $3 AND id = $4 AND version = $13 AND status = 1
        RETURNING {PROJECTION_COLUMNS}
        "#,
    );
    let row = sqlx::query(&query)
        .bind(to_i64("tenant_id", current.scope.tenant_id)?)
        .bind(to_i64("organization_id", current.scope.organization_id)?)
        .bind(to_i64("site_publication_id", current.site_publication_id)?)
        .bind(to_i64("source_file_projection_id", current.id)?)
        .bind(publication_state.as_str())
        .bind(visibility.as_str())
        .bind(require_id("actor_id", actor_id)?)
        .bind(now)
        .bind(now)
        .bind(now)
        .bind(pin_current_version)
        .bind(advance_public_version)
        .bind(to_i64("expected_page_version", expected_page_version)?)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(sql_error)?
        .ok_or(WikiPersistenceError::StaleVersion {
            resource: "wiki_source_file",
            id: current.id,
            expected: expected_page_version,
        })?;
    projection_from_row(&row)
}

async fn advance_navigation_and_search_generations(
    store: &SqlxWikiPersistenceStore,
    transaction: &mut Transaction<'_, Any>,
    publication: &WikiPublication,
    actor_id: u64,
    expected_version: u64,
    now: &str,
) -> Result<WikiPublication, WikiPersistenceError> {
    let updated_at = store.dialect.sql_timestamp_expr("$6");
    let query = format!(
        r#"
        UPDATE kb_site_publication
        SET navigation_generation = navigation_generation + 1,
            search_generation = search_generation + 1,
            updated_by = $4,
            updated_at = {updated_at},
            version = version + 1
        WHERE tenant_id = $1 AND organization_id = $2 AND id = $3
          AND version = $5 AND status = 1
        RETURNING {PUBLICATION_COLUMNS}
        "#,
    );
    let row = sqlx::query(&query)
        .bind(to_i64("tenant_id", publication.scope.tenant_id)?)
        .bind(to_i64(
            "organization_id",
            publication.scope.organization_id,
        )?)
        .bind(to_i64("site_publication_id", publication.id)?)
        .bind(require_id("actor_id", actor_id)?)
        .bind(to_i64("expected_publication_version", expected_version)?)
        .bind(now)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(sql_error)?
        .ok_or(WikiPersistenceError::StaleVersion {
            resource: "wiki_publication",
            id: publication.id,
            expected: expected_version,
        })?;
    publication_from_row(&row)
}

async fn load_publication_for_space(
    _store: &SqlxWikiPersistenceStore,
    transaction: &mut Transaction<'_, Any>,
    scope: WikiPersistenceScope,
    space_id: u64,
) -> Result<WikiPublication, WikiPersistenceError> {
    let query = format!(
        "SELECT {PUBLICATION_COLUMNS} FROM kb_site_publication WHERE tenant_id = $1 AND organization_id = $2 AND space_id = $3 AND status = 1",
    );
    let row = sqlx::query(&query)
        .bind(to_i64("tenant_id", scope.tenant_id)?)
        .bind(to_i64("organization_id", scope.organization_id)?)
        .bind(require_id("space_id", space_id)?)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(sql_error)?
        .ok_or(WikiPersistenceError::NotFound {
            resource: "wiki_publication_for_space",
            id: space_id,
        })?;
    publication_from_row(&row)
}

async fn load_page(
    transaction: &mut Transaction<'_, Any>,
    scope: WikiPersistenceScope,
    site_publication_id: u64,
    source_file_uuid: &str,
) -> Result<WikiSourceProjection, WikiPersistenceError> {
    let query = format!(
        "SELECT {PROJECTION_COLUMNS} FROM kb_source_file_projection WHERE tenant_id = $1 AND organization_id = $2 AND site_publication_id = $3 AND uuid = $4 AND status = 1",
    );
    let row = sqlx::query(&query)
        .bind(to_i64("tenant_id", scope.tenant_id)?)
        .bind(to_i64("organization_id", scope.organization_id)?)
        .bind(to_i64("site_publication_id", site_publication_id)?)
        .bind(source_file_uuid)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(sql_error)?
        .ok_or(WikiPersistenceError::NotFound {
            resource: "wiki_source_file",
            id: 0,
        })?;
    projection_from_row(&row)
}

fn page_event(
    command: PageCommand,
    old_public: bool,
    new_public: bool,
) -> (&'static str, &'static str) {
    match command {
        PageCommand::Publish(_) if old_public => (ROUTE_CHANGED_EVENT, "REPUBLISH"),
        PageCommand::Publish(_) => (ROUTE_CHANGED_EVENT, "PUBLISH"),
        PageCommand::Unpublish => (ROUTE_REVOKED_EVENT, "UNPUBLISH"),
        PageCommand::ChangeVisibility(WikiVisibility::Private) if old_public => {
            (ROUTE_REVOKED_EVENT, "VISIBILITY_PRIVATE")
        }
        PageCommand::ChangeVisibility(_) if new_public => {
            (ROUTE_CHANGED_EVENT, "VISIBILITY_CHANGE")
        }
        PageCommand::ChangeVisibility(_) => (ROUTE_CHANGED_EVENT, "VISIBILITY_CHANGE"),
    }
}

struct LifecycleEvent<'a> {
    publication: &'a WikiPublication,
    page: Option<&'a WikiSourceProjection>,
    event_type: &'a str,
    operation: &'a str,
    route: Option<&'a str>,
    page_public_version: Option<u64>,
    now: &'a str,
}

async fn append_lifecycle_event(
    store: &SqlxWikiPersistenceStore,
    transaction: &mut Transaction<'_, Any>,
    event: LifecycleEvent<'_>,
) -> Result<(), WikiPersistenceError> {
    let outbox_id = store.next_id()?;
    let event_uuid = uuid();
    let payload = json!({
        "id": event_uuid,
        "type": event.event_type,
        "source": "sdkwork-knowledgebase",
        "specversion": "1.0",
        "time": event.now,
        "tenantId": event.publication.scope.tenant_id.to_string(),
        "organizationId": event.publication.scope.organization_id.to_string(),
        "subject": format!("wiki-publication:{}", event.publication.uuid),
        "sequenceNo": outbox_id.to_string(),
        "data": {
            "providerResourceUuid": event.publication.uuid,
            "providerGeneration": event.publication.provider_generation.to_string(),
            "navigationGeneration": event.publication.navigation_generation.to_string(),
            "searchGeneration": event.publication.search_generation.to_string(),
            "sourceFileUuid": event.page.map(|value| value.uuid.as_str()),
            "route": event.route,
            "pagePublicVersion": event.page_public_version.map(|value| value.to_string()),
            "previousPagePublicVersion": event.page_public_version.map(|value| value.saturating_sub(1).to_string()),
            "operation": event.operation,
        }
    });
    let payload_json = serde_json::to_string(&payload)
        .map_err(|error| WikiPersistenceError::Internal(error.to_string()))?;
    let payload_expr = store.dialect.sql_json_expr("$7");
    let created_at = store.dialect.sql_timestamp_expr("$9");
    let query = format!(
        r#"
        INSERT INTO kb_outbox_event (
            id, uuid, tenant_id, aggregate_type, aggregate_id, event_type,
            payload, status, created_at, version
        ) VALUES ($1, $2, $3, $4, $5, $6, {payload_expr}, $8, {created_at}, 0)
        "#,
    );
    sqlx::query(&query)
        .bind(outbox_id)
        .bind(event_uuid)
        .bind(to_i64("tenant_id", event.publication.scope.tenant_id)?)
        .bind("wiki_publication")
        .bind(to_i64("aggregate_id", event.publication.id)?)
        .bind(event.event_type)
        .bind(payload_json)
        .bind(0_i32)
        .bind(event.now)
        .execute(&mut **transaction)
        .await
        .map_err(sql_error)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn append_lifecycle_audit(
    store: &SqlxWikiPersistenceStore,
    transaction: &mut Transaction<'_, Any>,
    publication: &WikiPublication,
    page: Option<&WikiSourceProjection>,
    event_type: &str,
    operation: &str,
    actor_id: u64,
    audit: &WikiLifecycleAuditContext,
    disposition: WikiLifecycleDisposition,
    now: &str,
) -> Result<(), WikiPersistenceError> {
    let payload = json!({
        "organizationId": publication.scope.organization_id.to_string(),
        "spaceId": publication.space_id.to_string(),
        "publicationUuid": publication.uuid,
        "publicationVersion": publication.version.to_string(),
        "sourceFileUuid": page.map(|value| value.uuid.as_str()),
        "pageVersion": page.map(|value| value.version.to_string()),
        "pagePublicVersion": page.map(|value| value.page_public_version.to_string()),
        "visibility": page.map(|value| value.visibility.as_str()),
        "operation": operation,
        "disposition": match disposition {
            WikiLifecycleDisposition::Changed => "CHANGED",
            WikiLifecycleDisposition::Unchanged => "UNCHANGED",
        },
    });
    let payload_json = serde_json::to_string(&payload)
        .map_err(|error| WikiPersistenceError::Internal(error.to_string()))?;
    let payload_expr = store.dialect.sql_json_expr("$12");
    let created_at = store.dialect.sql_timestamp_expr("$13");
    let query = format!(
        r#"
        INSERT INTO kb_audit_event (
            id, uuid, tenant_id, event_type, actor_type, actor_id,
            resource_type, resource_id, result, request_id, trace_id,
            payload, created_at, version
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
            {payload_expr}, {created_at}, $14
        )
        "#,
    );
    let resource_id = page.map_or(publication.id, |value| value.id);
    let resource_type = if page.is_some() {
        "wiki_source_file"
    } else {
        "wiki_publication"
    };
    sqlx::query(&query)
        .bind(store.next_id()?)
        .bind(uuid())
        .bind(to_i64("tenant_id", publication.scope.tenant_id)?)
        .bind(event_type)
        .bind("user")
        .bind(actor_id.to_string())
        .bind(resource_type)
        .bind(to_i64("audit_resource_id", resource_id)?)
        .bind("success")
        .bind(&audit.request_id)
        .bind(audit.trace_id.as_deref())
        .bind(payload_json)
        .bind(now)
        .bind(0_i64)
        .execute(&mut **transaction)
        .await
        .map_err(sql_error)?;
    Ok(())
}

fn validate_page_request(
    scope: WikiPersistenceScope,
    space_id: u64,
    source_file_uuid: &str,
    actor_id: u64,
    audit: &WikiLifecycleAuditContext,
) -> Result<(), WikiPersistenceError> {
    validate_scope(scope)?;
    require_id("space_id", space_id)?;
    require_id("actor_id", actor_id)?;
    require_text("source_file_uuid", source_file_uuid, 64)?;
    validate_audit_context(audit)?;
    Ok(())
}

fn validate_audit_context(audit: &WikiLifecycleAuditContext) -> Result<(), WikiPersistenceError> {
    require_text("audit.request_id", &audit.request_id, 128)?;
    if let Some(trace_id) = audit.trace_id.as_deref() {
        require_text("audit.trace_id", trace_id, 128)?;
    }
    Ok(())
}

fn ensure_expected_version(
    resource: &'static str,
    id: u64,
    actual: u64,
    expected: u64,
) -> Result<(), WikiPersistenceError> {
    if actual != expected {
        return Err(WikiPersistenceError::StaleVersion {
            resource,
            id,
            expected,
        });
    }
    Ok(())
}
