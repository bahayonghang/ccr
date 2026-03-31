use std::sync::LazyLock;
use std::time::Duration;

pub static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .user_agent(concat!("ccr/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .pool_max_idle_per_host(5)
        .build()
        .expect("Failed to create HTTP client")
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
