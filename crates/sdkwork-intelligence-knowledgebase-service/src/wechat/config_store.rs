use crate::ports::knowledge_drive_storage::{
    HeadKnowledgeObjectRequest, KnowledgeDriveStorage, KnowledgeStorageError,
    PutKnowledgeObjectRequest,
};
use crate::wechat::secret_cipher::{decrypt_optional_secret, encrypt_optional_secret};
use sdkwork_knowledgebase_contract::wechat::{
    KnowledgeWechatApplet, KnowledgeWechatOfficialAccount,
};
use sdkwork_utils_rust::{is_blank, sha256_hash};
use serde::{Deserialize, Serialize};

const CONFIG_LOGICAL_PATH: &str = "wechat/v1/config.json";
const CONFIG_OBJECT_ROLE: &str = "wechat_config";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TenantWechatConfig {
    #[serde(default)]
    official_accounts: Vec<KnowledgeWechatOfficialAccount>,
    #[serde(default)]
    applets: Vec<KnowledgeWechatApplet>,
}

pub struct WechatConfigStore<'a> {
    drive: &'a dyn KnowledgeDriveStorage,
    tenant_space_uuid: String,
}

impl<'a> WechatConfigStore<'a> {
    pub fn new(drive: &'a dyn KnowledgeDriveStorage, tenant_id: &str) -> Self {
        Self {
            drive,
            tenant_space_uuid: tenant_config_space_uuid(tenant_id),
        }
    }

    pub async fn load_official_accounts(
        &self,
    ) -> Result<Vec<KnowledgeWechatOfficialAccount>, KnowledgeStorageError> {
        let config = self.load_config().await?;
        Ok(config
            .official_accounts
            .into_iter()
            .map(redact_official_account)
            .collect())
    }

    pub async fn replace_official_accounts(
        &self,
        accounts: Vec<KnowledgeWechatOfficialAccount>,
    ) -> Result<Vec<KnowledgeWechatOfficialAccount>, KnowledgeStorageError> {
        validate_official_accounts(&accounts)?;
        let existing = self.load_config().await.unwrap_or_default();
        let mut config = existing;
        config.official_accounts =
            merge_official_account_secrets(accounts, &config.official_accounts);
        self.save_config(&config).await?;
        Ok(config
            .official_accounts
            .into_iter()
            .map(redact_official_account)
            .collect())
    }

    pub async fn load_applets(&self) -> Result<Vec<KnowledgeWechatApplet>, KnowledgeStorageError> {
        let config = self.load_config().await?;
        Ok(config.applets.into_iter().map(redact_applet).collect())
    }

    pub async fn replace_applets(
        &self,
        applets: Vec<KnowledgeWechatApplet>,
    ) -> Result<Vec<KnowledgeWechatApplet>, KnowledgeStorageError> {
        validate_applets(&applets)?;
        let existing = self.load_config().await.unwrap_or_default();
        let mut config = existing;
        config.applets = merge_applet_secrets(applets, &config.applets);
        self.save_config(&config).await?;
        Ok(config.applets.into_iter().map(redact_applet).collect())
    }

    pub async fn find_official_account(
        &self,
        account_id: &str,
    ) -> Result<Option<KnowledgeWechatOfficialAccount>, KnowledgeStorageError> {
        let config = self.load_config().await?;
        Ok(config
            .official_accounts
            .into_iter()
            .find(|account| account.id == account_id))
    }

    async fn load_config(&self) -> Result<TenantWechatConfig, KnowledgeStorageError> {
        let head_request =
            HeadKnowledgeObjectRequest::managed_artifact(CONFIG_LOGICAL_PATH, CONFIG_OBJECT_ROLE)
                .with_space_uuid(self.tenant_space_uuid.as_str());
        let object_ref = match self.drive.head_object(head_request).await {
            Ok(object_ref) => object_ref,
            Err(KnowledgeStorageError::NotFound(_)) => return Ok(TenantWechatConfig::default()),
            Err(error) => return Err(error),
        };
        let body = self.drive.get_object_text(&object_ref).await?;
        let mut config: TenantWechatConfig = serde_json::from_str(&body).map_err(|error| {
            KnowledgeStorageError::Internal(format!("invalid wechat config json: {error}"))
        })?;
        decrypt_config_secrets(&mut config)?;
        Ok(config)
    }

