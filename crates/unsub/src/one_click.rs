use std::net::{IpAddr, SocketAddr};

use url::{Host, Url};

use crate::{Error, REQUEST_TIMEOUT, Result};

pub fn is_public_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let [a, b, ..] = v4.octets();
            !(v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.is_broadcast()
                || v4.is_documentation()
                || v4.is_unspecified()
                || v4.is_multicast()
                || a == 0
                || (a == 100 && (64..=127).contains(&b))
                || (a == 198 && (b == 18 || b == 19))
                || a >= 240)
        }
        IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return is_public_address(IpAddr::V4(mapped));
            }
            let first = v6.segments()[0];
            !(v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
                || first == 0x2001 && v6.segments()[1] == 0xdb8)
        }
    }
}

pub fn validate_one_click_url(raw: &str) -> Result<(Url, String)> {
    let url = Url::parse(raw).map_err(|_| Error::UnsafeUrl(raw.to_string()))?;
    if url.scheme() != "https" {
        return Err(Error::UnsafeUrl("only https is allowed".into()));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(Error::UnsafeUrl("credentials in url".into()));
    }
    match url.host() {
        Some(Host::Domain(domain)) if domain.contains('.') => {
            let domain = domain.to_string();
            Ok((url, domain))
        }
        _ => Err(Error::UnsafeUrl("host must be a public domain name".into())),
    }
}

async fn resolve_public(domain: &str, port: u16) -> Result<Vec<SocketAddr>> {
    let addresses: Vec<SocketAddr> = tokio::net::lookup_host((domain, port))
        .await
        .map_err(|error| Error::UnsafeUrl(format!("cannot resolve {domain}: {error}")))?
        .collect();
    if addresses.is_empty()
        || addresses
            .iter()
            .any(|address| !is_public_address(address.ip()))
    {
        return Err(Error::UnsafeUrl(format!(
            "{domain} resolves to a non-public address"
        )));
    }
    Ok(addresses)
}

pub async fn post_one_click(raw: &str) -> Result<u16> {
    let (url, domain) = validate_one_click_url(raw)?;
    let addresses = resolve_public(&domain, url.port_or_known_default().unwrap_or(443)).await?;
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(REQUEST_TIMEOUT)
        .resolve_to_addrs(&domain, &addresses)
        .user_agent(concat!("mail-sieve/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let response = client
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_public_targets() {
        for blocked in [
            "127.0.0.1",
            "10.1.2.3",
            "192.168.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "::1",
            "fd00::1",
            "fe80::1",
            "::ffff:10.0.0.1",
        ] {
            assert!(
                !is_public_address(blocked.parse().unwrap()),
                "{blocked} should be blocked"
            );
        }
        for allowed in ["1.1.1.1", "2606:4700:4700::1111"] {
            assert!(
                is_public_address(allowed.parse().unwrap()),
                "{allowed} should be allowed"
            );
        }
    }

    #[test]
    fn validates_url_shape() {
        assert!(validate_one_click_url("https://news.example.com/u?id=1").is_ok());
        assert!(validate_one_click_url("http://news.example.com/u").is_err());
        assert!(validate_one_click_url("https://10.0.0.1/u").is_err());
        assert!(validate_one_click_url("https://[::1]/u").is_err());
        assert!(validate_one_click_url("https://localhost/u").is_err());
        assert!(validate_one_click_url("https://user:pw@news.example.com/u").is_err());
    }
}
