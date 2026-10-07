"""Apply reviewed C3 session, temporary-setting, sync and startup messages."""

from migrate_semantics import replace

replace('commands/temp_cmd.rs', [
    ('ColorOutput::info("本小姐来帮你快速设置临时配置！");', 'ColorOutput::info("设置临时配置");'),
    ('println!();\n\n    // 提示信息\n    ColorOutput::info("提示:");\n    ColorOutput::info("   • 临时配置已立即生效");\n    ColorOutput::info("   • 执行 \'ccr switch <配置名>\' 可恢复为 TOML 配置");\n    ColorOutput::info("   • 执行 \'ccr current\' 可查看当前配置状态");\n    println!();', '// 提示信息\n    ColorOutput::info("重新应用配置时会恢复 TOML 配置中的值");\n    crate::commands::common::print_next_steps(&[("查看当前配置状态", "ccr current")]);'),
])
replace('commands/temp_token.rs', [
    ('println!();\n    ColorOutput::info("提示:");\n    ColorOutput::info("   • 临时配置已立即应用到 settings.json");\n    ColorOutput::info("   • 下次 switch 时将使用配置文件中的原始 token");\n    ColorOutput::info("   • 临时配置不会修改 toml 配置文件");', 'ColorOutput::info("临时配置已应用到 settings.json；toml 配置文件保持原值");\n    ColorOutput::info("下次 switch 时将使用配置文件中的原始 token");\n    crate::commands::common::print_next_steps(&[("查看临时覆盖状态", "ccr temp-token show")]);'),
    ('ColorOutput::success(&format!(\n                "当前有 {} 个字段被临时覆盖",', 'ColorOutput::info(&format!(\n                "当前有 {} 个字段被临时覆盖",'),
    ('ColorOutput::info("使用 \'ccr temp-token set <TOKEN>\' 设置临时token");', 'crate::commands::common::print_next_steps(&[("查看临时设置帮助", "ccr temp-token set --help")]);'),
])
replace('commands/sessions_cmd.rs', [
    ('ColorOutput::warning("未找到任何 session");\n        ColorOutput::info("提示: 运行 \'ccr sessions reindex\' 重建索引");', 'ColorOutput::info("未找到任何 session");\n        crate::commands::common::print_next_steps(&[("重建会话索引", "ccr sessions reindex")]);'),
    ('ColorOutput::warning(&format!("未找到匹配 \'{}\' 的 session", query));', 'ColorOutput::info(&format!("未找到匹配 \'{}\' 的 session", query));'),
    ('ColorOutput::info(&format!("恢复命令: {}", s.resume_command()));', 'crate::commands::common::print_next_steps(&[("恢复会话", &s.resume_command())]);'),
    ('ColorOutput::info(&format!("执行: {}", cmd));\n                ColorOutput::warning("注意: 自动执行功能尚未实现，请手动运行上述命令");\n                println!();\n                println!("  {}", cmd);', 'ColorOutput::warning("自动执行功能尚未实现，请手动运行命令");\n                crate::commands::common::print_next_steps(&[("恢复会话", &cmd)]);'),
    ('ColorOutput::info("开始索引 sessions...");', 'ColorOutput::step("开始索引 sessions...");'),
    ('ColorOutput::success("索引完成");', 'if stats.errors > 0 {\n        ColorOutput::warning("索引完成，部分文件未成功索引");\n    } else {\n        ColorOutput::success("索引完成");\n    }'),
])
replace('commands/provider_cmd.rs', [
    ('ColorOutput::info("使用 \'ccr list\' 查看可用配置");', 'crate::commands::common::print_next_steps(&[("查看可用配置", "ccr list")]);'),
    ('ColorOutput::warning("没有可用的配置");', 'ColorOutput::info("没有可用的配置");'),
    ('println!();\n        ColorOutput::info("提示: 使用 \'ccr provider test <name> --verbose\' 查看单个 Provider 详情");', 'crate::commands::common::print_next_steps(&[("查看 Provider 测试选项", "ccr provider test --help")]);'),
])
replace('commands/codex/env.rs', [
    ('ColorOutput::warning("当前 Profile 不需要额外环境变量导出");', 'ColorOutput::info("当前 Profile 不需要额外环境变量导出");'),
])
replace('commands/codex/quota.rs', [
    ('ColorOutput::warning("未找到已保存的 Codex 账号");\n        ColorOutput::info("使用 `ccr codex auth save <name>` 保存账号后再查询配额");', 'ColorOutput::info("未找到已保存的 Codex 账号");\n        crate::commands::common::print_next_steps(&[("查看账号保存帮助", "ccr codex auth save --help")]);'),
    ('ColorOutput::success("Codex 账号配额余额");', 'ColorOutput::info("Codex 账号配额余额");'),
])
replace('commands/codex/fix.rs', [
    ('ColorOutput::step("进程清理（skipped）");', 'ColorOutput::info(&ColorOutput::format_status(ccr_core::core::logging::OutputStatus::Skipped, "进程清理", std::io::IsTerminal::is_terminal(&std::io::stdout())));'),
    ('ColorOutput::step("上游 doctor（skipped）");', 'ColorOutput::info(&ColorOutput::format_status(ccr_core::core::logging::OutputStatus::Skipped, "上游 doctor", std::io::IsTerminal::is_terminal(&std::io::stdout())));'),
    ('ColorOutput::info("需要上游健康检查时运行 ccr codex fix --doctor");', 'crate::commands::common::print_next_steps(&[("运行上游健康检查", "ccr codex fix --doctor")]);'),
    ('ColorOutput::success("未发现残留的 Codex app-server 进程");', 'ColorOutput::info("未发现残留的 Codex app-server 进程");'),
    ('ColorOutput::info("可运行 ccr codex fix --repair-runtime 显式修复本地漂移");', 'crate::commands::common::print_next_steps(&[("显式修复本地漂移", "ccr codex fix --repair-runtime")]);'),
])
replace('commands/profile/current.rs', [
    ('ColorOutput::info("提示: 使用 `ccr current --verbose` 查看路径、环境变量和完整配置详情");', 'crate::commands::common::print_next_steps(&[("查看完整配置详情", "ccr current --verbose")]);'),
    ('ColorOutput::info("提示: * 标记的为必需环境变量");', 'ColorOutput::info("* 标记的为必需环境变量");'),
])
replace('commands/profile/switch.rs', [
    ('"提示: 从 {} {} 切换到 {} {}",\n        old_current,\n        "→",\n        config_name,\n        "✓"', '"已从 {} 切换到 {}",\n        old_current,\n        config_name'),
])
replace('services/ui_service.rs', [
    ('ColorOutput::success("当前已是最新版本，无需更新");', 'ColorOutput::info("当前已是最新版本，无需更新");'),
    ('ColorOutput::warning("提示: 按 Ctrl+C 停止服务");', 'ColorOutput::info("按 Ctrl+C 停止服务");'),
    ('println!();\n        ColorOutput::info("快速安装:");\n        ColorOutput::info("  cargo install just");\n        println!();', 'crate::commands::common::print_next_steps(&[("安装 just", "cargo install just")]);'),
    ('ColorOutput::warning("下载中 (这可能需要几分钟)...");', 'ColorOutput::step("下载中 (这可能需要几分钟)...");'),
    ('ColorOutput::info("正在检查远程版本...");', 'ColorOutput::step("检查远程版本");'),
    ('ColorOutput::info("检查 just 工具...");', 'ColorOutput::step("检查 just 工具");'),
    ('ColorOutput::info("检查项目依赖...");', 'ColorOutput::step("检查项目依赖");'),
    ('ColorOutput::info("正在安装依赖 (这可能需要几分钟)...");', 'ColorOutput::step("安装依赖 (这可能需要几分钟)");'),
    ('ColorOutput::info("构建生产版本...");', 'ColorOutput::step("构建生产版本");'),
    ('ColorOutput::info("提示: CCR UI 是一个完整的 Vue 3 + Tauri 应用");', 'ColorOutput::info("CCR UI 是一个完整的 Vue 3 + Tauri 应用");'),
])
replace('sync/commands.rs', [
    ('println!();\n\n    ColorOutput::info("可用命令:");\n    println!("  ccr sync status    # 查看同步状态");\n    println!("  ccr sync push      # 上传配置到云端");\n    println!("  ccr sync pull      # 从云端下载配置");\n    println!();', 'crate::commands::common::print_next_steps(&[("查看同步状态", "ccr sync status")]);'),
    ('ColorOutput::warning("未选择任何同步内容，操作取消");', 'ColorOutput::info("未选择任何同步内容，操作取消");'),
    ('ColorOutput::warning("暂无注册的同步文件夹");', 'ColorOutput::info("暂无注册的同步文件夹");'),
    ('ColorOutput::info("使用 \'ccr sync folder add\' 添加文件夹");', 'crate::commands::common::print_next_steps(&[("添加同步文件夹", "ccr sync folder add")]);'),
    ('ColorOutput::info("提示: 使用 \'ccr sync <folder> push\' 开始同步");', 'crate::commands::common::print_next_steps(&[("查看文件夹同步帮助", "ccr sync --help")]);'),
    ('ColorOutput::success("  本地路径: ✓ 存在 (目录)");', 'ColorOutput::key_value("本地路径", "存在 (目录)", 2);'),
    ('ColorOutput::success("  本地路径: ✓ 存在 (文件)");', 'ColorOutput::key_value("本地路径", "存在 (文件)", 2);'),
    ('ColorOutput::warning("  本地路径: ✗ 不存在");', 'ColorOutput::warning("本地路径不存在");'),
    ('ColorOutput::warning(&format!("文件夹 \'{}\' 已经是启用状态", name));', 'ColorOutput::info(&format!("文件夹 \'{}\' 已经是启用状态", name));'),
    ('ColorOutput::warning(&format!("文件夹 \'{}\' 已经是禁用状态", name));', 'ColorOutput::info(&format!("文件夹 \'{}\' 已经是禁用状态", name));'),
    ('ColorOutput::warning("没有启用的同步文件夹");', 'ColorOutput::info("没有启用的同步文件夹");'),
    ('ColorOutput::info("使用 \'ccr sync folder list\' 查看所有文件夹");', 'crate::commands::common::print_next_steps(&[("查看同步文件夹", "ccr sync folder list")]);'),
    ('ColorOutput::info("使用 \'ccr sync folder enable <name>\' 启用该文件夹");', 'crate::commands::common::print_next_steps(&[("查看启用文件夹帮助", "ccr sync folder enable --help")]);'),
])
print('Remaining reviewed groups migrated.')
