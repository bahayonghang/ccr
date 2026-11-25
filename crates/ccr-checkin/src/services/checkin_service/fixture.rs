//! Isolated development transport. The service uses this exact provider for every request.
use super::*;
use crate::managers::checkin::builtin_providers::resolve_builtin_for_provider;
use reqwest::Url;
use std::net::IpAddr;

impl CheckinService {
    pub fn with_loopback_fixture(
        directory: PathBuf,
        db: ccr_db::database::DbAccess,
        provider: CheckinProvider,
    ) -> Result<Self> {
        let denied =
            || CheckinServiceError::Provider("Fixture requires a custom loopback provider".into());
        let base = Url::parse(&provider.base_url).map_err(|_| denied())?;
        let literal = base.host_str().unwrap_or_default().trim_matches(['[', ']']);
        if provider.builtin_id.is_some()
            || resolve_builtin_for_provider(&provider).is_some()
            || !matches!(base.scheme(), "http" | "https")
            || !literal.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
            || !base.username().is_empty()
            || base.password().is_some()
            || base.query().is_some()
            || base.fragment().is_some()
        {
            return Err(denied());
        }
        // The service concatenates the stored base and path. Validate those exact targets.
        for path in [
            &provider.checkin_path,
            &provider.balance_path,
            &provider.user_info_path,
        ] {
            let target = Url::parse(&format!(
                "{}{path}",
                provider.base_url.trim_end_matches('/')
            ))
            .map_err(|_| denied())?;
            if !path.starts_with('/')
                || target.origin() != base.origin()
                || !target.username().is_empty()
                || target.password().is_some()
            {
                return Err(denied());
            }
        }
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|_| CheckinServiceError::Network("Cannot create fixture client".into()))?;
        let mut service = Self::with_db_and_client(directory, db, client);
        service.pinned_provider = Some(std::sync::Arc::new(provider));
        Ok(service)
    }
}
