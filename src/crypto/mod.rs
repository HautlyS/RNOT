use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use std::path::PathBuf;

const NONCE_SIZE: usize = 12;

pub struct TokenEncryption {
    key_file: PathBuf,
}

impl TokenEncryption {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            key_file: config_dir.join(".key"),
        }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let key = self.get_or_create_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)?;

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(BASE64.encode(&result))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        let key = self.get_or_create_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key)?;

        let decoded = BASE64.decode(encrypted)?;

        if decoded.len() < NONCE_SIZE {
            return Err(anyhow::anyhow!("Invalid encrypted data"));
        }

        let (nonce_bytes, ciphertext) = decoded.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

        Ok(String::from_utf8(plaintext)?)
    }

    fn get_or_create_key(&self) -> Result<[u8; 32]> {
        if self.key_file.exists() {
            let key_data = std::fs::read(&self.key_file)?;
            let decoded = BASE64.decode(&key_data)?;

            let mut key = [0u8; 32];
            key.copy_from_slice(&decoded[..32]);
            return Ok(key);
        }

        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);

        let encoded = BASE64.encode(key);
        std::fs::write(&self.key_file, encoded)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.key_file, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(key)
    }
}
