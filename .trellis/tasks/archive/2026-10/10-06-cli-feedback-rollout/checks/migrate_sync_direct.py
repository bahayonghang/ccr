"""Migrate direct Sync status printers while preserving their stdout stream."""

import re
from pathlib import Path

path = Path('crates/ccr-cli/src/sync/commands.rs')
source = path.read_text(encoding='utf-8')
decorator = r'(?:\.\w+\(\))*'
pattern = re.compile(r'println!\(\s*"\{\}\s+\{\}",\s*"([✓⚠ℹ])"' + decorator + r',\s*"([^"\n]*)"' + decorator + r',?\s*\)')


def status(match):
    marker, message = match.groups()
    method = {'✓': 'success', '⚠': 'warning', 'ℹ': 'info'}[marker]
    return f'ColorOutput::{method}("{message}")'


source, count = pattern.subn(status, source)
source = re.sub(r'println!\("\{\} 已取消(上传|下载)", "ℹ"' + decorator + r'\)', r'ColorOutput::info("已取消\1")', source)
source = re.sub(r'println!\("   \{\}", "✓ (上传成功|下载成功)"' + decorator + r'\)', r'ColorOutput::success("\1")', source)
source = re.sub(r'println!\("   \{\} \{\}", "✗ (上传失败|下载失败):"' + decorator + r', e\)',
                r'println!("{}", ColorOutput::format_status(OutputStatus::Error, &format!("\1: {e}"), io::stdout().is_terminal()))', source)
source = re.sub(r'print!\("(?:🔍 |🚀 |💾 |⬇️  )([^"\n]*)"\)',
                r'print!("{}", ColorOutput::format_status(OutputStatus::Step, "\1", io::stdout().is_terminal()))', source)
source = source.replace('use ccr_core::core::logging::ColorOutput;', 'use ccr_core::core::logging::{ColorOutput, OutputStatus};')
source = source.replace('use std::io::{self, Write};', 'use std::io::{self, IsTerminal, Write};')

for before, after in [
    ('println!("   💡 提示: 运行 {} 首次上传", "ccr sync push".cyan());', 'crate::commands::common::print_next_steps(&[("首次上传配置", "ccr sync push")]);'),
    ('println!("📝 配置步骤:");\n        println!("   1. 运行 {} 开始配置", "ccr sync config".cyan());\n        println!("   2. 输入 WebDAV 服务器信息");\n        println!("   3. 测试连接成功后即可使用");', 'crate::commands::common::print_next_steps(&[("配置 WebDAV 同步", "ccr sync config")]);'),
    ('println!();\n        println!("   💡 提示: 首次使用需要先上传配置到云端");\n        println!("   运行命令: {}", "ccr sync push".cyan());\n        println!();', 'crate::commands::common::print_next_steps(&[("首次上传配置", "ccr sync push")]);'),
    ('println!();\n        println!("   💡 提示: 首次使用需要先上传配置到云端");\n        println!(\n            "   运行命令: {}",\n            format!("ccr sync {} push", folder_name).cyan()\n        );\n        println!();', 'crate::commands::common::print_next_steps(&[("查看文件夹同步帮助", "ccr sync --help")]);'),
    ('println!();\n        println!(\n            "  💡 提示: 运行 {} 首次上传",\n            format!("ccr sync {} push", folder_name).cyan()\n        );', 'crate::commands::common::print_next_steps(&[("查看文件夹同步帮助", "ccr sync --help")]);'),
    ('println!("💡 下一步: 运行 {} 查看配置", "ccr list".cyan());', 'crate::commands::common::print_next_steps(&[("查看配置", "ccr list")]);'),
    ('println!("📊 同步信息:");', 'ColorOutput::info("同步信息:");'),
    ('println!("   • 本地路径: {}", sync_path.display().to_string().cyan());', 'ColorOutput::key_value("本地路径", &sync_path.display().to_string(), 2);'),
    ('println!("   • 本地路径: {}", local_path.display().to_string().cyan());', 'ColorOutput::key_value("本地路径", &local_path.display().to_string(), 2);'),
    ('println!("   • 远程路径: {}", sync_config.remote_path.cyan());', 'ColorOutput::key_value("远程路径", &sync_config.remote_path, 2);'),
    ('println!("   • 远程路径: {}", folder.remote_path.cyan());', 'ColorOutput::key_value("远程路径", &folder.remote_path, 2);'),
    ('println!("   • 服务器: {}", sync_config.webdav_url.dimmed());', 'ColorOutput::key_value("服务器", &sync_config.webdav_url, 2);'),
    ('println!("   • 服务器: {}", webdav_config.url.dimmed());', 'ColorOutput::key_value("服务器", &webdav_config.url, 2);'),
    ('println!("   • 变化项: {}", changed_count.to_string().cyan());', 'ColorOutput::key_value("变化项", &changed_count.to_string(), 2);'),
    ('println!("   • 跳过项: {}", skipped_count.to_string().cyan());', 'ColorOutput::key_value("跳过项", &skipped_count.to_string(), 2);'),
    ('println!(\n            "   📁 备份位置: {}",\n            backup_path.display().to_string().dimmed()\n        );', 'ColorOutput::key_value("备份位置", &backup_path.display().to_string(), 2);'),
    ('println!(\n            "   📁 备份位置: {}",\n            backup_path.display().to_string().dimmed()\n        );', 'ColorOutput::key_value("备份位置", &backup_path.display().to_string(), 2);'),
    ('println!("  成功: {}", success_count.to_string().green());', 'ColorOutput::key_value("成功", &success_count.to_string(), 2);'),
    ('println!("  失败: {}", failed_count.to_string().red());', 'ColorOutput::key_value("失败", &failed_count.to_string(), 2);'),
]:
    source = source.replace(before, after)

path.write_text(source, encoding='utf-8', newline='')
print('Direct Sync status calls migrated:', count)
