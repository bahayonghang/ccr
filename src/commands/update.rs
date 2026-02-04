// 🔄 update 命令实现 - 自动更新 CCR
// 📦 从 GitHub 仓库更新到最新版本(使用 cargo install)

#![allow(clippy::unused_async)]

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use std::process::{Command, Stdio};

/// 🔄 执行自更新
///
/// 执行流程:
/// 1. 📋 显示当前版本
/// 2. 🤔 询问用户确认(非 check 模式)
/// 3. 🔄 执行 cargo install --git --force(实时显示进度)
/// 4. ✅ 显示更新结果
///
/// 参数:
/// - check_only: 仅检查更新,不执行安装
///
/// 依赖:
/// - 需要本地安装 Rust 和 cargo
/// - 需要能访问 GitHub
pub async fn update_command(check_only: bool, branch: &str) -> Result<()> {
    use crate::core::CCR_GITHUB_REPO;

    ColorOutput::title("CCR 自动更新");
    println!();

    let current_version = env!("CARGO_PKG_VERSION");
    let repo_url = format!("https://github.com/{}", CCR_GITHUB_REPO);
    ColorOutput::key_value("当前版本", current_version, 2);
    ColorOutput::key_value("仓库地址", &repo_url, 2);
    ColorOutput::key_value("更新分支", branch, 2);
    println!();

    if check_only {
        ColorOutput::separator();
        println!();
        ColorOutput::info("检查模式 - 不会执行实际更新");
        println!();
        ColorOutput::step("更新命令预览");
        println!(
            "  cargo install --git {} ccr --branch {} --force",
            repo_url, branch
        );
        println!();
        ColorOutput::info("💡 提示: 运行 'ccr update' 执行更新(去掉 --check 参数)");
        println!();
        return Ok(());
    }

    // 确认更新
    println!();
    let confirmed = tokio::task::spawn_blocking(|| -> Result<bool> {
        Ok(ColorOutput::ask_confirmation("确认更新到最新版本?", true))
    })
    .await
    .map_err(|e| CcrError::FileIoError(format!("读取用户输入失败: {e}")))??;

    if !confirmed {
        println!();
        ColorOutput::info("已取消更新");
        println!();
        return Ok(());
    }

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::step("开始更新 CCR");
    println!();
    ColorOutput::info("执行命令:");
    println!(
        "  cargo install --git {} ccr --branch {} --force",
        repo_url, branch
    );
    println!();
    ColorOutput::separator();
    println!();

    let repo_url_for_task = repo_url.clone();
    let branch_for_task = branch.to_string();

    // 执行 cargo install,实时显示输出
    let status = tokio::task::spawn_blocking(move || -> Result<std::process::ExitStatus> {
        let mut child = Command::new("cargo")
            .args([
                "install",
                "--git",
                repo_url_for_task.as_str(),
                "ccr", // 指定包名
                "--branch",
                branch_for_task.as_str(),
                "--force",
            ])
            .stdout(Stdio::inherit()) // 实时显示标准输出
            .stderr(Stdio::inherit()) // 实时显示标准错误
            .spawn()
            .map_err(|e| {
                CcrError::ExternalCommandError(format!(
                    "无法启动 cargo 命令: {}\n\n可能原因：\n  • 未安装 Rust 工具链\n  • cargo 不在系统 PATH 中",
                    e
                ))
            })?;

        // 等待命令执行完成
        child
            .wait()
            .map_err(|e| CcrError::ExternalCommandError(format!("等待 cargo 命令完成失败: {}", e)))
    })
    .await
    .map_err(|e| CcrError::ExternalCommandError(format!("执行更新任务失败: {}", e)))??;

    println!();
    ColorOutput::separator();
    println!();

    if status.success() {
        ColorOutput::success("🎉 更新成功完成");
        println!();
        ColorOutput::info("后续步骤:");
        println!("  1. 运行 'ccr version' 查看新版本信息");
        println!("  2. 运行 'ccr --help' 查看新功能");
        println!();
    } else {
        ColorOutput::error("❌ 更新失败");
        println!();
        ColorOutput::info("可能的原因:");
        println!("  • 网络连接问题(无法访问 GitHub)");
        println!("  • Git 未安装或配置不正确");
        println!("  • 权限不足(无法写入 ~/.cargo/bin)");
        println!("  • Rust 工具链版本过旧");
        println!();
        ColorOutput::info("解决方案:");
        println!("  1. 检查网络连接: ping github.com");
        println!("  2. 更新 Rust 工具链: rustup update");
        println!("  3. 检查 cargo 版本: cargo --version");
        println!(
            "  4. 手动安装: cargo install --git {} ccr --branch {} --force",
            repo_url, branch
        );
        println!();

        return Err(CcrError::UpdateError(format!(
            "更新失败,退出码: {}",
            status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_command_check_only() {
        // 测试 --check 模式不会实际执行更新
        // 这个测试只验证函数能正常返回
        let result = update_command(true, "main").await;
        // check_only 模式应该总是成功返回
        assert!(result.is_ok());
    }
}
