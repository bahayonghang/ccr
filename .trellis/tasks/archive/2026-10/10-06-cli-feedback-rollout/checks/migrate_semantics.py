"""Apply explicitly reviewed semantic presentation replacements for C3."""

from pathlib import Path


def replace(filename, pairs):
    path = Path('crates/ccr-cli/src') / filename
    source = path.read_text(encoding='utf-8')
    for before, after in pairs:
        if before not in source:
            if after in source:
                continue
            raise ValueError(f'Missing reviewed text in {filename}: {before}')
        source = source.replace(before, after)
    path.write_text(source, encoding='utf-8', newline='')


replace('commands/data/budget.rs', [
    ('ColorOutput::success("预算控制已启用");\n    } else {', 'ColorOutput::key_value("预算控制", "已启用", 2);\n    } else {'),
    ('ColorOutput::warning("预算控制已禁用");\n        ColorOutput::info("使用 `ccr budget set --enable` 启用预算控制");', 'ColorOutput::key_value("预算控制", "已禁用", 2);\n        crate::commands::common::print_next_steps(&[("启用预算控制", "ccr budget set --enable")]);'),
    ('ColorOutput::info("使用 `ccr budget set --help` 查看可用选项");', 'crate::commands::common::print_next_steps(&[("查看配置选项", "ccr budget set --help")]);'),
    ('println!();\n    ColorOutput::info("提示: 使用 `ccr budget status` 查看当前预算状态");', 'crate::commands::common::print_next_steps(&[("查看预算状态", "ccr budget status")]);'),
    ('ColorOutput::info("提示: 预算控制状态未改变，使用 `ccr budget set --enable/--disable` 修改");', 'ColorOutput::info("预算控制状态未改变");\n    crate::commands::common::print_next_steps(&[("查看预算配置选项", "ccr budget set --help")]);'),
])
replace('commands/data/export.rs', [
    ('ColorOutput::info("提示: 请妥善保管导出文件,避免泄露");', 'ColorOutput::info("请妥善保管导出文件,避免泄露");'),
    ('ColorOutput::info("提示: 使用 --no-secrets 参数可导出不含密钥的配置");', 'ColorOutput::info("使用 --no-secrets 参数可导出不含密钥的配置");'),
    ('ColorOutput::info("提示: 不使用 --no-secrets 可导出完整配置(包含密钥)");', 'ColorOutput::info("不使用 --no-secrets 可导出完整配置(包含密钥)");'),
])
replace('commands/data/import.rs', [
    ('println!();\n    ColorOutput::info("提示: 运行 \'ccr list\' 查看所有配置");', 'crate::commands::common::print_next_steps(&[("查看配置", "ccr list")]);'),
])
replace('commands/data/history.rs', [
    ('ColorOutput::info("提示: 使用 ccr history --clear 清理历史记录");', 'crate::commands::common::print_next_steps(&[("查看历史记录选项", "ccr history --help")]);'),
])
replace('commands/data/pricing.rs', [
    ('ColorOutput::warning("未配置任何模型定价");\n        ColorOutput::info(\n            "使用 `ccr pricing set <模型名> --input <价格> --output <价格>` 添加定价",\n        );', 'ColorOutput::info("未配置任何模型定价");\n        crate::commands::common::print_next_steps(&[("查看定价设置选项", "ccr pricing set --help")]);'),
    ('ColorOutput::info("提示: 使用 --verbose 查看缓存定价详情");', 'crate::commands::common::print_next_steps(&[("查看缓存定价详情", "ccr pricing list --verbose")]);'),
    ('println!();\n    ColorOutput::info("提示: 使用 `ccr pricing list` 查看所有定价配置");', 'crate::commands::common::print_next_steps(&[("查看定价配置", "ccr pricing list")]);'),
    ('ColorOutput::info("提示: 使用 `ccr pricing list` 查看默认配置");', 'crate::commands::common::print_next_steps(&[("查看默认定价", "ccr pricing list")]);'),
])
replace('commands/data/stats.rs', [
    ('ColorOutput::success("Token 使用:");', 'ColorOutput::info("Token 使用:");'),
    ('ColorOutput::success("按平台分组:");', 'ColorOutput::info("按平台分组:");'),
    ('ColorOutput::success("按模型分组:");', 'ColorOutput::info("按模型分组:");'),
    ('ColorOutput::success("按项目分组:");', 'ColorOutput::info("按项目分组:");'),
    ('ColorOutput::success(&format!("成本最高的 {} 个会话:", limit));', 'ColorOutput::info(&format!("成本最高的 {} 个会话:", limit));'),
    ('ColorOutput::success("每日趋势:");', 'ColorOutput::info("每日趋势:");'),
    ('ColorOutput::warning("目录不存在，无需清理");', 'ColorOutput::info("目录不存在，无需清理");'),
    ('ColorOutput::warning("没有找到需要清理的文件");', 'ColorOutput::info("没有找到需要清理的文件");'),
    ('ColorOutput::warning("模拟运行模式，未实际删除");', 'ColorOutput::info("模拟运行模式，未实际删除");'),
    ('ColorOutput::warning("已取消");', 'ColorOutput::info("已取消");'),
])
replace('commands/lifecycle/clean.rs', [
    ('ColorOutput::warning("模拟运行模式(不会实际删除文件)");', 'ColorOutput::info("模拟运行模式(不会实际删除文件)");'),
    ('ColorOutput::success("没有需要清理的文件");', 'ColorOutput::info("没有需要清理的文件");'),
    ('ColorOutput::success("没有找到需要清理的规划文件");', 'ColorOutput::info("没有找到需要清理的规划文件");'),
    ('ColorOutput::info("提示: 使用 --dry-run 参数可以先预览将要删除的文件");', 'ColorOutput::info("使用 --dry-run 参数可以先预览将要删除的文件");'),
    ('println!();\n        ColorOutput::info("提示: 运行 \'ccr clean backups\' (不带 --dry-run) 执行实际清理");', 'crate::commands::common::print_next_steps(&[("执行实际清理", &format!("ccr clean backups --days {days}"))]);'),
    ('println!();\n        let rerun_hint = if all { "ccr clean --all" } else { "ccr clean" };\n        ColorOutput::info(&format!("提示: 运行 \'{rerun_hint}\' 执行实际清理"));', 'let rerun_hint = if all { "ccr clean --all" } else { "ccr clean" };\n        crate::commands::common::print_next_steps(&[("执行实际清理", rerun_hint)]);'),
    ('ColorOutput::success(&format!("释放空间: {:.2} MB", size_mb));', 'ColorOutput::key_value("释放空间", &format!("{size_mb:.2} MB"), 2);'),
    ('ColorOutput::success(&format!(\n            "释放空间: {:.2} MB",\n            result.total_size as f64 / 1024.0 / 1024.0\n        ));', 'ColorOutput::key_value("释放空间", &format!("{:.2} MB", result.total_size as f64 / 1024.0 / 1024.0), 2);'),
])
replace('commands/lifecycle/clear.rs', [
    ('ColorOutput::success("settings.json 中没有 CCR 托管的 Claude 环境变量，无需清理");', 'ColorOutput::info("settings.json 中没有 CCR 托管的 Claude 环境变量，无需清理");'),
    ('"提示: Claude Code 将无法正常工作，直到您重新执行 ccr switch 切换配置"', '"清除托管环境变量后，需要重新应用配置以恢复 Claude Code 的对应设置"'),
    ('println!();\n    ColorOutput::info("提示:");\n    ColorOutput::info("   • 使用 \'ccr switch <配置名>\' 重新应用配置");\n    ColorOutput::info("   • 使用 \'ccr list\' 查看可用配置");\n    ColorOutput::info(&format!(\n        "   • 如需恢复，可使用备份文件: {}",\n        backup_path.display()\n    ));', 'ColorOutput::key_value("恢复备份", &backup_path.display().to_string(), 2);\n    crate::commands::common::print_next_steps(&[("查看可用配置", "ccr list")]);'),
])
replace('commands/lifecycle/init.rs', [
    ('ColorOutput::info("提示:");\n            println!("  • 查看平台列表: ccr platform list");\n            println!("  • 初始化 Claude 模板: ccr claude profile init");\n            println!("  • 初始化 Codex 模板: ccr codex profile init");\n            println!("  • 初始化 Grok 模板: ccr grok profile init");\n            println!("  • 强制重新初始化: ccr init --force");\n            println!();', 'crate::commands::common::print_next_steps(&[("查看平台列表", "ccr platform list")]);'),
    ('ColorOutput::info("提示: 现有配置会自动备份");', 'ColorOutput::info("现有配置会自动备份");'),
    ('ColorOutput::info("后续步骤:");\n    println!("  1. 使用 \'ccr platform list\' 查看所有平台");\n    println!(\n        "  2. 使用 \'ccr claude profile init\'、\'ccr codex profile init\' 或 \'ccr grok profile init\' 初始化平台模板"\n    );\n    println!("  3. 使用 \'ccr add\' 添加配置 profile");\n    println!("  4. 使用 \'ccr list\' 查看配置列表");\n    println!();\n\n    ColorOutput::info("提示:");\n    println!("  • 查看帮助: ccr --help");\n    println!();', 'crate::commands::common::print_next_steps(&[("查看平台列表", "ccr platform list")]);'),
])
replace('commands/update.rs', [
    ('println!();\n    ColorOutput::info("提示: 运行 \'ccr update\' 执行更新(去掉 --check 参数)");\n    println!();', 'crate::commands::common::print_next_steps(&[("执行更新", "ccr update")]);'),
    ('println!();\n    ColorOutput::info("后续步骤:");\n    println!("  1. 运行 \'ccr version\' 查看新版本信息");\n    println!("  2. 运行 \'ccr --help\' 查看新功能");\n    println!();', 'crate::commands::common::print_next_steps(&[("查看版本信息", "ccr version")]);'),
])

print('Reviewed semantic groups migrated.')
