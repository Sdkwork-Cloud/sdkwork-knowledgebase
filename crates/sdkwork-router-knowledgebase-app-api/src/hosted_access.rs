use axum::http::StatusCode;
use sdkwork_intelligence_knowledgebase_service::{
    okf::{OkfBundleFileRegistryService, OkfBundleInitializerService},
    ports::knowledge_ingestion_job_store::IngestionJobStore,
    space::KnowledgeSpaceService,
};
use sdkwork_knowledgebase_contract::{
    ingest::IngestionJob, GrantKnowledgeSpaceMemberRequest, KnowledgeDocument, KnowledgeSpace,
    KnowledgeSpaceMember, KnowledgeSpaceMemberList, KnowledgeSpaceMemberRole,
    KnowledgeSpaceMemberSubjectType, UpdateKnowledgeSpaceRequest,
};
use sdkwork_intelligence_knowledgebase_service::ports::knowledge_access_control::{
    KnowledgeAccessRole, KnowledgeSpaceMember as ServiceSpaceMember, KnowledgeSubjectType,
};

use crate::{
    error::ApiError,
    hosted::map_okf_concept_store_error,
    ports::KnowledgeAppRequestContext,
    runtime::KnowledgebaseRuntime,
    ApiResult,
};

pub(crate) fn ensure_runtime_tenant(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
) -> ApiResult<()> {
    if context.tenant_id != runtime.tenant_id() {
        return Err(ApiError::new(
            StatusCode::FORBIDDEN,
            "tenant_id_mismatch",
            "authenticated tenant does not match configured runtime tenant",
        ));
    }
    Ok(())
}

pub(crate) fn require_actor_id(context: &KnowledgeAppRequestContext) -> ApiResult<String> {
    context
        .actor_id
        .map(|value| value.to_string())
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::UNAUTHORIZED,
                "missing_actor_id",
                "authenticated actor_id is required for this operation",
            )
        })
}

pub(crate) async fn require_space_access(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    space_id: u64,
) -> ApiResult<KnowledgeSpace> {
    ensure_runtime_tenant(runtime, context)?;
    let actor_id = require_actor_id(context)?;
    let file_registry = OkfBundleFileRegistryService::new(runtime.okf_bundle_file_store());
    let okf_initializer = OkfBundleInitializerService::new(runtime.drive_storage())
        .with_registry(&file_registry)
        .with_drive_workspace(runtime.drive_workspace());
    KnowledgeSpaceService::new(runtime.space_store(), &okf_initializer)
        .with_drive_context(runtime.tenant_id_str(), runtime.operator_id())
        .with_drive_space_provisioner(runtime.drive_space_provisioner())
        .with_access_control(runtime.access_control())
        .get_space_with_access_check(space_id, &context.tenant_id.to_string(), &actor_id)
        .await
        .map_err(ApiError::from)
}

pub(crate) async fn require_document_access(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    document_id: u64,
) -> ApiResult<KnowledgeDocument> {
    ensure_runtime_tenant(runtime, context)?;
    let document = runtime
        .document_store()
        .get_document_by_id(document_id)
        .await
        .map_err(ApiError::from)?;
    require_space_access(runtime, context, document.space_id).await?;
    Ok(document)
}

pub(crate) async fn require_ingest_access(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    ingest_id: u64,
) -> ApiResult<IngestionJob> {
    ensure_runtime_tenant(runtime, context)?;
    let job = runtime
        .ingestion_job_store()
        .get_job(ingest_id)
        .await
        .map_err(ApiError::from)?;
    require_space_access(runtime, context, job.space_id).await?;
    Ok(job)
}

pub(crate) async fn require_okf_concept_space_access(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    concept_row_id: u64,
) -> ApiResult<KnowledgeSpace> {
    ensure_runtime_tenant(runtime, context)?;
    let concept = runtime
        .okf_concept_store()
        .get_concept_by_row_id(concept_row_id)
        .await
        .map_err(map_okf_concept_store_error)?;
    require_space_access(runtime, context, concept.space_id).await
}

