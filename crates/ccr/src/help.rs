// 🎨 自定义帮助渲染模块
// 提供彩色、分隔线、表格化、响应式的帮助输出

use colored::*;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ColumnConstraint, ContentArrangement, Table, Width,
    presets::UTF8_FULL,
};
// 仅在启用 tui 特性时引入 crossterm，避免 --no-default-features 构建失败
#[cfg(feature = "tui")]
use crossterm::terminal;
use std::cmp::min;

/// 获取终端宽度
/// - 当启用 `tui` 特性时，使用 crossterm 检测终端宽度
/// - 当未启用 `tui` 特性时，回退到环境变量 `COLUMNS` 或默认 80 列
#[cfg(feature = "tui")]
fn term_width() -> usize {
    match terminal::size() {
        Ok((w, _)) => w as usize,
        Err(_) => std::env::var("COLUMNS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(80),
    }
}

#[cfg(not(feature = "tui"))]
fn term_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

/// 统一创建表格，响应式宽度与美观样式
fn make_table(width: usize, headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_width(min((width.saturating_sub(4)) as u16, 120));

    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| {
            Cell::new(h)
                .fg(TableColor::Cyan)
                .add_attribute(Attribute::Bold)
        })
        .collect();
    table.set_header(header_cells);
    table
}

fn section_title(title: &str) {
    let line = "─".repeat(60).dimmed().to_string();
    println!("{}", line);
    println!("{}", title.blue().bold());
    println!("{}", line);
}

fn decorate_top_border() {
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════════╗".cyan()
    );
}

fn decorate_bottom_border() {
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════╝".cyan()
    );
}

/// 高亮命令名（亮绿色）
fn cmd(text: &str) -> String {
    text.bright_green().bold().to_string()
}

/// 高亮参数（黄色）
fn param(text: &str) -> String {
    text.yellow().bold().to_string()
}

/// 高亮选项（青色）
fn opt(text: &str) -> String {
    text.cyan().bold().to_string()
}

/// 高亮重要提示（红色）
fn important(text: &str) -> String {
    text.red().bold().to_string()
}

