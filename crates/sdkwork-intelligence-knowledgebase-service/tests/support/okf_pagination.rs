use sdkwork_intelligence_knowledgebase_service::ports::knowledge_okf_concept_store::KnowledgeOkfConceptStoreError;
use sdkwork_utils_rust::MAX_LIST_PAGE_SIZE;

pub(crate) fn validated_okf_test_page_size(
    page_size: u32,
) -> Result<usize, KnowledgeOkfConceptStoreError> {
    let max_page_size = u32::try_from(MAX_LIST_PAGE_SIZE).map_err(|_| {
        KnowledgeOkfConceptStoreError::Internal(
            "MAX_LIST_PAGE_SIZE must fit the OKF u32 page-size contract".to_string(),
        )
    })?;
    if !(1..=max_page_size).contains(&page_size) {
        return Err(KnowledgeOkfConceptStoreError::Internal(format!(
            "invalid OKF page size: {page_size}"
        )));
    }
    usize::try_from(page_size).map_err(|_| {
        KnowledgeOkfConceptStoreError::Internal(format!(
            "OKF page size does not fit usize: {page_size}"
        ))
    })
}
