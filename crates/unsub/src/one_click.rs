use crate::guard::{pinned_client, resolve_public, validate_public_url};
use crate::{Error, Result};

pub async fn post_one_click(raw: &str) -> Result<u16> {
    let (url, domain) = validate_public_url(raw)?;
    let addresses = resolve_public(&domain, url.port_or_known_default().unwrap_or(443)).await?;
    let response = pinned_client(&domain, &addresses)?
        .post(url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body("List-Unsubscribe=One-Click")
        .send()
        .await?;
    let status = response.status();
    if status.is_success() || status.is_redirection() {
        Ok(status.as_u16())
    } else {
        Err(Error::Status(status.as_u16()))
    }
}
