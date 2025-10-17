'use client';

import { useState, useEffect } from 'react';
import { Cloud, CloudUpload, CloudDownload, Info, CheckCircle, XCircle, RefreshCw, AlertCircle, Home, Server, User, FolderOpen, Settings } from 'lucide-react';
import Navbar from '@/components/layout/Navbar';
import Link from 'next/link';

interface SyncConfigDetails {
  enabled: boolean;
  webdav_url: string;
  username: string;
  remote_path: string;
  auto_sync: boolean;
  remote_file_exists: boolean | null;
}

interface SyncStatus {
  success: boolean;
  output: string;
  configured: boolean;
  config: SyncConfigDetails | null;
}

interface SyncInfo {
  feature_name: string;
  description: string;
  supported_services: string[];
  setup_steps: string[];
  security_notes: string[];
}

export default function SyncManagement() {
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null);
  const [syncInfo, setSyncInfo] = useState<SyncInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [operating, setOperating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [operationResult, setOperationResult] = useState<{ success: boolean; message: string } | null>(null);

  const loadSyncStatus = async () => {
    try {
      setLoading(true);
      setError(null);
      
      // 获取同步状态
      const statusRes = await fetch('http://localhost:8081/api/sync/status');
      const statusData = await statusRes.json();
      
      if (statusData.success && statusData.data) {
        setSyncStatus(statusData.data);
      } else {
        setError(statusData.message || 'Failed to load sync status');
      }

      // 获取同步信息
      const infoRes = await fetch('http://localhost:8081/api/sync/info');
      const infoData = await infoRes.json();
      
      if (infoData.success && infoData.data) {
        setSyncInfo(infoData.data);
      }
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
      
      const res = await fetch('http://localhost:8081/api/sync/push', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ force }),
      });
      
      const data = await res.json();
      
      if (data.success && data.data) {
        setOperationResult({
          success: data.data.success,
          message: data.data.output || data.data.error
        });
        
        if (data.data.success) {
          await loadSyncStatus();
        }
      } else {
        setOperationResult({
          success: false,
          message: data.message || 'Failed to push config'
        });
      }
    } catch (err) {
      setOperationResult({
        success: false,
        message: err instanceof Error ? err.message : 'Failed to push config'
      });
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
      
      const res = await fetch('http://localhost:8081/api/sync/pull', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ force }),
      });
      
      const data = await res.json();
      
      if (data.success && data.data) {
        setOperationResult({
          success: data.data.success,
          message: data.data.output || data.data.error
        });
        
        if (data.data.success) {
          await loadSyncStatus();
        }
      } else {
        setOperationResult({
          success: false,
          message: data.message || 'Failed to pull config'
        });
      }
    } catch (err) {
      setOperationResult({
        success: false,
        message: err instanceof Error ? err.message : 'Failed to pull config'
      });
    } finally {
      setOperating(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50">
      <Navbar />
      
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-8 pb-8">
        {/* 顶部标题栏 + 返回按钮 */}
        <div className="mb-8">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <Cloud className="w-8 h-8 text-blue-600" />
              <h1 className="text-3xl font-bold text-gray-900">WebDAV 云同步</h1>
            </div>
            <Link
              href="/configs"
              className="flex items-center gap-2 px-4 py-2 bg-white hover:bg-gray-50 text-gray-700 rounded-lg border border-gray-300 transition-colors shadow-sm"
            >
              <Home className="w-4 h-4" />
              <span className="font-medium">返回首页</span>
            </Link>
          </div>
          <p className="text-gray-600">
            使用 WebDAV 协议同步配置文件到云端存储（坚果云、Nextcloud、ownCloud 等）
          </p>
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
            {/* 同步状态卡片 */}
            <div className="lg:col-span-2 space-y-6">
              {/* 配置状态 */}
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
                              <div className="text-sm font-mono text-gray-900 break-all">
                                {syncStatus.config.webdav_url}
                              </div>
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
                              <div className="text-sm font-mono text-gray-900">
                                {syncStatus.config.username}
                              </div>
                            </div>
                          </div>
                        </div>

                        {/* 远程路径 */}
                        <div className="bg-white border border-gray-200 rounded-lg p-4">
                          <div className="flex items-start gap-3">
                            <div className="p-2 bg-green-50 rounded-lg">
                              <FolderOpen className="w-5 h-5 text-green-600" />
                            </div>
                            <div className="flex-1">
                              <div className="text-xs text-gray-500 mb-1">远程路径</div>
                              <div className="text-sm font-mono text-gray-900">
                                {syncStatus.config.remote_path}
                              </div>
                            </div>
                          </div>
                        </div>

                        {/* 状态信息 */}
                        <div className="grid grid-cols-2 gap-3">
                          {/* 自动同步 */}
                          <div className="bg-white border border-gray-200 rounded-lg p-4">
                            <div className="flex items-start gap-3">
                              <div className="p-2 bg-orange-50 rounded-lg">
                                <Settings className="w-5 h-5 text-orange-600" />
                              </div>
                              <div className="flex-1">
                                <div className="text-xs text-gray-500 mb-1">自动同步</div>
                                <div className={`text-sm font-semibold ${
                                  syncStatus.config.auto_sync ? 'text-green-600' : 'text-gray-400'
                                }`}>
                                  {syncStatus.config.auto_sync ? '✓ 已开启' : '✗ 已关闭'}
                                </div>
                              </div>
                            </div>
                          </div>

                          {/* 远程文件状态 */}
                          <div className="bg-white border border-gray-200 rounded-lg p-4">
                            <div className="flex items-start gap-3">
                              <div className="p-2 bg-cyan-50 rounded-lg">
                                <Cloud className="w-5 h-5 text-cyan-600" />
                              </div>
                              <div className="flex-1">
                                <div className="text-xs text-gray-500 mb-1">远程文件</div>
                                <div className={`text-sm font-semibold ${
                                  syncStatus.config.remote_file_exists === true
                                    ? 'text-green-600'
                                    : syncStatus.config.remote_file_exists === false
                                    ? 'text-yellow-600'
                                    : 'text-gray-400'
                                }`}>
                                  {syncStatus.config.remote_file_exists === true
                                    ? '✓ 已存在'
                                    : syncStatus.config.remote_file_exists === false
                                    ? '⚠ 不存在'
                                    : '未知'}
                                </div>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      <div className="flex items-center gap-2 text-yellow-700 bg-yellow-50 px-4 py-3 rounded-lg">
                        <AlertCircle className="w-5 h-5" />
                        <span className="font-medium">同步功能未配置</span>
                      </div>
                      
                      <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                        <p className="text-sm text-blue-800 mb-3">
                          请在终端运行以下命令配置 WebDAV 同步：
                        </p>
                        <code className="block bg-white px-4 py-2 rounded border border-blue-300 text-sm font-mono text-blue-900">
                          ccr sync config
                        </code>
                      </div>
                    </div>
                  )}
                </div>
              </div>

              {/* 操作按钮 */}
              {syncStatus?.configured && (
                <div className="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
                  <div className="px-6 py-4 border-b border-gray-200">
                    <h3 className="font-semibold text-gray-900">同步操作</h3>
                  </div>
                  
                  <div className="p-6 space-y-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <button
                        onClick={() => handlePush(false)}
                        disabled={operating}
                        className="flex items-center justify-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-medium rounded-lg transition-colors"
                      >
                        <CloudUpload className="w-5 h-5" />
                        上传到云端 (Push)
                      </button>
                      
                      <button
                        onClick={() => handlePull(false)}
                        disabled={operating}
                        className="flex items-center justify-center gap-2 px-6 py-3 bg-green-600 hover:bg-green-700 disabled:bg-gray-400 text-white font-medium rounded-lg transition-colors"
                      >
                        <CloudDownload className="w-5 h-5" />
                        从云端下载 (Pull)
                      </button>
                    </div>

                    <button
                      onClick={() => loadSyncStatus()}
                      disabled={operating}
                      className="w-full flex items-center justify-center gap-2 px-6 py-2 bg-gray-100 hover:bg-gray-200 disabled:bg-gray-50 text-gray-700 font-medium rounded-lg transition-colors"
                    >
                      <RefreshCw className={`w-4 h-4 ${operating ? 'animate-spin' : ''}`} />
                      刷新状态
                    </button>
                  </div>
                </div>
              )}

              {/* 操作结果 */}
              {operationResult && (
                <div className={`rounded-xl shadow-sm border p-6 ${
                  operationResult.success
                    ? 'bg-green-50 border-green-200'
                    : 'bg-red-50 border-red-200'
                }`}>
                  <div className="flex items-start gap-3">
                    {operationResult.success ? (
                      <CheckCircle className="w-6 h-6 text-green-600 flex-shrink-0 mt-0.5" />
                    ) : (
                      <XCircle className="w-6 h-6 text-red-600 flex-shrink-0 mt-0.5" />
                    )}
                    <div className="flex-1">
                      <h3 className={`font-semibold mb-2 ${
                        operationResult.success ? 'text-green-900' : 'text-red-900'
                      }`}>
                        {operationResult.success ? '操作成功' : '操作失败'}
                      </h3>
                      <pre className={`text-sm whitespace-pre-wrap font-mono ${
                        operationResult.success ? 'text-green-800' : 'text-red-800'
                      }`}>
                        {operationResult.message}
                      </pre>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* 功能说明侧边栏 */}
            <div className="space-y-6">
              {syncInfo && (
                <>
                  {/* 功能介绍 */}
                  <div className="bg-white rounded-xl shadow-sm border border-gray-200 overflow-hidden">
                    <div className="bg-gradient-to-r from-indigo-600 to-purple-600 px-6 py-4">
                      <h3 className="font-bold text-white flex items-center gap-2">
                        <Info className="w-5 h-5" />
                        功能说明
                      </h3>
                    </div>
                    
                    <div className="p-6 space-y-4">
                      <div>
                        <h4 className="font-semibold text-gray-900 mb-2">功能介绍</h4>
                        <p className="text-sm text-gray-600">{syncInfo.description}</p>
                      </div>

                      <div>
                        <h4 className="font-semibold text-gray-900 mb-2">支持的服务</h4>
                        <ul className="space-y-1">
                          {syncInfo.supported_services.map((service, idx) => (
                            <li key={idx} className="text-sm text-gray-600 flex items-center gap-2">
                              <span className="w-1.5 h-1.5 bg-blue-500 rounded-full"></span>
                              {service}
                            </li>
                          ))}
                        </ul>
                      </div>

                      <div>
                        <h4 className="font-semibold text-gray-900 mb-2">配置步骤</h4>
                        <ol className="space-y-2">
                          {syncInfo.setup_steps.map((step, idx) => (
                            <li key={idx} className="text-sm text-gray-600 flex gap-2">
                              <span className="font-semibold text-blue-600 flex-shrink-0">{idx + 1}.</span>
                              <span>{step}</span>
                            </li>
                          ))}
                        </ol>
                      </div>

                      <div>
                        <h4 className="font-semibold text-gray-900 mb-2">安全提示</h4>
                        <ul className="space-y-2">
                          {syncInfo.security_notes.map((note, idx) => (
                            <li key={idx} className="text-xs text-gray-500 flex items-start gap-2">
                              <span className="text-yellow-600 flex-shrink-0 mt-0.5">⚠️</span>
                              <span>{note}</span>
                            </li>
                          ))}
                        </ul>
                      </div>
                    </div>
                  </div>
                </>
              )}
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
