'use client';

import { CheckCircle, Trash2, Download, RefreshCw, Zap } from 'lucide-react';
import { useEffect, useState } from 'react';
import { getSystemInfo, getVersion, checkUpdate, updateCCR } from '@/lib/api/client';
import type { SystemInfo, VersionInfo, UpdateCheckResponse } from '@/lib/types';
import UpdateModal from '../ui/UpdateModal';

interface LeftSidebarProps {
  currentConfig: string;
  totalConfigs: number;
  historyCount: number;
  onValidate?: () => void;
  onClean?: () => void;
}

export default function LeftSidebar({
  currentConfig,
  totalConfigs,
  historyCount,
  onValidate,
  onClean,
}: LeftSidebarProps) {
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [versionInfo, setVersionInfo] = useState<VersionInfo | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateCheckResponse | null>(null);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [updateStage, setUpdateStage] = useState<'confirm' | 'updating' | 'success' | 'error'>('confirm');
  const [updateOutput, setUpdateOutput] = useState('');
  const [updateError, setUpdateError] = useState('');

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

  const handleCheckUpdate = async () => {
    setIsCheckingUpdate(true);
    try {
      const data = await checkUpdate();
      setUpdateInfo(data);
    } catch (err) {
      console.error('Failed to check for updates:', err);
    } finally {
      setIsCheckingUpdate(false);
    }
  };

  const handleOpenUpdateModal = () => {
    setUpdateStage('confirm');
    setUpdateOutput('');
    setUpdateError('');
    setShowUpdateModal(true);
  };

  const handleConfirmUpdate = async () => {
    setUpdateStage('updating');
    setUpdateOutput('开始更新 CCR...\n');
    
    try {
      const result = await updateCCR();
      
      if (result.success) {
        setUpdateOutput(result.output || '更新完成！');
        setUpdateStage('success');
        // 重新获取版本信息
        setTimeout(() => {
          const loadVersionInfo = async () => {
            try {
              const data = await getVersion();
              setVersionInfo(data);
              // 清除更新信息，以便下次可以重新检查
              setUpdateInfo(null);
            } catch (err) {
              console.error('Failed to reload version info:', err);
            }
          };
          loadVersionInfo();
        }, 1000);
      } else {
        setUpdateOutput(result.output || '');
        setUpdateError(result.error || '更新失败');
        setUpdateStage('error');
      }
    } catch (err) {
      console.error('Failed to update CCR:', err);
      setUpdateError(err instanceof Error ? err.message : '更新过程中出现错误');
      setUpdateStage('error');
    }
  };

  const handleCloseUpdateModal = () => {
    setShowUpdateModal(false);
  };

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
    <aside
      className="rounded-xl p-5 h-fit sticky top-5 glass-effect"
      style={{
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-small)',
      }}
    >
      {/* 当前配置 */}
      <section className="mb-6">
        <h2
          className="text-xs font-semibold uppercase tracking-wider mb-3"
          style={{ color: 'var(--text-secondary)' }}
        >
          当前配置
        </h2>
        <div
          className="relative overflow-hidden rounded-lg p-4"
          style={{
            background: 'linear-gradient(135deg, var(--bg-tertiary), var(--bg-secondary))',
            border: '1px solid var(--accent-primary)',
          }}
        >
          {/* 扫描动画线 */}
          <div
            className="absolute top-0 left-0 w-full h-0.5 animate-scan"
            style={{
              background: 'linear-gradient(90deg, transparent, var(--accent-primary), transparent)',
            }}
            aria-hidden="true"
          />

          <div className="flex items-center space-x-2 mb-2">
            <span
              className="w-1.5 h-1.5 rounded-full animate-pulse"
              style={{
                background: 'var(--accent-success)',
                boxShadow: '0 0 10px var(--glow-success)',
              }}
              aria-label="状态：活跃"
            />
            <span className="text-xs" style={{ color: 'var(--text-secondary)' }}>
              ACTIVE
            </span>
          </div>
          <div
            className="text-lg font-bold uppercase font-mono tracking-wide break-all"
            style={{ color: 'var(--text-primary)' }}
          >
            {currentConfig || '-'}
          </div>
        </div>
      </section>

      {/* 系统信息 */}
      {systemInfo && (
        <section className="mb-6">
          <h2
            className="text-xs font-semibold uppercase tracking-wider mb-3"
            style={{ color: 'var(--text-secondary)' }}
          >
            系统信息
          </h2>
          <div
            className="rounded-lg p-3 space-y-2.5"
            style={{
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border-color)',
            }}
          >
            <SystemInfoItem icon="💻" label="主机" value={systemInfo.hostname} />
            <SystemInfoItem icon="🖥️" label="系统" value={`${systemInfo.os} ${systemInfo.os_version}`} />
            <SystemInfoItem icon="⚙️" label="CPU" value={`${systemInfo.cpu_cores} 核心`} />
            <SystemInfoItem
              icon="📊"
              label="CPU 使用率"
              value={`${Math.round(systemInfo.cpu_usage)}%`}
              progress={systemInfo.cpu_usage}
            />
            <SystemInfoItem
              icon="💾"
              label="内存"
              value={`${systemInfo.used_memory_gb.toFixed(1)} GB / ${systemInfo.total_memory_gb.toFixed(1)} GB`}
              progress={systemInfo.memory_usage_percent}
            />
            <SystemInfoItem icon="⏱️" label="运行时间" value={formatUptime(systemInfo.uptime_seconds)} />
          </div>
        </section>
      )}

      {/* 统计 */}
      <section className="mb-6">
        <h2
          className="text-xs font-semibold uppercase tracking-wider mb-3"
          style={{ color: 'var(--text-secondary)' }}
        >
          统计
        </h2>
        <div className="grid grid-cols-2 gap-2.5">
          <StatCard label="总配置" value={totalConfigs} />
          <StatCard label="历史" value={historyCount} />
        </div>
      </section>

      {/* 版本信息 */}
      {versionInfo && (
        <section className="mb-6">
          <h2
            className="text-xs font-semibold uppercase tracking-wider mb-3"
            style={{ color: 'var(--text-secondary)' }}
          >
            版本信息
          </h2>
          <div
            className="rounded-lg p-3 space-y-2"
            style={{
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border-color)',
            }}
          >
            <div className="flex items-center justify-between">
              <span className="text-xs font-medium" style={{ color: 'var(--text-muted)' }}>
                当前版本
              </span>
              <span className="text-sm font-bold font-mono" style={{ color: 'var(--accent-primary)' }}>
                v{versionInfo.current_version}
              </span>
            </div>

            {updateInfo && (
              <div className="mt-3 pt-3" style={{ borderTop: '1px solid var(--border-color)' }}>
                {updateInfo.has_update ? (
                  <>
                    <div className="flex items-center space-x-2 mb-2">
                      <span
                        className="w-1.5 h-1.5 rounded-full animate-pulse"
                        style={{
                          background: 'var(--accent-success)',
                          boxShadow: '0 0 10px var(--glow-success)',
                        }}
                      />
                      <span className="text-xs font-semibold" style={{ color: 'var(--accent-success)' }}>
                        发现新版本
                      </span>
                    </div>
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-xs" style={{ color: 'var(--text-muted)' }}>
                        最新版本
                      </span>
                      <span className="text-sm font-bold font-mono" style={{ color: 'var(--accent-success)' }}>
                        v{updateInfo.latest_version}
                      </span>
                    </div>
                    <a
                      href={updateInfo.release_url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="w-full px-3 py-2 rounded-lg font-semibold text-xs transition-all flex items-center justify-center space-x-2 text-white hover:scale-105"
                      style={{
                        background: 'linear-gradient(135deg, var(--accent-success), var(--accent-primary))',
                        boxShadow: '0 0 20px var(--glow-success)',
                      }}
                    >
                      <Download className="w-3.5 h-3.5" />
                      <span>下载更新</span>
                    </a>
                  </>
                ) : (
                  <div className="text-xs text-center py-1" style={{ color: 'var(--text-muted)' }}>
                    已是最新版本
                  </div>
                )}
              </div>
            )}

            <div className="space-y-2">
              <button
                onClick={handleCheckUpdate}
                disabled={isCheckingUpdate}
                className="w-full px-3 py-2 rounded-lg font-semibold text-xs transition-all flex items-center justify-center space-x-2 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
                style={{
                  background: 'var(--bg-secondary)',
                  color: 'var(--text-primary)',
                  border: '1px solid var(--border-color)',
                }}
              >
                <RefreshCw className={`w-3.5 h-3.5 ${isCheckingUpdate ? 'animate-spin' : ''}`} />
                <span>{isCheckingUpdate ? '检查中...' : '检查更新'}</span>
              </button>

              {/* 立即更新按钮 - 始终显示，有新版本时高亮 */}
              <button
                onClick={handleOpenUpdateModal}
                className={`w-full px-3 py-2 rounded-lg font-semibold text-xs transition-all flex items-center justify-center space-x-2 text-white hover:scale-105 ${
                  updateInfo?.has_update ? 'animate-pulse-subtle' : ''
                }`}
                style={{
                  background: updateInfo?.has_update
                    ? 'linear-gradient(135deg, var(--accent-success), var(--accent-primary))'
                    : 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                  boxShadow: updateInfo?.has_update
                    ? '0 0 20px var(--glow-success)'
                    : '0 0 20px var(--glow-primary)',
                }}
              >
                <Zap className="w-3.5 h-3.5" />
                <span>立即更新</span>
              </button>
            </div>
          </div>
        </section>
      )}

      {/* 更新对话框 */}
      <UpdateModal
        isOpen={showUpdateModal}
        onClose={handleCloseUpdateModal}
        onConfirm={handleConfirmUpdate}
        stage={updateStage}
        output={updateOutput}
        error={updateError}
      />

      {/* 操作按钮 */}
      <div className="space-y-2.5">
        {onValidate && (
          <button
            onClick={onValidate}
            className="w-full px-4 py-2.5 rounded-lg font-semibold text-sm transition-all flex items-center justify-center space-x-2 hover:scale-105"
            style={{
              background: 'var(--bg-tertiary)',
              color: 'var(--text-primary)',
              border: '1px solid var(--border-color)',
            }}
            aria-label="验证配置"
          >
            <CheckCircle className="w-4 h-4" />
            <span>验证</span>
          </button>
        )}
        {onClean && (
          <button
            onClick={onClean}
            className="w-full px-4 py-2.5 rounded-lg font-semibold text-sm transition-all flex items-center justify-center space-x-2 text-white hover:scale-105"
            style={{
              background: 'var(--accent-warning)',
              boxShadow: '0 0 20px rgba(245, 158, 11, 0.3)',
            }}
            aria-label="清理备份"
          >
            <Trash2 className="w-4 h-4" />
            <span>清理备份</span>
          </button>
        )}
      </div>
    </aside>
  );
}

