use crate::guard::{pinned_client, resolve_public, validate_public_url};
use crate::{Error, Result};

pub async fn fetch_public_text(raw: &str, max_bytes: usize) -> Result<String> {
    let (url, domain) = validate_public_url(raw)?;
    let addresses = resolve_public(&domain, url.port_or_known_default().unwrap_or(443)).await?;
    let mut response = pinned_client(&domain, &addresses)?.get(url).send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(Error::Status(status.as_u16()));
    }
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(Error::TooLarge(max_bytes));
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len() + chunk.len() > max_bytes {
            return Err(Error::TooLarge(max_bytes));
        }
        body.extend_from_slice(&chunk);
    }
    Ok(String::from_utf8_lossy(&body).into_owned())
}
