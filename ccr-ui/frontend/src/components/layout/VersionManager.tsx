'use client';

import { Download, RefreshCw, Zap } from 'lucide-react';
import { useEffect, useState } from 'react';
import { getVersion, checkUpdate, updateCCR } from '@/lib/api/client';
import type { VersionInfo, UpdateCheckResponse } from '@/lib/types';
import UpdateModal from '../ui/UpdateModal';

export default function VersionManager() {
  const [versionInfo, setVersionInfo] = useState<VersionInfo | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateCheckResponse | null>(null);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [updateStage, setUpdateStage] = useState<'confirm' | 'updating' | 'success' | 'error'>('confirm');
  const [updateOutput, setUpdateOutput] = useState('');
  const [updateError, setUpdateError] = useState('');

  useEffect(() => {
    loadVersionInfo();
  }, []);

  const loadVersionInfo = async () => {
    try {
      const data = await getVersion();
      setVersionInfo(data);
    } catch (err) {
      console.error('Failed to load version info:', err);
    }
  };

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
        setTimeout(() => {
          loadVersionInfo();
          setUpdateInfo(null);
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

  if (!versionInfo) return null;

  return (
    <>
      <div
        className="rounded-lg p-4"
        style={{
          background: 'var(--bg-tertiary)',
          border: '1px solid var(--border-color)',
        }}
      >
        {/* 版本信息头部 */}
        <div className="flex items-center justify-between mb-3">
          <span className="text-xs font-semibold uppercase tracking-wider" style={{ color: 'var(--text-secondary)' }}>
            版本管理
          </span>
          <Zap className="w-4 h-4" style={{ color: 'var(--accent-primary)' }} />
        </div>

        {/* 当前版本 */}
        <div className="mb-3">
          <div className="text-xs mb-1" style={{ color: 'var(--text-muted)' }}>当前版本</div>
          <div
            className="text-2xl font-bold font-mono tracking-wide"
            style={{ color: 'var(--accent-primary)' }}
          >
            v{versionInfo.current_version}
          </div>
        </div>

        {/* 更新信息 */}
        {updateInfo && updateInfo.has_update && (
          <div
            className="mb-3 p-2.5 rounded-lg"
            style={{
              background: 'rgba(16, 185, 129, 0.1)',
              border: '1px solid var(--accent-success)',
            }}
          >
            <div className="flex items-center justify-between mb-1.5">
              <div className="flex items-center space-x-1.5">
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
              <span className="text-sm font-bold font-mono" style={{ color: 'var(--accent-success)' }}>
                v{updateInfo.latest_version}
              </span>
            </div>
            {updateInfo.release_url && (
              <a
                href={updateInfo.release_url}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs underline hover:no-underline"
                style={{ color: 'var(--accent-success)' }}
              >
                查看更新日志 →
              </a>
            )}
          </div>
        )}

        {updateInfo && !updateInfo.has_update && (
          <div className="mb-3 text-xs text-center py-1.5" style={{ color: 'var(--text-muted)' }}>
            ✓ 已是最新版本
          </div>
        )}

        {/* 操作按钮 */}
        <div className="grid grid-cols-2 gap-2">
          <button
            onClick={handleCheckUpdate}
            disabled={isCheckingUpdate}
            className="px-3 py-2 rounded-lg font-semibold text-xs transition-all flex items-center justify-center space-x-1.5 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
            style={{
              background: 'var(--bg-secondary)',
              color: 'var(--text-primary)',
              border: '1px solid var(--border-color)',
            }}
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isCheckingUpdate ? 'animate-spin' : ''}`} />
            <span>{isCheckingUpdate ? '检查中' : '检查更新'}</span>
          </button>

          <button
            onClick={handleOpenUpdateModal}
            className={`px-3 py-2 rounded-lg font-semibold text-xs transition-all flex items-center justify-center space-x-1.5 text-white hover:scale-105 ${
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

      {/* 更新对话框 */}
      <UpdateModal
        isOpen={showUpdateModal}
        onClose={handleCloseUpdateModal}
        onConfirm={handleConfirmUpdate}
        stage={updateStage}
        output={updateOutput}
        error={updateError}
      />
    </>
  );
}