function SystemInfoItem({
  icon,
  label,
  value,
  progress,
}: {
  icon: string;
  label: string;
  value: string;
  progress?: number;
}) {
  return (
    <div
      className="flex items-start space-x-2.5 p-2 rounded-lg transition-all hover:translate-x-0.5"
      style={{ background: 'var(--bg-secondary)' }}
    >
      <div className="text-xl flex-shrink-0 w-6 text-center" aria-hidden="true">{icon}</div>
      <div className="flex-1 min-w-0">
        <div
          className="text-xs uppercase tracking-wide font-semibold mb-1"
          style={{ color: 'var(--text-muted)' }}
        >
          {label}
        </div>
        {progress !== undefined && (
          <div
            className="w-full h-1.5 rounded-full overflow-hidden mb-1 relative"
            style={{ background: 'var(--bg-tertiary)' }}
            role="progressbar"
            aria-valuenow={Math.min(progress, 100)}
            aria-valuemin={0}
            aria-valuemax={100}
          >
            <div
              className="h-full rounded-full transition-all duration-500 relative overflow-hidden"
              style={{
                width: `${Math.min(progress, 100)}%`,
                background: 'linear-gradient(90deg, var(--accent-primary), var(--accent-secondary))',
              }}
            >
              <div
                className="absolute top-0 left-0 w-full h-full animate-shimmer"
                style={{
                  background: 'linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent)',
                }}
                aria-hidden="true"
              />
            </div>
          </div>
        )}
        <div className="text-xs font-medium font-mono truncate" style={{ color: 'var(--text-primary)' }}>
          {value}
        </div>
      </div>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: number }) {
  return (
    <div
      className="p-2.5 rounded-lg text-center transition-all hover:scale-105"
      style={{
        background: 'var(--bg-tertiary)',
        border: '1px solid var(--border-color)',
      }}
    >
      <div className="text-lg font-bold" style={{ color: 'var(--accent-primary)' }}>
        {value}
      </div>
      <div className="text-xs mt-1" style={{ color: 'var(--text-muted)' }}>
        {label}
      </div>
    </div>
  );
}

