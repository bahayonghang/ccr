// 🎯 temp-token 命令实现 - 临时token管理
// 💎 用于临时覆盖当前配置的token,不修改永久配置文件
//
// 执行流程:
// 1. set - 设置临时token并立即应用到当前settings.json
// 2. show - 显示当前临时配置
// 3. clear - 清除临时配置

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::managers::SettingsManager;
use crate::managers::temp_override::{TempOverride, TempOverrideManager};
use crate::utils::mask_sensitive;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};

/// 🎯 设置临时token
///
/// 参数:
/// - token: 临时使用的token
/// - base_url: 可选的临时base_url
/// - model: 可选的临时model
///
/// 工作流程:
/// 1. 读取当前 settings.json
/// 2. 应用临时覆盖
/// 3. 保存 settings.json
/// 4. 立即清除临时配置（不保存到文件）
pub async fn temp_token_set(
    token: &str,
    base_url: Option<String>,
    model: Option<String>,
) -> Result<()> {
    ColorOutput::title("设置临时 Token");
    println!();

    // 创建设置管理器
    let settings_manager = SettingsManager::with_default()?;

    // 读取当前设置
    let mut current_settings = settings_manager.load_async().await?;

    // 创建临时配置
    let mut temp_override = TempOverride::new(token.to_string());
    temp_override.base_url = base_url.clone();
    temp_override.model = model.clone();

    ColorOutput::info("🎯 正在应用临时配置到当前设置...");

    // 应用临时覆盖到当前设置
    if let Some(temp_token) = &temp_override.auth_token {
        current_settings
            .env
            .insert("ANTHROPIC_AUTH_TOKEN".to_string(), temp_token.clone());
    }

    if let Some(temp_base_url) = &temp_override.base_url {
        current_settings
            .env
            .insert("ANTHROPIC_BASE_URL".to_string(), temp_base_url.clone());
    }

    if let Some(temp_model) = &temp_override.model {
        current_settings
            .env
            .insert("ANTHROPIC_MODEL".to_string(), temp_model.clone());
    }

    if let Some(temp_small_model) = &temp_override.small_fast_model {
        current_settings.env.insert(
            "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
            temp_small_model.clone(),
        );
    }

    // 保存更新后的设置
    settings_manager
        .save_atomic_async(&current_settings)
        .await?;

    ColorOutput::success("✅ 临时配置已应用到当前设置");
    println!();

    // 显示配置详情
    display_temp_override(&temp_override);

    // 提示信息
    println!();
    ColorOutput::info("💡 提示:");
    ColorOutput::info("   • 临时配置已立即应用到 settings.json");
    ColorOutput::info("   • 下次 switch 时将使用配置文件中的原始 token");
    ColorOutput::info("   • 临时配置不会修改 toml 配置文件");

    Ok(())
}

/// 📊 显示当前临时配置
pub async fn temp_token_show() -> Result<()> {
    ColorOutput::title("临时配置状态");
    println!();

    let manager = TempOverrideManager::with_default()?;

    match manager.load_async().await? {
        Some(temp_override) => {
            ColorOutput::success(&format!(
                "✅ 当前有 {} 个字段被临时覆盖",
                temp_override.override_count()
            ));
            println!();
            display_temp_override(&temp_override);
        }
        None => {
            ColorOutput::info("📝 当前没有设置临时配置");
            println!();
            ColorOutput::info("使用 'ccr temp-token set <TOKEN>' 设置临时token");
        }
    }

    Ok(())
}

/// 🧹 清除临时配置
pub async fn temp_token_clear() -> Result<()> {
    ColorOutput::title("清除临时配置");
    println!();

    let manager = TempOverrideManager::with_default()?;

    if !manager.exists_async().await? {
        ColorOutput::info("📝 当前没有临时配置需要清除");
        return Ok(());
    }

    manager.clear_async().await?;
    ColorOutput::success("✅ 临时配置已清除");
    println!();
    ColorOutput::info("💡 现在将使用 toml 配置文件中的设置");

    Ok(())
}

/// 📊 显示临时配置详情表格
fn display_temp_override(temp_override: &TempOverride) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("字段")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("临时值")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    // Auth Token
    if let Some(token) = &temp_override.auth_token {
        table.add_row(vec![
            Cell::new("Auth Token")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(mask_sensitive(token)).fg(TableColor::DarkGrey),
        ]);
    }

    // Base URL
    if let Some(base_url) = &temp_override.base_url {
        table.add_row(vec![
            Cell::new("Base URL").fg(TableColor::Yellow),
            Cell::new(base_url).fg(TableColor::Blue),
        ]);
    }

    // Model
    if let Some(model) = &temp_override.model {
        table.add_row(vec![
            Cell::new("主模型").fg(TableColor::Yellow),
            Cell::new(model).fg(TableColor::Magenta),
        ]);
    }

    // Small Fast Model
    if let Some(small_model) = &temp_override.small_fast_model {
        table.add_row(vec![
            Cell::new("快速小模型").fg(TableColor::Yellow),
            Cell::new(small_model).fg(TableColor::Magenta),
        ]);
    }

    // 创建时间
    let created_time = temp_override
        .created_at
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S");
    table.add_row(vec![
        Cell::new("创建时间").fg(TableColor::DarkGrey),
        Cell::new(created_time.to_string()).fg(TableColor::DarkGrey),
    ]);

    // 备注
    if let Some(note) = &temp_override.note {
        table.add_row(vec![
            Cell::new("备注").fg(TableColor::DarkGrey),
            Cell::new(note).fg(TableColor::White),
        ]);
    }

    println!("{}", table);
}
