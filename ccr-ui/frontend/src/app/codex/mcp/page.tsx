'use client';

import { useState, useEffect } from 'react';
import { Server, Plus, Edit2, Trash2, Globe, Terminal } from 'lucide-react';
import {
  listCodexMcpServers,
  addCodexMcpServer,
  updateCodexMcpServer,
  deleteCodexMcpServer,
  listConfigs,
  getHistory
} from '@/lib/api/client';
import type { CodexMcpServer, CodexMcpServerRequest } from '@/lib/types';
import Navbar from '@/components/layout/Navbar';
import StatusHeader from '@/components/layout/StatusHeader';
import CollapsibleSidebar from '@/components/layout/CollapsibleSidebar';

export default function CodexMcpManagement() {
  const [servers, setServers] = useState<CodexMcpServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [currentConfig, setCurrentConfig] = useState<string>('');
  const [totalConfigs, setTotalConfigs] = useState(0);
  const [historyCount, setHistoryCount] = useState(0);
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingServer, setEditingServer] = useState<CodexMcpServer | null>(null);
  const [serverType, setServerType] = useState<'stdio' | 'http'>('stdio');
  const [formData, setFormData] = useState<CodexMcpServerRequest>({
    command: '',
    args: [],
    env: {},
    cwd: '',
    startup_timeout_ms: 20000,
    url: '',
    bearer_token: '',
  });
  const [argInput, setArgInput] = useState('');
  const [envKey, setEnvKey] = useState('');
  const [envValue, setEnvValue] = useState('');

  const loadServers = async () => {
    try {
      setLoading(true);
      const data = await listCodexMcpServers();
      setServers(data);

      // 加载系统配置信息
      try {
        const configData = await listConfigs();
        setCurrentConfig(configData.current_config);
        setTotalConfigs(configData.configs.length);

        const historyData = await getHistory();
        setHistoryCount(historyData.total);
      } catch (err) {
        console.error('Failed to load system info:', err);
      }
    } catch (err) {
      console.error('Failed to load Codex MCP servers:', err);
      alert('加载 Codex MCP 服务器失败');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadServers();
  }, []);

  const handleAdd = () => {
    setShowAddForm(true);
    setEditingServer(null);
    setServerType('stdio');
    setFormData({
      command: 'npx',
      args: [],
      env: {},
      cwd: '',
      startup_timeout_ms: 20000,
      url: '',
      bearer_token: '',
    });
    setArgInput('');
  };

  const handleEdit = (server: CodexMcpServer) => {
    setEditingServer(server);
    setShowAddForm(true);

    // 判断服务器类型
    if (server.url) {
      setServerType('http');
    } else {
      setServerType('stdio');
    }

    setFormData({
      command: server.command || '',
      args: server.args || [],
      env: server.env || {},
      cwd: server.cwd || '',
      startup_timeout_ms: server.startup_timeout_ms || 20000,
      url: server.url || '',
      bearer_token: server.bearer_token || '',
    });
    setArgInput((server.args || []).join(' '));
  };

  const handleSubmit = async () => {
    // 验证必填字段
    if (serverType === 'stdio' && !formData.command) {
      alert('STDIO 服务器必须填写命令');
      return;
    }
    if (serverType === 'http' && !formData.url) {
      alert('HTTP 服务器必须填写 URL');
      return;
    }

    // Parse args from input
    const args = serverType === 'stdio' ? argInput.split(' ').filter(a => a.trim()) : [];

    const request: CodexMcpServerRequest = serverType === 'stdio'
      ? {
          command: formData.command,
          args,
          env: formData.env,
          cwd: formData.cwd || undefined,
          startup_timeout_ms: formData.startup_timeout_ms,
        }
      : {
          url: formData.url,
          bearer_token: formData.bearer_token || undefined,
        };

    try {
      if (editingServer) {
        await updateCodexMcpServer(editingServer.name, request);
        alert('✓ Codex MCP 服务器更新成功');
      } else {
        await addCodexMcpServer(request);
        alert('✓ Codex MCP 服务器添加成功');
      }
      setShowAddForm(false);
      await loadServers();
    } catch (err) {
      alert(`操作失败: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  const handleDelete = async (name: string) => {
    if (!confirm(`确定删除 Codex MCP 服务器 "${name}" 吗？`)) return;

    try {
      await deleteCodexMcpServer(name);
      alert('✓ Codex MCP 服务器删除成功');
      await loadServers();
    } catch (err) {
      alert(`删除失败: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  const addEnvVar = () => {
    if (envKey && envValue) {
      setFormData(prev => ({
        ...prev,
        env: { ...prev.env, [envKey]: envValue }
      }));
      setEnvKey('');
      setEnvValue('');
    }
  };

  const removeEnvVar = (key: string) => {
    setFormData(prev => {
      const newEnv = { ...prev.env };
      delete newEnv[key];
      return { ...prev, env: newEnv };
    });
  };

  return (
    <div style={{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }}>
      <div className="max-w-[1800px] mx-auto">
        <Navbar />
        <StatusHeader currentConfig={currentConfig} totalConfigs={totalConfigs} historyCount={historyCount} />

        <div className="grid grid-cols-[auto_1fr] gap-4">
          <CollapsibleSidebar />

          <main className="rounded-xl p-6 glass-effect" style={{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }}>
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-3">
                <Server className="w-6 h-6" style={{ color: 'var(--accent-primary)' }} />
                <h1 className="text-2xl font-bold" style={{ color: 'var(--text-primary)' }}>Codex MCP 服务器管理</h1>
              </div>
              <button
                onClick={handleAdd}
                className="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2"
                style={{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }}
              >
                <Plus className="w-4 h-4" />
                添加 MCP 服务器
              </button>
            </div>

            <div className="mb-4 p-3 rounded-lg" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)' }}>
              <div className="text-sm" style={{ color: 'var(--text-muted)' }}>
                <strong style={{ color: 'var(--text-primary)' }}>配置文件位置:</strong> <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }}>~/.codex/config.toml</code>
              </div>
            </div>

            {loading ? (
              <div className="flex justify-center py-20">
                <div className="w-12 h-12 rounded-full border-4 border-transparent animate-spin" style={{ borderTopColor: 'var(--accent-primary)', borderRightColor: 'var(--accent-secondary)' }} />
              </div>
            ) : (
              <div className="space-y-3">
                {servers.length === 0 ? (
                  <div className="text-center py-10" style={{ color: 'var(--text-muted)' }}>
                    暂无 Codex MCP 服务器配置
                  </div>
                ) : (
                  servers.map((server) => (
                    <div key={server.name} className="rounded-lg p-4" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)' }}>
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <h3 className="text-lg font-bold font-mono" style={{ color: 'var(--text-primary)' }}>{server.name}</h3>
                            {server.url ? (
                              <span className="px-2 py-0.5 rounded text-xs font-semibold uppercase flex items-center gap-1" style={{ background: 'var(--accent-secondary)', color: 'white' }}>
                                <Globe className="w-3 h-3" />
                                HTTP
                              </span>
                            ) : (
                              <span className="px-2 py-0.5 rounded text-xs font-semibold uppercase flex items-center gap-1" style={{ background: 'var(--accent-primary)', color: 'white' }}>
                                <Terminal className="w-3 h-3" />
                                STDIO
                              </span>
                            )}
                          </div>
                          <div className="space-y-2 text-sm">
                            {server.command && (
                              <>
                                <div><span style={{ color: 'var(--text-muted)' }}>命令:</span> <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }}>{server.command}</code></div>
                                <div><span style={{ color: 'var(--text-muted)' }}>参数:</span> <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }}>{(server.args || []).join(' ')}</code></div>
                                {server.cwd && <div><span style={{ color: 'var(--text-muted)' }}>工作目录:</span> <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }}>{server.cwd}</code></div>}
                                {server.startup_timeout_ms && <div><span style={{ color: 'var(--text-muted)' }}>启动超时:</span> <span className="ml-2" style={{ color: 'var(--text-primary)' }}>{server.startup_timeout_ms}ms</span></div>}
                              </>
                            )}
                            {server.url && (
                              <>
                                <div><span style={{ color: 'var(--text-muted)' }}>URL:</span> <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--accent-secondary)' }}>{server.url}</code></div>
                                {server.bearer_token && <div><span style={{ color: 'var(--text-muted)' }}>Bearer Token:</span> <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }}>••••••••</code></div>}
                              </>
                            )}
                            {server.env && Object.keys(server.env).length > 0 && (
                              <div>
                                <span style={{ color: 'var(--text-muted)' }}>环境变量:</span>
                                <div className="ml-2 mt-1 space-y-1">
                                  {Object.entries(server.env).map(([key, value]) => (
                                    <div key={key} className="text-xs font-mono px-2 py-1 rounded" style={{ background: 'var(--bg-secondary)' }}>
                                      <span style={{ color: 'var(--accent-secondary)' }}>{key}</span>=<span style={{ color: 'var(--text-primary)' }}>{value}</span>
                                    </div>
                                  ))}
                                </div>
                              </div>
                            )}
                          </div>
                        </div>
                        <div className="flex gap-2">
                          <button onClick={() => handleEdit(server)} className="p-2 rounded-lg transition-all hover:scale-110" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }} title="编辑">
                            <Edit2 className="w-4 h-4" />
                          </button>
                          <button onClick={() => handleDelete(server.name)} className="p-2 rounded-lg transition-all hover:scale-110" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }} title="删除">
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </div>
                    </div>
                  ))
                )}
              </div>
            )}

            {/* Add/Edit Form Modal */}
            {showAddForm && (
              <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                <div className="rounded-xl p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)' }}>
                  <h2 className="text-xl font-bold mb-4" style={{ color: 'var(--text-primary)' }}>
                    {editingServer ? '编辑 Codex MCP 服务器' : '添加 Codex MCP 服务器'}
                  </h2>

                  {/* Server Type Selector */}
                  <div className="mb-4">
                    <label className="block text-sm font-semibold mb-2" style={{ color: 'var(--text-secondary)' }}>服务器类型</label>
                    <div className="flex gap-3">
                      <button
                        onClick={() => setServerType('stdio')}
                        className={`flex-1 px-4 py-2 rounded-lg font-semibold text-sm flex items-center justify-center gap-2 ${serverType === 'stdio' ? 'text-white' : ''}`}
                        style={{
                          background: serverType === 'stdio' ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' : 'var(--bg-tertiary)',
                          border: '1px solid var(--border-color)',
                          color: serverType === 'stdio' ? 'white' : 'var(--text-primary)'
                        }}
                      >
                        <Terminal className="w-4 h-4" />
                        STDIO (本地子进程)
                      </button>
                      <button
                        onClick={() => setServerType('http')}
                        className={`flex-1 px-4 py-2 rounded-lg font-semibold text-sm flex items-center justify-center gap-2 ${serverType === 'http' ? 'text-white' : ''}`}
                        style={{
                          background: serverType === 'http' ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' : 'var(--bg-tertiary)',
                          border: '1px solid var(--border-color)',
                          color: serverType === 'http' ? 'white' : 'var(--text-primary)'
                        }}
                      >
                        <Globe className="w-4 h-4" />
                        HTTP (远程服务器)
                      </button>
                    </div>
                  </div>

                  <div className="space-y-4">
                    {serverType === 'stdio' ? (
                      <>
                        <div>
                          <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>命令 *</label>
                          <input type="text" value={formData.command} onChange={(e) => setFormData({ ...formData, command: e.target.value })} className="w-full px-3 py-2 rounded-lg font-mono" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="例如: npx 或 node" />
                        </div>

                        <div>
                          <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>参数 *</label>
                          <input type="text" value={argInput} onChange={(e) => setArgInput(e.target.value)} className="w-full px-3 py-2 rounded-lg font-mono" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="例如: -y @upstash/context7-mcp" />
                          <div className="text-xs mt-1" style={{ color: 'var(--text-muted)' }}>用空格分隔多个参数</div>
                        </div>

                        <div>
                          <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>工作目录 (可选)</label>
                          <input type="text" value={formData.cwd} onChange={(e) => setFormData({ ...formData, cwd: e.target.value })} className="w-full px-3 py-2 rounded-lg font-mono" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="例如: /path/to/directory" />
                        </div>

                        <div>
                          <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>启动超时 (毫秒)</label>
                          <input type="number" value={formData.startup_timeout_ms} onChange={(e) => setFormData({ ...formData, startup_timeout_ms: parseInt(e.target.value) || 20000 })} className="w-full px-3 py-2 rounded-lg" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="20000" />
                        </div>

                        <div>
                          <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>环境变量</label>
                          <div className="flex gap-2 mb-2">
                            <input type="text" value={envKey} onChange={(e) => setEnvKey(e.target.value)} className="flex-1 px-3 py-2 rounded-lg font-mono" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="KEY" />
                            <input type="text" value={envValue} onChange={(e) => setEnvValue(e.target.value)} className="flex-1 px-3 py-2 rounded-lg font-mono" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="VALUE" />
                            <button onClick={addEnvVar} className="px-4 py-2 rounded-lg font-semibold text-sm text-white" style={{ background: 'var(--accent-primary)' }}>
                              添加
                            </button>
                          </div>
                          <div className="space-y-1">
                            {Object.entries(formData.env || {}).map(([key, value]) => (
                              <div key={key} className="flex items-center justify-between px-3 py-2 rounded" style={{ background: 'var(--bg-secondary)' }}>
                                <code className="text-sm font-mono" style={{ color: 'var(--text-primary)' }}>{key}={value}</code>
                                <button onClick={() => removeEnvVar(key)} className="text-xs" style={{ color: 'var(--accent-danger)' }}>删除</button>
                              </div>
                            ))}
                          </div>
                        </div>
                      </>
                    ) : (
                      <>
                        <div>
                          <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>服务器 URL *</label>
                          <input type="text" value={formData.url} onChange={(e) => setFormData({ ...formData, url: e.target.value })} className="w-full px-3 py-2 rounded-lg font-mono" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="例如: https://mcp.example.com/mcp" />
                        </div>

                        <div>
                          <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>Bearer Token (可选)</label>
                          <input type="password" value={formData.bearer_token} onChange={(e) => setFormData({ ...formData, bearer_token: e.target.value })} className="w-full px-3 py-2 rounded-lg font-mono" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="your-bearer-token" />
                        </div>
                      </>
                    )}
                  </div>

                  <div className="flex gap-3 mt-6">
                    <button onClick={handleSubmit} className="flex-1 px-4 py-2 rounded-lg font-semibold text-white" style={{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }}>
                      {editingServer ? '更新' : '添加'}
                    </button>
                    <button onClick={() => setShowAddForm(false)} className="flex-1 px-4 py-2 rounded-lg font-semibold" style={{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }}>
                      取消
                    </button>
                  </div>
                </div>
              </div>
            )}
          </main>
        </div>
      </div>
    </div>
  );
}
