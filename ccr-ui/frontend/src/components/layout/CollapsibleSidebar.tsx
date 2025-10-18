'use client';

import { useState } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Settings, Terminal, ChevronLeft, ChevronRight, Menu, ChevronDown, ChevronUp, Zap, Server, Command, Bot, Puzzle, Sparkles, Gem, Workflow, Cloud, Code2, ArrowLeftRight } from 'lucide-react';

// 导航菜单结构 - 支持层级菜单
const navigationGroups = [
  // CCR 命令执行 - 包含所有 CLI 客户端
  {
    title: 'CCR 命令执行',
    icon: Terminal,
    defaultExpanded: false,
    items: [
      { name: 'CCR 命令', href: '/commands/ccr', icon: Zap },
      { name: 'Claude Code 命令', href: '/commands/claude-code', icon: Zap },
      { name: 'Claude 命令', href: '/commands/claude', icon: Zap },
      { name: 'Qwen 命令', href: '/commands/qwen', icon: Sparkles },
      { name: 'Gemini 命令', href: '/commands/gemini', icon: Gem },
      { name: 'IFLOW 命令', href: '/commands/iflow', icon: Workflow },
    ],
  },
  // 各个 CLI 工具配置管理
  {
    title: 'Claude Code',
    icon: Zap,
    defaultExpanded: true, // 默认展开
    items: [
      { name: '配置管理', href: '/configs', icon: Settings },
      { name: '☁️ 云同步', href: '/sync', icon: Cloud },
      { name: 'MCP 管理', href: '/mcp', icon: Server },
      { name: 'Slash Commands', href: '/slash-commands', icon: Command },
      { name: 'Agents 管理', href: '/agents', icon: Bot },
      { name: '插件管理', href: '/plugins', icon: Puzzle },
    ],
  },
  {
    title: 'Qwen',
    icon: Sparkles,
    defaultExpanded: false,
    items: [
      { name: 'MCP 管理', href: '/qwen/mcp', icon: Server },
      { name: 'Slash Commands', href: '/qwen/slash-commands', icon: Command },
      { name: 'Agents 管理', href: '/qwen/agents', icon: Bot },
      { name: '插件管理', href: '/qwen/plugins', icon: Puzzle },
    ],
  },
  {
    title: 'Gemini Cli',
    icon: Gem,
    defaultExpanded: false,
    items: [
      { name: 'MCP 管理', href: '/gemini-cli/mcp', icon: Server },
      { name: 'Slash Commands', href: '/gemini-cli/slash-commands', icon: Command },
      { name: 'Agents 管理', href: '/gemini-cli/agents', icon: Bot },
      { name: '插件管理', href: '/gemini-cli/plugins', icon: Puzzle },
    ],
  },
  {
    title: 'IFLOW',
    icon: Workflow,
    defaultExpanded: false,
    items: [
      { name: 'MCP 管理', href: '/iflow/mcp', icon: Server },
      { name: 'Slash Commands', href: '/iflow/slash-commands', icon: Command },
      { name: 'Agents 管理', href: '/iflow/agents', icon: Bot },
      { name: '插件管理', href: '/iflow/plugins', icon: Puzzle },
    ],
  },
  {
    title: 'Codex',
    icon: Code2,
    defaultExpanded: false,
    items: [
      { name: 'MCP 管理', href: '/codex/mcp', icon: Server },
      { name: 'Profile 管理', href: '/codex/profiles', icon: Settings },
    ],
  },
  // 配置转换器
  {
    title: '配置转换器',
    icon: ArrowLeftRight,
    defaultExpanded: false,
    items: [
      { name: 'CLI 配置转换', href: '/converter', icon: ArrowLeftRight },
    ],
  },
];

