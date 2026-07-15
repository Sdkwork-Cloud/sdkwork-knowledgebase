#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sdkwork_intelligence_knowledgebase_rpc_bin::run_group_knowledge_space_lifecycle_rpc_from_env()
        .await
        .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync>)
}
