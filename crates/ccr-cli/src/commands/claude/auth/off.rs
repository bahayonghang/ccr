//! `ccr claude auth off`

#![allow(clippy::unused_async)]

use crate::application::{AuthOffPath, auth_off_for_platform};
use crate::models::Platform;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct AuthOffJson {
    ok: bool,
    changed: bool,
    path: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile_pointer: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
}

pub async fn off_command(json: bool) -> Result<()> {
    let result = auth_off_for_platform(Platform::Claude)?;
    print_auth_off(
        json,
        result.changed,
        result.path,
        result.profile_pointer,
        result.warnings,
    )
}

pub(crate) fn print_auth_off(
    json: bool,
    changed: bool,
    path: AuthOffPath,
    profile_pointer: Option<String>,
    warnings: Vec<String>,
) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&AuthOffJson {
                ok: true,
                changed,
                path: path.as_str(),
                profile_pointer,
                warnings,
            })?
        );
        return Ok(());
    }

    if changed {
        match path {
            AuthOffPath::File => ColorOutput::success("已登出官方运行时登录（已删除本地凭据文件）"),
            AuthOffPath::NativeLogout => {
                ColorOutput::success("已调用官方 logout，官方运行时登录已登出")
            }
        }
    } else {
        ColorOutput::info("当前没有可清除的官方凭据文件；无需执行 auth off");
    }

    for warning in warnings {
        ColorOutput::warning(&warning);
    }
    Ok(())
}
