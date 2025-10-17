'use client';

import { Activity, Cpu, HardDrive, Clock, Server, ChevronDown, ChevronUp } from 'lucide-react';
import { useEffect, useState } from 'react';
import { getSystemInfo } from '@/lib/api/client';
import type { SystemInfo } from '@/lib/types';
import VersionManager from './VersionManager';

interface StatusHeaderProps {
  currentConfig: string;
  totalConfigs: number;
  historyCount: number;
}

export default function StatusHeader({
  currentConfig,
  totalConfigs,
  historyCount,
}: StatusHeaderProps) {
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);

  // 初始状态统一为 false，避免水合错误
  const [isCollapsed, setIsCollapsed] = useState(false);

  // 当折叠状态改变时，保存到 localStorage
  const toggleCollapsed = () => {
    const newState = !isCollapsed;
    setIsCollapsed(newState);
    if (typeof window !== 'undefined') {
      localStorage.setItem('ccr-status-header-collapsed', String(newState));
    }
  };

  // 组件挂载后从 localStorage 读取折叠状态
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('ccr-status-header-collapsed');
      if (saved === 'true') {
        setIsCollapsed(true);
      }
    }
  }, []);

  useEffect(() => {
    const loadSystemInfo = async () => {
      try {
        const data = await getSystemInfo();
        setSystemInfo(data);
      } catch (err) {
        console.error('Failed to load system info:', err);
      }
    };

    loadSystemInfo();
    const interval = setInterval(loadSystemInfo, 5000);
    return () => clearInterval(interval);
  }, []);

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (days > 0) {
      return `${days}天 ${hours}时`;
    } else if (hours > 0) {
      return `${hours}时 ${minutes}分`;
    } else {
      return `${minutes}分钟`;
    }
  };

  return (
    <div
      className="rounded-xl mb-5 glass-effect overflow-hidden relative"
      style={{
        border: '2px solid var(--accent-primary)',
        boxShadow: '0 0 30px rgba(139, 92, 246, 0.25), 0 8px 32px rgba(0, 0, 0, 0.2)',
        background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.05), rgba(168, 85, 247, 0.03))',
      }}
    >
      {/* 顶部装饰条 */}
      <div
        className="absolute top-0 left-0 w-full h-1"
        style={{
          background: 'linear-gradient(90deg, var(--accent-primary), var(--accent-secondary), var(--accent-primary))',
          opacity: 0.8,
        }}
        aria-hidden="true"
      />

      {/* 折叠按钮和标题栏 */}
      <div
        className="flex items-center justify-between px-5 py-4 cursor-pointer hover:bg-opacity-50 transition-all"
        onClick={toggleCollapsed}
        style={{
          borderBottom: isCollapsed ? 'none' : '1px solid var(--border-color)',
        }}
      >
        <div className="flex items-center gap-3">
          <div
            className="w-2 h-2 rounded-full animate-pulse"
            style={{
              background: 'var(--accent-success)',
              boxShadow: '0 0 10px var(--glow-success)',
            }}
            aria-label="状态：活跃"
          />
          <h2 className="text-base font-bold uppercase tracking-wider" style={{ color: 'var(--text-primary)' }}>
            系统状态面板
          </h2>
          <span
            className="px-2.5 py-0.5 rounded-lg text-xs font-semibold uppercase"
            style={{
              background: 'var(--accent-primary)',
              color: 'white',
            }}
          >
            实时监控
          </span>
        </div>
        <button
          className="p-2 rounded-lg transition-all hover:bg-opacity-20 hover:scale-110"
          style={{
            background: 'rgba(139, 92, 246, 0.1)',
            color: 'var(--accent-primary)',
          }}
          aria-label={isCollapsed ? '展开' : '收起'}
          onClick={(e) => {
            e.stopPropagation();
            toggleCollapsed();
          }}
        >
          {isCollapsed ? <ChevronDown className="w-5 h-5" /> : <ChevronUp className="w-5 h-5" />}
        </button>
      </div>

      {/* 可折叠内容 */}
      <div
        className="transition-all duration-300 ease-in-out"
        style={{
          maxHeight: isCollapsed ? '0' : '1000px',
          opacity: isCollapsed ? 0 : 1,
          overflow: 'hidden',
        }}
      >
        <div className="p-5">
          {/* 第一行：当前配置、统计信息、版本管理 */}
          <div className="grid grid-cols-1 lg:grid-cols-[2fr_1fr_1fr] gap-4 mb-4">
        {/* 当前激活配置 - 占据更大空间 */}
        <div
          className="relative overflow-hidden rounded-lg p-5"
          style={{
            background: 'linear-gradient(135deg, var(--bg-tertiary), var(--bg-secondary))',
            border: '2px solid var(--accent-primary)',
            boxShadow: '0 0 20px rgba(139, 92, 246, 0.3), inset 0 0 20px rgba(139, 92, 246, 0.05)',
          }}
        >
          <div
            className="absolute top-0 left-0 w-full h-0.5 animate-scan"
            style={{
              background: 'linear-gradient(90deg, transparent, var(--accent-primary), transparent)',
            }}
            aria-hidden="true"
          />
          <div className="flex items-center justify-between mb-3">
            <span className="text-xs font-semibold uppercase tracking-wider" style={{ color: 'var(--text-secondary)' }}>
              当前配置
            </span>
            <span
              className="w-2 h-2 rounded-full animate-pulse"
              style={{
                background: 'var(--accent-success)',
                boxShadow: '0 0 10px var(--glow-success)',
              }}
              aria-label="状态：活跃"
            />
          </div>
          <div
            className="text-2xl font-bold uppercase font-mono tracking-wide truncate"
            style={{ color: 'var(--text-primary)' }}
            title={currentConfig}
          >
            {currentConfig || '-'}
          </div>
        </div>

        {/* 统计信息卡片 */}
        <div
          className="rounded-lg p-4"
          style={{
            background: 'var(--bg-tertiary)',
            border: '2px solid var(--accent-secondary)',
            boxShadow: '0 0 15px rgba(168, 85, 247, 0.2), inset 0 0 15px rgba(168, 85, 247, 0.05)',
          }}
        >
          <div className="text-xs font-semibold uppercase tracking-wider mb-3" style={{ color: 'var(--text-secondary)' }}>
            统计信息
          </div>
          <div className="space-y-3">
            <StatItem icon={<Server className="w-4 h-4" />} label="总配置" value={totalConfigs} />
            <StatItem icon={<Activity className="w-4 h-4" />} label="历史记录" value={historyCount} />
          </div>
        </div>

        {/* 版本管理 */}
        <VersionManager />
      </div>

      {/* 第二行：系统信息 */}
      {systemInfo && (
        <div
          className="rounded-lg p-4"
          style={{
            background: 'var(--bg-tertiary)',
            border: '2px solid rgba(34, 197, 94, 0.5)',
            boxShadow: '0 0 15px rgba(34, 197, 94, 0.15), inset 0 0 15px rgba(34, 197, 94, 0.05)',
          }}
        >
          <div className="flex items-center justify-between mb-3">
            <div className="text-xs font-semibold uppercase tracking-wider" style={{ color: 'var(--text-secondary)' }}>
              系统信息
            </div>
            <div className="text-xs" style={{ color: 'var(--text-muted)' }}>
              每5秒自动刷新
            </div>
          </div>
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
            <SystemMetric
              icon={<Server className="w-4 h-4" />}
              label="主机"
              value={systemInfo.hostname}
            />
            <SystemMetric
              icon={<Cpu className="w-4 h-4" />}
              label="CPU"
              value={`${systemInfo.cpu_cores} 核 · ${Math.round(systemInfo.cpu_usage)}%`}
              progress={systemInfo.cpu_usage}
            />
            <SystemMetric
              icon={<HardDrive className="w-4 h-4" />}
              label="内存"
              value={`${systemInfo.used_memory_gb.toFixed(1)}/${systemInfo.total_memory_gb.toFixed(1)} GB`}
              progress={systemInfo.memory_usage_percent}
            />
            <SystemMetric
              icon={<Activity className="w-4 h-4" />}
              label="系统"
              value={systemInfo.os}
            />
            <SystemMetric
              icon={<Clock className="w-4 h-4" />}
              label="运行时间"
              value={formatUptime(systemInfo.uptime_seconds)}
            />
          </div>
        </div>
      )}
        </div>
      </div>
    </div>
  );
}

