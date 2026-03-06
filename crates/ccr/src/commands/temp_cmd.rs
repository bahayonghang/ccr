// 🎯 temp 命令实现 - 临时配置快速设置
// 💎 无需依赖现有 TOML 配置，直接交互式输入 base_url、token、model
//    并立即写入 settings.json
//
// 与 temp-token 的区别：
// - temp-token: 基于现有 TOML 配置的临时覆盖
// - temp: 完全独立的临时配置，无需任何预设配置
//
// 执行流程:
// 1. 交互式提示输入 base_url (必填)
// 2. 交互式提示输入 token (必填)
// 3. 交互式提示输入 model (可选，智能解析)
// 4. 直接写入 settings.json

#![allow(clippy::unused_async)]

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::SettingsManager;
use crate::utils::mask_sensitive;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};
use std::io::{self, Write};

/// 🎯 临时配置快速设置（交互式）
///
/// 完全独立的临时配置入口，无需依赖现有 TOML 配置。
/// 通过交互式提示输入 base_url、token、model，并直接写入 settings.json。
///
/// 适用场景：
/// - 快速测试新的 API 提供商
/// - 使用临时 token 进行短期测试
/// - 不想创建永久配置时的快速切换
pub async fn temp_command() -> Result<()> {
    ColorOutput::title("临时配置快速设置");
    println!();

    ColorOutput::info("📝 本小姐来帮你快速设置临时配置！");
    ColorOutput::info("   此配置将直接写入 settings.json，无需创建 TOML 配置");
    ColorOutput::info("   下次执行 'ccr switch' 时将恢复为 TOML 配置中的值");
    println!();
    ColorOutput::separator();
    println!();

    // 1-3. 交互式输入临时配置
    let (base_url, token, model) =
        tokio::task::spawn_blocking(|| -> Result<(String, String, Option<String>)> {
            let base_url = prompt_base_url()?;
            let token = prompt_token()?;
            let model = prompt_model_with_smart_parse();
            Ok((base_url, token, model))
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取用户输入失败: {e}")))??;

    println!();
    ColorOutput::separator();
    println!();

    // 显示配置预览
    ColorOutput::step("配置预览");
    println!();
    display_temp_config(&base_url, &token, model.as_deref());

    println!();

    // 确认应用
    let confirmed = tokio::task::spawn_blocking(|| -> Result<bool> {
        Ok(ColorOutput::ask_confirmation("确认应用此临时配置?", true))
    })
    .await
    .map_err(|e| CcrError::FileIoError(format!("读取用户输入失败: {e}")))??;

    if !confirmed {
        println!();
        ColorOutput::info("已取消操作");
        return Ok(());
    }

    println!();
    ColorOutput::separator();
    println!();

    // 应用配置到 settings.json
    ColorOutput::step("应用临时配置");
    apply_temp_config(&base_url, &token, model.as_deref()).await?;

    ColorOutput::success("✅ 临时配置已应用到 settings.json");
    println!();

    // 提示信息
    ColorOutput::info("💡 提示:");
    ColorOutput::info("   • 临时配置已立即生效");
    ColorOutput::info("   • 执行 'ccr switch <配置名>' 可恢复为 TOML 配置");
    ColorOutput::info("   • 执行 'ccr current' 可查看当前配置状态");
    println!();

    Ok(())
}

/// 提示输入 Base URL（必填）
fn prompt_base_url() -> Result<String> {
    ColorOutput::info("1️⃣ 请输入 Base URL (API 端点地址)");
    println!("   示例: https://api.anthropic.com");
    println!("         https://api.example.com/v1");
    println!();

    loop {
        print!("* Base URL: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() {
            ColorOutput::warning("   Base URL 是必填项，请重新输入");
            continue;
        }

        // 简单验证 URL 格式
        if !input.starts_with("http://") && !input.starts_with("https://") {
            ColorOutput::warning("   URL 应以 http:// 或 https:// 开头");
            continue;
        }

        return Ok(input);
    }
}

/// 提示输入 Token（必填）
fn prompt_token() -> Result<String> {
    println!();
    ColorOutput::info("2️⃣ 请输入 Auth Token (认证令牌)");
    println!("   示例: sk-ant-api03-xxxxx");
    println!("         sk-xxxxx");
    println!();

    loop {
        print!("* Auth Token: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() {
            ColorOutput::warning("   Auth Token 是必填项，请重新输入");
            continue;
        }

        return Ok(input);
    }
}

/// 提示输入 Model（可选，智能解析）
///
/// 智能解析规则：
/// - 如果输入为空，返回 None
/// - 如果输入包含已知的模型名称模式，直接使用
/// - 如果输入不完整，尝试智能补全
fn prompt_model_with_smart_parse() -> Option<String> {
    println!();
    ColorOutput::info("3️⃣ 请输入 Model (模型名称，可选)");
    println!("   示例: claude-sonnet-4-20250514");
    println!("         claude-3-5-sonnet-20241022");
    println!("         gpt-4o");
    println!("   提示: 直接按 Enter 跳过，使用服务商默认模型");
    println!();

    print!("  Model: ");
    io::stdout().flush().ok()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let input = input.trim().to_string();

    if input.is_empty() {
        println!("   ℹ️  已跳过，将使用服务商默认模型");
        return None;
    }

    // 智能解析模型名称
    let parsed_model = smart_parse_model(&input);

    if parsed_model != input {
        println!("   🧠 智能解析: {} → {}", input, parsed_model);
    }

    Some(parsed_model)
}

/// 智能解析模型名称
///
/// 支持的简写和补全：
/// - "sonnet" / "claude-sonnet" → 最新 sonnet 模型
/// - "opus" / "claude-opus" → 最新 opus 模型
/// - "haiku" / "claude-haiku" → 最新 haiku 模型
/// - "gpt4" / "gpt-4" → gpt-4o
/// - "gpt4o" → gpt-4o
/// - 其他输入保持原样
fn smart_parse_model(input: &str) -> String {
    let input_lower = input.to_lowercase();

    // Claude 模型简写解析
    if input_lower == "sonnet" || input_lower == "claude-sonnet" {
        return "claude-sonnet-4-20250514".to_string();
    }
    if input_lower == "opus" || input_lower == "claude-opus" {
        return "claude-opus-4-20250514".to_string();
    }
    if input_lower == "haiku" || input_lower == "claude-haiku" {
        return "claude-3-5-haiku-20241022".to_string();
    }

    // GPT 模型简写解析
    if input_lower == "gpt4" || input_lower == "gpt-4" || input_lower == "gpt4o" {
        return "gpt-4o".to_string();
    }
    if input_lower == "gpt4-mini" || input_lower == "gpt-4o-mini" {
        return "gpt-4o-mini".to_string();
    }

    // Gemini 模型简写解析
    if input_lower == "gemini" || input_lower == "gemini-pro" {
        return "gemini-2.0-flash".to_string();
    }

    // 其他输入保持原样
    input.to_string()
}

/// 显示临时配置预览表格
fn display_temp_config(base_url: &str, token: &str, model: Option<&str>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("字段")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("值")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    // Base URL
    table.add_row(vec![
        Cell::new("Base URL")
            .fg(TableColor::Yellow)
            .add_attribute(Attribute::Bold),
        Cell::new(base_url).fg(TableColor::Blue),
    ]);

    // Auth Token (脱敏显示)
    table.add_row(vec![
        Cell::new("Auth Token")
            .fg(TableColor::Yellow)
            .add_attribute(Attribute::Bold),
        Cell::new(mask_sensitive(token)).fg(TableColor::DarkGrey),
    ]);

    // Model
    if let Some(m) = model {
        table.add_row(vec![
            Cell::new("Model")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(m).fg(TableColor::Magenta),
        ]);
    } else {
        table.add_row(vec![
            Cell::new("Model").fg(TableColor::Yellow),
            Cell::new("(使用默认)").fg(TableColor::DarkGrey),
        ]);
    }

    println!("{}", table);
}

/// 应用临时配置到 settings.json
async fn apply_temp_config(base_url: &str, token: &str, model: Option<&str>) -> Result<()> {
    let settings_manager = SettingsManager::with_default()?;

    // 加载当前设置
    let mut current_settings = settings_manager.load_async().await?;

    // 应用临时配置
    current_settings
        .env
        .insert("ANTHROPIC_BASE_URL".to_string(), base_url.to_string());
    current_settings
        .env
        .insert("ANTHROPIC_AUTH_TOKEN".to_string(), token.to_string());

    if let Some(m) = model {
        current_settings
            .env
            .insert("ANTHROPIC_MODEL".to_string(), m.to_string());
    }

    // 保存设置
    settings_manager
        .save_atomic_async(&current_settings)
        .await?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_parse_model_sonnet() {
        assert_eq!(smart_parse_model("sonnet"), "claude-sonnet-4-20250514");
        assert_eq!(
            smart_parse_model("claude-sonnet"),
            "claude-sonnet-4-20250514"
        );
    }

    #[test]
    fn test_smart_parse_model_opus() {
        assert_eq!(smart_parse_model("opus"), "claude-opus-4-20250514");
        assert_eq!(smart_parse_model("claude-opus"), "claude-opus-4-20250514");
    }

    #[test]
    fn test_smart_parse_model_haiku() {
        assert_eq!(smart_parse_model("haiku"), "claude-3-5-haiku-20241022");
    }

    #[test]
    fn test_smart_parse_model_gpt() {
        assert_eq!(smart_parse_model("gpt4"), "gpt-4o");
        assert_eq!(smart_parse_model("gpt-4"), "gpt-4o");
        assert_eq!(smart_parse_model("gpt4o"), "gpt-4o");
    }

    #[test]
    fn test_smart_parse_model_gemini() {
        assert_eq!(smart_parse_model("gemini"), "gemini-2.0-flash");
        assert_eq!(smart_parse_model("gemini-pro"), "gemini-2.0-flash");
    }

    #[test]
    fn test_smart_parse_model_passthrough() {
        // 完整的模型名称应该保持不变
        assert_eq!(
            smart_parse_model("claude-3-5-sonnet-20241022"),
            "claude-3-5-sonnet-20241022"
        );
        assert_eq!(smart_parse_model("custom-model-v1"), "custom-model-v1");
    }
}
