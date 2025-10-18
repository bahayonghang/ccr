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
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900">
      {/* 动态背景效果 */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-1/4 -left-1/4 w-96 h-96 bg-blue-500/30 rounded-full blur-3xl animate-pulse"></div>
        <div className="absolute bottom-1/4 -right-1/4 w-96 h-96 bg-purple-500/30 rounded-full blur-3xl animate-pulse" style={{ animationDelay: '1s' }}></div>
      </div>

      <div className="relative z-10 container mx-auto px-4 py-12">
        {/* 返回首页按钮 */}
        <Link
          href="/"
          className="inline-flex items-center gap-2 mb-8 text-gray-400 hover:text-blue-400 transition-colors group"
        >
          <Home className="w-5 h-5 group-hover:-translate-x-1 transition-transform" />
          <span>返回首页</span>
        </Link>

        {/* 头部区域 */}
        <div className="mb-16">
          <div className="flex items-center gap-4 mb-4">
            <div className="p-4 bg-gradient-to-br from-blue-500 to-cyan-500 rounded-2xl">
              <Code2 className="w-12 h-12 text-white" />
            </div>
            <div>
              <h1 className="text-5xl font-bold bg-gradient-to-r from-blue-400 via-cyan-400 to-blue-400 bg-clip-text text-transparent">
                Claude Code
              </h1>
              <p className="text-xl text-gray-300 mt-2">
                Claude Code 配置管理中心
              </p>
            </div>
          </div>
          <p className="text-gray-400 max-w-2xl">
            管理您的 Claude Code 配置、MCP 服务器、Agents、插件和自定义命令。支持云端同步和历史记录追踪。
          </p>
        </div>

        {/* 功能模块网格 */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {subModules.map((module, index) => (
            <Link
              key={module.href}
              href={module.href}
              className="group relative"
              style={{ animationDelay: `${index * 100}ms` }}
            >
              <div className="glass-card p-8 h-full border border-gray-700/50 hover:border-gray-500/50 transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                {/* 渐变边框效果 */}
                <div className={`absolute inset-0 rounded-xl bg-gradient-to-br ${module.gradient} opacity-0 group-hover:opacity-20 transition-opacity duration-300`}></div>

                {/* 图标和徽章 */}
                <div className="flex items-start justify-between mb-4">
                  <div className={`inline-flex p-4 rounded-xl bg-gradient-to-br ${module.gradient}`}>
                    <div className="text-white">
                      {module.icon}
                    </div>
                  </div>
                  {module.badge && (
                    <span className="px-3 py-1 text-xs font-semibold bg-blue-500/20 text-blue-300 rounded-full border border-blue-500/30">
                      {module.badge}
                    </span>
                  )}
                </div>

                {/* 标题和描述 */}
                <h3 className="text-2xl font-bold text-white mb-3 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text group-hover:from-blue-400 group-hover:to-cyan-400 transition-all">
                  {module.title}
                </h3>
                <p className="text-gray-400 text-sm mb-6 line-clamp-2">
                  {module.description}
                </p>

                {/* 查看详情按钮 */}
                <div className="flex items-center gap-2 text-gray-500 group-hover:text-blue-400 transition-colors">
                  <span className="text-sm font-medium">查看详情</span>
                  <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
                </div>
              </div>
            </Link>
          ))}
        </div>

        {/* 底部统计信息（可选） */}
        <div className="mt-16 glass-card p-8 border border-gray-700/50">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 text-center">
            <div>
              <p className="text-3xl font-bold text-white mb-2">6</p>
              <p className="text-gray-400 text-sm">功能模块</p>
            </div>
            <div>
              <p className="text-3xl font-bold text-white mb-2">云同步</p>
              <p className="text-gray-400 text-sm">WebDAV 支持</p>
            </div>
            <div>
              <p className="text-3xl font-bold text-white mb-2">完整</p>
              <p className="text-gray-400 text-sm">功能覆盖</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
