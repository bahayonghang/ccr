// update 命令实现 - 自动更新 CCR

use crate::error::{CcrError, Result};
use crate::logging::ColorOutput;
use std::process::Command;

/// 执行自更新
pub fn update_command(check_only: bool) -> Result<()> {
    ColorOutput::title("CCR 自动更新");
    println!();

    let current_version = env!("CARGO_PKG_VERSION");
    ColorOutput::info(&format!("当前版本: {}", current_version));
    println!();

    if check_only {
        ColorOutput::info("将使用 cargo 从 GitHub 获取最新版本");
        ColorOutput::info("执行命令: cargo install --git https://github.com/bahayonghang/ccr --force");
        println!();
        ColorOutput::info("运行 'ccr update' 进行更新");
        return Ok(());
    }

    // 确认更新
    println!();
    if !ColorOutput::ask_confirmation("是否更新到最新版本?", true) {
        ColorOutput::info("已取消更新");
        return Ok(());
    }

    println!();
    ColorOutput::step("正在更新 CCR...");
    ColorOutput::info("执行: cargo install --git https://github.com/bahayonghang/ccr --force");
    println!();

    // 执行 cargo install
    let output = Command::new("cargo")
        .args(&[
            "install",
            "--git",
            "https://github.com/bahayonghang/ccr",
            "--force",
        ])
        .output()
        .map_err(|e| CcrError::ConfigError(format!("执行 cargo 命令失败: {}", e)))?;

    if output.status.success() {
        println!();
        ColorOutput::separator();
        println!();
        ColorOutput::success("✓ 更新成功");
        ColorOutput::info("提示: 运行 'ccr version' 查看新版本");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        ColorOutput::error("更新失败");
        println!();
        ColorOutput::info("错误信息:");
        println!("{}", stderr);
        println!();
        ColorOutput::info("可能的原因:");
        println!("  • 未安装 Rust 工具链（需要 cargo）");
        println!("  • 网络连接问题");
        println!("  • Git 访问权限问题");
        println!();
        ColorOutput::info("解决方案:");
        println!("  1. 确保已安装 Rust: https://rustup.rs/");
        println!("  2. 检查网络连接");
        println!("  3. 手动执行: cargo install --git https://github.com/bahayonghang/ccr --force");
        
        return Err(CcrError::ConfigError("更新失败".into()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_command_check_only() {
        // 测试 --check 模式不会实际执行更新
        // 这个测试只验证函数能正常返回
        let result = update_command(true);
        // check_only 模式应该总是成功返回
        assert!(result.is_ok());
    }
}

