//! `ccr grok auth` handlers.

#![allow(clippy::unused_async)]

use crate::commands::claude::auth::off::print_auth_off;
use crate::services::GrokAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct GrokAuthCurrentJson {
    logged_in: bool,
}

pub async fn current_command(json: bool) -> Result<()> {
    let current = GrokAuthService::new().current()?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&GrokAuthCurrentJson {
                logged_in: current.logged_in,
            })?
        );
        return Ok(());
    }

    ColorOutput::title("Grok 官方会话");
    if current.logged_in {
        ColorOutput::success("已检测到官方会话文件（auth.json 存在）");
    } else {
        ColorOutput::info("未检测到官方会话；运行时可回退 XAI_API_KEY");
    }
    Ok(())
}

pub async fn off_command(json: bool) -> Result<()> {
    let result = GrokAuthService::new().off()?;
    print_auth_off(
        json,
        result.changed,
        result.path,
        result.profile_pointer,
        result.warnings,
    )
}
