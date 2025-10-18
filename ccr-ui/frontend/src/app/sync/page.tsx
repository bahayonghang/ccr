'use client';

import { useState, useEffect } from 'react';
import { Cloud, CloudUpload, CloudDownload, Info, CheckCircle, XCircle, RefreshCw, AlertCircle, Home, Server, User, FolderOpen, Settings } from 'lucide-react';
import Navbar from '@/components/layout/Navbar';
import Link from 'next/link';
import { getSyncStatus, getSyncInfo, pushSync, pullSync } from '@/lib/api/client';
import type { SyncStatusResponse, SyncInfoResponse } from '@/lib/types';

export default function SyncManagement() {
  const [syncStatus, setSyncStatus] = useState<SyncStatusResponse | null>(null);
  const [syncInfo, setSyncInfo] = useState<SyncInfoResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [operating, setOperating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [operationResult, setOperationResult] = useState<{ success: boolean; message: string } | null>(null);

  const loadSyncStatus = async () => {
    try {
      setLoading(true);
      setError(null);
      const statusData = await getSyncStatus();
      setSyncStatus(statusData);

      const infoData = await getSyncInfo();
      setSyncInfo(infoData);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load sync status');
      console.error('Error loading sync status:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadSyncStatus();
  }, []);

  const handlePush = async (force: boolean = false) => {
    if (!force && !confirm('确定要上传配置到云端吗？\n如果远程文件已存在将会被覆盖。')) {
      return;
    }
    try {
      setOperating(true);
      setOperationResult(null);
      const data = await pushSync({ force });
      setOperationResult({ success: data.success, message: data.output || data.error });
      if (data.success) {
        await loadSyncStatus();
      }
    } catch (err) {
      setOperationResult({ success: false, message: err instanceof Error ? err.message : 'Failed to push config' });
    } finally {
      setOperating(false);
    }
  };

  const handlePull = async (force: boolean = false) => {
    if (!force && !confirm('确定要从云端下载配置吗？\n本地配置将被覆盖（会先备份）。')) {
      return;
    }
    try {
      setOperating(true);
      setOperationResult(null);
      const data = await pullSync({ force });
      setOperationResult({ success: data.success, message: data.output || data.error });
      if (data.success) {
        await loadSyncStatus();
      }
    } catch (err) {
      setOperationResult({ success: false, message: err instanceof Error ? err.message : 'Failed to pull config' });
    } finally {
      setOperating(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50">
      <Navbar />
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-8 pb-8">
        <div className="mb-8">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <Cloud className="w-8 h-8 text-blue-600" />
              <h1 className="text-3xl font-bold text-gray-900">WebDAV 云同步</h1>
            </div>
            <Link href="/configs" className="flex items-center gap-2 px-4 py-2 bg-white hover:bg-gray-50 text-gray-700 rounded-lg border border-gray-300 transition-colors shadow-sm">
              <Home className="w-4 h-4" />
              <span className="font-medium">返回首页</span>
            </Link>
          </div>
          <p className="text-gray-600">使用 WebDAV 协议同步配置文件到云端存储（坚果云、Nextcloud、ownCloud 等）</p>
        </div>

        {loading ? (
          <div className="flex items-center justify-center py-12">
            <RefreshCw className="w-8 h-8 text-blue-600 animate-spin" />
          </div>
        ) : error ? (
          <div className="bg-red-50 border border-red-200 rounded-lg p-4 flex items-start gap-3">
            <XCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
            <div>
              <h3 className="font-semibold text-red-900 mb-1">加载失败</h3>
              <p className="text-sm text-red-700">{error}</p>
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div className="lg:col-span-2 space-y-6">
              <div className="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
                <div className="bg-gradient-to-r from-blue-600 to-indigo-600 px-6 py-4">
                  <h2 className="text-xl font-bold text-white flex items-center gap-2">
                    <Cloud className="w-6 h-6" />
                    同步状态
                  </h2>
                </div>
                <div className="p-6">
                  {syncStatus?.configured && syncStatus.config ? (
                    <div className="space-y-4">
                      <div className="flex items-center gap-2 text-green-700 bg-green-50 px-4 py-3 rounded-lg">
                        <CheckCircle className="w-5 h-5" />
                        <span className="font-medium">同步功能已配置</span>
                      </div>
                      {/* 配置详情卡片 */}
                      <div className="grid grid-cols-1 gap-3">
                        {/* WebDAV 服务器 */}
                        <div className="bg-white border border-gray-200 rounded-lg p-4">
                          <div className="flex items-start gap-3">
                            <div className="p-2 bg-blue-50 rounded-lg">
                              <Server className="w-5 h-5 text-blue-600" />
                            </div>
                            <div className="flex-1">
                              <div className="text-xs text-gray-500 mb-1">WebDAV 服务器</div>
                              <div className="text-sm font-mono text-gray-900 break-all">{syncStatus.config.webdav_url}</div>
                            </div>
                          </div>
                        </div>
                        {/* 用户名 */}
                        <div className="bg-white border border-gray-200 rounded-lg p-4">
                          <div className="flex items-start gap-3">
                            <div className="p-2 bg-purple-50 rounded-lg">
                              <User className="w-5 h-5 text-purple-600" />
                            </div>
                            <div className="flex-1">
                              <div className="text-xs text-gray-500 mb-1">用户名</div>
                              <div className="text-sm font-mono text-gray-900">{syncStatus.config.username}</div>
                            </div>
                          </div>
                        </div>
                        {/* 远程路径 */}
                        <div className="bg-white border border-gray-200 rounded-lg p-4">
                          <div className="flex items-start gap-3">
                            <div className="p-2 bg-pink-50 rounded-lg">
                              <FolderOpen className="w-5 h-5 text-pink-600" />
                            </div>
                            <div className="flex-1">
                              <div className="text-xs text-gray-500 mb-1">远程路径</div>
                              <div className="text-sm font-mono text-gray-900 break-all">{syncStatus.config.remote_path}</div>
                            </div>
                          </div>
                        </div>
                        {/* 自动同步 */}
                        <div className="bg-white border border-gray-200 rounded-lg p-4">
                          <div className="flex items-start gap-3">
                            <div className="p-2 bg-amber-50 rounded-lg">
                              <Settings className="w-5 h-5 text-amber-600" />
                            </div>
                            <div className="flex-1">
                              <div className="text-xs text-gray-500 mb-1">自动同步</div>
                              <div className="text-sm text-gray-900">{syncStatus.config.auto_sync ? '✓ 开启' : '✗ 关闭'}</div>
                            </div>
                          </div>
                        </div>
                        {/* 远程文件存在 */}
                        {typeof syncStatus.config.remote_file_exists === 'boolean' && (
                          <div className="bg-white border border-gray-200 rounded-lg p-4">
                            <div className="flex items-start gap-3">
                              <div className="p-2 bg-green-50 rounded-lg">
                                <Info className="w-5 h-5 text-green-600" />
                              </div>
                              <div className="flex-1">
                                <div className="text-xs text-gray-500 mb-1">远程文件状态</div>
                                <div className="text-sm text-gray-900">{syncStatus.config.remote_file_exists ? '✓ 存在' : '✗ 不存在'}</div>
                              </div>
                            </div>
                          </div>
                        )}
                      </div>
                      {/* 操作按钮 */}
                      <div className="flex flex-wrap gap-3">
                        <button onClick={() => handlePush(false)} disabled={operating} className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg shadow-sm disabled:opacity-50"><CloudUpload className="w-4 h-4" /><span>上传到云端</span></button>
                        <button onClick={() => handlePull(false)} disabled={operating} className="flex items-center gap-2 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg shadow-sm disabled:opacity-50"><CloudDownload className="w-4 h-4" /><span>从云端下载</span></button>
                        <button onClick={() => handlePush(true)} disabled={operating} className="px-3 py-2 text-sm rounded-lg border border-blue-200 text-blue-700 bg-blue-50 hover:bg-blue-100">强制上传</button>
                        <button onClick={() => handlePull(true)} disabled={operating} className="px-3 py-2 text-sm rounded-lg border border-indigo-200 text-indigo-700 bg-indigo-50 hover:bg-indigo-100">强制下载</button>
                      </div>
                      {operationResult && (
                        <div className={`mt-4 p-4 rounded-lg border ${operationResult.success ? 'bg-green-50 border-green-200 text-green-700' : 'bg-red-50 border-red-200 text-red-700'}`}>
                          <div className="flex items-center gap-2">
                            {operationResult.success ? <CheckCircle className="w-5 h-5" /> : <AlertCircle className="w-5 h-5" />}
                            <span className="font-medium">{operationResult.success ? '操作成功' : '操作失败'}</span>
                          </div>
                          <pre className="mt-2 text-xs whitespace-pre-wrap">{operationResult.message}</pre>
                        </div>
                      )}
                    </div>
                  ) : (
                    <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 flex items-start gap-3">
                      <AlertCircle className="w-5 h-5 text-yellow-700 flex-shrink-0 mt-0.5" />
                      <div>
                        <h3 className="font-semibold text-yellow-900 mb-1">同步功能未配置</h3>
                        <p className="text-sm text-yellow-800">请在终端中运行 <code className="font-mono">ccr sync config</code> 设置 WebDAV 连接</p>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
            {/* 右侧信息栏 */}
            <aside className="space-y-6">
              <div className="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
                <div className="bg-gradient-to-r from-purple-600 to-pink-600 px-6 py-4">
                  <h2 className="text-xl font-bold text-white flex items-center gap-2"><Info className="w-6 h-6" />功能说明</h2>
                </div>
                <div className="p-6">
                  {syncInfo ? (
                    <div className="space-y-4">
                      <div>
                        <h3 className="text-base font-semibold text-gray-900">{syncInfo.feature_name}</h3>
                        <p className="text-sm text-gray-700 mt-1">{syncInfo.description}</p>
                      </div>
                      <div>
                        <h4 className="text-sm font-semibold text-gray-900">支持的服务</h4>
                        <ul className="mt-2 list-disc list-inside text-sm text-gray-700 space-y-1">
                          {syncInfo.supported_services.map((s) => (<li key={s}>{s}</li>))}
                        </ul>
                      </div>
                      <div>
                        <h4 className="text-sm font-semibold text-gray-900">配置步骤</h4>
                        <ol className="mt-2 list-decimal list-inside text-sm text-gray-700 space-y-1">
                          {syncInfo.setup_steps.map((s) => (<li key={s}>{s}</li>))}
                        </ol>
                      </div>
                      <div>
                        <h4 className="text-sm font-semibold text-gray-900">安全说明</h4>
                        <ul className="mt-2 list-disc list-inside text-sm text-gray-700 space-y-1">
                          {syncInfo.security_notes.map((s) => (<li key={s}>{s}</li>))}
                        </ul>
                      </div>
                    </div>
                  ) : (
                    <div className="text-sm text-gray-600">加载中...</div>
                  )}
                </div>
              </div>
            </aside>
          </div>
        )}
      </main>
    </div>
  );
}