/// 打印顶层帮助
pub fn print_top_help() {
    let w = term_width();
    decorate_top_border();

    // 顶部标题与简介
    println!(
        "{} {}",
        cmd("ccr"),
        env!("CARGO_PKG_VERSION").to_string().white()
    );
    println!(
        "{}",
        "🎯 Claude Code Configuration Switcher (Rust Version)"
            .white()
            .bold()
    );

    println!("{}", "一个强大的 Claude Code 配置管理工具,支持：".white());
    println!("    • 多套配置快速切换");
    println!("    • 完整的操作审计追踪");
    println!("    • 自动备份和恢复");
    println!("    • 配置导入导出");
    println!("    • Web 可视化界面");

    // 用法
    section_title("用法 (Usage)");
    println!(
        "{} {} {} {} {}",
        "Usage:".bold(),
        cmd("ccr"),
        opt("[选项]"),
        param("[配置名称]"),
        param("[命令]")
    );

    // 快速开始
    section_title("快速开始");
    println!("  {}  # 初始化配置文件", cmd("ccr init"));
    println!("  {}  # 查看所有配置", cmd("ccr list"));
    println!("  {}  # 切换配置", cmd("ccr switch <名称>"));
    println!("  {}  # 快捷切换(省略 switch)", cmd("ccr anthropic"));
    println!("  {}  # 打开 TUI 配置选择器", cmd("ccr"));

    // 多平台切换
    section_title("多平台切换");
    println!("  {}  # 查看所有支持的平台", cmd("ccr platform list"));
    println!("  {}  # 查看当前平台", cmd("ccr platform current"));
    println!(
        "  {}  # 切换到 Codex 平台",
        cmd("ccr platform switch codex")
    );
    println!(
        "  {}  # 切换到 Gemini 平台",
        cmd("ccr platform switch gemini")
    );
    println!();
    println!(
        "  {} Claude, Codex, Gemini, Qwen, Aider 等平台",
        "支持的平台:".white().bold()
    );

    // 获取帮助
    section_title("获取帮助");
    println!("  {}  # 显示此帮助", cmd("ccr --help"));
    println!("  {}  # 显示特定命令的帮助", cmd("ccr <命令> --help"));

    // 命令分组显示
    let wide = w >= 100;
    let headers: Vec<&str> = if wide {
        vec!["命令", "说明", "示例"]
    } else {
        vec!["命令", "说明"]
    };

    // 配置管理
    section_title("配置管理");
    let mut conf_table = make_table(w, &headers);
    // 列宽约束：命令列固定，说明列最小 20，示例列最大 28
    if wide {
        conf_table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(14)),
            ColumnConstraint::LowerBoundary(Width::Fixed(20)),
            ColumnConstraint::UpperBoundary(Width::Fixed(28)),
        ]);
    } else {
        conf_table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(14)),
            ColumnConstraint::LowerBoundary(Width::Fixed(20)),
        ]);
    }
    let add_row = |table: &mut Table, name: &str, desc: &str, example: Option<&str>| {
        let name_cell = Cell::new(name)
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold);
        let desc_cell = Cell::new(desc);
        if wide {
            let ex = Cell::new(example.unwrap_or("-")).fg(TableColor::Yellow);
            table.add_row(vec![name_cell, desc_cell, ex]);
        } else {
            table.add_row(vec![name_cell, desc_cell]);
        }
    };
    add_row(
        &mut conf_table,
        "init",
        "初始化配置文件",
        Some("ccr init --force"),
    );
    add_row(
        &mut conf_table,
        "list",
        "列出所有可用的配置方案",
        Some("ccr list"),
    );
    add_row(
        &mut conf_table,
        "status",
        "显示当前激活的配置状态",
        Some("ccr status"),
    );

    add_row(
        &mut conf_table,
        "switch",
        "切换到指定的配置方案",
        Some("ccr switch anthropic"),
    );
    add_row(
        &mut conf_table,
        "add",
        "添加新的配置方案(交互式)",
        Some("ccr add"),
    );
    add_row(
        &mut conf_table,
        "delete",
        "删除指定的配置方案",
        Some("ccr delete <name>"),
    );
    add_row(
        &mut conf_table,
        "validate",
        "验证配置文件和设置的完整性",
        Some("ccr validate"),
    );
    add_row(
        &mut conf_table,
        "optimize",
        "优化配置文件结构",
        Some("ccr optimize"),
    );
    add_row(
        &mut conf_table,
        "history",
        "查看配置操作的历史记录",
        Some("ccr history -l 50 -t switch"),
    );
    add_row(
        &mut conf_table,
        "export",
        "导出配置到文件",
        Some("ccr export -o conf.toml --no-secrets"),
    );
    add_row(
        &mut conf_table,
        "import",
        "从文件导入配置",
        Some("ccr import config.toml --merge"),
    );
    add_row(
        &mut conf_table,
        "clean",
        "清理过期的备份文件",
        Some("ccr clean -d 30 --dry-run"),
    );
    add_row(
        &mut conf_table,
        "clear",
        "清理 CCR 写入的配置",
        Some("ccr clear --force"),
    );
    println!("{}", conf_table);

    // 用户界面
    section_title("用户界面");
    let mut ui_table = make_table(w, &headers);
    if wide {
        ui_table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(10)),
            ColumnConstraint::LowerBoundary(Width::Fixed(20)),
            ColumnConstraint::UpperBoundary(Width::Fixed(28)),
        ]);
    } else {
        ui_table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(10)),
            ColumnConstraint::LowerBoundary(Width::Fixed(20)),
        ]);
    }
    add_row(
        &mut ui_table,
        "web",
        "启动轻量级 Legacy Web 界面 / Web API（兼容与编程访问，新的浏览器端推荐使用 ccr ui）",
        Some("ccr web --host 0.0.0.0 --port 19527 --no-browser"),
    );
    add_row(
        &mut ui_table,
        "ui",
        "启动 CCR UI (推荐 Web 界面，完整 Web 应用)",
        Some("ccr ui -p 15173"),
    );
    println!("{}", ui_table);
    println!();
    println!(
        "  {} 直接运行 {} 即可打开 TUI 配置选择器（选择 Claude/Codex 平台配置）",
        "💡".yellow(),
        cmd("ccr")
    );

    // 同步与平台
    section_title("同步与平台");
    let mut sp_table = make_table(w, &headers);
    if wide {
        sp_table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(12)),
            ColumnConstraint::LowerBoundary(Width::Fixed(20)),
            ColumnConstraint::UpperBoundary(Width::Fixed(28)),
        ]);
    } else {
        sp_table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(12)),
            ColumnConstraint::LowerBoundary(Width::Fixed(20)),
        ]);
    }
    add_row(
        &mut sp_table,
        "sync",
        "WebDAV 配置同步",
        Some("ccr sync status"),
    );
    add_row(
        &mut sp_table,
        "platform",
        "多平台管理(切换/列表/状态)",
        Some("ccr platform switch codex"),
    );
    add_row(
        &mut sp_table,
        "temp-token",
        "临时Token管理",
        Some("ccr temp-token show"),
    );
    println!("{}", sp_table);

    // 统计与维护
    section_title("统计与维护");
    let mut mv_table = make_table(w, &headers);
    if wide {
        mv_table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(12)),
            ColumnConstraint::LowerBoundary(Width::Fixed(20)),
            ColumnConstraint::UpperBoundary(Width::Fixed(28)),
        ]);
    } else {
        mv_table.set_constraints(vec![
            ColumnConstraint::Absolute(Width::Fixed(12)),
            ColumnConstraint::LowerBoundary(Width::Fixed(20)),
        ]);
    }
    add_row(
        &mut mv_table,
        "stats",
        "统计与分析",
        Some("ccr stats cost --today"),
    );
    add_row(
        &mut mv_table,
        "update",
        "从 GitHub 更新到最新版本",
        Some("ccr update --check"),
    );
    add_row(
        &mut mv_table,
        "version",
        "显示详细的版本信息",
        Some("ccr version"),
    );
    println!("{}", mv_table);

    // 选项表格（重要参数加 ★）
    section_title("选项 (Options)");
    let mut opt_table = make_table(w, &headers);
    let star = "★".red().bold().to_string();
    let o = |t: &str| {
        Cell::new(t)
            .fg(TableColor::Cyan)
            .add_attribute(Attribute::Bold)
    };
    let p = |t: &str| {
        Cell::new(t)
            .fg(TableColor::Yellow)
            .add_attribute(Attribute::Bold)
    };

    if wide {
        opt_table.add_row(vec![
            o("-y, --yes"),
            Cell::new(format!("{} 自动确认模式（跳过所有确认提示）", star)),
            Cell::new("ccr -y delete test"),
        ]);
        opt_table.add_row(vec![
            o("-h, --help"),
            Cell::new("显示帮助信息（使用 '-h' 查看简短摘要）"),
            Cell::new("ccr --help"),
        ]);
        opt_table.add_row(vec![
            o("-V, --version"),
            Cell::new("显示版本信息"),
            Cell::new("ccr --version"),
        ]);
        opt_table.add_row(vec![
            p("[CONFIG_NAME]"),
            Cell::new(format!("{} 直接切换到指定配置(快捷方式)", star)),
            Cell::new("ccr anthropic"),
        ]);
    } else {
        opt_table.add_row(vec![
            o("-y, --yes"),
            Cell::new(format!("{} 自动确认模式（跳过所有确认提示）", star)),
        ]);
        opt_table.add_row(vec![o("-h, --help"), Cell::new("显示帮助信息")]);
        opt_table.add_row(vec![o("-V, --version"), Cell::new("显示版本信息")]);
        opt_table.add_row(vec![
            p("[CONFIG_NAME]"),
            Cell::new(format!("{} 直接切换配置", star)),
        ]);
    }
    println!("{}", opt_table);

    // 参数（Arguments）- 保留原内容
    section_title("参数 (Arguments)");
    println!("  {}", param("[CONFIG_NAME]"));
    println!("      直接切换到指定配置(快捷方式,无需输入 switch 子命令)");
    println!(
        "      示例：{}  等同于  {}",
        cmd("ccr anthropic"),
        cmd("ccr switch anthropic")
    );

    // 重要提示
    section_title("重要提示");
    println!(
        "{}",
        important("警告：使用 --yes 将跳过所有确认提示，请谨慎执行涉及删除/覆盖的操作！")
    );

    // 底部
    println!("\n{}", "更多帮助: ccr <命令> --help".blue());
    decorate_bottom_border();
}

