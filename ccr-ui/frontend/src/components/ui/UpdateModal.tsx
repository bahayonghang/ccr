'use client';

import { X, AlertTriangle, CheckCircle, Loader2, AlertCircle } from 'lucide-react';
import { useEffect } from 'react';

interface UpdateModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  stage: 'confirm' | 'updating' | 'success' | 'error';
  output?: string;
  error?: string;
}

export default function UpdateModal({
  isOpen,
  onClose,
  onConfirm,
  stage,
  output = '',
  error = '',
}: UpdateModalProps) {
  // 按 Escape 关闭（仅在非更新中状态）
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && stage !== 'updating') {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
      return () => document.removeEventListener('keydown', handleEscape);
    }
  }, [isOpen, stage, onClose]);

  if (!isOpen) return null;

  return (
    <>
      {/* 背景遮罩 */}
      <div
        className="fixed inset-0 z-50 flex items-center justify-center p-4"
        style={{
          background: 'rgba(0, 0, 0, 0.6)',
          backdropFilter: 'blur(8px)',
        }}
        onClick={(e) => {
          if (e.target === e.currentTarget && stage !== 'updating') {
            onClose();
          }
        }}
      >
        {/* 模态框 */}
        <div
          className="relative rounded-2xl shadow-2xl w-full max-w-2xl max-h-[80vh] overflow-hidden animate-modal-in"
          style={{
            background: 'var(--bg-primary)',
            border: '1px solid var(--border-color)',
            boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
          }}
        >
          {/* 顶部装饰线 */}
          <div
            className="h-1"
            style={{
              background: stage === 'error'
                ? 'var(--accent-error)'
                : stage === 'success'
                ? 'var(--accent-success)'
                : 'linear-gradient(90deg, var(--accent-primary), var(--accent-secondary))',
            }}
          />

          {/* 头部 */}
          <div className="px-6 py-5 flex items-center justify-between border-b" style={{ borderColor: 'var(--border-color)' }}>
            <div className="flex items-center space-x-3">
              {stage === 'confirm' && (
                <AlertTriangle className="w-6 h-6" style={{ color: 'var(--accent-warning)' }} />
              )}
              {stage === 'updating' && (
                <Loader2 className="w-6 h-6 animate-spin" style={{ color: 'var(--accent-primary)' }} />
              )}
              {stage === 'success' && (
                <CheckCircle className="w-6 h-6" style={{ color: 'var(--accent-success)' }} />
              )}
              {stage === 'error' && (
                <AlertCircle className="w-6 h-6" style={{ color: 'var(--accent-error)' }} />
              )}
              
              <h2 className="text-xl font-bold" style={{ color: 'var(--text-primary)' }}>
                {stage === 'confirm' && '确认更新'}
                {stage === 'updating' && '正在更新...'}
                {stage === 'success' && '更新成功'}
                {stage === 'error' && '更新失败'}
              </h2>
            </div>

            {stage !== 'updating' && (
              <button
                onClick={onClose}
                className="p-2 rounded-lg transition-all hover:scale-110"
                style={{
                  background: 'var(--bg-tertiary)',
                  color: 'var(--text-secondary)',
                }}
                aria-label="关闭"
              >
                <X className="w-5 h-5" />
              </button>
            )}
          </div>

          {/* 内容区域 */}
          <div className="px-6 py-5 overflow-y-auto max-h-[60vh]">
            {stage === 'confirm' && (
              <div className="space-y-4">
                <p className="text-base leading-relaxed" style={{ color: 'var(--text-primary)' }}>
                  确定要立即更新 CCR 吗？
                </p>
                <div
                  className="rounded-lg p-4 space-y-2"
                  style={{
                    background: 'var(--bg-secondary)',
                    border: '1px solid var(--border-color)',
                  }}
                >
                  <p className="text-sm font-semibold" style={{ color: 'var(--text-secondary)' }}>
                    ⚠️ 注意事项：
                  </p>
                  <ul className="text-sm space-y-1.5 ml-6 list-disc" style={{ color: 'var(--text-muted)' }}>
                    <li>更新过程可能需要几分钟时间</li>
                    <li>更新期间请勿关闭此窗口</li>
                    <li>更新完成后需要刷新页面</li>
                    <li>建议在更新前保存当前工作</li>
                  </ul>
                </div>
              </div>
            )}

            {stage === 'updating' && (
              <div className="space-y-4">
                <div className="flex items-center space-x-3">
                  <Loader2 className="w-5 h-5 animate-spin" style={{ color: 'var(--accent-primary)' }} />
                  <p className="text-base font-medium" style={{ color: 'var(--text-primary)' }}>
                    正在执行更新，请稍候...
                  </p>
                </div>

                {/* 输出日志 */}
                {output && (
                  <div
                    className="rounded-lg p-4 font-mono text-xs overflow-x-auto"
                    style={{
                      background: 'var(--bg-secondary)',
                      border: '1px solid var(--border-color)',
                      color: 'var(--text-secondary)',
                      maxHeight: '300px',
                      overflowY: 'auto',
                    }}
                  >
                    <pre className="whitespace-pre-wrap">{output}</pre>
                  </div>
                )}

                {/* 进度动画 */}
                <div className="relative h-2 rounded-full overflow-hidden" style={{ background: 'var(--bg-tertiary)' }}>
                  <div
                    className="h-full animate-progress-bar"
                    style={{
                      background: 'linear-gradient(90deg, var(--accent-primary), var(--accent-secondary))',
                    }}
                  />
                </div>
              </div>
            )}

            {stage === 'success' && (
              <div className="space-y-4">
                <div
                  className="rounded-lg p-4 flex items-start space-x-3"
                  style={{
                    background: 'rgba(34, 197, 94, 0.1)',
                    border: '1px solid var(--accent-success)',
                  }}
                >
                  <CheckCircle className="w-5 h-5 mt-0.5 flex-shrink-0" style={{ color: 'var(--accent-success)' }} />
                  <div className="space-y-2 flex-1">
                    <p className="text-base font-semibold" style={{ color: 'var(--accent-success)' }}>
                      CCR 已成功更新！
                    </p>
                    <p className="text-sm" style={{ color: 'var(--text-secondary)' }}>
                      更新已完成，建议刷新页面以使用最新版本。
                    </p>
                  </div>
                </div>

                {/* 输出日志 */}
                {output && (
                  <details className="cursor-pointer">
                    <summary className="text-sm font-medium mb-2" style={{ color: 'var(--text-secondary)' }}>
                      查看更新日志
                    </summary>
                    <div
                      className="rounded-lg p-4 font-mono text-xs overflow-x-auto"
                      style={{
                        background: 'var(--bg-secondary)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-secondary)',
                        maxHeight: '200px',
                        overflowY: 'auto',
                      }}
                    >
                      <pre className="whitespace-pre-wrap">{output}</pre>
                    </div>
                  </details>
                )}
              </div>
            )}

            {stage === 'error' && (
              <div className="space-y-4">
                <div
                  className="rounded-lg p-4 flex items-start space-x-3"
                  style={{
                    background: 'rgba(239, 68, 68, 0.1)',
                    border: '1px solid var(--accent-error)',
                  }}
                >
                  <AlertCircle className="w-5 h-5 mt-0.5 flex-shrink-0" style={{ color: 'var(--accent-error)' }} />
                  <div className="space-y-2 flex-1">
                    <p className="text-base font-semibold" style={{ color: 'var(--accent-error)' }}>
                      更新失败
                    </p>
                    <p className="text-sm" style={{ color: 'var(--text-secondary)' }}>
                      更新过程中出现错误，请查看错误信息并重试。
                    </p>
                  </div>
                </div>

                {/* 错误信息 */}
                {error && (
                  <div
                    className="rounded-lg p-4 font-mono text-xs overflow-x-auto"
                    style={{
                      background: 'var(--bg-secondary)',
                      border: '1px solid var(--accent-error)',
                      color: 'var(--accent-error)',
                      maxHeight: '200px',
                      overflowY: 'auto',
                    }}
                  >
                    <pre className="whitespace-pre-wrap">{error}</pre>
                  </div>
                )}

                {/* 输出日志 */}
                {output && (
                  <details className="cursor-pointer">
                    <summary className="text-sm font-medium mb-2" style={{ color: 'var(--text-secondary)' }}>
                      查看详细日志
                    </summary>
                    <div
                      className="rounded-lg p-4 font-mono text-xs overflow-x-auto"
                      style={{
                        background: 'var(--bg-secondary)',
                        border: '1px solid var(--border-color)',
                        color: 'var(--text-secondary)',
                        maxHeight: '200px',
                        overflowY: 'auto',
                      }}
                    >
                      <pre className="whitespace-pre-wrap">{output}</pre>
                    </div>
                  </details>
                )}
              </div>
            )}
          </div>

          {/* 底部按钮 */}
          <div
            className="px-6 py-4 flex items-center justify-end space-x-3 border-t"
            style={{ borderColor: 'var(--border-color)', background: 'var(--bg-secondary)' }}
          >
            {stage === 'confirm' && (
              <>
                <button
                  onClick={onClose}
                  className="px-5 py-2.5 rounded-lg font-semibold text-sm transition-all hover:scale-105"
                  style={{
                    background: 'var(--bg-tertiary)',
                    color: 'var(--text-primary)',
                    border: '1px solid var(--border-color)',
                  }}
                >
                  取消
                </button>
                <button
                  onClick={onConfirm}
                  className="px-5 py-2.5 rounded-lg font-semibold text-sm transition-all text-white hover:scale-105"
                  style={{
                    background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                    boxShadow: '0 0 20px var(--glow-primary)',
                  }}
                >
                  确认更新
                </button>
              </>
            )}

            {stage === 'updating' && (
              <p className="text-sm" style={{ color: 'var(--text-muted)' }}>
                更新中，请勿关闭窗口...
              </p>
            )}

            {(stage === 'success' || stage === 'error') && (
              <>
                <button
                  onClick={onClose}
                  className="px-5 py-2.5 rounded-lg font-semibold text-sm transition-all hover:scale-105"
                  style={{
                    background: 'var(--bg-tertiary)',
                    color: 'var(--text-primary)',
                    border: '1px solid var(--border-color)',
                  }}
                >
                  关闭
                </button>
                {stage === 'success' && (
                  <button
                    onClick={() => window.location.reload()}
                    className="px-5 py-2.5 rounded-lg font-semibold text-sm transition-all text-white hover:scale-105"
                    style={{
                      background: 'linear-gradient(135deg, var(--accent-success), var(--accent-primary))',
                      boxShadow: '0 0 20px var(--glow-success)',
                    }}
                  >
                    刷新页面
                  </button>
                )}
              </>
            )}
          </div>
        </div>
      </div>

      <style jsx>{`
        @keyframes modal-in {
          from {
            opacity: 0;
            transform: scale(0.95) translateY(-10px);
          }
          to {
            opacity: 1;
            transform: scale(1) translateY(0);
          }
        }

        @keyframes progress-bar {
          0% {
            width: 0%;
          }
          100% {
            width: 100%;
          }
        }

        .animate-modal-in {
          animation: modal-in 0.3s ease-out;
        }

        .animate-progress-bar {
          animation: progress-bar 2s ease-in-out infinite;
        }
      `}</style>
    </>
  );
}

