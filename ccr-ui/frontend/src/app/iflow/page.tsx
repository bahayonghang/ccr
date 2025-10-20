'use client';

import Link from 'next/link';
import { Activity, Server, Users, Puzzle, Terminal, ArrowRight, Home } from 'lucide-react';

export default function IflowPage() {
  const subModules = [
    {
      title: 'MCP 服务器',
      description: 'IFLOW MCP 服务器配置管理',
      icon: <Server className="w-8 h-8" />,
      href: '/iflow/mcp',
      gradient: 'from-indigo-500 to-blue-500'
    },
    {
      title: 'Agents',
      description: 'AI Agent 配置和管理',
      icon: <Users className="w-8 h-8" />,
      href: '/iflow/agents',
      gradient: 'from-blue-500 to-purple-500'
    },
    {
      title: '插件管理',
      description: '插件启用和配置',
      icon: <Puzzle className="w-8 h-8" />,
      href: '/iflow/plugins',
      gradient: 'from-purple-500 to-pink-500'
    },
    {
      title: 'Slash Commands',
      description: '自定义命令管理',
      icon: <Terminal className="w-8 h-8" />,
      href: '/iflow/slash-commands',
      gradient: 'from-pink-500 to-indigo-500'
    },
  ];

  return (
    <div className="min-h-screen relative" style={{ background: 'var(--bg-primary)' }}>
      {/* 🎨 动态背景装饰 - 液态玻璃风格 */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none -z-10">
        <div
          className="absolute top-20 right-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
          style={{ background: 'linear-gradient(135deg, #6366f1 0%, #3b82f6 100%)' }}
        />
        <div
          className="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
          style={{
            background: 'linear-gradient(135deg, #4f46e5 0%, #2563eb 100%)',
            animationDelay: '1s'
          }}
        />
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
          style={{
            background: 'linear-gradient(135deg, #3b82f6 0%, #6366f1 100%)',
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
              style={{ background: 'linear-gradient(135deg, #6366f1, #3b82f6)' }}
            >
              <Activity className="w-12 h-12 text-white" />
            </div>
            <div>
              <h1
                className="text-5xl font-bold bg-gradient-to-r bg-clip-text text-transparent"
                style={{
                  backgroundImage: 'linear-gradient(to right, #6366f1, #3b82f6, #6366f1)'
                }}
              >
                IFLOW
              </h1>
              <p
                className="text-xl mt-2"
                style={{ color: 'var(--text-primary)' }}
              >
                内部工作流配置中心
              </p>
            </div>
          </div>
          <p
            className="max-w-2xl"
            style={{ color: 'var(--text-secondary)' }}
          >
            管理 IFLOW MCP 服务器、Agents、插件和工作流自动化配置。
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {subModules.map((module, index) => (
            <Link
              key={module.href}
              href={module.href}
              className="group relative"
              style={{ animationDelay: `${index * 100}ms` }}
            >
              <div
                className="glass-card p-8 h-full transition-all duration-300 hover:scale-105 hover:-translate-y-2"
                style={{
                  border: '1px solid var(--border-color)',
                }}
              >
                <div className={`absolute inset-0 rounded-xl bg-gradient-to-br ${module.gradient} opacity-0 group-hover:opacity-20 transition-opacity duration-300`}></div>
                <div className={`inline-flex p-4 rounded-xl bg-gradient-to-br ${module.gradient} mb-4`}>
                  <div className="text-white">{module.icon}</div>
                </div>
                <h3
                  className="text-2xl font-bold mb-3 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text group-hover:from-indigo-400 group-hover:to-blue-400 transition-all"
                  style={{ color: 'var(--text-primary)' }}
                >
                  {module.title}
                </h3>
                <p
                  className="text-sm mb-6 line-clamp-2"
                  style={{ color: 'var(--text-muted)' }}
                >
                  {module.description}
                </p>
                <div
                  className="flex items-center gap-2 group-hover:text-indigo-400 transition-colors"
                  style={{ color: 'var(--text-muted)' }}
                >
                  <span className="text-sm font-medium">查看详情</span>
                  <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
                </div>
              </div>
            </Link>
          ))}
        </div>
      </div>
    </div>
  );
}
