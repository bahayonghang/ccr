'use client';

import { useState, useEffect } from 'react';
import { User, Plus, Edit2, Trash2 } from 'lucide-react';
import {
  listCodexProfiles,
  addCodexProfile,
  updateCodexProfile,
  deleteCodexProfile,
  listConfigs,
  getHistory
} from '@/lib/api/client';
import type { CodexProfile, CodexProfileRequest } from '@/lib/types';
import Navbar from '@/components/layout/Navbar';
import StatusHeader from '@/components/layout/StatusHeader';
import CollapsibleSidebar from '@/components/layout/CollapsibleSidebar';

export default function CodexProfileManagement() {
  const [profiles, setProfiles] = useState<CodexProfile[]>([]);
  const [loading, setLoading] = useState(true);
  const [currentConfig, setCurrentConfig] = useState<string>('');
  const [totalConfigs, setTotalConfigs] = useState(0);
  const [historyCount, setHistoryCount] = useState(0);
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingProfile, setEditingProfile] = useState<CodexProfile | null>(null);
  const [formData, setFormData] = useState<CodexProfileRequest>({
    model: '',
    approval_policy: '',
    sandbox_mode: '',
    model_reasoning_effort: '',
  });

  const loadProfiles = async () => {
    try {
      setLoading(true);
      const data = await listCodexProfiles();
      setProfiles(data);

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
      console.error('Failed to load Codex profiles:', err);
      alert('加载 Codex Profiles 失败');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadProfiles();
  }, []);

  const handleAdd = () => {
    setShowAddForm(true);
    setEditingProfile(null);
    setFormData({
      model: 'gpt-5',
      approval_policy: 'on-request',
      sandbox_mode: 'workspace-write',
      model_reasoning_effort: 'medium',
    });
  };

  const handleEdit = (profile: CodexProfile) => {
    setEditingProfile(profile);
    setShowAddForm(true);
    setFormData({
      model: profile.model || '',
      approval_policy: profile.approval_policy || '',
      sandbox_mode: profile.sandbox_mode || '',
      model_reasoning_effort: profile.model_reasoning_effort || '',
    });
  };

  const handleSubmit = async () => {
    const request: CodexProfileRequest = {
      model: formData.model || undefined,
      approval_policy: formData.approval_policy || undefined,
      sandbox_mode: formData.sandbox_mode || undefined,
      model_reasoning_effort: formData.model_reasoning_effort || undefined,
    };

    try {
      if (editingProfile) {
        await updateCodexProfile(editingProfile.name, request);
        alert('✓ Codex Profile 更新成功');
      } else {
        await addCodexProfile(request);
        alert('✓ Codex Profile 添加成功');
      }
      setShowAddForm(false);
      await loadProfiles();
    } catch (err) {
      alert(`操作失败: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  const handleDelete = async (name: string) => {
    if (!confirm(`确定删除 Codex Profile "${name}" 吗？`)) return;

    try {
      await deleteCodexProfile(name);
      alert('✓ Codex Profile 删除成功');
      await loadProfiles();
    } catch (err) {
      alert(`删除失败: ${err instanceof Error ? err.message : 'Unknown error'}`);
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
                <User className="w-6 h-6" style={{ color: 'var(--accent-primary)' }} />
                <h1 className="text-2xl font-bold" style={{ color: 'var(--text-primary)' }}>Codex Profile 管理</h1>
              </div>
              <button
                onClick={handleAdd}
                className="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2"
                style={{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }}
              >
                <Plus className="w-4 h-4" />
                添加 Profile
              </button>
            </div>

            <div className="mb-4 p-3 rounded-lg" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)' }}>
              <div className="text-sm space-y-1" style={{ color: 'var(--text-muted)' }}>
                <div><strong style={{ color: 'var(--text-primary)' }}>配置文件位置:</strong> <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }}>~/.codex/config.toml</code></div>
                <div><strong style={{ color: 'var(--text-primary)' }}>使用方法:</strong> <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }}>codex --profile dev</code></div>
              </div>
            </div>

            {loading ? (
              <div className="flex justify-center py-20">
                <div className="w-12 h-12 rounded-full border-4 border-transparent animate-spin" style={{ borderTopColor: 'var(--accent-primary)', borderRightColor: 'var(--accent-secondary)' }} />
              </div>
            ) : (
              <div className="space-y-3">
                {profiles.length === 0 ? (
                  <div className="text-center py-10" style={{ color: 'var(--text-muted)' }}>
                    暂无 Codex Profile 配置
                  </div>
                ) : (
                  profiles.map((profile) => (
                    <div key={profile.name} className="rounded-lg p-4" style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)' }}>
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <h3 className="text-lg font-bold font-mono" style={{ color: 'var(--text-primary)' }}>{profile.name}</h3>
                          </div>
                          <div className="grid grid-cols-2 gap-3 text-sm">
                            {profile.model && (
                              <div>
                                <span style={{ color: 'var(--text-muted)' }}>模型:</span>
                                <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }}>{profile.model}</code>
                              </div>
                            )}
                            {profile.approval_policy && (
                              <div>
                                <span style={{ color: 'var(--text-muted)' }}>批准策略:</span>
                                <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }}>{profile.approval_policy}</code>
                              </div>
                            )}
                            {profile.sandbox_mode && (
                              <div>
                                <span style={{ color: 'var(--text-muted)' }}>沙盒模式:</span>
                                <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }}>{profile.sandbox_mode}</code>
                              </div>
                            )}
                            {profile.model_reasoning_effort && (
                              <div>
                                <span style={{ color: 'var(--text-muted)' }}>推理深度:</span>
                                <code className="ml-2 px-2 py-1 rounded font-mono" style={{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }}>{profile.model_reasoning_effort}</code>
                              </div>
                            )}
                          </div>
                        </div>
                        <div className="flex gap-2">
                          <button onClick={() => handleEdit(profile)} className="p-2 rounded-lg transition-all hover:scale-110" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }} title="编辑">
                            <Edit2 className="w-4 h-4" />
                          </button>
                          <button onClick={() => handleDelete(profile.name)} className="p-2 rounded-lg transition-all hover:scale-110" style={{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }} title="删除">
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
                    {editingProfile ? '编辑 Codex Profile' : '添加 Codex Profile'}
                  </h2>

                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>模型 (Model)</label>
                      <select
                        value={formData.model}
                        onChange={(e) => setFormData({ ...formData, model: e.target.value })}
                        className="w-full px-3 py-2 rounded-lg"
                        style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }}
                      >
                        <option value="">-- 选择模型 --</option>
                        <option value="gpt-5">gpt-5</option>
                        <option value="gpt-5-codex">gpt-5-codex</option>
                        <option value="gpt-4">gpt-4</option>
                        <option value="ollama">ollama (本地模型)</option>
                      </select>
                    </div>

                    <div>
                      <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>批准策略 (Approval Policy)</label>
                      <select
                        value={formData.approval_policy}
                        onChange={(e) => setFormData({ ...formData, approval_policy: e.target.value })}
                        className="w-full px-3 py-2 rounded-lg"
                        style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }}
                      >
                        <option value="">-- 选择批准策略 --</option>
                        <option value="auto">auto - 自动执行（工作区内）</option>
                        <option value="on-request">on-request - 按需批准</option>
                        <option value="read-only">read-only - 仅读取</option>
                        <option value="full-access">full-access - 完全访问</option>
                      </select>
                      <div className="text-xs mt-1" style={{ color: 'var(--text-muted)' }}>
                        控制 Codex 的执行权限级别
                      </div>
                    </div>

                    <div>
                      <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>沙盒模式 (Sandbox Mode)</label>
                      <select
                        value={formData.sandbox_mode}
                        onChange={(e) => setFormData({ ...formData, sandbox_mode: e.target.value })}
                        className="w-full px-3 py-2 rounded-lg"
                        style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }}
                      >
                        <option value="">-- 选择沙盒模式 --</option>
                        <option value="workspace-write">workspace-write - 可写工作区</option>
                        <option value="workspace-read">workspace-read - 只读工作区</option>
                        <option value="full">full - 完整权限</option>
                      </select>
                      <div className="text-xs mt-1" style={{ color: 'var(--text-muted)' }}>
                        控制文件系统访问权限
                      </div>
                    </div>

                    <div>
                      <label className="block text-sm font-semibold mb-1" style={{ color: 'var(--text-secondary)' }}>推理深度 (Reasoning Effort)</label>
                      <select
                        value={formData.model_reasoning_effort}
                        onChange={(e) => setFormData({ ...formData, model_reasoning_effort: e.target.value })}
                        className="w-full px-3 py-2 rounded-lg"
                        style={{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }}
                      >
                        <option value="">-- 选择推理深度 --</option>
                        <option value="low">low - 低（快速）</option>
                        <option value="medium">medium - 中等</option>
                        <option value="high">high - 高（深度思考）</option>
                      </select>
                      <div className="text-xs mt-1" style={{ color: 'var(--text-muted)' }}>
                        控制模型的推理复杂度
                      </div>
                    </div>
                  </div>

                  <div className="flex gap-3 mt-6">
                    <button onClick={handleSubmit} className="flex-1 px-4 py-2 rounded-lg font-semibold text-white" style={{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }}>
                      {editingProfile ? '更新' : '添加'}
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
