// update 命令实现 - 自动更新 CCR

use crate::error::{CcrError, Result};
use crate::logging::ColorOutput;

/// 执行自更新
pub fn update_command(check_only: bool) -> Result<()> {
    ColorOutput::title("CCR 自动更新");
    println!();

    let current_version = env!("CARGO_PKG_VERSION");
    ColorOutput::info(&format!("当前版本: {}", current_version));
    println!();

    // 检查是否有新版本
    ColorOutput::step("步骤 1/3: 检查更新");
    
    let status = check_for_updates()?;
    
    match status {
        UpdateStatus::UpToDate => {
            ColorOutput::success("已是最新版本");
            return Ok(());
        }
        UpdateStatus::UpdateAvailable { version, url } => {
            ColorOutput::info(&format!("发现新版本: {}", version));
            
            if check_only {
                ColorOutput::info("运行 'ccr update' 进行更新");
                return Ok(());
            }

            println!();
            
            // 确认更新
            if !ColorOutput::ask_confirmation(
                &format!("是否更新到版本 {}?", version),
                true,
            ) {
                ColorOutput::info("已取消更新");
                return Ok(());
            }

            println!();
            ColorOutput::step("步骤 2/3: 下载新版本");
            download_and_install(&version, &url)?;

            println!();
            ColorOutput::step("步骤 3/3: 验证安装");
            verify_installation()?;

            println!();
            ColorOutput::separator();
            println!();
            ColorOutput::success(&format!("✓ 成功更新到版本 {}", version));
            ColorOutput::info("请重新启动终端或运行 'ccr version' 查看新版本");
        }
    }

    Ok(())
}

/// 更新状态
enum UpdateStatus {
    UpToDate,
    UpdateAvailable { version: String, url: String },
}

/// 检查更新
fn check_for_updates() -> Result<UpdateStatus> {
    let current_version = env!("CARGO_PKG_VERSION");
    
    ColorOutput::info("正在从 GitHub 检查最新版本...");

    // 构建 self_update
    let releases = self_update::backends::github::ReleaseList::configure()
        .repo_owner("bahayonghang")
        .repo_name("ccr")
        .build()
        .map_err(|e| CcrError::NetworkError(format!("配置更新检查失败: {}", e)))?
        .fetch()
        .map_err(|e| CcrError::NetworkError(format!("获取版本列表失败: {}", e)))?;

    if releases.is_empty() {
        return Err(CcrError::NetworkError("未找到任何发布版本".into()));
    }

    // 获取最新版本
    let latest = &releases[0];
    let latest_version = latest.version.trim_start_matches('v');

    ColorOutput::info(&format!("最新版本: {}", latest_version));

    // 比较版本
    if version_compare(latest_version, current_version) {
        // 获取当前平台的下载链接
        let asset_name = get_asset_name();
        let download_url = latest
            .asset_for(&asset_name, Some(get_target_triple()))
            .map(|asset| asset.download_url.clone())
            .ok_or_else(|| {
                CcrError::NetworkError(format!(
                    "未找到适用于当前平台的安装包: {}",
                    asset_name
                ))
            })?;

        Ok(UpdateStatus::UpdateAvailable {
            version: latest_version.to_string(),
            url: download_url,
        })
    } else {
        Ok(UpdateStatus::UpToDate)
    }
}

/// 下载并安装新版本
fn download_and_install(_version: &str, _url: &str) -> Result<()> {
    ColorOutput::info("正在下载...");

    let target = get_target_triple();
    
    let status = self_update::backends::github::Update::configure()
        .repo_owner("bahayonghang")
        .repo_name("ccr")
        .target(target)
        .bin_name("ccr")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()
        .map_err(|e| CcrError::NetworkError(format!("配置更新失败: {}", e)))?
        .update()
        .map_err(|e| CcrError::NetworkError(format!("更新失败: {}", e)))?;

    ColorOutput::success(&format!("已下载版本 {}", status.version()));

    Ok(())
}

/// 验证安装
fn verify_installation() -> Result<()> {
    ColorOutput::info("验证安装...");
    
    // 尝试运行 --version 命令验证
    let output = std::process::Command::new("ccr")
        .arg("--version")
        .output()
        .map_err(|e| CcrError::ConfigError(format!("验证安装失败: {}", e)))?;

    if output.status.success() {
        ColorOutput::success("安装验证成功");
        Ok(())
    } else {
        Err(CcrError::ConfigError("安装验证失败".into()))
    }
}

/// 比较版本号（简单实现）
/// 如果 new_version > current_version 返回 true
fn version_compare(new_version: &str, current_version: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let new_parts = parse_version(new_version);
    let current_parts = parse_version(current_version);

    for i in 0..std::cmp::max(new_parts.len(), current_parts.len()) {
        let new = new_parts.get(i).unwrap_or(&0);
        let current = current_parts.get(i).unwrap_or(&0);

        if new > current {
            return true;
        } else if new < current {
            return false;
        }
    }

    false
}

/// 获取目标平台三元组
fn get_target_triple() -> &'static str {
    // 根据编译目标返回对应的三元组
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "x86_64-unknown-linux-gnu";
    
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "aarch64-unknown-linux-gnu";

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "x86_64-apple-darwin";
    
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "aarch64-apple-darwin";

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "x86_64-pc-windows-msvc";

    // 默认返回（用于其他平台）
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    return "unknown";
}

/// 获取资产名称
fn get_asset_name() -> String {
    let target = get_target_triple();
    format!("ccr-{}", target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compare() {
        assert!(version_compare("0.2.0", "0.1.0"));
        assert!(version_compare("1.0.0", "0.9.9"));
        assert!(version_compare("0.1.1", "0.1.0"));
        assert!(!version_compare("0.1.0", "0.1.0"));
        assert!(!version_compare("0.1.0", "0.2.0"));
    }

    #[test]
    fn test_get_target_triple() {
        let target = get_target_triple();
        assert!(!target.is_empty());
        assert_ne!(target, "unknown");
    }
}

