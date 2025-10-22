#!/usr/bin/env python3
"""
批量优化 Vue 页面模板
移除 Navbar/StatusHeader，添加 Breadcrumb
"""

import re
import os
from pathlib import Path

# 模块配置
MODULES = {
    'codex': {
        'color': '#ec4899',
        'icon': 'Boxes',
        'label': 'Codex',
        'path': '/codex',
        'pages': {
            'CodexMcpView.vue': {'name': 'MCP 服务器', 'path': '/codex/mcp', 'icon': 'Server', 'desc': 'Model Context Protocol 服务器配置和管理'},
            'CodexProfilesView.vue': {'name': 'Profiles 配置', 'path': '/codex/profiles', 'icon': 'User', 'desc': 'Codex 配置文件管理'},
            'CodexAgentsView.vue': {'name': 'Agents', 'path': '/codex/agents', 'icon': 'Bot', 'desc': 'AI Agent 配置和管理'},
            'CodexSlashCommandsView.vue': {'name': 'Slash Commands', 'path': '/codex/slash-commands', 'icon': 'Command', 'desc': '自定义命令管理和配置'},
            'CodexPluginsView.vue': {'name': '插件管理', 'path': '/codex/plugins', 'icon': 'Puzzle', 'desc': '插件启用/禁用和配置管理'},
        }
    },
    'gemini': {
        'color': '#8b5cf6',
        'icon': 'Sparkles',
        'label': 'Gemini CLI',
        'path': '/gemini-cli',
        'pages': {
            'GeminiMcpView.vue': {'name': 'MCP 服务器', 'path': '/gemini-cli/mcp', 'icon': 'Server', 'desc': 'Model Context Protocol 服务器配置和管理'},
            'GeminiAgentsView.vue': {'name': 'Agents', 'path': '/gemini-cli/agents', 'icon': 'Bot', 'desc': 'AI Agent 配置和管理'},
            'GeminiSlashCommandsView.vue': {'name': 'Slash Commands', 'path': '/gemini-cli/slash-commands', 'icon': 'Command', 'desc': '自定义命令管理和配置'},
            'GeminiPluginsView.vue': {'name': '插件管理', 'path': '/gemini-cli/plugins', 'icon': 'Puzzle', 'desc': '插件启用/禁用和配置管理'},
        }
    },
    'qwen': {
        'color': '#14b8a6',
        'icon': 'Zap',
        'label': 'Qwen',
        'path': '/qwen',
        'pages': {
            'QwenMcpView.vue': {'name': 'MCP 服务器', 'path': '/qwen/mcp', 'icon': 'Server', 'desc': 'Model Context Protocol 服务器配置和管理'},
            'QwenAgentsView.vue': {'name': 'Agents', 'path': '/qwen/agents', 'icon': 'Bot', 'desc': 'AI Agent 配置和管理'},
            'QwenSlashCommandsView.vue': {'name': 'Slash Commands', 'path': '/qwen/slash-commands', 'icon': 'Command', 'desc': '自定义命令管理和配置'},
            'QwenPluginsView.vue': {'name': '插件管理', 'path': '/qwen/plugins', 'icon': 'Puzzle', 'desc': '插件启用/禁用和配置管理'},
        }
    },
    'iflow': {
        'color': '#f97316',
        'icon': 'Flame',
        'label': 'iFlow',
        'path': '/iflow',
        'pages': {
            'IflowMcpView.vue': {'name': 'MCP 服务器', 'path': '/iflow/mcp', 'icon': 'Server', 'desc': 'Model Context Protocol 服务器配置和管理'},
            'IflowAgentsView.vue': {'name': 'Agents', 'path': '/iflow/agents', 'icon': 'Bot', 'desc': 'AI Agent 配置和管理'},
            'IflowSlashCommandsView.vue': {'name': 'Slash Commands', 'path': '/iflow/slash-commands', 'icon': 'Command', 'desc': '自定义命令管理和配置'},
            'IflowPluginsView.vue': {'name': '插件管理', 'path': '/iflow/plugins', 'icon': 'Puzzle', 'desc': '插件启用/禁用和配置管理'},
        }
    }
}

def generate_breadcrumb(module_icon, module_label, module_path, page_name, page_path, page_icon, color):
    """生成 Breadcrumb 代码"""
    return f"""      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          {{ label: '首页', path: '/', icon: Home }},
          {{ label: '{module_label}', path: '{module_path}', icon: {module_icon} }},
          {{ label: '{page_name}', path: '{page_path}', icon: {page_icon} }}
        ]"
        moduleColor="{color}"
      />
"""

def rgb_from_hex(hex_color):
    """从十六进制颜色转换为 RGB"""
    hex_color = hex_color.lstrip('#')
    r, g, b = tuple(int(hex_color[i:i+2], 16) for i in (0, 2, 4))
    return r, g, b

def process_file(filepath, module_config, page_config):
    """处理单个文件"""
    module_color = module_config['color']
    module_label = module_config['label']
    module_path = module_config['path']
    module_icon = module_config['icon']
    
    page_name = page_config['name']
    page_path = page_config['path']
    page_icon = page_config['icon']
    page_desc = page_config['desc']
    
    r, g, b = rgb_from_hex(module_color)
    
    print(f"  处理 {os.path.basename(filepath)}...")
    
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # 1. 移除 Navbar 和 StatusHeader（如果存在）
    content = re.sub(r'\s*<Navbar\s*/>\s*\n', '', content)
    content = re.sub(r'\s*<StatusHeader[\s\S]*?/>\s*\n', '', content)
    
    # 2. 在 <div class="grid grid-cols-[auto_1fr] gap-4"> 之前添加 Breadcrumb
    if '<Breadcrumb' not in content:
        breadcrumb = generate_breadcrumb(
            module_icon, module_label, module_path,
            page_name, page_path, page_icon, module_color
        )
        content = re.sub(
            r'(\s*<div class="grid grid-cols-\[auto_1fr\] gap-4">)',
            f'{breadcrumb}\\1',
            content
        )
    
    # 3. 优化标题区域（更复杂，需要分情况处理）
    # 这部分由于结构差异较大，建议手动处理或使用更复杂的逻辑
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)
    
    return True

def main():
    base_dir = Path(__file__).parent
    
    for module_key, module_config in MODULES.items():
        print(f"\n🔧 优化 {module_config['label']} 模块...")
        for filename, page_config in module_config['pages'].items():
            filepath = base_dir / filename
            if filepath.exists():
                try:
                    process_file(filepath, module_config, page_config)
                    print(f"    ✓ {filename}")
                except Exception as e:
                    print(f"    ✗ {filename}: {e}")
            else:
                print(f"    ⚠ {filename} 不存在")
    
    print("\n🎉 批量优化完成！")
    print("\n⚠️  注意：标题区域需要手动优化，请参考已完成的页面（如 McpView.vue）")

if __name__ == '__main__':
    main()