/// 打印子命令帮助（复用 clap 生成的详细帮助，并加边框）
pub fn print_subcommand_help(name: &str) {
    use clap::CommandFactory;
    let mut root_cmd = crate::Cli::command();
    let w = term_width();
    decorate_top_border();
    println!("{} {}", cmd("ccr"), env!("CARGO_PKG_VERSION"));
    println!("子命令帮助: {}", name.blue().bold());
    println!("{}", "─".repeat(min(w, 60)).dimmed());

    // 尝试找到匹配子命令并打印其长帮助
    if let Some(sc) = root_cmd.find_subcommand_mut(name) {
        let mut buf = Vec::new();
        sc.write_long_help(&mut buf).ok();
        let s = String::from_utf8_lossy(&buf);
        println!("{}", s);
    } else {
        println!("未找到子命令: {}", name.red());
    }

    println!("{}", "─".repeat(min(w, 60)).dimmed());
    decorate_bottom_border();
}

/// 打印嵌套子命令帮助（如 "codex auth"）
///
/// 支持多级嵌套的子命令路径
/// 示例: print_nested_subcommand_help(&["codex", "auth"])
pub fn print_nested_subcommand_help(path: &[&str]) {
    use clap::CommandFactory;
    let mut cmd = crate::Cli::command();
    let w = term_width();

    // 逐级查找子命令
    for name in path {
        if let Some(sc) = cmd.find_subcommand_mut(name) {
            cmd = sc.clone();
        } else {
            decorate_top_border();
            println!("未找到子命令: {}", path.join(" ").red());
            decorate_bottom_border();
            return;
        }
    }

    // 打印帮助
    decorate_top_border();
    println!("{} {}", self::cmd("ccr"), env!("CARGO_PKG_VERSION"));
    println!("子命令帮助: {}", path.join(" ").blue().bold());
    println!("{}", "─".repeat(min(w, 60)).dimmed());

    let mut buf = Vec::new();
    cmd.write_long_help(&mut buf).ok();
    let s = String::from_utf8_lossy(&buf);
    println!("{}", s);

    println!("{}", "─".repeat(min(w, 60)).dimmed());
    decorate_bottom_border();
}
