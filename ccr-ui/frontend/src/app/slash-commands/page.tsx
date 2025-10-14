'use client';

import { useState, useEffect, useMemo } from 'react';
import { Command, Plus, Edit2, Trash2, Power, PowerOff, Search, X, Folder, Home } from 'lucide-react';
import { 
  listSlashCommands,
  addSlashCommand, 
  updateSlashCommand, 
  deleteSlashCommand, 
  toggleSlashCommand,
  listConfigs,
  getHistory
} from '@/lib/api/client';
import type { SlashCommand, SlashCommandRequest } from '@/lib/types';
import Navbar from '@/components/layout/Navbar';
import StatusHeader from '@/components/layout/StatusHeader';
import CollapsibleSidebar from '@/components/layout/CollapsibleSidebar';

export default function SlashCommandsManagement() {
  const [commands, setCommands] = useState<SlashCommand[]>([]);
  const [folders, setFolders] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [currentConfig, setCurrentConfig] = useState<string>('');
  const [totalConfigs, setTotalConfigs] = useState(0);
  const [historyCount, setHistoryCount] = useState(0);
  const [selectedFolder, setSelectedFolder] = useState<string>(''); // '' = all, '__root__' = root only
  const [searchQuery, setSearchQuery] = useState('');
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingCommand, setEditingCommand] = useState<SlashCommand | null>(null);
  const [formData, setFormData] = useState<SlashCommandRequest>({
    name: '',
    description: '',
    command: '',
    args: [],
    disabled: false,
  });

  const loadCommands = async () => {
    try {
      setLoading(true);
      const data = await listSlashCommands();
      setCommands(data.commands);
      setFolders(data.folders || []);

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
      console.error('Failed to load slash commands:', err);
      alert('加载 Slash Commands 失败');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadCommands();
  }, []);

  // 过滤和排序后的命令列表
  const filteredAndSortedCommands = useMemo(() => {
    let filtered = commands;

    // 按文件夹过滤
    if (selectedFolder === '__root__') {
      filtered = commands.filter(cmd => !cmd.folder || cmd.folder === '');
    } else if (selectedFolder) {
      filtered = commands.filter(cmd => cmd.folder === selectedFolder);
    }

    // 搜索过滤
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(cmd => 
        cmd.name.toLowerCase().includes(query) ||
        cmd.description.toLowerCase().includes(query) ||
        cmd.command.toLowerCase().includes(query)
      );
    }

    // 按名称排序
    return filtered.sort((a, b) => a.name.localeCompare(b.name));
  }, [commands, selectedFolder, searchQuery]);

  // 统计信息
  const stats = useMemo(() => {
    const rootCount = commands.filter(c => !c.folder || c.folder === '').length;
    const folderCounts: Record<string, number> = {};
    folders.forEach(f => {
      folderCounts[f] = commands.filter(c => c.folder === f).length;
    });
    return { rootCount, folderCounts, total: commands.length };
  }, [commands, folders]);

  const handleAdd = () => {
    setShowAddForm(true);
    setEditingCommand(null);
    setFormData({
      name: '',
      description: '',
      command: '',
      args: [],
      disabled: false,
    });
  };

  const handleEdit = (cmd: SlashCommand) => {
    setEditingCommand(cmd);
    setShowAddForm(true);
    setFormData({
      name: cmd.name,
      description: cmd.description,
      command: cmd.command,
      args: cmd.args || [],
      disabled: cmd.disabled || false,
    });
  };

  const handleSubmit = async () => {
    if (!formData.name || !formData.command) {
      alert('请填写必填字段');
      return;
    }

    const request: SlashCommandRequest = {
      ...formData,
      args: formData.args && formData.args.length > 0 ? formData.args : undefined,
    };

    try {
      if (editingCommand) {
        await updateSlashCommand(editingCommand.name, request);
      } else {
        await addSlashCommand(request);
      }
      setShowAddForm(false);
      setEditingCommand(null);
      loadCommands();
    } catch (err) {
      console.error('操作失败:', err);
      alert('操作失败');
    }
  };

  const handleDelete = async (name: string) => {
    if (!confirm(`确定要删除命令 "${name}" 吗？`)) return;
    
    try {
      await deleteSlashCommand(name);
      loadCommands();
    } catch (err) {
      console.error('删除失败:', err);
      alert('删除失败');
    }
  };

  const handleToggle = async (name: string) => {
    try {
      await toggleSlashCommand(name);
      loadCommands();
    } catch (err) {
      console.error('切换状态失败:', err);
      alert('切换状态失败');
    }
  };

  return (
    <div style={{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }}>
      {/* 顶部导航和状态栏 - 横跨所有栏目 */}
      <div className="mb-6">
        <Navbar />
        <StatusHeader currentConfig={currentConfig} totalConfigs={totalConfigs} historyCount={historyCount} />
      </div>

      <div style={{ display: 'flex', gap: '0' }}>
        {/* 左侧大目录 - CollapsibleSidebar */}
        <CollapsibleSidebar />
        
        {/* 小的子目录导航 */}
        <div style={{ 
          width: '180px',
          background: 'var(--bg-secondary)', 
          borderRight: '1px solid var(--border-color)',
          padding: '12px 8px',
          overflowY: 'auto',
          height: 'calc(100vh - 40px)',
          flexShrink: 0
        }}>
          <h4 style={{ 
            color: 'var(--text-muted)', 
            fontSize: '11px', 
            fontWeight: '600',
            marginBottom: '8px',
            marginLeft: '8px',
            textTransform: 'uppercase',
            letterSpacing: '0.5px'
          }}>
            文件夹
          </h4>
          
          {/* 全部 */}
          <div
            onClick={() => setSelectedFolder('')}
            style={{
              padding: '6px 8px',
              borderRadius: '6px',
              cursor: 'pointer',
              marginBottom: '4px',
              background: selectedFolder === '' ? 'var(--accent-primary)' : 'transparent',
              color: selectedFolder === '' ? '#fff' : 'var(--text-primary)',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              transition: 'all 0.2s',
              fontSize: '13px'
            }}
            onMouseEnter={(e) => {
              if (selectedFolder !== '') e.currentTarget.style.background = 'var(--bg-tertiary)';
            }}
            onMouseLeave={(e) => {
              if (selectedFolder !== '') e.currentTarget.style.background = 'transparent';
            }}
          >
            <Folder className="w-3.5 h-3.5" />
            <span className="flex-1">全部</span>
            <span style={{ fontSize: '11px', opacity: 0.7 }}>{stats.total}</span>
          </div>

          {/* 根目录 */}
          <div
            onClick={() => setSelectedFolder('__root__')}
            style={{
              padding: '6px 8px',
              borderRadius: '6px',
              cursor: 'pointer',
              marginBottom: '4px',
              background: selectedFolder === '__root__' ? 'var(--accent-primary)' : 'transparent',
              color: selectedFolder === '__root__' ? '#fff' : 'var(--text-primary)',
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              transition: 'all 0.2s',
              fontSize: '13px'
            }}
            onMouseEnter={(e) => {
              if (selectedFolder !== '__root__') e.currentTarget.style.background = 'var(--bg-tertiary)';
            }}
            onMouseLeave={(e) => {
              if (selectedFolder !== '__root__') e.currentTarget.style.background = 'transparent';
            }}
          >
            <Home className="w-3.5 h-3.5" />
            <span className="flex-1">根目录</span>
            <span style={{ fontSize: '11px', opacity: 0.7 }}>{stats.rootCount}</span>
          </div>

          {/* 子文件夹 */}
          {folders.map(folder => (
            <div
              key={folder}
              onClick={() => setSelectedFolder(folder)}
              style={{
                padding: '6px 8px',
                borderRadius: '6px',
                cursor: 'pointer',
                marginBottom: '4px',
                background: selectedFolder === folder ? 'var(--accent-primary)' : 'transparent',
                color: selectedFolder === folder ? '#fff' : 'var(--text-primary)',
                display: 'flex',
                alignItems: 'center',
                gap: '6px',
                transition: 'all 0.2s',
                fontSize: '13px'
              }}
              onMouseEnter={(e) => {
                if (selectedFolder !== folder) e.currentTarget.style.background = 'var(--bg-tertiary)';
              }}
              onMouseLeave={(e) => {
                if (selectedFolder !== folder) e.currentTarget.style.background = 'transparent';
              }}
            >
              <Folder className="w-3.5 h-3.5" />
              <span className="flex-1">{folder}</span>
              <span style={{ fontSize: '11px', opacity: 0.7 }}>{stats.folderCounts[folder] || 0}</span>
            </div>
          ))}
        </div>

        {/* 主内容区域 */}
        <div style={{ flex: 1, minWidth: 0 }}>
          <div className="max-w-[1600px] mx-auto">
            {/* 标题和搜索 */}
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-4">
                <h2 className="text-2xl font-bold" style={{ color: 'var(--text-primary)' }}>
                  <Command className="inline-block w-7 h-7 mr-2" />
                  Slash Commands 管理
                </h2>
                <span className="px-3 py-1 rounded-full text-sm font-medium" style={{ 
                  background: 'var(--accent-primary)', 
                  color: '#fff' 
                }}>
                  {filteredAndSortedCommands.length}/{stats.total}
                </span>
              </div>
              <button
                onClick={handleAdd}
                className="px-4 py-2 rounded-lg font-medium transition-all hover:scale-105"
                style={{ 
                  background: 'var(--accent-primary)', 
                  color: '#fff' 
                }}
              >
                <Plus className="inline-block w-5 h-5 mr-2" />
                添加 Command
              </button>
            </div>

            {/* 搜索框 */}
            <div className="mb-4">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5" style={{ color: 'var(--text-muted)' }} />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="搜索命令名称、描述或命令..."
                  className="w-full pl-11 pr-10 py-3 rounded-lg transition-all focus:outline-none focus:ring-2"
                  style={{ 
                    background: 'var(--bg-tertiary)', 
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-primary)',
                    '--tw-ring-color': 'var(--accent-primary)'
                  } as any}
                />
                {searchQuery && (
                  <button
                    onClick={() => setSearchQuery('')}
                    className="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 rounded-lg hover:bg-opacity-20 transition-all"
                    style={{ color: 'var(--text-muted)' }}
                  >
                    <X className="w-4 h-4" />
                  </button>
                )}
              </div>
              {searchQuery && (
                <p className="mt-2 text-sm" style={{ color: 'var(--text-muted)' }}>
                  找到 <span style={{ color: 'var(--accent-primary)', fontWeight: 'bold' }}>{filteredAndSortedCommands.length}</span> 个匹配的命令
                </p>
              )}
            </div>

            {/* Commands 列表 */}
            <div className="space-y-4">
              {loading ? (
                <div className="text-center py-10" style={{ color: 'var(--text-muted)' }}>
                  加载中...
                </div>
              ) : commands.length === 0 ? (
                <div className="text-center py-10" style={{ color: 'var(--text-muted)' }}>
                  暂无 Slash Commands 配置
                </div>
              ) : filteredAndSortedCommands.length === 0 ? (
                <div className="text-center py-10" style={{ color: 'var(--text-muted)' }}>
                  <Search className="w-12 h-12 mx-auto mb-3 opacity-50" />
                  <p>未找到匹配的命令</p>
                  <p className="text-sm mt-2">尝试使用其他关键词搜索或切换文件夹</p>
                </div>
              ) : (
                filteredAndSortedCommands.map((cmd) => (
                  <div
                    key={cmd.name}
                    className="p-6 rounded-xl transition-all hover:scale-[1.01]"
                    style={{ 
                      background: 'var(--bg-secondary)', 
                      border: '1px solid var(--border-color)' 
                    }}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-2">
                          <h3 className="text-xl font-semibold" style={{ color: 'var(--text-primary)' }}>
                            {cmd.name}
                          </h3>
                          {cmd.folder && (
                            <span className="px-2 py-1 rounded text-xs font-medium" style={{ 
                              background: 'var(--bg-tertiary)', 
                              color: 'var(--text-muted)' 
                            }}>
                              📁 {cmd.folder}
                            </span>
                          )}
                          {cmd.disabled && (
                            <span className="px-2 py-1 rounded text-xs font-medium" style={{ 
                              background: '#fef3c7', 
                              color: '#92400e' 
                            }}>
                              已禁用
                            </span>
                          )}
                        </div>
                        <p className="mb-2" style={{ color: 'var(--text-secondary)', fontSize: '14px' }}>
                          <strong>Command:</strong> {cmd.command}
                        </p>
                        <p className="text-sm" style={{ color: 'var(--text-muted)' }}>
                          {cmd.description}
                        </p>
                      </div>
                      <div className="flex gap-2 ml-4">
                        <button
                          onClick={() => handleToggle(cmd.name)}
                          className="p-2 rounded-lg transition-all hover:scale-110"
                          style={{ 
                            background: cmd.disabled ? '#fef3c7' : '#d1fae5',
                            color: cmd.disabled ? '#92400e' : '#065f46'
                          }}
                          title={cmd.disabled ? "启用" : "禁用"}
                        >
                          {cmd.disabled ? <PowerOff className="w-5 h-5" /> : <Power className="w-5 h-5" />}
                        </button>
                        <button
                          onClick={() => handleEdit(cmd)}
                          className="p-2 rounded-lg transition-all hover:scale-110"
                          style={{ 
                            background: 'var(--bg-tertiary)', 
                            color: 'var(--accent-primary)' 
                          }}
                          title="编辑"
                        >
                          <Edit2 className="w-5 h-5" />
                        </button>
                        <button
                          onClick={() => handleDelete(cmd.name)}
                          className="p-2 rounded-lg transition-all hover:scale-110"
                          style={{ 
                            background: '#fee2e2', 
                            color: '#991b1b' 
                          }}
                          title="删除"
                        >
                          <Trash2 className="w-5 h-5" />
                        </button>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>

      {/* 添加/编辑表单弹窗 */}
      {showAddForm && (
        <div
          className="fixed inset-0 flex items-center justify-center z-50"
          style={{ background: 'rgba(0, 0, 0, 0.5)' }}
          onClick={() => setShowAddForm(false)}
        >
          <div
            className="p-8 rounded-2xl w-full max-w-2xl max-h-[80vh] overflow-y-auto"
            style={{ background: 'var(--bg-secondary)' }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="text-2xl font-bold mb-6" style={{ color: 'var(--text-primary)' }}>
              {editingCommand ? '编辑 Command' : '添加 Command'}
            </h3>
            <div className="space-y-4">
              <div>
                <label className="block mb-2 font-medium" style={{ color: 'var(--text-secondary)' }}>
                  名称 *
                </label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg focus:outline-none focus:ring-2"
                  style={{ 
                    background: 'var(--bg-tertiary)', 
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-primary)',
                    '--tw-ring-color': 'var(--accent-primary)'
                  } as any}
                />
              </div>
              <div>
                <label className="block mb-2 font-medium" style={{ color: 'var(--text-secondary)' }}>
                  Command *
                </label>
                <input
                  type="text"
                  value={formData.command}
                  onChange={(e) => setFormData({ ...formData, command: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg focus:outline-none focus:ring-2"
                  style={{ 
                    background: 'var(--bg-tertiary)', 
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-primary)',
                    '--tw-ring-color': 'var(--accent-primary)'
                  } as any}
                />
              </div>
              <div>
                <label className="block mb-2 font-medium" style={{ color: 'var(--text-secondary)' }}>
                  描述
                </label>
                <textarea
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  rows={4}
                  className="w-full px-4 py-2 rounded-lg focus:outline-none focus:ring-2"
                  style={{ 
                    background: 'var(--bg-tertiary)', 
                    border: '1px solid var(--border-color)',
                    color: 'var(--text-primary)',
                    '--tw-ring-color': 'var(--accent-primary)'
                  } as any}
                />
              </div>
            </div>
            <div className="flex gap-3 mt-6">
              <button
                onClick={handleSubmit}
                className="flex-1 px-6 py-3 rounded-lg font-medium transition-all hover:scale-105"
                style={{ 
                  background: 'var(--accent-primary)', 
                  color: '#fff' 
                }}
              >
                {editingCommand ? '保存' : '添加'}
              </button>
              <button
                onClick={() => setShowAddForm(false)}
                className="flex-1 px-6 py-3 rounded-lg font-medium transition-all hover:scale-105"
                style={{ 
                  background: 'var(--bg-tertiary)', 
                  color: 'var(--text-secondary)' 
                }}
              >
                取消
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
