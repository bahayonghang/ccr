'use client';

import { CheckCircle, Trash2, Download, RefreshCw, Zap } from 'lucide-react';
import { useEffect, useState } from 'react';
import { getVersion, checkUpdate, updateCCR } from '@/lib/api/client';
import type { VersionInfo, UpdateCheckResponse } from '@/lib/types';
import UpdateModal from '../ui/UpdateModal';

interface LeftSidebarProps {
  onValidate?: () => void;
  onClean?: () => void;
}

export default function LeftSidebar({
  onValidate,
  onClean,
}: LeftSidebarProps) {
  const [versionInfo, setVersionInfo] = useState<VersionInfo | null>(null);
  const [updateInfo, setUpdateInfo] = useState<UpdateCheckResponse | null>(null);
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [updateStage, setUpdateStage] = useState<'confirm' | 'updating' | 'success' | 'error'>('confirm');
  const [updateOutput, setUpdateOutput] = useState('');
  const [updateError, setUpdateError] = useState('');

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

  return (
    <aside
      className="rounded-xl p-4 h-fit sticky top-5 glass-effect"
      style={{
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-small)',
      }}
    >

      {/* 版本管理 */}
      {versionInfo && (
        <section className="mb-4">
          <h2
            className="text-xs font-semibold uppercase tracking-wider mb-2.5"
            style={{ color: 'var(--text-secondary)' }}
          >
            版本管理
          </h2>
          <div
            className="rounded-lg p-2.5 space-y-2"
            style={{
              background: 'var(--bg-tertiary)',
              border: '1px solid var(--border-color)',
            }}
          >

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
      <div className="space-y-2">
        {onValidate && (
          <button
            onClick={onValidate}
            className="w-full px-3 py-2 rounded-lg font-semibold text-xs transition-all flex items-center justify-center space-x-1.5 hover:scale-105"
            style={{
              background: 'var(--bg-tertiary)',
              color: 'var(--text-primary)',
              border: '1px solid var(--border-color)',
            }}
            aria-label="验证配置"
          >
            <CheckCircle className="w-3.5 h-3.5" />
            <span>验证配置</span>
          </button>
        )}
        {onClean && (
          <button
            onClick={onClean}
            className="w-full px-3 py-2 rounded-lg font-semibold text-xs transition-all flex items-center justify-center space-x-1.5 text-white hover:scale-105"
            style={{
              background: 'var(--accent-warning)',
              boxShadow: '0 0 15px rgba(245, 158, 11, 0.3)',
            }}
            aria-label="清理备份"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>清理备份</span>
          </button>
        )}
      </div>
    </aside>
  );
}