function StatItem({ icon, label, value }: { icon: React.ReactNode; label: string; value: number }) {
  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-2" style={{ color: 'var(--text-muted)' }}>
        {icon}
        <span className="text-xs font-medium">{label}</span>
      </div>
      <div className="text-xl font-bold font-mono" style={{ color: 'var(--accent-primary)' }}>
        {value}
      </div>
    </div>
  );
}

function SystemMetric({
  icon,
  label,
  value,
  progress,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  progress?: number;
}) {
  // 根据进度确定边框颜色
  const getBorderColor = () => {
    if (progress === undefined) {
      return 'rgba(139, 92, 246, 0.4)'; // 紫色 - 无进度条的指标
    }
    if (progress > 80) {
      return 'rgba(239, 68, 68, 0.5)'; // 红色 - 高负载
    }
    return 'rgba(34, 197, 94, 0.5)'; // 绿色 - 正常
  };

  const getGlowColor = () => {
    if (progress === undefined) {
      return 'rgba(139, 92, 246, 0.15)';
    }
    if (progress > 80) {
      return 'rgba(239, 68, 68, 0.2)';
    }
    return 'rgba(34, 197, 94, 0.15)';
  };

  return (
    <div
      className="rounded-lg p-3 space-y-2 transition-all duration-300 hover:scale-[1.02]"
      style={{
        background: 'linear-gradient(135deg, var(--bg-secondary), var(--bg-tertiary))',
        border: `1.5px solid ${getBorderColor()}`,
        boxShadow: `0 0 12px ${getGlowColor()}, inset 0 0 12px rgba(255, 255, 255, 0.02)`,
      }}
    >
      <div className="flex items-center gap-1.5" style={{ color: 'var(--text-muted)' }}>
        {icon}
        <span className="text-xs font-medium uppercase tracking-wide">{label}</span>
      </div>
      {progress !== undefined && (
        <div
          className="w-full h-1.5 rounded-full overflow-hidden"
          style={{
            background: 'var(--bg-primary)',
            border: '1px solid rgba(255, 255, 255, 0.05)',
          }}
          role="progressbar"
          aria-valuenow={Math.min(progress, 100)}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div
            className="h-full rounded-full transition-all duration-500"
            style={{
              width: `${Math.min(progress, 100)}%`,
              background: progress > 80
                ? 'linear-gradient(90deg, var(--accent-warning), var(--accent-danger))'
                : 'linear-gradient(90deg, var(--accent-primary), var(--accent-secondary))',
              boxShadow: progress > 80
                ? '0 0 8px var(--accent-danger)'
                : '0 0 8px var(--accent-primary)',
            }}
          />
        </div>
      )}
      <div className="text-sm font-semibold font-mono truncate" style={{ color: 'var(--text-primary)' }} title={value}>
        {value}
      </div>
    </div>
  );
}

