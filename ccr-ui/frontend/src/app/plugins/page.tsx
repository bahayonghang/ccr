'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { Puzzle, Plus, Edit2, Trash2, Power, PowerOff, Home } from 'lucide-react';
import { 
  listPlugins, 
  addPlugin, 
  updatePlugin, 
  deletePlugin, 
  togglePlugin,
  listConfigs,
  getHistory
} from '@/lib/api/client';
import type { Plugin as PluginType, PluginRequest } from '@/lib/types';
import Navbar from '@/components/layout/Navbar';
import StatusHeader from '@/components/layout/StatusHeader';
import CollapsibleSidebar from '@/components/layout/CollapsibleSidebar';

export default function PluginsManagement() {
  const [plugins, setPlugins] = useState<PluginType[]>([]);
  const [loading, setLoading] = useState(true);
  const [currentConfig, setCurrentConfig] = useState<string>('');
  const [totalConfigs, setTotalConfigs] = useState(0);
  const [historyCount, setHistoryCount] = useState(0);
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingPlugin, setEditingPlugin] = useState<PluginType | null>(null);
  const [formData, setFormData] = useState<PluginRequest>({
    id: '',
    name: '',
    version: '',
    enabled: true,
    config: undefined,
  });
  const [configJson, setConfigJson] = useState('');

  const loadPlugins = async () => {
    try {
      setLoading(true);
      const data = await listPlugins();
      setPlugins(data);

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
      console.error('Failed to load plugins:', err);
      alert('加载插件失败');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadPlugins();
  }, []);

  const handleAdd = () => {
    setShowAddForm(true);
    setEditingPlugin(null);
    setFormData({
      id: '',
      name: '',
      version: '1.0.0',
      enabled: true,
      config: undefined,
    });
    setConfigJson('');
  };

  const handleEdit = (plugin: PluginType) => {
    setEditingPlugin(plugin);
    setShowAddForm(true);
    setFormData({
      id: plugin.id,
      name: plugin.name,
      version: plugin.version,
      enabled: plugin.enabled,
      config: plugin.config,
    });
    setConfigJson(plugin.config ? JSON.stringify(plugin.config, null, 2) : '');
  };

  const handleSubmit = async () => {
    if (!formData.id || !formData.name || !formData.version) {
      alert('请填写必填字段');
      return;
    }

    // Parse config JSON if provided
    let config = undefined;
    if (configJson.trim()) {
      try {
        config = JSON.parse(configJson);
      } catch (err) {
        alert('配置 JSON 格式错误');
        return;
      }
    }

    const request: PluginRequest = {
      ...formData,
      config,
    };

    try {
      if (editingPlugin) {
        await updatePlugin(editingPlugin.id, request);
        alert('✓ 插件更新成功');
      } else {
        await addPlugin(request);
        alert('✓ 插件添加成功');
      }
      setShowAddForm(false);
      await loadPlugins();
    } catch (err) {
      alert(`操作失败: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm(`确定删除插件 "${id}" 吗？`)) return;

    try {
      await deletePlugin(id);
      alert('✓ 插件删除成功');
      await loadPlugins();
    } catch (err) {
      alert(`删除失败: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  const handleToggle = async (id: string) => {
    try {
      await togglePlugin(id);
      await loadPlugins();
    } catch (err) {
      alert(`切换失败: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
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
                <Puzzle className="w-6 h-6" style={{ color: 'var(--accent-primary)' }} />
                <h1 className="text-2xl font-bold" style={{ color: 'var(--text-primary)' }}>插件管理</h1>
              </div>
              <div className="flex items-center gap-3">
                <Link
                  href="/"
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                  style={{
                    background: 'var(--bg-secondary)',
                    color: 'var(--text-secondary)',
                    border: '1px solid var(--border-color)',
                  }}
                >
                  <Home className="w-4 h-4" />
                  <span>返回首页</span>
                </Link>
                <button
                  onClick={handleAdd}
                  className="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2"
                  style={{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }}
                >
                  <Plus className="w-4 h-4" />
                  添加插件
                </button>
              </div>
            </div>

            {loading ? (
              <div className="flex justify-center py-20">
                <div className="w-12 h-12 rounded-full border-4 border-transparent animate-spin" style={{ borderTopColor: 'var(--accent-primary)', borderRightColor: 'var(--accent-secondary)' }} />
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {plugins.length === 0 ? (
                  <div className="col-span-full text-center py-10" style={{ color: 'var(--text-muted)' }}>
                    暂无插件配置
                  </div>
                ) : (
                  plugins.map((plugin) => (
                    <div
                      key={plugin.id}
                      className="group rounded-lg p-4 transition-all duration-300"
                      style={{
                        background: 'rgba(255, 255, 255, 0.7)',
                        border: '1px solid rgba(99, 102, 241, 0.12)',
                        outline: 'none',
                        cursor: 'default'
                      }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.background = 'rgba(255, 255, 255, 0.9)';
                        e.currentTarget.style.borderColor = 'rgba(99, 102, 241, 0.24)';
                        e.currentTarget.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -2px rgba(0, 0, 0, 0.08)';
                        e.currentTarget.style.transform = 'translateY(-2px)';
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.background = 'rgba(255, 255, 255, 0.7)';
                        e.currentTarget.style.borderColor = 'rgba(99, 102, 241, 0.12)';
                        e.currentTarget.style.boxShadow = 'none';
                        e.currentTarget.style.transform = 'translateY(0)';
                      }}
                    >
                      <div className="flex items-start justify-between mb-3">
                        <div className="flex items-center gap-2">
                          <Puzzle className="w-5 h-5" style={{ color: plugin.enabled ? 'var(--accent-success)' : 'var(--text-muted)' }} />
                          <h3 className="text-base font-bold" style={{ color: 'var(--text-primary)' }}>{plugin.name}</h3>
                        </div>
                        {!plugin.enabled && (
                          <span className="px-2 py-0.5 rounded text-xs font-semibold uppercase" style={{ background: 'var(--accent-danger)', color: 'white' }}>
                            已禁用
                          </span>
                        )}
                      </div>

                      <div className="space-y-2 text-sm mb-4">
                        <div><span style={{ color: 'var(--text-muted)' }}>ID:</span> <code className="ml-2 text-xs font-mono" style={{ color: 'var(--accent-primary)' }}>{plugin.id}</code></div>
                        <div><span style={{ color: 'var(--text-muted)' }}>版本:</span> <code className="ml-2 text-xs font-mono" style={{ color: 'var(--text-secondary)' }}>{plugin.version}</code></div>
                      </div>

                      <div className="flex gap-2">
                        <button onClick={() => handleToggle(plugin.id)} className="flex-1 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all hover:scale-105" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: plugin.enabled ? 'var(--accent-warning)' : 'var(--accent-success)' }}>
                          {plugin.enabled ? <><PowerOff className="w-3 h-3 inline mr-1" />禁用</> : <><Power className="w-3 h-3 inline mr-1" />启用</>}
                        </button>
                        <button onClick={() => handleEdit(plugin)} className="p-1.5 rounded-lg transition-all hover:scale-110" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }} title="编辑">
                          <Edit2 className="w-3 h-3" />
                        </button>
                        <button onClick={() => handleDelete(plugin.id)} className="p-1.5 rounded-lg transition-all hover:scale-110" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }} title="删除">
                          <Trash2 className="w-3 h-3" />
                        </button>
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
                    {editingPlugin ? '编辑插件' : '添加插件'}
                  </h2>

                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>插件 ID *</label>
                      <input type="text" value={formData.id} onChange={(e) => setFormData({ ...formData, id: e.target.value })} className="w-full px-3 py-2 rounded-lg font-mono" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="例如: my-awesome-plugin" disabled={!!editingPlugin} />
                    </div>

                    <div>
                      <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>插件名称 *</label>
                      <input type="text" value={formData.name} onChange={(e) => setFormData({ ...formData, name: e.target.value })} className="w-full px-3 py-2 rounded-lg" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="例如: My Awesome Plugin" />
                    </div>

                    <div>
                      <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>版本 *</label>
                      <input type="text" value={formData.version} onChange={(e) => setFormData({ ...formData, version: e.target.value })} className="w-full px-3 py-2 rounded-lg font-mono" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder="例如: 1.0.0" />
                    </div>

                    <div>
                      <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>配置 JSON（可选）</label>
                      <textarea value={configJson} onChange={(e) => setConfigJson(e.target.value)} rows={6} className="w-full px-3 py-2 rounded-lg font-mono text-xs" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }} placeholder={'{\n  "key": "value"\n}'} />
                      <div className="text-xs mt-1" style={{ color: 'var(--text-muted)' }}>插件的自定义配置（JSON 格式）</div>
                    </div>

                    <div className="flex items-center gap-2">
                      <input type="checkbox" id="enabled-plugin" checked={formData.enabled} onChange={(e) => setFormData({ ...formData, enabled: e.target.checked })} className="w-4 h-4" />
                      <label htmlFor="enabled-plugin" className="text-sm" style={{ color: 'var(--text-secondary)' }}>启用此插件</label>
                    </div>
                  </div>

                  <div className="flex gap-3 mt-6">
                    <button onClick={handleSubmit} className="flex-1 px-4 py-2 rounded-lg font-semibold text-white" style={{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }}>
                      {editingPlugin ? '更新' : '添加'}
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
