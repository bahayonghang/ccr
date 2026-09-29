//! Terminal adapter for the shared read-only validation report.

use crate::services::validate_service::{DiagnosticReport, DiagnosticSeverity, diagnose};
use ccr_core::core::error::{CcrError, Result};

/// Compatibility entry point for embedded callers. No process exit occurs here.
pub async fn validate_command() -> Result<()> {
    let report = validate_report_command();
    if report.has_errors() {
        return Err(CcrError::ValidationError(
            "Validation reported failed checks".into(),
        ));
    }
    Ok(())
}

/// The binary consumes severity and the existing CcrError exit-code mapping.
pub fn validate_report_command() -> DiagnosticReport {
    let report = diagnose();
    println!("配置验证报告 / Validation report");
    for check in &report.checks {
        let severity = match check.severity {
            DiagnosticSeverity::Info => "OK",
            DiagnosticSeverity::Warning => "WARN",
            DiagnosticSeverity::Error => "ERROR",
        };
        let platform = check
            .platform
            .map_or("CCR", |platform| platform.display_name());
        println!(
            "[{severity}/{}] {platform} / {}: {}",
            check.category.as_str(),
            check.target,
            check.message
        );
    }
    println!("Validation exit code: {}", report.exit_code());
    report
}
