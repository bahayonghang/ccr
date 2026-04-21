//! 🩺 ccr doctor 命令
//!
//! 聚合 CCR 本地环境、平台配置、当前 profile、认证状态与可选在线探活。

use crate::services::doctor_service::{DoctorReport, DoctorRunOptions, DoctorService};
use ccr_core::core::error::Result;
use clap::Args;
use std::io::{self, Write};

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
    #[arg(long, value_parser = ["claude", "codex", "gemini", "qwen", "droid"], conflicts_with = "all_platforms")]
    pub platform: Option<String>,
}

pub async fn doctor_command(args: DoctorArgs) -> Result<()> {
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

    if report.has_failures() {
        let _ = io::stdout().flush();
        std::process::exit(1);
    }

    Ok(())
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
        println!("{} {}", check.status.label(), check.summary);

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
    use crate::services::doctor_service::DoctorStatus;

    #[test]
    fn doctor_renderer_prints_expected_summary() {
        let mut report = DoctorReport {
            scope: "global + current platform (claude)".to_string(),
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
