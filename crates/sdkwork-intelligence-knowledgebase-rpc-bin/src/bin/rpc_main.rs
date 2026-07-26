#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    install_process_crypto_provider()?;
    sdkwork_intelligence_knowledgebase_rpc_bin::run_group_knowledge_space_lifecycle_rpc_from_env()
        .await
        .map_err(|error| Box::new(error) as Box<dyn std::error::Error + Send + Sync>)
}

fn install_process_crypto_provider() -> Result<(), std::io::Error> {
    if rustls::crypto::CryptoProvider::get_default().is_some() {
        return Ok(());
    }

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| std::io::Error::other("failed to install the process-level Rustls provider"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_crypto_provider_before_rpc_bootstrap() {
        install_process_crypto_provider().expect("crypto provider should install");
        assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    }
}
