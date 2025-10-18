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
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-1/4 -left-1/4 w-96 h-96 bg-purple-500/30 rounded-full blur-3xl animate-pulse"></div>
        <div className="absolute bottom-1/4 -right-1/4 w-96 h-96 bg-pink-500/30 rounded-full blur-3xl animate-pulse" style={{ animationDelay: '1s' }}></div>
      </div>

      <div className="relative z-10 container mx-auto px-4 py-12">
        <Link href="/" className="inline-flex items-center gap-2 mb-8 text-gray-400 hover:text-purple-400 transition-colors group">
          <Home className="w-5 h-5 group-hover:-translate-x-1 transition-transform" />
          <span>返回首页</span>
        </Link>

        <div className="mb-16">
          <div className="flex items-center gap-4 mb-4">
            <div className="p-4 bg-gradient-to-br from-purple-500 to-pink-500 rounded-2xl">
              <Settings className="w-12 h-12 text-white" />
            </div>
            <div>
              <h1 className="text-5xl font-bold bg-gradient-to-r from-purple-400 via-pink-400 to-purple-400 bg-clip-text text-transparent">
                Codex
              </h1>
              <p className="text-xl text-gray-300 mt-2">AI 编程助手配置中心</p>
            </div>
          </div>
          <p className="text-gray-400 max-w-2xl">
            管理 Codex MCP 服务器、Profiles 和基础配置，支持 STDIO 和 HTTP 双协议。
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {subModules.map((module, index) => (
            <Link key={module.href} href={module.href} className="group relative" style={{ animationDelay: `${index * 100}ms` }}>
              <div className="glass-card p-8 h-full border border-gray-700/50 hover:border-gray-500/50 transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                <div className={`absolute inset-0 rounded-xl bg-gradient-to-br ${module.gradient} opacity-0 group-hover:opacity-20 transition-opacity duration-300`}></div>
                <div className="flex items-start justify-between mb-4">
                  <div className={`inline-flex p-4 rounded-xl bg-gradient-to-br ${module.gradient}`}>
                    <div className="text-white">{module.icon}</div>
                  </div>
                  {module.badge && (
                    <span className="px-3 py-1 text-xs font-semibold bg-purple-500/20 text-purple-300 rounded-full border border-purple-500/30">
                      {module.badge}
                    </span>
                  )}
                </div>
                <h3 className="text-2xl font-bold text-white mb-3 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text group-hover:from-purple-400 group-hover:to-pink-400 transition-all">
                  {module.title}
                </h3>
                <p className="text-gray-400 text-sm mb-6 line-clamp-2">{module.description}</p>
                <div className="flex items-center gap-2 text-gray-500 group-hover:text-purple-400 transition-colors">
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