export default function CollapsibleSidebar() {
  const [collapsed, setCollapsed] = useState(false);
  const [expandedGroups, setExpandedGroups] = useState<Record<string, boolean>>(
    // 初始化展开状态
    navigationGroups.reduce((acc, group) => {
      acc[group.title] = group.defaultExpanded ?? false;
      return acc;
    }, {} as Record<string, boolean>)
  );
  const pathname = usePathname();

  const toggleGroup = (groupTitle: string) => {
    setExpandedGroups(prev => ({
      ...prev,
      [groupTitle]: !prev[groupTitle],
    }));
  };

  return (
    <aside
      className={`rounded-xl p-4 h-fit sticky top-5 transition-all duration-300 glass-effect ${
        collapsed ? 'w-16' : 'w-64'
      }`}
      style={{
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-small)',
      }}
    >
      {/* 切换按钮 */}
      <div className="flex items-center justify-between mb-4">
        {!collapsed && (
          <div
            className="text-xs font-semibold uppercase tracking-wider"
            style={{ color: 'var(--text-secondary)' }}
          >
            导航菜单
          </div>
        )}
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="p-2 rounded-lg transition-all hover:scale-110"
          style={{
            background: 'var(--bg-tertiary)',
            border: '1px solid var(--border-color)',
            color: 'var(--text-secondary)',
          }}
          title={collapsed ? '展开菜单' : '收起菜单'}
          aria-label={collapsed ? '展开菜单' : '收起菜单'}
        >
          {collapsed ? (
            <ChevronRight className="w-4 h-4" />
          ) : (
            <ChevronLeft className="w-4 h-4" />
          )}
        </button>
      </div>

      {/* 导航链接 - 层级菜单 */}
      <nav className="space-y-2" aria-label="主导航">
        {navigationGroups.map((group, groupIndex) => {
          const GroupIcon = group.icon;
          const isExpanded = expandedGroups[group.title];
          const hasActiveChild = group.items.some(item => pathname === item.href);

          return (
            <div key={group.title} className="space-y-1">
              {/* 分隔线（折叠状态且非第一个分组） */}
              {collapsed && groupIndex > 0 && (
                <div
                  className="h-px mx-2 my-2"
                  style={{ background: 'var(--border-color)' }}
                  aria-hidden="true"
                />
              )}

              {/* 分组头部 */}
              <button
                  onClick={() => !collapsed && toggleGroup(group.title)}
                  className={`w-full flex items-center ${
                    collapsed ? 'justify-center' : 'justify-between'
                  } px-4 py-3 rounded-lg transition-all hover:scale-[1.02] ${
                    hasActiveChild ? 'text-white' : ''
                  }`}
                  style={{
                    background: hasActiveChild
                      ? 'linear-gradient(135deg, rgba(139, 92, 246, 0.2), rgba(168, 85, 247, 0.2))'
                      : 'var(--bg-tertiary)',
                    border: `1px solid ${hasActiveChild ? 'var(--accent-primary)' : 'var(--border-color)'}`,
                    color: hasActiveChild ? 'var(--accent-primary)' : 'var(--text-primary)',
                  }}
                  title={collapsed ? group.title : undefined}
                  aria-expanded={!collapsed && isExpanded}
                  aria-label={`${group.title} 菜单组`}
                >
                  <div className={`flex items-center ${collapsed ? '' : 'space-x-3'}`}>
                    <GroupIcon className="w-5 h-5 flex-shrink-0" />
                    {!collapsed && (
                      <span className="font-semibold text-sm">{group.title}</span>
                    )}
                  </div>
                  {!collapsed && (
                    isExpanded ? (
                      <ChevronUp className="w-4 h-4" />
                    ) : (
                      <ChevronDown className="w-4 h-4" />
                    )
                  )}
                </button>

              {/* 子菜单项 - 仅在展开状态且非折叠时显示 */}
              {!collapsed && isExpanded && (
                <div className="ml-2 space-y-1 border-l-2" style={{ borderColor: 'var(--border-color)' }}>
                  {group.items.map((item) => {
                    const isActive = pathname === item.href;
                    const Icon = item.icon;

                    return (
                      <Link
                        key={item.href}
                        href={item.href}
                        className={`flex items-center space-x-3 px-4 py-2.5 ml-2 rounded-lg transition-all relative overflow-hidden group hover:translate-x-0.5 ${
                          isActive ? 'text-white' : ''
                        }`}
                        style={{
                          background: isActive
                            ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                            : 'transparent',
                          border: `1px solid ${isActive ? 'var(--accent-primary)' : 'transparent'}`,
                          boxShadow: isActive ? '0 0 15px var(--glow-primary)' : undefined,
                          color: isActive ? 'white' : 'var(--text-secondary)',
                        }}
                        aria-current={isActive ? 'page' : undefined}
                      >
                        {/* 左侧指示器 */}
                        <span
                          className={`absolute left-0 top-0 w-1 h-full transition-all ${
                            isActive ? 'scale-y-100' : 'scale-y-0 group-hover:scale-y-50'
                          }`}
                          style={{ background: 'var(--accent-primary)' }}
                          aria-hidden="true"
                        />
                        
                        <Icon className="w-4 h-4 flex-shrink-0" />
                        <span className="font-medium text-sm">{item.name}</span>
                      </Link>
                    );
                  })}
                </div>
              )}

              {/* 折叠状态下，显示子菜单作为独立项 */}
              {collapsed && (
                <div className="space-y-1">
                  {group.items.map((item) => {
                    const isActive = pathname === item.href;
                    const Icon = item.icon;

                    return (
                      <Link
                        key={item.href}
                        href={item.href}
                        className={`flex items-center justify-center px-4 py-3 rounded-lg transition-all relative overflow-hidden group ${
                          isActive ? 'text-white' : ''
                        }`}
                        style={{
                          background: isActive
                            ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                            : 'var(--bg-tertiary)',
                          border: `1px solid ${isActive ? 'var(--accent-primary)' : 'var(--border-color)'}`,
                          boxShadow: isActive ? '0 0 15px var(--glow-primary)' : undefined,
                          color: isActive ? 'white' : 'var(--text-secondary)',
                        }}
                        title={item.name}
                        aria-current={isActive ? 'page' : undefined}
                      >
                        <Icon className="w-5 h-5 flex-shrink-0" />
                      </Link>
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </nav>

      {/* 收起状态提示 */}
      {collapsed && (
        <div className="mt-4 text-center">
          <button
            onClick={() => setCollapsed(false)}
            className="p-2 rounded-lg transition-all hover:scale-110"
            style={{
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border-color)',
              color: 'var(--text-muted)',
            }}
            title="展开菜单"
            aria-label="展开菜单"
          >
            <Menu className="w-4 h-4" />
          </button>
        </div>
      )}
    </aside>
  );
}