pub(crate) async fn create_space_with_context(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    request: sdkwork_knowledgebase_contract::CreateKnowledgeSpaceRequest,
) -> ApiResult<KnowledgeSpace> {
    ensure_runtime_tenant(runtime, context)?;
    let file_registry = OkfBundleFileRegistryService::new(runtime.okf_bundle_file_store());
    let okf_initializer = OkfBundleInitializerService::new(runtime.drive_storage())
        .with_registry(&file_registry)
        .with_drive_workspace(runtime.drive_workspace());
    KnowledgeSpaceService::new(runtime.space_store(), &okf_initializer)
        .with_drive_context(runtime.tenant_id_str(), runtime.operator_id())
        .with_drive_space_provisioner(runtime.drive_space_provisioner())
        .with_access_control(runtime.access_control())
        .create_space(request)
        .await
        .map_err(ApiError::from)
}

pub(crate) async fn update_space_with_context(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    space_id: u64,
    request: UpdateKnowledgeSpaceRequest,
) -> ApiResult<KnowledgeSpace> {
    ensure_runtime_tenant(runtime, context)?;
    let actor_id = require_actor_id(context)?;
    let file_registry = OkfBundleFileRegistryService::new(runtime.okf_bundle_file_store());
    let okf_initializer = OkfBundleInitializerService::new(runtime.drive_storage())
        .with_registry(&file_registry)
        .with_drive_workspace(runtime.drive_workspace());
    KnowledgeSpaceService::new(runtime.space_store(), &okf_initializer)
        .with_drive_context(runtime.tenant_id_str(), runtime.operator_id())
        .with_drive_space_provisioner(runtime.drive_space_provisioner())
        .with_access_control(runtime.access_control())
        .update_space(
            space_id,
            &context.tenant_id.to_string(),
            &actor_id,
            request,
        )
        .await
        .map_err(ApiError::from)
}

pub(crate) async fn delete_space_with_context(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    space_id: u64,
) -> ApiResult<()> {
    ensure_runtime_tenant(runtime, context)?;
    let actor_id = require_actor_id(context)?;
    let file_registry = OkfBundleFileRegistryService::new(runtime.okf_bundle_file_store());
    let okf_initializer = OkfBundleInitializerService::new(runtime.drive_storage())
        .with_registry(&file_registry)
        .with_drive_workspace(runtime.drive_workspace());
    KnowledgeSpaceService::new(runtime.space_store(), &okf_initializer)
        .with_drive_context(runtime.tenant_id_str(), runtime.operator_id())
        .with_drive_space_provisioner(runtime.drive_space_provisioner())
        .with_access_control(runtime.access_control())
        .delete_space(space_id, &context.tenant_id.to_string(), &actor_id)
        .await
        .map_err(ApiError::from)
}

fn map_member_role(role: KnowledgeAccessRole) -> KnowledgeSpaceMemberRole {
    match role {
        KnowledgeAccessRole::Reader => KnowledgeSpaceMemberRole::Reader,
        KnowledgeAccessRole::Writer => KnowledgeSpaceMemberRole::Writer,
        KnowledgeAccessRole::Owner => KnowledgeSpaceMemberRole::Owner,
    }
}

fn map_member_subject_type(
    subject_type: KnowledgeSubjectType,
) -> KnowledgeSpaceMemberSubjectType {
    match subject_type {
        KnowledgeSubjectType::User => KnowledgeSpaceMemberSubjectType::User,
        KnowledgeSubjectType::Group => KnowledgeSpaceMemberSubjectType::Group,
        KnowledgeSubjectType::Domain => KnowledgeSpaceMemberSubjectType::Domain,
        KnowledgeSubjectType::App => KnowledgeSpaceMemberSubjectType::App,
    }
}

fn map_contract_member(member: ServiceSpaceMember) -> KnowledgeSpaceMember {
    KnowledgeSpaceMember {
        subject_type: map_member_subject_type(member.subject_type),
        subject_id: member.subject_id,
        role: map_member_role(member.role),
        inherited: member.inherited,
    }
}

