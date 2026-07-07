use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use sdkwork_utils_rust::is_blank;
use sha2::{Digest, Sha256};
use thiserror::Error;

const ENCRYPTED_PREFIX: &str = "kbenc:v1:";
const NONCE_LEN: usize = 12;

#[cfg(test)]
pub(crate) static SECRET_CIPHER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Debug, Error)]
pub enum SecretCipherError {
    #[error("secret encryption failed: {0}")]
    Encrypt(String),
    #[error("secret decryption failed: {0}")]
    Decrypt(String),
    #[error("secret encryption key is not configured")]
    MissingKey,
}

/// Returns true when a master key is available from env or key file.
pub fn encryption_key_configured() -> bool {
    resolve_key_material().is_some()
}

/// Encrypts a secret for at-rest storage. Requires a configured master key.
pub fn encrypt_secret(plaintext: &str) -> Result<String, SecretCipherError> {
    if is_blank(Some(plaintext)) {
        return Ok(String::new());
    }
    if plaintext.starts_with(ENCRYPTED_PREFIX) {
        return Ok(plaintext.to_string());
    }
    let Some(key_material) = resolve_key_material() else {
        return Err(SecretCipherError::MissingKey);
    };

    let cipher = Aes256Gcm::new(GenericArray::from_slice(&derive_aes256_key(&key_material)));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|error| SecretCipherError::Encrypt(error.to_string()))?;

    let mut packed = nonce_bytes.to_vec();
    packed.extend(ciphertext);
    Ok(format!(
        "{ENCRYPTED_PREFIX}{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, packed)
    ))
}

/// Decrypts a stored secret. Plaintext legacy values pass through unchanged.
pub fn decrypt_secret(value: &str) -> Result<String, SecretCipherError> {
    if is_blank(Some(value)) {
        return Ok(String::new());
    }
    if !value.starts_with(ENCRYPTED_PREFIX) {
        return Ok(value.to_string());
    }

    let Some(key_material) = resolve_key_material() else {
        return Err(SecretCipherError::MissingKey);
    };

    let encoded = value.strip_prefix(ENCRYPTED_PREFIX).unwrap_or(value);
    let packed = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
        .map_err(|error| SecretCipherError::Decrypt(error.to_string()))?;
    if packed.len() <= NONCE_LEN {
        return Err(SecretCipherError::Decrypt(
            "ciphertext shorter than nonce length".to_string(),
        ));
    }

    let (nonce_bytes, ciphertext) = packed.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&derive_aes256_key(&key_material)));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|error| SecretCipherError::Decrypt(error.to_string()))?;
    String::from_utf8(plaintext).map_err(|error| SecretCipherError::Decrypt(error.to_string()))
}

pub fn encrypt_optional_secret(value: Option<String>) -> Result<Option<String>, SecretCipherError> {
    match value {
        None => Ok(None),
        Some(entry) => encrypt_secret(entry.as_str()).map(Some),
    }
}

pub fn decrypt_optional_secret(value: Option<String>) -> Result<Option<String>, SecretCipherError> {
    match value {
        None => Ok(None),
        Some(entry) => decrypt_secret(entry.as_str()).map(Some),
    }
}

fn resolve_key_material() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY_FILE") {
        if !is_blank(Some(path.as_str())) {
            if let Ok(contents) = std::fs::read_to_string(path.trim()) {
                let trimmed = contents.trim();
                if !is_blank(Some(trimmed)) {
                    return Some(trimmed.as_bytes().to_vec());
                }
            }
        }
    }

    std::env::var("SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY")
        .ok()
        .filter(|value| !is_blank(Some(value.as_str())))
        .map(|value| value.trim().as_bytes().to_vec())
}

fn derive_aes256_key(material: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(material);
    digest.into()
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::SECRET_CIPHER_TEST_LOCK;

    pub struct TestEncryptionKeyGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
    }

    impl TestEncryptionKeyGuard {
        pub fn with_key(key: &str) -> Self {
            let lock = SECRET_CIPHER_TEST_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            std::env::set_var("SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY", key);
            Self { _lock: lock }
        }

        pub fn without_key() -> Self {
            let lock = SECRET_CIPHER_TEST_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            std::env::remove_var("SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY");
            std::env::remove_var("SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY_FILE");
            Self { _lock: lock }
        }
    }

    impl Drop for TestEncryptionKeyGuard {
        fn drop(&mut self) {
            std::env::remove_var("SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY");
            std::env::remove_var("SDKWORK_KNOWLEDGEBASE_SECRETS_ENCRYPTION_KEY_FILE");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::TestEncryptionKeyGuard;
    use super::*;

    #[test]
    fn encrypt_roundtrip_when_key_configured() {
        let _guard = TestEncryptionKeyGuard::with_key("integration-test-master-key");
        let encrypted = encrypt_secret("super-secret").expect("encrypt secret");
        assert!(encrypted.starts_with(ENCRYPTED_PREFIX));
        let decrypted = decrypt_secret(&encrypted).expect("decrypt secret");
        assert_eq!(decrypted, "super-secret");
    }

    #[test]
    fn encrypt_requires_master_key() {
        let _guard = TestEncryptionKeyGuard::without_key();
        let error = encrypt_secret("super-secret").expect_err("encrypt without key");
        assert!(matches!(error, SecretCipherError::MissingKey));
    }

    #[test]
    fn plaintext_legacy_values_pass_through_on_decrypt() {
        let _guard = TestEncryptionKeyGuard::without_key();
        let value = "legacy-plain-secret";
        assert_eq!(decrypt_secret(value).expect("legacy decrypt"), value);
    }
}
