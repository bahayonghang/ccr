'use client';

import { Activity, Cpu, HardDrive, Clock, Server, Zap } from 'lucide-react';
import { useEffect, useState } from 'react';
import { getSystemInfo, getVersion } from '@/lib/api/client';
import type { SystemInfo, VersionInfo } from '@/lib/types';

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
  const [versionInfo, setVersionInfo] = useState<VersionInfo | null>(null);

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

  useEffect(() => {
    const loadVersionInfo = async () => {
      try {
        const data = await getVersion();
        setVersionInfo(data);
      } catch (err) {
        console.error('Failed to load version info:', err);
      }
    };

    loadVersionInfo();
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
      className="rounded-xl p-5 mb-5 glass-effect"
      style={{
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-medium)',
      }}
    >
      {/* 第一行：当前配置和统计 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
        {/* 当前激活配置 */}
        <div
          className="relative overflow-hidden rounded-lg p-4"
          style={{
            background: 'linear-gradient(135deg, var(--bg-tertiary), var(--bg-secondary))',
            border: '1px solid var(--accent-primary)',
          }}
        >
          <div
            className="absolute top-0 left-0 w-full h-0.5 animate-scan"
            style={{
              background: 'linear-gradient(90deg, transparent, var(--accent-primary), transparent)',
            }}
            aria-hidden="true"
          />
          <div className="flex items-center justify-between mb-2">
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
            className="text-xl font-bold uppercase font-mono tracking-wide truncate"
            style={{ color: 'var(--text-primary)' }}
            title={currentConfig}
          >
            {currentConfig || '-'}
          </div>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-2 gap-3">
          <StatCard icon={<Server className="w-4 h-4" />} label="总配置" value={totalConfigs} />
          <StatCard icon={<Activity className="w-4 h-4" />} label="历史记录" value={historyCount} />
        </div>

        {/* 版本信息 */}
        {versionInfo && (
          <div
            className="rounded-lg p-4"
            style={{
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border-color)',
            }}
          >
            <div className="flex items-center justify-between mb-2">
              <span className="text-xs font-semibold uppercase tracking-wider" style={{ color: 'var(--text-secondary)' }}>
                CCR 版本
              </span>
              <Zap className="w-4 h-4" style={{ color: 'var(--accent-primary)' }} />
            </div>
            <div
              className="text-xl font-bold font-mono tracking-wide"
              style={{ color: 'var(--accent-primary)' }}
            >
              v{versionInfo.current_version}
            </div>
          </div>
        )}
      </div>

      {/* 第二行：系统信息 */}
      {systemInfo && (
        <div
          className="rounded-lg p-4"
          style={{
            background: 'var(--bg-tertiary)',
            border: '1px solid var(--border-color)',
          }}
        >
          <div className="text-xs font-semibold uppercase tracking-wider mb-3" style={{ color: 'var(--text-secondary)' }}>
            系统信息
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
            <SystemMetric
              icon={<Server className="w-4 h-4" />}
              label="主机"
              value={systemInfo.hostname}
            />
            <SystemMetric
              icon={<Cpu className="w-4 h-4" />}
              label="CPU"
              value={`${systemInfo.cpu_cores} 核心`}
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
            <SystemMetric
              icon={<Activity className="w-4 h-4" />}
              label="CPU 使用率"
              value={`${Math.round(systemInfo.cpu_usage)}%`}
              progress={systemInfo.cpu_usage}
            />
          </div>
        </div>
      )}
    </div>
  );
}

function StatCard({ icon, label, value }: { icon: React.ReactNode; label: string; value: number }) {
  return (
    <div
      className="p-3 rounded-lg transition-all hover:scale-105"
      style={{
        background: 'var(--bg-tertiary)',
        border: '1px solid var(--border-color)',
      }}
    >
      <div className="flex items-center gap-2 mb-1" style={{ color: 'var(--text-muted)' }}>
        {icon}
        <span className="text-xs font-medium">{label}</span>
      </div>
      <div className="text-2xl font-bold" style={{ color: 'var(--accent-primary)' }}>
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
  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-1.5" style={{ color: 'var(--text-muted)' }}>
        {icon}
        <span className="text-xs font-medium uppercase tracking-wide">{label}</span>
      </div>
      {progress !== undefined && (
        <div
          className="w-full h-1.5 rounded-full overflow-hidden"
          style={{ background: 'var(--bg-secondary)' }}
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

