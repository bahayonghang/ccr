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
  HardDrive,
  TrendingUp
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { getSystemInfo, getVersion } from '@/lib/api/client';

interface ModuleCard {
  title: string;
  description: string;
  icon: React.ReactNode;
  href: string;
  color: string;
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
      icon: <Code2 className="w-7 h-7" />,
      href: '/claude-code',
      color: '#6366f1',
      stats: '核心模块'
    },
    {
      title: 'Codex',
      description: 'MCP 服务器、Profiles、基础配置管理',
      icon: <Settings className="w-7 h-7" />,
      href: '/codex',
      color: '#8b5cf6',
      stats: 'AI 编程助手'
    },
    {
      title: 'Gemini CLI',
      description: 'Google Gemini 配置管理和工具集成',
      icon: <Sparkles className="w-7 h-7" />,
      href: '/gemini-cli',
      color: '#f59e0b',
      stats: 'Google AI'
    },
    {
      title: 'Qwen',
      description: '阿里通义千问配置管理和服务集成',
      icon: <Zap className="w-7 h-7" />,
      href: '/qwen',
      color: '#10b981',
      stats: '国产大模型'
    },
    {
      title: 'IFLOW',
      description: '内部工作流配置和自动化管理',
      icon: <Activity className="w-7 h-7" />,
      href: '/iflow',
      color: '#3b82f6',
      stats: '工作流引擎'
    },
    {
      title: '命令执行中心',
      description: '统一的 CLI 命令执行和管理界面',
      icon: <Terminal className="w-7 h-7" />,
      href: '/commands',
      color: '#64748b',
      stats: '多 CLI 支持'
    },
    {
      title: '配置转换器',
      description: '跨 CLI 工具的配置格式转换',
      icon: <TrendingUp className="w-7 h-7" />,
      href: '/converter',
      color: '#f97316',
      stats: '格式互转'
    },
    {
      title: '云同步',
      description: 'WebDAV 云端配置同步和备份',
      icon: <Cloud className="w-7 h-7" />,
      href: '/sync',
      color: '#06b6d4',
      stats: '自动备份'
    },
  ];

  return (
    <div className="min-h-screen relative">
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

      <div className="relative z-10 container mx-auto px-6 py-16">
        {/* 🌟 头部区域 - Material Design */}
        <div className="text-center mb-16 animate-fade-in">
          <div className="inline-block mb-6">
            <div className="flex items-center justify-center w-20 h-20 rounded-3xl glass-card mb-6 mx-auto">
              <Code2 className="w-10 h-10" style={{ color: '#6366f1' }} />
            </div>
          </div>

          <h1 className="text-6xl md:text-7xl font-bold mb-6 bg-gradient-to-r from-[#6366f1] via-[#8b5cf6] to-[#ec4899] bg-clip-text text-transparent">
            CCR UI
          </h1>

          <p className="text-2xl font-medium mb-3" style={{ color: 'var(--text-primary)' }}>
            Claude Code 配置管理中心
          </p>

          <p className="text-base max-w-2xl mx-auto mb-8" style={{ color: 'var(--text-secondary)' }}>
            现代化的多 CLI 工具配置管理解决方案，支持 Claude、Codex、Gemini 等多种 AI 平台
          </p>

          {version && (
            <div
              className="inline-flex items-center gap-2 px-5 py-2.5 glass-card text-sm font-semibold animate-slide-in-right"
              style={{ color: 'var(--accent-primary)' }}
            >
              <Sparkles className="w-4 h-4" />
              <span>v{version}</span>
            </div>
          )}
        </div>

        {/* 📊 系统状态卡片 - Material Design */}
        {systemInfo && (
          <div className="mb-16 grid grid-cols-1 md:grid-cols-3 gap-6">
            <div
              className="glass-card p-6 hover:scale-105 transition-all duration-300 cursor-pointer group"
              style={{ animationDelay: '0.1s' }}
            >
              <div className="flex items-center gap-4">
                <div
                  className="p-4 rounded-2xl"
                  style={{ background: 'rgba(99, 102, 241, 0.1)' }}
                >
                  <Cpu className="w-7 h-7" style={{ color: '#6366f1' }} />
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium mb-1" style={{ color: 'var(--text-muted)' }}>
                    CPU 使用率
                  </p>
                  <p className="text-3xl font-bold" style={{ color: 'var(--text-primary)' }}>
                    {systemInfo.cpu_usage?.toFixed(1) || '0.0'}%
                  </p>
                </div>
              </div>
            </div>

            <div
              className="glass-card p-6 hover:scale-105 transition-all duration-300 cursor-pointer group"
              style={{ animationDelay: '0.2s' }}
            >
              <div className="flex items-center gap-4">
                <div
                  className="p-4 rounded-2xl"
                  style={{ background: 'rgba(139, 92, 246, 0.1)' }}
                >
                  <HardDrive className="w-7 h-7" style={{ color: '#8b5cf6' }} />
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium mb-1" style={{ color: 'var(--text-muted)' }}>
                    内存使用
                  </p>
                  <p className="text-3xl font-bold" style={{ color: 'var(--text-primary)' }}>
                    {systemInfo.memory_usage_percent?.toFixed(1) || '0.0'}%
                  </p>
                </div>
              </div>
            </div>

            <div
              className="glass-card p-6 hover:scale-105 transition-all duration-300 cursor-pointer group"
              style={{ animationDelay: '0.3s' }}
            >
              <div className="flex items-center gap-4">
                <div
                  className="p-4 rounded-2xl"
                  style={{ background: 'rgba(16, 185, 129, 0.1)' }}
                >
                  <Activity className="w-7 h-7" style={{ color: '#10b981' }} />
                </div>
                <div className="flex-1">
                  <p className="text-sm font-medium mb-1" style={{ color: 'var(--text-muted)' }}>
                    系统平台
                  </p>
                  <p className="text-lg font-bold truncate" style={{ color: 'var(--text-primary)' }}>
                    {systemInfo.os} {systemInfo.os_version}
                  </p>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* 🎯 功能模块卡片网格 - Material Design */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {modules.map((module, index) => (
            <Link
              key={module.href}
              href={module.href}
              className="group block"
              style={{ animationDelay: `${index * 0.05}s` }}
            >
              <div className="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
                {/* 图标区域 */}
                <div className="mb-5">
                  <div
                    className="inline-flex p-4 rounded-2xl"
                    style={{ background: `${module.color}15` }}
                  >
                    <div style={{ color: module.color }}>
                      {module.icon}
                    </div>
                  </div>
                </div>

                {/* 标题和描述 */}
                <h3
                  className="text-xl font-bold mb-3 group-hover:text-transparent group-hover:bg-gradient-to-r group-hover:bg-clip-text group-hover:from-[#6366f1] group-hover:to-[#8b5cf6] transition-all"
                  style={{ color: 'var(--text-primary)' }}
                >
                  {module.title}
                </h3>

                <p
                  className="text-sm mb-4 leading-relaxed line-clamp-2"
                  style={{ color: 'var(--text-secondary)' }}
                >
                  {module.description}
                </p>

                {/* 底部信息 */}
                <div className="flex items-center justify-between mt-auto">
                  {module.stats && (
                    <span
                      className="text-xs font-semibold px-3 py-1.5 rounded-full"
                      style={{
                        background: 'var(--bg-secondary)',
                        color: 'var(--text-muted)',
                        border: '1px solid var(--border-color)'
                      }}
                    >
                      {module.stats}
                    </span>
                  )}
                  <ArrowRight
                    className="w-5 h-5 ml-auto group-hover:translate-x-1 transition-transform"
                    style={{ color: module.color }}
                  />
                </div>
              </div>
            </Link>
          ))}
        </div>

        {/* 🌈 底部信息 */}
        <div className="mt-20 text-center">
          <p className="text-sm mb-2" style={{ color: 'var(--text-muted)' }}>
            现代化的配置管理解决方案 · 支持多种 AI CLI 工具
          </p>
          <p className="text-xs" style={{ color: 'var(--text-muted)' }}>
            Claude Code • Codex • Gemini • Qwen • IFLOW
          </p>
        </div>
      </div>
    </div>
  );
}
