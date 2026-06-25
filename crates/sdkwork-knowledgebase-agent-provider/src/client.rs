use sdkwork_agent_kernel::{KnowledgeDocument, KnowledgeDocumentFilter};

pub trait KnowledgebaseRetrievalClient {
    fn retrieve(
        &self,
        request: sdkwork_knowledgebase_contract::KnowledgeRetrievalRequest,
    ) -> Result<sdkwork_knowledgebase_contract::KnowledgeRetrievalResult, String>;

    fn read_document(&self, document_id: &str) -> Result<KnowledgeDocument, String>;

    fn list_documents(
        &self,
        filter: KnowledgeDocumentFilter,
    ) -> Result<Vec<KnowledgeDocument>, String>;
}
