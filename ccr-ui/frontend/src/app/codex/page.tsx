'use client';

import Link from 'next/link';
import { Settings, Server, Users, Puzzle, Terminal, ArrowRight, Home } from 'lucide-react';

export default function CodexPage() {
  const subModules = [
    {
      title: 'MCP 服务器',
      description: 'Codex MCP 服务器配置管理（STDIO + HTTP）',
      icon: <Server className="w-8 h-8" />,
      href: '/codex/mcp',
      gradient: 'from-purple-500 to-pink-500'
    },
    {
      title: 'Profiles',
      description: 'Codex Profile 配置和管理',
      icon: <Users className="w-8 h-8" />,
      href: '/codex/profiles',
      gradient: 'from-blue-500 to-cyan-500'
    },
    {
      title: '基础配置',
      description: 'Model、Approval Policy、Sandbox 等基础设置',
      icon: <Settings className="w-8 h-8" />,
      href: '/codex/config',
      gradient: 'from-orange-500 to-red-500',
      badge: '配置中心'
    },
  ];

  return (
    <div className="min-h-screen relative" style={{ background: 'var(--bg-primary)' }}>
      {/* 🎨 动态背景装饰 - 液态玻璃风格 */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none -z-10">
        <div
          className="absolute top-20 right-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
          style={{ background: 'linear-gradient(135deg, #8b5cf6 0%, #ec4899 100%)' }}
        />
        <div
          className="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
          style={{
            background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)',
            animationDelay: '1s'
          }}
        />
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
          style={{
            background: 'linear-gradient(135deg, #ec4899 0%, #f59e0b 100%)',
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
              style={{ background: 'linear-gradient(135deg, #8b5cf6, #ec4899)' }}
            >
              <Settings className="w-12 h-12 text-white" />
            </div>
            <div>
              <h1
                className="text-5xl font-bold bg-gradient-to-r bg-clip-text text-transparent"
                style={{
                  backgroundImage: 'linear-gradient(to right, #8b5cf6, #ec4899, #8b5cf6)'
                }}
              >
                Codex
              </h1>
              <p
                className="text-xl mt-2"
                style={{ color: 'var(--text-primary)' }}
              >
                AI 编程助手配置中心
              </p>
            </div>
          </div>
          <p
            className="max-w-2xl"
            style={{ color: 'var(--text-secondary)' }}
          >
            管理 Codex MCP 服务器、Profiles 和基础配置，支持 STDIO 和 HTTP 双协议。
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
                <div className="flex items-start justify-between mb-4">
                  <div className={`inline-flex p-4 rounded-xl bg-gradient-to-br ${module.gradient}`}>
                    <div className="text-white">{module.icon}</div>
                  </div>
                  {module.badge && (
                    <span
                      className="px-3 py-1 text-xs font-semibold rounded-full"
                      style={{
                        background: 'rgba(139, 92, 246, 0.2)',
                        color: '#c084fc',
                        border: '1px solid rgba(139, 92, 246, 0.3)'
                      }}
                    >
                      {module.badge}
                    </span>
                  )}
                </div>
                <h3
                  className="text-2xl font-bold mb-3 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text group-hover:from-purple-400 group-hover:to-pink-400 transition-all"
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
                  className="flex items-center gap-2 group-hover:text-purple-400 transition-colors"
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
