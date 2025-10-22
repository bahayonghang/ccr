#!/usr/bin/env python3
"""
批量更新所有第三级页面的 CollapsibleSidebar 组件，添加 module 参数
"""

import re
from pathlib import Path

# 页面和对应的 module 映射
PAGE_MODULE_MAP = {
    # Claude Code 模块
    'ConfigsView.vue': 'claude-code',
    'SyncView.vue': 'claude-code',
    'McpView.vue': 'claude-code',
    'AgentsView.vue': 'claude-code',
    'SlashCommandsView.vue': 'claude-code',
    'PluginsView.vue': 'claude-code',
    
    # Codex 模块
    'CodexMcpView.vue': 'codex',
    'CodexProfilesView.vue': 'codex',
    'CodexAgentsView.vue': 'codex',
    'CodexSlashCommandsView.vue': 'codex',
    'CodexPluginsView.vue': 'codex',
    
    # Gemini CLI 模块
    'GeminiMcpView.vue': 'gemini-cli',
    'GeminiAgentsView.vue': 'gemini-cli',
    'GeminiSlashCommandsView.vue': 'gemini-cli',
    'GeminiPluginsView.vue': 'gemini-cli',
    
    # Qwen 模块
    'QwenMcpView.vue': 'qwen',
    'QwenAgentsView.vue': 'qwen',
    'QwenSlashCommandsView.vue': 'qwen',
    'QwenPluginsView.vue': 'qwen',
    
    # iFlow 模块
    'IflowMcpView.vue': 'iflow',
    'IflowAgentsView.vue': 'iflow',
    'IflowSlashCommandsView.vue': 'iflow',
    'IflowPluginsView.vue': 'iflow',
    
    # Converter
    'ConverterView.vue': 'converter',
    
    # Commands
    'CommandsView.vue': 'commands',
}

def update_file(filepath, module_name):
    """更新单个文件中的 CollapsibleSidebar 组件"""
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 查找 <CollapsibleSidebar /> 并替换为 <CollapsibleSidebar module="xxx" />
    # 支持多种格式：
    # 1. <CollapsibleSidebar />
    # 2. <CollapsibleSidebar/>
    # 3. <CollapsibleSidebar   />
    
    patterns = [
        (r'<CollapsibleSidebar\s*/>', f'<CollapsibleSidebar module="{module_name}" />'),
        (r'<CollapsibleSidebar/>', f'<CollapsibleSidebar module="{module_name}" />'),
    ]
    
    modified = False
    for pattern, replacement in patterns:
        if re.search(pattern, content):
            content = re.sub(pattern, replacement, content)
            modified = True
            break
    
    if modified:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    
    return False

def main():
    base_dir = Path('.')
    updated_count = 0
    skipped_count = 0
    
    print("🔧 开始批量更新 CollapsibleSidebar...")
    print()
    
    for filename, module_name in PAGE_MODULE_MAP.items():
        filepath = base_dir / filename
        
        if not filepath.exists():
            print(f"⚠  {filename} - 文件不存在")
            skipped_count += 1
            continue
        
        try:
            if update_file(filepath, module_name):
                print(f"✓ {filename} -> module=\"{module_name}\"")
                updated_count += 1
            else:
                print(f"⊘ {filename} - 已经有 module 或格式不匹配")
                skipped_count += 1
        except Exception as e:
            print(f"✗ {filename} - 错误: {e}")
            skipped_count += 1
    
    print()
    print(f"🎉 批量更新完成！")
    print(f"  ✓ 成功更新: {updated_count} 个文件")
    print(f"  ⊘ 跳过: {skipped_count} 个文件")

if __name__ == '__main__':
    main()
