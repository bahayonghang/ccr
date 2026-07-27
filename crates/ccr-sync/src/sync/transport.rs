use ccr_core::core::error::{CcrError, Result};
use reqwest_dav::re_exports::reqwest::{Client as HttpClient, redirect::Policy};
use reqwest_dav::re_exports::url::{Host, Url};
use std::time::Duration;

pub const INSECURE_LOOPBACK_HTTP_ENV: &str = "CCR_ALLOW_INSECURE_WEBDAV_HTTP";

pub fn insecure_loopback_http_enabled() -> bool {
    std::env::var(INSECURE_LOOPBACK_HTTP_ENV)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn validate_webdav_url(raw: &str, allow_insecure_loopback: bool) -> Result<Url> {
    let url =
        Url::parse(raw).map_err(|_| transport_error("invalid_url", "WebDAV 地址不是有效 URL"))?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err(transport_error(
            "embedded_credentials",
            "WebDAV 地址不能内嵌用户名或密码",
        ));
    }

    match url.scheme() {
        "https" => Ok(url),
        "http" if allow_insecure_loopback && is_loopback(&url) => Ok(url),
        "http" => Err(transport_error(
            "https_required",
            "WebDAV 默认要求 HTTPS；HTTP 仅允许 loopback 且必须显式启用开发标志",
        )),
        _ => Err(transport_error(
            "unsupported_scheme",
            "WebDAV 地址只支持 HTTPS",
        )),
    }
}

pub(crate) fn build_http_client(allow_insecure_loopback: bool) -> Result<HttpClient> {
    HttpClient::builder()
        .timeout(Duration::from_secs(120))
        .redirect(Policy::custom(move |attempt| {
            if attempt.previous().len() >= 5 {
                return attempt.error("WebDAV redirect limit exceeded");
            }
            if validate_webdav_url(attempt.url().as_str(), allow_insecure_loopback).is_err() {
                return attempt.error("WebDAV redirect rejected by HTTPS policy");
            }
            attempt.follow()
        }))
        .build()
        .map_err(|error| {
            transport_error("client", &format!("创建 WebDAV HTTP 客户端失败: {error}"))
        })
}

fn is_loopback(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    }
}

fn transport_error(code: &str, message: &str) -> CcrError {
    CcrError::SyncError(format!("sync_transport_{code}: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_is_required_except_explicit_loopback_development_mode() {
        assert!(validate_webdav_url("https://dav.example.com/dav/", false).is_ok());
        assert!(validate_webdav_url("http://dav.example.com/dav/", true).is_err());
        assert!(validate_webdav_url("http://localhost:8080/dav/", false).is_err());
        assert!(validate_webdav_url("http://localhost:8080/dav/", true).is_ok());
        assert!(validate_webdav_url("http://127.0.0.1:8080/dav/", true).is_ok());
        assert!(validate_webdav_url("http://[::1]:8080/dav/", true).is_ok());
    }

    #[test]
    fn credentials_and_non_http_schemes_are_rejected() {
        assert!(validate_webdav_url("https://user:pass@dav.example.com/", false).is_err());
        assert!(validate_webdav_url("file:///tmp/dav", false).is_err());
    }
}
