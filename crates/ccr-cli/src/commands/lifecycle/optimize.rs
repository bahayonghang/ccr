// 🔄 optimize 命令实现 - 优化配置文件结构
// 📋 按字母顺序排列配置节,提升可读性

#![allow(clippy::unused_async)]

use crate::services::ConfigService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;

/// 🔄 优化配置文件结构
///
/// 功能说明:
/// 1. 📖 加载当前配置文件
/// 2. 🔤 按照配置节名称的字母顺序重新排列
/// 3. 💾 保存优化后的配置文件
/// 4. ✅ 保持所有配置内容不变,仅调整顺序
///
/// 使用场景:
/// - 手动编辑配置文件后,配置节顺序混乱
/// - 希望配置文件更容易阅读和维护
/// - 统一团队配置文件的格式风格
pub async fn optimize_command() -> Result<()> {
    ColorOutput::title("配置文件优化");
    println!();

    // 加载配置文件
    ColorOutput::step("加载配置文件");
    let config_service = ConfigService::with_default()?;
    let config_path = config_service
        .config_manager()
        .config_path()
        .display()
        .to_string();

    let mut config = config_service.load_config()?;
    ColorOutput::success(&format!("配置文件: {}", config_path));

    // 显示优化前的配置节顺序
    println!();
    ColorOutput::step("当前配置节顺序");
    let original_order: Vec<String> = config.sections.keys().cloned().collect();
    for (index, name) in original_order.iter().enumerate() {
        println!("  {}. {}", index + 1, name);
    }

    // 执行排序
    println!();
    ColorOutput::step("按字母顺序优化");
    config.sort_sections();

    // 显示优化后的配置节顺序
    let optimized_order: Vec<String> = config.sections.keys().cloned().collect();
    println!();
    for (index, name) in optimized_order.iter().enumerate() {
        println!("  {}. {}", index + 1, name);
    }

    // 检查是否有变化
    if original_order == optimized_order {
        println!();
        ColorOutput::info("配置节顺序已是最优,无需调整");
        println!();
        return Ok(());
    }

    // 保存配置文件
    println!();
    ColorOutput::step("保存优化后的配置");
    config_service.save_config(&config)?;
    ColorOutput::success("配置文件已优化并保存");

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::title("优化完成");
    println!();
    ColorOutput::success(&format!(
        "✓ 配置节已按字母顺序排列(共 {} 个)",
        optimized_order.len()
    ));
    ColorOutput::info("配置内容保持不变,仅调整了顺序");
    println!();

    Ok(())
}
