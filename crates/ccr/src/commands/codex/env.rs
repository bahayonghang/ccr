//! 🌐 codex env 命令实现
//!
//! 输出当前或指定 profile 需要的环境变量导出脚本。

#![allow(clippy::unused_async)]

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::models::PlatformConfig;
use crate::platforms::codex::CodexPlatform;

pub async fn env_command(profile_name: Option<&str>) -> Result<()> {
    let platform = CodexPlatform::new()?;
    let resolved_name = match profile_name {
        Some(name) => name.to_string(),
        None => platform.get_current_profile()?.ok_or_else(|| {
            CcrError::ConfigError("当前没有激活的 Codex profile，请先切换 profile".into())
        })?,
    };

    let script = platform.export_profile_shell_script(&resolved_name)?;
    if script.trim().is_empty() {
        ColorOutput::warning("当前 Profile 不需要额外环境变量导出");
        return Ok(());
    }

    println!("{}", script);
    Ok(())
}
