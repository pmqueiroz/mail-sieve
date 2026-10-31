use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const NONCE_LEN: usize = 24;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StoredCredentials {
    Password {
        password: String,
    },
    OAuth2 {
        provider: String,
        refresh_token: String,
        access_token: Option<String>,
        expires_at: Option<i64>,
    },
}

#[derive(Clone)]
pub struct Vault {
    cipher: XChaCha20Poly1305,
}

impl Vault {
    pub fn new(key: &[u8; 32]) -> Vault {
        Vault {
            cipher: XChaCha20Poly1305::new_from_slice(key).expect("key is 32 bytes"),
        }
    }

    pub fn seal(&self, credentials: &StoredCredentials) -> Result<Vec<u8>> {
        let plaintext =
            serde_json::to_vec(credentials).map_err(|e| Error::Internal(e.to_string()))?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        getrandom::fill(&mut nonce_bytes).map_err(|e| Error::Internal(e.to_string()))?;
        let nonce = XNonce::from(nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_slice())
            .map_err(|_| Error::Internal("credential encryption failed".into()))?;
        let mut sealed = nonce_bytes.to_vec();
        sealed.extend(ciphertext);
        Ok(sealed)
    }

    pub fn open(&self, sealed: &[u8]) -> Result<StoredCredentials> {
        if sealed.len() <= NONCE_LEN {
            return Err(Error::Internal("stored credentials are corrupt".into()));
        }
        let (nonce_bytes, ciphertext) = sealed.split_at(NONCE_LEN);
        let nonce_bytes: [u8; NONCE_LEN] = nonce_bytes.try_into().expect("split at nonce length");
        let plaintext = self
            .cipher
            .decrypt(&XNonce::from(nonce_bytes), ciphertext)
            .map_err(|_| {
                Error::Internal(
                    "cannot decrypt stored credentials; was the secret key changed?".into(),
                )
            })?;
        serde_json::from_slice(&plaintext).map_err(|e| Error::Internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_rejects_wrong_key() {
        let vault = Vault::new(&[7; 32]);
        let credentials = StoredCredentials::Password {
            password: "hunter2".into(),
        };
        let sealed = vault.seal(&credentials).unwrap();
        assert!(!String::from_utf8_lossy(&sealed).contains("hunter2"));
        assert_eq!(vault.open(&sealed).unwrap(), credentials);
        assert_ne!(sealed, vault.seal(&credentials).unwrap());
        assert!(Vault::new(&[8; 32]).open(&sealed).is_err());
        assert!(vault.open(&sealed[..10]).is_err());
    }
}
