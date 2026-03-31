use std::sync::LazyLock;
use std::time::Duration;

pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(concat!("ccr/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(5)
        .build()
        .unwrap_or_else(|err| {
            tracing::warn!(error = %err, "创建定制 HTTP client 失败，回退到默认客户端");
            reqwest::Client::new()
        })
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_client_builds_request() {
        let request = HTTP_CLIENT.get("http://example.invalid").build();
        assert!(request.is_ok());
    }
}
