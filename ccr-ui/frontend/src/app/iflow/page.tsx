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
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-indigo-900 to-slate-900">
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-1/4 -left-1/4 w-96 h-96 bg-indigo-500/30 rounded-full blur-3xl animate-pulse"></div>
        <div className="absolute bottom-1/4 -right-1/4 w-96 h-96 bg-blue-500/30 rounded-full blur-3xl animate-pulse" style={{ animationDelay: '1s' }}></div>
      </div>

      <div className="relative z-10 container mx-auto px-4 py-12">
        <Link href="/" className="inline-flex items-center gap-2 mb-8 text-gray-400 hover:text-indigo-400 transition-colors group">
          <Home className="w-5 h-5 group-hover:-translate-x-1 transition-transform" />
          <span>返回首页</span>
        </Link>

        <div className="mb-16">
          <div className="flex items-center gap-4 mb-4">
            <div className="p-4 bg-gradient-to-br from-indigo-500 to-blue-500 rounded-2xl">
              <Activity className="w-12 h-12 text-white" />
            </div>
            <div>
              <h1 className="text-5xl font-bold bg-gradient-to-r from-indigo-400 via-blue-400 to-indigo-400 bg-clip-text text-transparent">
                IFLOW
              </h1>
              <p className="text-xl text-gray-300 mt-2">内部工作流配置中心</p>
            </div>
          </div>
          <p className="text-gray-400 max-w-2xl">
            管理 IFLOW MCP 服务器、Agents、插件和工作流自动化配置。
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {subModules.map((module, index) => (
            <Link key={module.href} href={module.href} className="group relative" style={{ animationDelay: `${index * 100}ms` }}>
              <div className="glass-card p-8 h-full border border-gray-700/50 hover:border-gray-500/50 transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                <div className={`absolute inset-0 rounded-xl bg-gradient-to-br ${module.gradient} opacity-0 group-hover:opacity-20 transition-opacity duration-300`}></div>
                <div className={`inline-flex p-4 rounded-xl bg-gradient-to-br ${module.gradient} mb-4`}>
                  <div className="text-white">{module.icon}</div>
                </div>
                <h3 className="text-2xl font-bold text-white mb-3 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text group-hover:from-indigo-400 group-hover:to-blue-400 transition-all">
                  {module.title}
                </h3>
                <p className="text-gray-400 text-sm mb-6 line-clamp-2">{module.description}</p>
                <div className="flex items-center gap-2 text-gray-500 group-hover:text-indigo-400 transition-colors">
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
