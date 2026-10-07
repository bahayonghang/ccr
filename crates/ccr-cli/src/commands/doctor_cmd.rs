//! 🩺 ccr doctor 命令
//!
//! 聚合 CCR 本地环境、平台配置、当前 profile、认证状态与可选在线探活。

use crate::models::Platform;
use crate::services::doctor_service::{
    DoctorReport, DoctorRunOptions, DoctorService, DoctorStatus,
};
use ccr_core::core::error::Result;
use ccr_core::core::logging::{ColorOutput, OutputStatus};
use clap::Args;
use std::io::IsTerminal;

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    /// 以 JSON 输出诊断结果
    #[arg(long)]
    pub json: bool,

    /// 输出额外的路径、细节与建议
    #[arg(short, long)]
    pub verbose: bool,

    /// 启用在线 Provider 探活
    #[arg(long)]
    pub online: bool,

    /// 检查所有已配置平台
    #[arg(long, conflicts_with = "platform")]
    pub all_platforms: bool,

    /// 仅检查指定平台
    #[arg(long, value_parser = clap::builder::PossibleValuesParser::new(Platform::all().into_iter().map(|platform| platform.short_name())), conflicts_with = "all_platforms")]
    pub platform: Option<String>,
}

pub async fn doctor_command(args: DoctorArgs) -> Result<()> {
    let report = doctor_report_command(args).await?;
    if report.has_failures() {
        return Err(ccr_core::CcrError::ValidationError(
            "Doctor reported failed checks".into(),
        ));
    }
    Ok(())
}

/// Render a typed report; only the binary chooses the process exit code.
pub async fn doctor_report_command(args: DoctorArgs) -> Result<DoctorReport> {
    let service = DoctorService::new();
    let report = service
        .run(&DoctorRunOptions {
            online: args.online,
            all_platforms: args.all_platforms,
            platform: args.platform.clone(),
        })
        .await;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        render_report(&report, args.verbose);
    }

    Ok(report)
}

fn render_report(report: &DoctorReport, verbose: bool) {
    println!("CCR doctor");
    println!("==========");
    println!();
    println!("Scope: {}", report.scope);
    println!(
        "Online checks: {}",
        if report.online { "enabled" } else { "disabled" }
    );
    println!();

    for check in &report.checks {
        let status = match check.status {
            DoctorStatus::Ok => OutputStatus::Success,
            DoctorStatus::Warn => OutputStatus::Warning,
            DoctorStatus::Fail => OutputStatus::Error,
            DoctorStatus::Skip => OutputStatus::Skipped,
        };
        println!(
            "{}",
            ColorOutput::format_status(status, &check.summary, std::io::stdout().is_terminal())
        );

        if verbose {
            if let Some(path) = &check.path {
                println!("       path: {}", path);
            }
            if let Some(detail) = &check.detail {
                println!("       detail: {}", detail);
            }
            if let Some(recommendation) = &check.recommendation {
                println!("       recommendation: {}", recommendation);
            }
        }
    }

    println!();
    println!(
        "Results: {} passed, {} warnings, {} failed, {} skipped",
        report.summary.passed,
        report.summary.warnings,
        report.summary.failed,
        report.summary.skipped
    );

    if report.summary.failed == 0 && report.summary.warnings == 0 {
        println!();
        println!("All checks passed! CCR is ready.");
    } else if report.summary.failed == 0 {
        println!();
        println!("Doctor completed with warnings.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    #[ignore = "invoked by the isolated renderer output test"]
    fn doctor_renderer_probe() {
        ColorOutput::configure_cli_output();
        let report = DoctorReport {
            scope: "synthetic renderer".to_string(),
            online: false,
            summary: crate::services::doctor_service::DoctorSummary {
                passed: 1,
                warnings: 1,
                failed: 1,
                skipped: 1,
            },
            checks: [
                (DoctorStatus::Ok, "passed check"),
                (DoctorStatus::Warn, "warning check"),
                (DoctorStatus::Fail, "failed check"),
                (DoctorStatus::Skip, "skipped check"),
            ]
            .into_iter()
            .map(
                |(status, summary)| crate::services::doctor_service::DoctorCheck {
                    id: summary.to_string(),
                    status,
                    summary: summary.to_string(),
                    path: None,
                    detail: None,
                    recommendation: None,
                },
            )
            .collect(),
        };
        println!("DOCTOR_RENDER_BEGIN");
        render_report(&report, false);
        println!("DOCTOR_RENDER_END");
    }

    #[test]
    fn doctor_renderer_keeps_all_statuses_on_stdout() {
        let output = Command::new(std::env::current_exe().expect("test executable path"))
            .args([
                "--exact",
                "commands::doctor_cmd::tests::doctor_renderer_probe",
                "--ignored",
                "--nocapture",
            ])
            .env("TERM", "dumb")
            .env("NO_COLOR", "1")
            .env_remove("CLICOLOR_FORCE")
            .output()
            .expect("renderer probe starts");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let stdout = std::str::from_utf8(&output.stdout).expect("UTF-8 renderer output");
        let report = stdout
            .split_once("DOCTOR_RENDER_BEGIN\n")
            .expect("renderer begin marker")
            .1
            .split_once("DOCTOR_RENDER_END\n")
            .expect("renderer end marker")
            .0;
        for expected in [
            "成功: passed check",
            "警告: warning check",
            "错误: failed check",
            "跳过: skipped check",
            "Results: 1 passed, 1 warnings, 1 failed, 1 skipped",
        ] {
            assert!(report.lines().any(|line| line == expected), "{report}");
        }
        assert!(!report.contains("All checks passed"));
        assert!(!report.contains('\u{1b}'));
    }

    #[test]
    fn doctor_renderer_prints_expected_summary() {
        let mut report = DoctorReport {
            scope: "global + configured Claude/Codex runtimes (claude)".to_string(),
            online: false,
            summary: Default::default(),
            checks: Vec::new(),
        };
        report.summary.passed = 1;
        report
            .checks
            .push(crate::services::doctor_service::DoctorCheck {
                id: "test".to_string(),
                status: DoctorStatus::Ok,
                summary: "Doctor renderer smoke test.".to_string(),
                path: None,
                detail: Some("detail".to_string()),
                recommendation: None,
            });

        render_report(&report, true);
    }
}
