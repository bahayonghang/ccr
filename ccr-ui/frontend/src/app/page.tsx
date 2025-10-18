'use client';

import Link from 'next/link';
import {
  Settings,
  Cloud,
  Terminal,
  Zap,
  Code2,
  Sparkles,
  ArrowRight,
  Activity,
  Cpu,
  HardDrive
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { getSystemInfo, getVersion } from '@/lib/api/client';

interface ModuleCard {
  title: string;
  description: string;
  icon: React.ReactNode;
  href: string;
  gradient: string;
  stats?: string;
}

export default function HomePage() {
  const [systemInfo, setSystemInfo] = useState<any>(null);
  const [version, setVersion] = useState<string>('');

  useEffect(() => {
    const loadData = async () => {
      try {
        const [sysInfo, versionInfo] = await Promise.all([
          getSystemInfo(),
          getVersion()
        ]);
        setSystemInfo(sysInfo);
        setVersion(versionInfo.current_version);
      } catch (error) {
        console.error('Failed to load dashboard data:', error);
      }
    };
    loadData();
  }, []);

  const modules: ModuleCard[] = [
    {
      title: 'Claude Code',
      description: '配置管理、云同步、MCP 服务器、Agents、插件',
      icon: <Code2 className="w-8 h-8" />,
      href: '/claude-code',
      gradient: 'from-blue-500 to-cyan-500',
      stats: '核心模块'
    },
    {
      title: 'Codex',
      description: 'MCP 服务器、Profiles、基础配置管理',
      icon: <Settings className="w-8 h-8" />,
      href: '/codex',
      gradient: 'from-purple-500 to-pink-500',
      stats: 'AI 编程助手'
    },
    {
      title: 'Gemini CLI',
      description: 'Google Gemini 配置管理和工具集成',
      icon: <Sparkles className="w-8 h-8" />,
      href: '/gemini-cli',
      gradient: 'from-orange-500 to-red-500',
      stats: 'Google AI'
    },
    {
      title: 'Qwen',
      description: '阿里通义千问配置管理和服务集成',
      icon: <Zap className="w-8 h-8" />,
      href: '/qwen',
      gradient: 'from-green-500 to-teal-500',
      stats: '国产大模型'
    },
    {
      title: 'IFLOW',
      description: '内部工作流配置和自动化管理',
      icon: <Activity className="w-8 h-8" />,
      href: '/iflow',
      gradient: 'from-indigo-500 to-blue-500',
      stats: '工作流引擎'
    },
    {
      title: '命令执行中心',
      description: '统一的 CLI 命令执行和管理界面',
      icon: <Terminal className="w-8 h-8" />,
      href: '/commands',
      gradient: 'from-gray-700 to-gray-900',
      stats: '多 CLI 支持'
    },
    {
      title: '配置转换器',
      description: '跨 CLI 工具的配置格式转换',
      icon: <Code2 className="w-8 h-8" />,
      href: '/converter',
      gradient: 'from-yellow-500 to-orange-500',
      stats: '格式互转'
    },
    {
      title: '云同步',
      description: 'WebDAV 云端配置同步和备份',
      icon: <Cloud className="w-8 h-8" />,
      href: '/sync',
      gradient: 'from-cyan-500 to-blue-500',
      stats: '自动备份'
    },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900">
      {/* 动态背景效果 */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="absolute top-1/4 -left-1/4 w-96 h-96 bg-blue-500/30 rounded-full blur-3xl animate-pulse"></div>
        <div className="absolute bottom-1/4 -right-1/4 w-96 h-96 bg-purple-500/30 rounded-full blur-3xl animate-pulse delay-1000"></div>
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-cyan-500/20 rounded-full blur-3xl animate-pulse delay-2000"></div>
      </div>

      <div className="relative z-10 container mx-auto px-4 py-12">
        {/* 头部区域 */}
        <div className="text-center mb-16">
          <h1 className="text-6xl font-bold mb-4 bg-gradient-to-r from-blue-400 via-purple-400 to-cyan-400 bg-clip-text text-transparent">
            CCR UI
          </h1>
          <p className="text-xl text-gray-300 mb-2">
            Claude Code 配置管理中心
          </p>
          <p className="text-gray-400">
            现代化的多 CLI 工具配置管理解决方案
          </p>
          {version && (
            <div className="mt-4 inline-block px-4 py-2 bg-blue-500/20 backdrop-blur-sm border border-blue-500/30 rounded-full text-blue-300 text-sm">
              v{version}
            </div>
          )}
        </div>

        {/* 系统状态卡片 */}
        {systemInfo && (
          <div className="mb-12 grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="glass-card p-6 border border-blue-500/30 hover:border-blue-500/50 transition-all">
              <div className="flex items-center gap-4">
                <div className="p-3 bg-blue-500/20 rounded-lg">
                  <Cpu className="w-6 h-6 text-blue-400" />
                </div>
                <div>
                  <p className="text-gray-400 text-sm">CPU 使用率</p>
                  <p className="text-2xl font-bold text-white">
                    {systemInfo.cpu_usage?.toFixed(1) || '0.0'}%
                  </p>
                </div>
              </div>
            </div>

            <div className="glass-card p-6 border border-purple-500/30 hover:border-purple-500/50 transition-all">
              <div className="flex items-center gap-4">
                <div className="p-3 bg-purple-500/20 rounded-lg">
                  <HardDrive className="w-6 h-6 text-purple-400" />
                </div>
                <div>
                  <p className="text-gray-400 text-sm">内存使用</p>
                  <p className="text-2xl font-bold text-white">
                    {systemInfo.memory_usage_percent?.toFixed(1) || '0.0'}%
                  </p>
                </div>
              </div>
            </div>

            <div className="glass-card p-6 border border-cyan-500/30 hover:border-cyan-500/50 transition-all">
              <div className="flex items-center gap-4">
                <div className="p-3 bg-cyan-500/20 rounded-lg">
                  <Activity className="w-6 h-6 text-cyan-400" />
                </div>
                <div>
                  <p className="text-gray-400 text-sm">系统</p>
                  <p className="text-lg font-bold text-white truncate">
                    {systemInfo.os} {systemInfo.os_version}
                  </p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* 功能模块卡片网格 */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {modules.map((module, index) => (
            <Link
              key={module.href}
              href={module.href}
              className="group relative"
              style={{ animationDelay: `${index * 100}ms` }}
            >
              <div className="glass-card p-6 h-full border border-gray-700/50 hover:border-gray-500/50 transition-all duration-300 hover:scale-105 hover:-translate-y-2">
                {/* 渐变边框效果 */}
                <div className={`absolute inset-0 rounded-xl bg-gradient-to-br ${module.gradient} opacity-0 group-hover:opacity-20 transition-opacity duration-300`}></div>

                {/* 图标 */}
                <div className={`inline-flex p-3 rounded-lg bg-gradient-to-br ${module.gradient} mb-4`}>
                  <div className="text-white">
                    {module.icon}
                  </div>
                </div>

                {/* 标题和描述 */}
                <h3 className="text-xl font-bold text-white mb-2 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text group-hover:from-blue-400 group-hover:to-cyan-400 transition-all">
                  {module.title}
                </h3>
                <p className="text-gray-400 text-sm mb-4 line-clamp-2">
                  {module.description}
                </p>

                {/* 统计信息 */}
                {module.stats && (
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-gray-500 bg-gray-800/50 px-2 py-1 rounded">
                      {module.stats}
                    </span>
                    <ArrowRight className="w-5 h-5 text-gray-600 group-hover:text-blue-400 group-hover:translate-x-1 transition-all" />
                  </div>
                )}
              </div>
            </Link>
          ))}
        </div>

        {/* 底部信息 */}
        <div className="mt-16 text-center text-gray-500 text-sm">
          <p>现代化的配置管理解决方案 • 支持多种 AI CLI 工具</p>
          <p className="mt-2">Claude Code • Codex • Gemini • Qwen • IFLOW</p>
        </div>
      </div>
    </div>
  );
}
