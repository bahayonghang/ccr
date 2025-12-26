// ➕ add 命令实现 - 交互式添加配置
// 📝 通过终端交互提示用户输入配置信息

#![allow(clippy::unused_async)]

use crate::commands::common::{prompt_optional, prompt_required, prompt_tags};
use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::managers::config::{ConfigSection, ProviderType};
use crate::services::ConfigService;
use crate::utils::Validatable;
use indexmap::IndexMap;
use std::io::{self, Write};

/// ➕ 交互式添加配置
///
/// 执行流程:
/// 1. 📝 提示用户输入配置信息
/// 2. ✅ 验证输入的有效性
/// 3. 💾 保存新配置
/// 4. 📊 显示添加结果
pub async fn add_command() -> Result<()> {
    ColorOutput::title("添加新配置");
    println!();

    ColorOutput::info("请按照提示输入配置信息");
    ColorOutput::info("标记 * 的为必填项，其他为可选项");
    println!();

    // 1. 配置名称（必需）
    let name = prompt_required("配置名称", "例如: my_provider")?;

    // 检查配置是否已存在
    let service = ConfigService::with_default()?;
    if service.load_config()?.sections.contains_key(&name) {
        ColorOutput::error(&format!("配置 '{}' 已存在", name));
        ColorOutput::info("提示: 使用 'ccr list' 查看已有配置");
        return Ok(());
    }

    println!();
    ColorOutput::separator();
    println!();

    // 2. 描述（可选）
    let description = prompt_optional("描述", "例如: 我的API提供商");

    // 3. Base URL（必需）
    let base_url = prompt_required("Base URL", "例如: https://api.example.com")?;

    // 4. Auth Token（必需）
    let auth_token = prompt_required("Auth Token", "例如: sk-ant-xxxxx")?;

    // 5. 模型（可选）
    let model = prompt_optional("主模型", "例如: claude-3-5-sonnet-20241022");

    // 6. 快速小模型（可选）
    let small_fast_model = prompt_optional("快速小模型", "例如: claude-3-5-haiku-20241022");

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::info("以下为分类字段（可选）");
    println!();

    // 7. 提供商（可选）
    let provider = prompt_optional("提供商名称", "例如: anyrouter, glm, moonshot");

    // 8. 提供商类型（可选）
    let provider_type = prompt_provider_type();

    // 9. 账号（可选）
    let account = prompt_optional("账号标识", "例如: github_5953");

    // 10. 标签（可选）
    let tags = prompt_tags();

    println!();
    ColorOutput::separator();
    println!();

    // 构建配置节
    let section = ConfigSection {
        description,
        base_url: Some(base_url),
        auth_token: Some(auth_token),
        model,
        small_fast_model,
        provider,
        provider_type,
        account,
        tags,
        usage_count: Some(0), // 初始使用次数为 0
        enabled: Some(true),  // 默认启用
        other: IndexMap::new(),
    };

    // 验证配置
    ColorOutput::step("验证配置");
    section.validate()?;
    ColorOutput::success("✓ 配置验证通过");
    println!();

    // 显示预览
    ColorOutput::step("配置预览");
    println!();
    println!("  配置名称: {}", name);
    if let Some(desc) = &section.description {
        println!("  描述: {}", desc);
    }
    println!(
        "  Base URL: {}",
        section.base_url.as_deref().unwrap_or("未设置")
    );
    println!(
        "  Auth Token: {}",
        ColorOutput::mask_sensitive(section.auth_token.as_deref().unwrap_or("未设置"))
    );
    if let Some(m) = &section.model {
        println!("  主模型: {}", m);
    }
    if let Some(sm) = &section.small_fast_model {
        println!("  快速小模型: {}", sm);
    }
    if let Some(p) = &section.provider {
        println!("  提供商: {}", p);
    }
    if let Some(pt) = &section.provider_type {
        println!("  提供商类型: {}", pt.display_name());
    }
    if let Some(acc) = &section.account {
        println!("  账号: {}", acc);
    }
    if let Some(t) = &section.tags
        && !t.is_empty()
    {
        println!("  标签: {}", t.join(", "));
    }
    println!();

    // 确认添加
    if !ColorOutput::ask_confirmation("确认添加此配置?", true) {
        println!();
        ColorOutput::info("已取消添加");
        return Ok(());
    }

    println!();
    ColorOutput::separator();
    println!();

    // 保存配置
    ColorOutput::step("保存配置");
    service.add_config(name.clone(), section)?;
    ColorOutput::success(&format!("✓ 配置 '{}' 添加成功", name));
    println!();

    ColorOutput::info("后续操作:");
    println!("  • 运行 'ccr list' 查看所有配置");
    println!("  • 运行 'ccr switch {}' 切换到此配置", name);
    println!();

    Ok(())
}

/// 提示用户选择提供商类型
fn prompt_provider_type() -> Option<ProviderType> {
    println!("  提供商类型:");
    println!("    1) 官方中转");
    println!("    2) 第三方模型");
    println!("    留空跳过");
    print!("  请选择 [1/2]: ");
    io::stdout().flush().expect("无法刷新标准输出");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("无法读取用户输入");
    let input = input.trim();

    match input {
        "1" => Some(ProviderType::OfficialRelay),
        "2" => Some(ProviderType::ThirdPartyModel),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    #[test]
    fn test_add_command_structure() {
        // 测试命令结构是否正确
        // 实际的交互式测试需要手动进行
    }
}
