// 🔄 update 命令实现 - 自动更新 CCR
// 📦 从 GitHub 仓库更新到最新版本(使用 cargo install)

#![allow(clippy::unused_async)]

#[path = "update_failure.rs"]
mod update_failure;

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use update_failure::handle_update_failure;

const CARGO_INSTALL_PACKAGE: &str = "ccr";

#[derive(Debug)]
struct UpdateExecutionResult {
    status: ExitStatus,
    stderr: String,
}

/// 🔄 执行自更新
pub async fn update_command(check_only: bool, branch: &str) -> Result<()> {
    use crate::core::CCR_GITHUB_REPO;

    let current_version = env!("CARGO_PKG_VERSION");
    let repo_url = format!("https://github.com/{CCR_GITHUB_REPO}");
    print_update_header(current_version, &repo_url, branch);

    if check_only {
        print_check_mode_preview(&repo_url, branch);
        return Ok(());
    }

    if !confirm_update().await? {
        print_cancelled();
        return Ok(());
    }

    print_update_command(&repo_url, branch);
    let result = run_update_install(repo_url.clone(), branch.to_string()).await?;
    print_post_update_separator();

    if result.status.success() {
        print_update_success();
        return Ok(());
    }

    handle_update_failure(&repo_url, branch, CARGO_INSTALL_PACKAGE, &result.stderr);
    let exit_code = result.status.code().unwrap_or(-1);
    Err(CcrError::UpdateError(format!("退出码: {exit_code}")))
}

fn print_update_header(current_version: &str, repo_url: &str, branch: &str) {
    ColorOutput::title("CCR 自动更新");
    println!();
    ColorOutput::key_value("当前版本", current_version, 2);
    ColorOutput::key_value("仓库地址", repo_url, 2);
    ColorOutput::key_value("更新分支", branch, 2);
    println!();
}

fn print_check_mode_preview(repo_url: &str, branch: &str) {
    ColorOutput::separator();
    println!();
    ColorOutput::info("检查模式 - 不会执行实际更新");
    println!();
    ColorOutput::step("更新命令预览");
    println!(
        "  cargo install --git {} {} --branch {} --force",
        repo_url, CARGO_INSTALL_PACKAGE, branch
    );
    println!();
    ColorOutput::info("💡 提示: 运行 'ccr update' 执行更新(去掉 --check 参数)");
    println!();
}

async fn confirm_update() -> Result<bool> {
    tokio::task::spawn_blocking(|| -> Result<bool> {
        Ok(ColorOutput::ask_confirmation("确认更新到最新版本?", true))
    })
    .await
    .map_err(|e| CcrError::FileIoError(format!("读取用户输入失败: {e}")))?
}

fn print_cancelled() {
    println!();
    ColorOutput::info("已取消更新");
    println!();
}

fn print_update_command(repo_url: &str, branch: &str) {
    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::step("开始更新 CCR");
    println!();
    ColorOutput::info("执行命令:");
    println!(
        "  cargo install --git {} {} --branch {} --force",
        repo_url, CARGO_INSTALL_PACKAGE, branch
    );
    println!();
    ColorOutput::separator();
    println!();
}

fn print_post_update_separator() {
    println!();
    ColorOutput::separator();
    println!();
}

fn print_update_success() {
    ColorOutput::success("🎉 更新成功完成");
    println!();
    ColorOutput::info("后续步骤:");
    println!("  1. 运行 'ccr version' 查看新版本信息");
    println!("  2. 运行 'ccr --help' 查看新功能");
    println!();
}

async fn run_update_install(repo_url: String, branch: String) -> Result<UpdateExecutionResult> {
    tokio::task::spawn_blocking(move || run_update_install_blocking(&repo_url, &branch))
        .await
        .map_err(|e| CcrError::ExternalCommandError(format!("执行更新任务失败: {e}")))?
}

fn run_update_install_blocking(repo_url: &str, branch: &str) -> Result<UpdateExecutionResult> {
    let mut child = Command::new("cargo")
        .args([
            "install",
            "--git",
            repo_url,
            CARGO_INSTALL_PACKAGE,
            "--branch",
            branch,
            "--force",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            CcrError::ExternalCommandError(format!(
                "无法启动 cargo 命令: {e}\n\n可能原因：\n  • 未安装 Rust 工具链\n  • cargo 不在系统 PATH 中"
            ))
        })?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CcrError::ExternalCommandError("无法捕获 cargo 标准错误输出".to_string()))?;
    let stderr_handle = std::thread::spawn(move || stream_and_capture_stderr(stderr));
    let status = child
        .wait()
        .map_err(|e| CcrError::ExternalCommandError(format!("等待 cargo 命令完成失败: {e}")))?;

    let stderr_output = join_stderr_handle(stderr_handle)?;
    Ok(UpdateExecutionResult {
        status,
        stderr: stderr_output,
    })
}

fn stream_and_capture_stderr(stderr: std::process::ChildStderr) -> std::io::Result<String> {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    let mut collected = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        eprint!("{line}");
        collected.push_str(&line);
    }

    Ok(collected)
}

fn join_stderr_handle(handle: std::thread::JoinHandle<std::io::Result<String>>) -> Result<String> {
    let stderr_result = handle.join().map_err(|_| {
        CcrError::ExternalCommandError("读取 cargo 错误输出线程异常退出".to_string())
    })?;
    stderr_result
        .map_err(|e| CcrError::ExternalCommandError(format!("读取 cargo 错误输出失败: {e}")))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_command_check_only() {
        let result = update_command(true, "main").await;
        assert!(result.is_ok());
    }
}
