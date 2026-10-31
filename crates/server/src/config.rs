use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::error::{Error, Result};

#[derive(Debug, Clone, clap::Args)]
pub struct Config {
    #[arg(long, env = "MAIL_SIEVE_DATA_DIR", default_value = "./data")]
    pub data_dir: PathBuf,
    #[arg(long, env = "MAIL_SIEVE_BIND", default_value = "127.0.0.1:8080")]
    pub bind: SocketAddr,
    #[arg(long, env = "MAIL_SIEVE_PUBLIC_URL")]
    pub public_url: Option<String>,
    #[arg(long, env = "MAIL_SIEVE_SECRET_KEY", hide_env_values = true)]
    pub secret_key: Option<String>,
    #[arg(long, env = "MAIL_SIEVE_ADMIN_PASSWORD", hide_env_values = true)]
    pub admin_password: Option<String>,
    #[arg(long, env = "MAIL_SIEVE_TRUSTED_HEADER")]
    pub trusted_header: Option<String>,
    #[arg(long, env = "MAIL_SIEVE_GOOGLE_CLIENT_ID")]
    pub google_client_id: Option<String>,
    #[arg(long, env = "MAIL_SIEVE_GOOGLE_CLIENT_SECRET", hide_env_values = true)]
    pub google_client_secret: Option<String>,
    #[arg(long, env = "MAIL_SIEVE_MICROSOFT_CLIENT_ID")]
    pub microsoft_client_id: Option<String>,
    #[arg(
        long,
        env = "MAIL_SIEVE_MICROSOFT_CLIENT_SECRET",
        hide_env_values = true
    )]
    pub microsoft_client_secret: Option<String>,
}

impl Config {
    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("mail-sieve.db")
    }

    pub fn plugins_dir(&self) -> PathBuf {
        self.data_dir.join("plugins")
    }

    pub fn secure_cookies(&self) -> bool {
        self.public_url
            .as_deref()
            .is_some_and(|url| url.starts_with("https://"))
    }

    pub fn public_base(&self) -> String {
        self.public_url
            .clone()
            .unwrap_or_else(|| format!("http://{}", self.bind))
            .trim_end_matches('/')
            .to_string()
    }

    pub fn load_secret_key(&self) -> Result<[u8; 32]> {
        match &self.secret_key {
            Some(encoded) => decode_key(encoded.trim()),
            None => load_or_create_key_file(&self.data_dir.join("secret.key")),
        }
    }
}

fn decode_key(encoded: &str) -> Result<[u8; 32]> {
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| Error::Internal("secret key is not valid base64".into()))?;
    bytes
        .try_into()
        .map_err(|_| Error::Internal("secret key must decode to 32 bytes".into()))
}

fn load_or_create_key_file(path: &Path) -> Result<[u8; 32]> {
    if path.exists() {
        let encoded = std::fs::read_to_string(path).map_err(|e| Error::Internal(e.to_string()))?;
        return decode_key(encoded.trim());
    }
    let mut key = [0u8; 32];
    getrandom::fill(&mut key).map_err(|e| Error::Internal(e.to_string()))?;
    write_private_file(path, STANDARD.encode(key).as_bytes())?;
    tracing::warn!(path = %path.display(), "generated a new secret key; back it up, credentials cannot be decrypted without it");
    Ok(key)
}

fn write_private_file(path: &Path, contents: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .map_err(|e| Error::Internal(e.to_string()))?;
    file.write_all(contents)
        .map_err(|e| Error::Internal(e.to_string()))
}
