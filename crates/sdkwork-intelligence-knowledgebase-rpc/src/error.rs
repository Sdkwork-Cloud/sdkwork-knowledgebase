use sdkwork_intelligence_knowledgebase_service::{
    group_space::KnowledgeGroupKnowledgeSpaceServiceError,
    ports::knowledge_group_space_binding_store::KnowledgeGroupSpaceBindingStoreError,
};
use tonic::Status;

pub fn map_group_knowledge_space_service_error(
    error: KnowledgeGroupKnowledgeSpaceServiceError,
) -> Status {
    match error {
        KnowledgeGroupKnowledgeSpaceServiceError::InvalidRequest(_) => {
            Status::invalid_argument("group knowledge-space request is invalid")
        }
        KnowledgeGroupKnowledgeSpaceServiceError::Denied(_) => {
            Status::permission_denied("group knowledge-space access is denied")
        }
        KnowledgeGroupKnowledgeSpaceServiceError::InvalidLifecycle(_) => {
            Status::failed_precondition(
                "group knowledge-space lifecycle does not allow this command",
            )
        }
        KnowledgeGroupKnowledgeSpaceServiceError::MembershipProjectionFencedByArchive => {
            Status::aborted("group knowledge-space membership projection was superseded by archive")
        }
        KnowledgeGroupKnowledgeSpaceServiceError::Provisioning(_) => {
            Status::internal("group knowledge-space provisioning did not complete")
        }
        KnowledgeGroupKnowledgeSpaceServiceError::Binding(binding_error) => {
            map_group_binding_store_error(binding_error)
        }
        KnowledgeGroupKnowledgeSpaceServiceError::SpaceStore(_)
        | KnowledgeGroupKnowledgeSpaceServiceError::Space(_)
        | KnowledgeGroupKnowledgeSpaceServiceError::AccessControl(_)
        | KnowledgeGroupKnowledgeSpaceServiceError::Authorization(_) => {
            Status::internal("group knowledge-space operation failed")
        }
    }
}

fn map_group_binding_store_error(error: KnowledgeGroupSpaceBindingStoreError) -> Status {
    match error {
        KnowledgeGroupSpaceBindingStoreError::InvalidRequest(_) => {
            Status::invalid_argument("group knowledge-space request is invalid")
        }
        KnowledgeGroupSpaceBindingStoreError::NotFound(_) => {
            Status::not_found("group knowledge-space binding was not found")
        }
        KnowledgeGroupSpaceBindingStoreError::Conflict(_) => {
            Status::aborted("group knowledge-space command conflicts with the current binding")
        }
        KnowledgeGroupSpaceBindingStoreError::InvalidLifecycle(_) => Status::failed_precondition(
            "group knowledge-space lifecycle does not allow this command",
        ),
        KnowledgeGroupSpaceBindingStoreError::Internal(_) => {
            Status::internal("group knowledge-space operation failed")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_conflicts_and_terminal_lifecycle_without_leaking_storage_details() {
        let conflict = map_group_knowledge_space_service_error(
            KnowledgeGroupKnowledgeSpaceServiceError::Binding(
                KnowledgeGroupSpaceBindingStoreError::Conflict("internal target value".to_string()),
            ),
        );
        assert_eq!(conflict.code(), tonic::Code::Aborted);
        assert!(!conflict.message().contains("internal target value"));

        let terminal = map_group_knowledge_space_service_error(
            KnowledgeGroupKnowledgeSpaceServiceError::InvalidLifecycle("archived".to_string()),
        );
        assert_eq!(terminal.code(), tonic::Code::FailedPrecondition);
    }
}
