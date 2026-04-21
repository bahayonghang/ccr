// 🎯 交互式提示工具
// 统一处理用户输入和确认逻辑

use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use std::io::{self, Write};

/// 提示用户输入必填项
///
/// # 参数
/// - `field_name`: 字段名称
/// - `hint`: 输入提示
///
/// # 返回
/// 用户输入的非空字符串
pub fn prompt_required(field_name: &str, hint: &str) -> Result<String> {
    loop {
        print!("* {}: ", field_name);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() {
            println!("  提示: {} ({})", hint, "必填".red());
            continue;
        }

        return Ok(input);
    }
}

/// 提示用户输入可选项
///
/// # 参数
/// - `field_name`: 字段名称
/// - `hint`: 输入提示
///
/// # 返回
/// 用户输入的字符串，如果为空则返回 None
pub fn prompt_optional(field_name: &str, hint: &str) -> Option<String> {
    print!("  {}: ", field_name);
    io::stdout().flush().ok()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let input = input.trim().to_string();

    if input.is_empty() {
        println!("  提示: {} (按 Enter 跳过)", hint);
        None
    } else {
        Some(input)
    }
}

/// 提示用户输入标签（逗号分隔）
pub fn prompt_tags() -> Option<Vec<String>> {
    print!("  标签 (逗号分隔): ");
    io::stdout().flush().ok()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok()?;
    let input = input.trim();

    if input.is_empty() {
        println!("  提示: 例如 'free,stable,high-speed' (按 Enter 跳过)");
        None
    } else {
        let tags: Vec<String> = input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if tags.is_empty() { None } else { Some(tags) }
    }
}

/// 确认操作
///
/// # 参数
/// - `message`: 确认消息
/// - `default`: 默认值（true = Y, false = N）
///
/// # 返回
/// 用户是否确认
#[expect(dead_code)]
pub fn confirm(message: &str, default: bool) -> bool {
    ColorOutput::ask_confirmation(message, default)
}

/// 确认危险操作（默认拒绝）
///
/// # 参数
/// - `message`: 确认消息
///
/// # 返回
/// 用户是否确认
#[expect(dead_code)]
pub fn confirm_dangerous(message: &str) -> bool {
    println!();
    ColorOutput::warning("⚠️  此操作可能造成数据丢失！");
    println!();

    print!("{} (y/N): ", message.bright_yellow().bold());
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);

    input.trim().eq_ignore_ascii_case("y")
}

/// 确认覆盖操作
///
/// # 参数
/// - `target`: 目标描述（如文件名）
///
/// # 返回
/// 用户是否确认覆盖
#[expect(dead_code)]
pub fn confirm_overwrite(target: &str) -> bool {
    println!();
    ColorOutput::warning(&format!("⚠️  {} 已存在，将被覆盖", target));
    println!();

    print!("确认覆盖? (y/N): ");
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);

    input.trim().eq_ignore_ascii_case("y")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    // 交互式函数难以自动测试，但确保编译通过
    #[test]
    fn test_module_compiles() {
        // 仅验证模块编译
    }
}