    async fn save_config(&self, config: &TenantWechatConfig) -> Result<(), KnowledgeStorageError> {
        let mut encrypted = config.clone();
        encrypt_config_secrets(&mut encrypted)?;
        let body = serde_json::to_vec(&encrypted).map_err(|error| {
            KnowledgeStorageError::Internal(format!("failed to encode wechat config: {error}"))
        })?;
        let checksum = format!("sha256:{}", sha256_hash(&body));
        self.drive
            .put_object(PutKnowledgeObjectRequest {
                logical_path: CONFIG_LOGICAL_PATH.to_string(),
                object_role: CONFIG_OBJECT_ROLE.to_string(),
                content_type: "application/json; charset=utf-8".to_string(),
                body,
                checksum_sha256_hex: Some(checksum),
                space_uuid: Some(self.tenant_space_uuid.clone()),
            })
            .await?;
        Ok(())
    }
}

fn tenant_config_space_uuid(tenant_id: &str) -> String {
    format!("tenant-{tenant_id}")
}

fn validate_official_accounts(
    accounts: &[KnowledgeWechatOfficialAccount],
) -> Result<(), KnowledgeStorageError> {
    for account in accounts {
        if is_blank(Some(account.id.as_str())) || is_blank(Some(account.app_id.as_str())) {
            return Err(KnowledgeStorageError::InvalidRequest(
                "official account id and appId are required".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_applets(applets: &[KnowledgeWechatApplet]) -> Result<(), KnowledgeStorageError> {
    for applet in applets {
        if is_blank(Some(applet.id.as_str())) || is_blank(Some(applet.app_id.as_str())) {
            return Err(KnowledgeStorageError::InvalidRequest(
                "applet id and appId are required".to_string(),
            ));
        }
    }
    Ok(())
}

fn merge_official_account_secrets(
    incoming: Vec<KnowledgeWechatOfficialAccount>,
    existing: &[KnowledgeWechatOfficialAccount],
) -> Vec<KnowledgeWechatOfficialAccount> {
    incoming
        .into_iter()
        .map(|mut account| {
            if secret_present(account.app_secret.as_deref()) {
                return account;
            }
            if let Some(previous) = existing.iter().find(|item| item.id == account.id) {
                account.app_secret = previous.app_secret.clone();
                if !secret_present(account.token.as_deref()) {
                    account.token = previous.token.clone();
                }
                if !secret_present(account.encoding_aes_key.as_deref()) {
                    account.encoding_aes_key = previous.encoding_aes_key.clone();
                }
            }
            account
        })
        .collect()
}

fn merge_applet_secrets(
    incoming: Vec<KnowledgeWechatApplet>,
    existing: &[KnowledgeWechatApplet],
) -> Vec<KnowledgeWechatApplet> {
    incoming
        .into_iter()
        .map(|mut applet| {
            if secret_present(applet.app_secret.as_deref()) {
                return applet;
            }
            if let Some(previous) = existing.iter().find(|item| item.id == applet.id) {
                applet.app_secret = previous.app_secret.clone();
                if !secret_present(applet.msg_token.as_deref()) {
                    applet.msg_token = previous.msg_token.clone();
                }
                if !secret_present(applet.msg_encoding_aes_key.as_deref()) {
                    applet.msg_encoding_aes_key = previous.msg_encoding_aes_key.clone();
                }
            }
            applet
        })
        .collect()
}

fn secret_present(value: Option<&str>) -> bool {
    value.is_some_and(|secret| !is_blank(Some(secret)))
}

fn redact_official_account(
    mut account: KnowledgeWechatOfficialAccount,
) -> KnowledgeWechatOfficialAccount {
    account.app_secret = None;
    account.token = None;
    account.encoding_aes_key = None;
    account
}

fn redact_applet(mut applet: KnowledgeWechatApplet) -> KnowledgeWechatApplet {
    applet.app_secret = None;
    applet.msg_token = None;
    applet.msg_encoding_aes_key = None;
    applet
}

fn decrypt_config_secrets(config: &mut TenantWechatConfig) -> Result<(), KnowledgeStorageError> {
    for account in &mut config.official_accounts {
        account.app_secret =
            decrypt_optional_secret(account.app_secret.clone()).map_err(cipher_storage_error)?;
        account.token =
            decrypt_optional_secret(account.token.clone()).map_err(cipher_storage_error)?;
        account.encoding_aes_key = decrypt_optional_secret(account.encoding_aes_key.clone())
            .map_err(cipher_storage_error)?;
    }
    for applet in &mut config.applets {
        applet.app_secret =
            decrypt_optional_secret(applet.app_secret.clone()).map_err(cipher_storage_error)?;
        applet.msg_token =
            decrypt_optional_secret(applet.msg_token.clone()).map_err(cipher_storage_error)?;
        applet.msg_encoding_aes_key = decrypt_optional_secret(applet.msg_encoding_aes_key.clone())
            .map_err(cipher_storage_error)?;
    }
    Ok(())
}

fn encrypt_config_secrets(config: &mut TenantWechatConfig) -> Result<(), KnowledgeStorageError> {
    for account in &mut config.official_accounts {
        account.app_secret =
            encrypt_optional_secret(account.app_secret.clone()).map_err(cipher_storage_error)?;
        account.token =
            encrypt_optional_secret(account.token.clone()).map_err(cipher_storage_error)?;
        account.encoding_aes_key = encrypt_optional_secret(account.encoding_aes_key.clone())
            .map_err(cipher_storage_error)?;
    }
    for applet in &mut config.applets {
        applet.app_secret =
            encrypt_optional_secret(applet.app_secret.clone()).map_err(cipher_storage_error)?;
        applet.msg_token =
            encrypt_optional_secret(applet.msg_token.clone()).map_err(cipher_storage_error)?;
        applet.msg_encoding_aes_key = encrypt_optional_secret(applet.msg_encoding_aes_key.clone())
            .map_err(cipher_storage_error)?;
    }
    Ok(())
}

fn cipher_storage_error(
    error: crate::wechat::secret_cipher::SecretCipherError,
) -> KnowledgeStorageError {
    KnowledgeStorageError::Internal(format!("wechat secret cipher error: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_official_account_secrets_preserves_existing_secret() {
        let existing = vec![KnowledgeWechatOfficialAccount {
            id: "1".to_string(),
            name: "A".to_string(),
            account_type: "subscription".to_string(),
            avatar: "A".to_string(),
            description: None,
            app_id: "wx1".to_string(),
            app_secret: Some("secret".to_string()),
            server_url: None,
            token: Some("token".to_string()),
            encoding_aes_key: None,
            encrypt_mode: None,
            domain_verify_file_name: None,
            domain_verify_file_content: None,
            js_secure_domains: None,
            web_auth_domains: None,
            business_domains: None,
            group: None,
        }];
        let incoming = vec![KnowledgeWechatOfficialAccount {
            id: "1".to_string(),
            name: "A".to_string(),
            account_type: "subscription".to_string(),
            avatar: "A".to_string(),
            description: None,
            app_id: "wx1".to_string(),
            app_secret: None,
            server_url: None,
            token: None,
            encoding_aes_key: None,
            encrypt_mode: None,
            domain_verify_file_name: None,
            domain_verify_file_content: None,
            js_secure_domains: None,
            web_auth_domains: None,
            business_domains: None,
            group: None,
        }];
        let merged = merge_official_account_secrets(incoming, &existing);
        assert_eq!(merged[0].app_secret.as_deref(), Some("secret"));
        assert_eq!(merged[0].token.as_deref(), Some("token"));
    }

    #[test]
    fn encrypt_config_secrets_writes_encrypted_prefix_when_key_configured() {
        let _guard = crate::wechat::secret_cipher::test_support::TestEncryptionKeyGuard::with_key(
            "config-store-test-key",
        );
        let mut config = TenantWechatConfig {
            official_accounts: vec![KnowledgeWechatOfficialAccount {
                id: "1".to_string(),
                name: "A".to_string(),
                account_type: "subscription".to_string(),
                avatar: "A".to_string(),
                description: None,
                app_id: "wx1".to_string(),
                app_secret: Some("secret".to_string()),
                server_url: None,
                token: None,
                encoding_aes_key: None,
                encrypt_mode: None,
                domain_verify_file_name: None,
                domain_verify_file_content: None,
                js_secure_domains: None,
                web_auth_domains: None,
                business_domains: None,
                group: None,
            }],
            applets: vec![],
        };
        encrypt_config_secrets(&mut config).expect("encrypt config secrets");
        assert!(config.official_accounts[0]
            .app_secret
            .as_deref()
            .unwrap()
            .starts_with("kbenc:v1:"));
        decrypt_config_secrets(&mut config).expect("decrypt config secrets");
        assert_eq!(
            config.official_accounts[0].app_secret.as_deref(),
            Some("secret")
        );
    }
}