pub(crate) fn parse_member_subject_type(
    subject_type: KnowledgeSpaceMemberSubjectType,
) -> KnowledgeSubjectType {
    match subject_type {
        KnowledgeSpaceMemberSubjectType::User => KnowledgeSubjectType::User,
        KnowledgeSpaceMemberSubjectType::Group => KnowledgeSubjectType::Group,
        KnowledgeSpaceMemberSubjectType::Domain => KnowledgeSubjectType::Domain,
        KnowledgeSpaceMemberSubjectType::App => KnowledgeSubjectType::App,
    }
}

pub(crate) fn parse_member_role(role: KnowledgeSpaceMemberRole) -> KnowledgeAccessRole {
    match role {
        KnowledgeSpaceMemberRole::Reader => KnowledgeAccessRole::Reader,
        KnowledgeSpaceMemberRole::Writer => KnowledgeAccessRole::Writer,
        KnowledgeSpaceMemberRole::Owner => KnowledgeAccessRole::Owner,
    }
}

fn space_service<'a>(
    runtime: &'a KnowledgebaseRuntime,
    okf_initializer: &'a OkfBundleInitializerService,
) -> KnowledgeSpaceService<'a> {
    KnowledgeSpaceService::new(runtime.space_store(), okf_initializer)
        .with_drive_context(runtime.tenant_id_str(), runtime.operator_id())
        .with_drive_space_provisioner(runtime.drive_space_provisioner())
        .with_access_control(runtime.access_control())
}

pub(crate) async fn list_space_members_with_context(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    space_id: u64,
) -> ApiResult<KnowledgeSpaceMemberList> {
    ensure_runtime_tenant(runtime, context)?;
    let actor_id = require_actor_id(context)?;
    let file_registry = OkfBundleFileRegistryService::new(runtime.okf_bundle_file_store());
    let okf_initializer = OkfBundleInitializerService::new(runtime.drive_storage())
        .with_registry(&file_registry)
        .with_drive_workspace(runtime.drive_workspace());
    let members = space_service(runtime, &okf_initializer)
        .list_space_members(space_id, &context.tenant_id.to_string(), &actor_id)
        .await
        .map_err(ApiError::from)?;
    Ok(KnowledgeSpaceMemberList {
        members: members.members.into_iter().map(map_contract_member).collect(),
        next_cursor: members.next_cursor,
    })
}

pub(crate) async fn grant_space_member_with_context(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    space_id: u64,
    request: GrantKnowledgeSpaceMemberRequest,
) -> ApiResult<()> {
    ensure_runtime_tenant(runtime, context)?;
    let actor_id = require_actor_id(context)?;
    if request.subject_id.trim().is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid_subject_id",
            "subjectId must not be blank",
        ));
    }
    let file_registry = OkfBundleFileRegistryService::new(runtime.okf_bundle_file_store());
    let okf_initializer = OkfBundleInitializerService::new(runtime.drive_storage())
        .with_registry(&file_registry)
        .with_drive_workspace(runtime.drive_workspace());
    space_service(runtime, &okf_initializer)
        .grant_space_member(
            space_id,
            &context.tenant_id.to_string(),
            parse_member_subject_type(request.subject_type),
            request.subject_id.trim(),
            parse_member_role(request.role),
            &actor_id,
        )
        .await
        .map_err(ApiError::from)
}

pub(crate) async fn revoke_space_member_with_context(
    runtime: &KnowledgebaseRuntime,
    context: &KnowledgeAppRequestContext,
    space_id: u64,
    subject_type: KnowledgeSpaceMemberSubjectType,
    subject_id: &str,
) -> ApiResult<()> {
    ensure_runtime_tenant(runtime, context)?;
    let actor_id = require_actor_id(context)?;
    if subject_id.trim().is_empty() {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid_subject_id",
            "subjectId must not be blank",
        ));
    }
    let file_registry = OkfBundleFileRegistryService::new(runtime.okf_bundle_file_store());
    let okf_initializer = OkfBundleInitializerService::new(runtime.drive_storage())
        .with_registry(&file_registry)
        .with_drive_workspace(runtime.drive_workspace());
    space_service(runtime, &okf_initializer)
        .revoke_space_member(
            space_id,
            &context.tenant_id.to_string(),
            parse_member_subject_type(subject_type),
            subject_id.trim(),
            &actor_id,
        )
        .await
        .map_err(ApiError::from)
}
