'use client';

import Link from 'next/link';
import {
  Settings,
  Cloud,
  Server,
  Terminal,
  Users,
  Puzzle,
  ArrowRight,
  Code2,
  Home
} from 'lucide-react';

interface SubModule {
  title: string;
  description: string;
  icon: React.ReactNode;
  href: string;
  gradient: string;
  badge?: string;
}

export default function ClaudeCodePage() {
  const subModules: SubModule[] = [
    {
      title: '配置管理',
      description: 'Claude Code 配置切换、验证、历史记录管理',
      icon: <Settings className="w-8 h-8" />,
      href: '/configs',
      gradient: 'from-blue-500 to-cyan-500',
      badge: '核心功能'
    },
    {
      title: '云同步',
      description: 'WebDAV 云端配置同步和自动备份',
      icon: <Cloud className="w-8 h-8" />,
      href: '/sync',
      gradient: 'from-cyan-500 to-blue-500',
      badge: '新功能'
    },
    {
      title: 'MCP 服务器',
      description: 'Model Context Protocol 服务器配置和管理',
      icon: <Server className="w-8 h-8" />,
      href: '/mcp',
      gradient: 'from-purple-500 to-pink-500'
    },
    {
      title: 'Slash Commands',
      description: '自定义命令管理和文件夹组织',
      icon: <Terminal className="w-8 h-8" />,
      href: '/slash-commands',
      gradient: 'from-orange-500 to-red-500'
    },
    {
      title: 'Agents',
      description: 'AI Agent 配置、工具绑定和模型管理',
      icon: <Users className="w-8 h-8" />,
      href: '/agents',
      gradient: 'from-green-500 to-teal-500'
    },
    {
      title: '插件管理',
      description: '插件启用/禁用和配置管理',
      icon: <Puzzle className="w-8 h-8" />,
      href: '/plugins',
      gradient: 'from-indigo-500 to-purple-500'
    },
  ];

  return (
    <div className="min-h-screen relative" style={{ background: 'var(--bg-primary)' }}>
      {/* 🎨 动态背景装饰 - 液态玻璃风格 */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none -z-10">
        <div
          className="absolute top-20 right-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
          style={{ background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)' }}
        />
        <div
          className="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
          style={{
            background: 'linear-gradient(135deg, #ec4899 0%, #f59e0b 100%)',
            animationDelay: '1s'
          }}
        />
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
          style={{
            background: 'linear-gradient(135deg, #10b981 0%, #3b82f6 100%)',
            animationDelay: '2s'
          }}
        />
      </div>

      <div className="relative z-10 container mx-auto px-6 py-12">
        {/* 返回首页按钮 */}
        <Link
          href="/"
          className="inline-flex items-center gap-2 mb-8 transition-colors group"
          style={{ color: 'var(--text-secondary)' }}
        >
          <Home className="w-5 h-5 group-hover:-translate-x-1 transition-transform" />
          <span className="group-hover:text-[var(--accent-primary)]">返回首页</span>
        </Link>

        {/* 头部区域 */}
        <div className="mb-16">
          <div className="flex items-center gap-4 mb-4">
            <div
              className="p-4 rounded-2xl"
              style={{ background: 'linear-gradient(135deg, #6366f1, #8b5cf6)' }}
            >
              <Code2 className="w-12 h-12 text-white" />
            </div>
            <div>
              <h1
                className="text-5xl font-bold bg-gradient-to-r bg-clip-text text-transparent"
                style={{
                  backgroundImage: 'linear-gradient(to right, #6366f1, #8b5cf6, #ec4899)'
                }}
              >
                Claude Code
              </h1>
              <p
                className="text-xl mt-2"
                style={{ color: 'var(--text-primary)' }}
              >
                Claude Code 配置管理中心
              </p>
            </div>
          </div>
          <p
            className="max-w-2xl"
            style={{ color: 'var(--text-secondary)' }}
          >
            管理您的 Claude Code 配置、MCP 服务器、Agents、插件和自定义命令。支持云端同步和历史记录追踪。
          </p>
        </div>

        {/* 功能模块网格 */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {subModules.map((module, index) => (
            <Link
              key={module.href}
              href={module.href}
              className="group relative block"
              style={{ animationDelay: `${index * 100}ms` }}
            >
              <div
                className="glass-card p-6 h-full hover:scale-105 transition-all duration-300"
                style={{
                  border: '1px solid var(--border-color)',
                  boxShadow: 'var(--shadow-md)',
                }}
              >
                {/* 图标和徽章 */}
                <div className="flex items-start justify-between mb-5">
                  <div
                    className="inline-flex p-4 rounded-2xl"
                    style={{ background: `rgba(99, 102, 241, 0.15)` }}
                  >
                    <div style={{ color: '#6366f1' }}>
                      {module.icon}
                    </div>
                  </div>
                  {module.badge && (
                    <span
                      className="px-3 py-1 text-xs font-semibold rounded-full"
                      style={{
                        background: 'var(--accent-warning)',
                        color: 'white',
                      }}
                    >
                      {module.badge}
                    </span>
                  )}
                </div>

                {/* 标题和描述 */}
                <h3
                  className="text-2xl font-bold mb-3 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text transition-all"
                  style={{
                    color: 'var(--text-primary)',
                    backgroundImage: 'linear-gradient(to right, #6366f1, #8b5cf6)',
                  }}
                >
                  {module.title}
                </h3>
                <p
                  className="text-sm mb-6 line-clamp-2 leading-relaxed"
                  style={{ color: 'var(--text-secondary)' }}
                >
                  {module.description}
                </p>

                {/* 查看详情按钮 */}
                <div
                  className="flex items-center gap-2 transition-colors"
                  style={{ color: 'var(--text-muted)' }}
                >
                  <span className="text-sm font-medium group-hover:text-[var(--accent-primary)]">查看详情</span>
                  <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
                </div>
              </div>
            </Link>
          ))}
        </div>

        {/* 底部统计信息 */}
        <div
          className="mt-16 glass-card p-8"
          style={{
            border: '1px solid var(--border-color)',
            boxShadow: 'var(--shadow-md)',
          }}
        >
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 text-center">
            <div>
              <p
                className="text-3xl font-bold mb-2"
                style={{ color: 'var(--text-primary)' }}
              >
                6
              </p>
              <p
                className="text-sm"
                style={{ color: 'var(--text-secondary)' }}
              >
                功能模块
              </p>
            </div>
            <div>
              <p
                className="text-3xl font-bold mb-2"
                style={{ color: 'var(--text-primary)' }}
              >
                云同步
              </p>
              <p
                className="text-sm"
                style={{ color: 'var(--text-secondary)' }}
              >
                WebDAV 支持
              </p>
            </div>
            <div>
              <p
                className="text-3xl font-bold mb-2"
                style={{ color: 'var(--text-primary)' }}
              >
                完整
              </p>
              <p
                className="text-sm"
                style={{ color: 'var(--text-secondary)' }}
              >
                功能覆盖
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
