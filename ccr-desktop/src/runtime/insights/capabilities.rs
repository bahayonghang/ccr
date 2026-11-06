use super::*;
pub use ccr_app::usage::capabilities::Report;

impl InsightsRequests {
    pub fn usage_capabilities(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<Report, InsightsError>, AdmissionError> {
        let paths = self.paths.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        let probe_cli = !self.services.is_fixture();
        self.services.executor.read(key, move |_, _| {
            if !local {
                return Err(InsightsError::LocalOnly);
            }
            let paths = paths.ok_or(InsightsError::Unconfigured)?;
            let handle = tokio::runtime::Handle::try_current().map_err(|_| InsightsError::Query)?;
            let mut report = handle.block_on(Report::detect(
                &AppPaths::from_root(paths.usage_root),
                probe_cli,
            ));
            report.cli_version = report.cli_version.map(|version| {
                ccr_core::core::log_redact::redact_log_text(&version)
                    .chars()
                    .take(256)
                    .collect()
            });
            Ok(report)
        })
    }

    pub fn capability_cli_probe_enabled(&self) -> bool {
        !self.services.is_fixture()
    }
}
