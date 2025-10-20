'use client';

import Link from 'next/link';
import { Zap, Server, Users, Puzzle, Terminal, ArrowRight, Home } from 'lucide-react';

export default function QwenPage() {
  const subModules = [
    {
      title: 'MCP 服务器',
      description: 'Qwen MCP 服务器配置（STDIO + HTTP + SSE）',
      icon: <Server className="w-8 h-8" />,
      href: '/qwen/mcp',
      gradient: 'from-green-500 to-teal-500'
    },
    {
      title: 'Agents',
      description: 'AI Agent 配置和管理',
      icon: <Users className="w-8 h-8" />,
      href: '/qwen/agents',
      gradient: 'from-teal-500 to-cyan-500'
    },
    {
      title: '插件管理',
      description: '插件启用和配置',
      icon: <Puzzle className="w-8 h-8" />,
      href: '/qwen/plugins',
      gradient: 'from-cyan-500 to-blue-500'
    },
    {
      title: 'Slash Commands',
      description: '自定义命令管理',
      icon: <Terminal className="w-8 h-8" />,
      href: '/qwen/slash-commands',
      gradient: 'from-blue-500 to-green-500'
    },
  ];

  return (
    <div className="min-h-screen relative" style={{ background: 'var(--bg-primary)' }}>
      {/* 🎨 动态背景装饰 - 液态玻璃风格 */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none -z-10">
        <div
          className="absolute top-20 right-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
          style={{ background: 'linear-gradient(135deg, #10b981 0%, #14b8a6 100%)' }}
        />
        <div
          className="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
          style={{
            background: 'linear-gradient(135deg, #059669 0%, #0d9488 100%)',
            animationDelay: '1s'
          }}
        />
        <div
          className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
          style={{
            background: 'linear-gradient(135deg, #14b8a6 0%, #06b6d4 100%)',
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
              style={{ background: 'linear-gradient(135deg, #10b981, #14b8a6)' }}
            >
              <Zap className="w-12 h-12 text-white" />
            </div>
            <div>
              <h1
                className="text-5xl font-bold bg-gradient-to-r bg-clip-text text-transparent"
                style={{
                  backgroundImage: 'linear-gradient(to right, #10b981, #14b8a6, #10b981)'
                }}
              >
                Qwen
              </h1>
              <p
                className="text-xl mt-2"
                style={{ color: 'var(--text-primary)' }}
              >
                阿里通义千问配置中心
              </p>
            </div>
          </div>
          <p
            className="max-w-2xl"
            style={{ color: 'var(--text-secondary)' }}
          >
            管理 Qwen MCP 服务器、Agents、插件和自定义命令,支持多种传输协议。
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
                  className="text-2xl font-bold mb-3 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text group-hover:from-green-400 group-hover:to-teal-400 transition-all"
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
                  className="flex items-center gap-2 group-hover:text-green-400 transition-colors"
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
