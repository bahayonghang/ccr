'use client';

import { useState, useEffect } from 'react';
import { AlertCircle, FileText, Building2, User } from 'lucide-react';
import { listConfigs, switchConfig, validateConfigs, getHistory } from '@/lib/api/client';
import type { ConfigItem, HistoryEntry } from '@/lib/types';
import Navbar from '@/components/layout/Navbar';
import LeftSidebar from '@/components/sidebar/LeftSidebar';
import RightSidebar from '@/components/sidebar/RightSidebar';
import CollapsibleSidebar from '@/components/layout/CollapsibleSidebar';
import HistoryList from '@/components/history/HistoryList';

type FilterType = 'all' | 'official_relay' | 'third_party_model' | 'uncategorized';

export default function ConfigManagement() {
  const [configs, setConfigs] = useState<ConfigItem[]>([]);
  const [currentConfig, setCurrentConfig] = useState<string>('');
  const [historyCount, setHistoryCount] = useState(0);
  const [historyEntries, setHistoryEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentFilter, setCurrentFilter] = useState<FilterType>('all');
  const [activeTab, setActiveTab] = useState<'configs' | 'history'>('configs');

  const loadConfigs = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await listConfigs();
      setConfigs(data.configs);
      setCurrentConfig(data.current_config);

      // 加载历史记录数量
      const historyData = await getHistory();
      setHistoryCount(historyData.total);
      setHistoryEntries(historyData.entries);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load configs');
      console.error('Error loading configs:', err);
    } finally {
      setLoading(false);
    }
  };

  const loadHistory = async () => {
    try {
      setHistoryLoading(true);
      const historyData = await getHistory();
      setHistoryEntries(historyData.entries);
      setHistoryCount(historyData.total);
    } catch (err) {
      console.error('Failed to load history:', err);
    } finally {
      setHistoryLoading(false);
    }
  };

  useEffect(() => {
    loadConfigs();
  }, []);

  useEffect(() => {
    if (activeTab === 'history') {
      loadHistory();
    }
  }, [activeTab]);

  const handleSwitch = async (configName: string) => {
    if (!confirm(`确定切换到配置 "${configName}" 吗？`)) return;

    try {
      await switchConfig(configName);
      alert(`✓ 成功切换到配置 "${configName}"`);
      await loadConfigs();
    } catch (err) {
      alert(`切换失败: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  const handleValidate = async () => {
    try {
      await validateConfigs();
      alert('✓ 配置验证通过');
    } catch (err) {
      alert(`验证失败: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  const handleConfigClick = (name: string) => {
    const element = document.getElementById(`config-${name}`);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'center' });
      // 闪烁效果
      element.style.transform = 'scale(1.02)';
      setTimeout(() => {
        element.style.transform = '';
      }, 300);
    }
  };

  // 根据当前筛选器过滤配置
  let filteredConfigs = configs;
  if (currentFilter === 'official_relay') {
    filteredConfigs = configs.filter(
      (c) => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay'
    );
  } else if (currentFilter === 'third_party_model') {
    filteredConfigs = configs.filter(
      (c) => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model'
    );
  } else if (currentFilter === 'uncategorized') {
    filteredConfigs = configs.filter((c) => !c.provider_type);
  }

  return (
    <div style={{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }}>
      <div className="max-w-[1800px] mx-auto">
        {/* 导航栏 */}
        <Navbar
          onRefresh={loadConfigs}
          onImport={() => alert('导入功能开发中')}
          onExport={() => alert('导出功能开发中')}
          onAdd={() => alert('添加功能开发中')}
        />

        {/* 四列布局：导航 + 状态侧边栏 + 主内容 + 配置导航 */}
        <div className="grid grid-cols-[auto_260px_1fr_220px] gap-4">
          {/* 可折叠导航 */}
          <CollapsibleSidebar />

          {/* 左侧状态侧边栏 */}
          <LeftSidebar
            currentConfig={currentConfig}
            totalConfigs={configs.length}
            historyCount={historyCount}
            onValidate={handleValidate}
            onClean={() => alert('清理备份功能开发中')}
          />

          {/* 主内容区 */}
          <main
            className="rounded-xl p-6 glass-effect"
            style={{
              border: '1px solid var(--border-color)',
              boxShadow: 'var(--shadow-small)',
            }}
          >
            {/* Tab 导航 */}
            <div
              className="flex gap-1.5 mb-5 p-1 rounded-lg"
              style={{ background: 'var(--bg-tertiary)' }}
              role="tablist"
            >
              <button
                onClick={() => setActiveTab('configs')}
                className={`flex-1 py-2 px-4 rounded-md text-sm font-semibold transition-all ${
                  activeTab === 'configs' ? 'text-white' : ''
                }`}
                style={{
                  background: activeTab === 'configs' ? 'var(--accent-primary)' : 'transparent',
                  color: activeTab === 'configs' ? 'white' : 'var(--text-secondary)',
                }}
                role="tab"
                aria-selected={activeTab === 'configs'}
                aria-controls="configs-panel"
              >
                配置列表
              </button>
              <button
                onClick={() => setActiveTab('history')}
                className={`flex-1 py-2 px-4 rounded-md text-sm font-semibold transition-all ${
                  activeTab === 'history' ? 'text-white' : ''
                }`}
                style={{
                  background: activeTab === 'history' ? 'var(--accent-primary)' : 'transparent',
                  color: activeTab === 'history' ? 'white' : 'var(--text-secondary)',
                }}
                role="tab"
                aria-selected={activeTab === 'history'}
                aria-controls="history-panel"
              >
                历史记录
              </button>
            </div>

            {/* 配置列表 Tab */}
            {activeTab === 'configs' && (
              <div id="configs-panel" role="tabpanel">
                {/* 筛选按钮 */}
                <div
                  className="flex gap-2 mb-5 p-2 rounded-lg"
                  style={{
                    background: 'var(--bg-tertiary)',
                    border: '1px solid var(--border-color)',
                  }}
                  role="group"
                  aria-label="配置筛选"
                >
                  {[
                    { type: 'all' as FilterType, label: '📋 全部配置' },
                    { type: 'official_relay' as FilterType, label: '🔄 官方中转' },
                    { type: 'third_party_model' as FilterType, label: '🤖 第三方模型' },
                    { type: 'uncategorized' as FilterType, label: '❓ 未分类' },
                  ].map(({ type, label }) => (
                    <button
                      key={type}
                      onClick={() => setCurrentFilter(type)}
                      className={`flex-1 py-2.5 px-4 rounded-lg text-sm font-semibold transition-all hover:scale-105 ${
                        currentFilter === type ? 'text-white shadow-lg' : ''
                      }`}
                      style={{
                        background:
                          currentFilter === type
                            ? 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))'
                            : 'transparent',
                        border: `1px solid ${currentFilter === type ? 'var(--accent-primary)' : 'var(--border-color)'}`,
                        color: currentFilter === type ? 'white' : 'var(--text-secondary)',
                        boxShadow: currentFilter === type ? '0 0 15px var(--glow-primary)' : undefined,
                      }}
                      aria-pressed={currentFilter === type}
                    >
                      {label}
                    </button>
                  ))}
                </div>

                {/* 加载状态 */}
                {loading && (
                  <div className="flex items-center justify-center py-20" role="status">
                    <div
                      className="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
                      style={{
                        borderTopColor: 'var(--accent-primary)',
                        borderRightColor: 'var(--accent-secondary)',
                      }}
                      aria-label="加载中"
                    />
                    <span className="sr-only">加载配置中...</span>
                  </div>
                )}

                {/* 错误状态 */}
                {error && (
                  <div
                    className="rounded-lg p-4 flex items-center space-x-2"
                    style={{
                      background: 'rgba(239, 68, 68, 0.1)',
                      border: '1px solid var(--accent-danger)',
                    }}
                    role="alert"
                  >
                    <AlertCircle style={{ color: 'var(--accent-danger)' }} aria-hidden="true" />
                    <span style={{ color: 'var(--accent-danger)' }}>Error: {error}</span>
                  </div>
                )}

                {/* 配置列表 */}
                {!loading && !error && (
                  <div className="space-y-3.5">
                    {filteredConfigs.length === 0 ? (
                      <div className="text-center py-10" style={{ color: 'var(--text-muted)' }}>
                        当前分类下暂无配置
                      </div>
                    ) : (
                      filteredConfigs.map((config) => (
                        <ConfigCard
                          key={config.name}
                          config={config}
                          onSwitch={handleSwitch}
                        />
                      ))
                    )}
                  </div>
                )}
              </div>
            )}

            {/* 历史记录 Tab */}
            {activeTab === 'history' && (
              <div id="history-panel" role="tabpanel">
                <HistoryList entries={historyEntries} loading={historyLoading} />
              </div>
            )}
          </main>

          {/* 右侧导航 */}
          <RightSidebar
            configs={configs}
            currentFilter={currentFilter}
            onConfigClick={handleConfigClick}
          />
        </div>
      </div>
    </div>
  );
}

function ConfigCard({ config, onSwitch }: { config: ConfigItem; onSwitch: (name: string) => void }) {
  // Provider 类型徽章
  const getProviderTypeBadge = () => {
    if (!config.provider_type) return null;

    const typeMap: Record<string, { text: string; class: string }> = {
      OfficialRelay: { text: '🔄 官方中转', class: 'official-relay' },
      official_relay: { text: '🔄 官方中转', class: 'official-relay' },
      ThirdPartyModel: { text: '🤖 第三方模型', class: 'third-party-model' },
      third_party_model: { text: '🤖 第三方模型', class: 'third-party-model' },
    };

    const type = typeMap[config.provider_type];
    if (!type) return null;

    return (
      <span
        className="inline-block px-2.5 py-0.5 rounded-lg text-xs font-semibold uppercase tracking-wide mr-2"
        style={{
          background: type.class === 'official-relay'
            ? 'rgba(59, 130, 246, 0.2)'
            : 'rgba(168, 85, 247, 0.2)',
          color: type.class === 'official-relay' ? '#3b82f6' : '#a855f7',
          border: `1px solid ${type.class === 'official-relay' ? 'rgba(59, 130, 246, 0.4)' : 'rgba(168, 85, 247, 0.4)'}`,
        }}
      >
        {type.text}
      </span>
    );
  };

  return (
    <article
      id={`config-${config.name}`}
      className={`rounded-lg p-4 transition-all hover:scale-[1.01] ${
        config.is_current ? 'ring-2' : ''
      }`}
      style={{
        background: config.is_current
          ? 'linear-gradient(135deg, rgba(139, 92, 246, 0.1), rgba(168, 85, 247, 0.1))'
          : 'var(--bg-tertiary)',
        border: `1px solid ${config.is_current ? 'var(--accent-primary)' : 'var(--border-color)'}`,
        boxShadow: config.is_current ? '0 0 20px var(--glow-primary)' : undefined,
      }}
    >
      {/* 头部 */}
      <header className="mb-3">
        <h3 className="flex items-center flex-wrap gap-2 mb-2">
          {getProviderTypeBadge()}
          <span className="text-base font-bold font-mono tracking-wide" style={{ color: 'var(--text-primary)' }}>
            {config.name}
          </span>
          {config.is_current && (
            <span
              className="px-2 py-0.5 rounded-lg text-xs font-semibold uppercase"
              style={{ background: 'var(--accent-success)', color: 'white' }}
            >
              当前
            </span>
          )}
          {config.is_default && (
            <span
              className="px-2 py-0.5 rounded-lg text-xs font-semibold uppercase"
              style={{ background: 'var(--accent-warning)', color: 'white' }}
            >
              默认
            </span>
          )}
        </h3>

        {/* 描述 */}
        <div
          className="flex items-center gap-1.5 p-2 px-3 rounded-md mb-2.5 transition-all hover:translate-x-0.5"
          style={{
            background: 'rgba(139, 92, 246, 0.08)',
            borderLeft: '3px solid var(--accent-primary)',
          }}
        >
          <FileText className="w-3.5 h-3.5 flex-shrink-0" style={{ opacity: 0.8 }} aria-hidden="true" />
          <span className="text-xs font-medium leading-relaxed" style={{ color: 'var(--text-secondary)' }}>
            {config.description || '无描述'}
          </span>
        </div>

        {/* Provider 信息 */}
        {config.provider && (
          <div className="flex flex-wrap gap-3 py-2">
            <div
              className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs transition-all"
              style={{
                background: 'var(--bg-secondary)',
                border: '1px solid var(--border-color)',
              }}
            >
              <Building2 className="w-3 h-3" aria-hidden="true" />
              <span style={{ color: 'var(--text-muted)' }}>提供商:</span>
              <span className="font-semibold font-mono" style={{ color: 'var(--accent-secondary)' }}>
                {config.provider}
              </span>
            </div>
            {config.account && (
              <div
                className="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs transition-all"
                style={{
                  background: 'var(--bg-secondary)',
                  border: '1px solid var(--border-color)',
                }}
              >
                <User className="w-3 h-3" aria-hidden="true" />
                <span style={{ color: 'var(--text-muted)' }}>账号:</span>
                <span className="font-semibold font-mono" style={{ color: 'var(--accent-success)' }}>
                  {config.account}
                </span>
              </div>
            )}
          </div>
        )}

        {/* 标签 */}
        {config.tags && config.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-2">
            {config.tags.map((tag) => (
              <span
                key={tag}
                className="px-2 py-0.5 rounded-md text-xs transition-all"
                style={{
                  background: 'var(--bg-secondary)',
                  border: '1px solid var(--border-color)',
                  color: 'var(--text-muted)',
                }}
              >
                {tag}
              </span>
            ))}
          </div>
        )}
      </header>

      {/* 详细信息 */}
      <div className="grid grid-cols-2 gap-2.5 mb-3">
        <DetailField label="Base URL" value={config.base_url} />
        <DetailField label="Auth Token" value={config.auth_token} />
        {config.model && <DetailField label="Model" value={config.model} />}
        {config.small_fast_model && <DetailField label="Small Fast Model" value={config.small_fast_model} />}
      </div>

      {/* 操作按钮 */}
      <div className="flex gap-2 flex-wrap">
        {!config.is_current && (
          <button
            onClick={() => onSwitch(config.name)}
            className="px-3 py-1.5 rounded-lg text-xs font-semibold transition-all text-white hover:scale-105"
            style={{
              background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
              boxShadow: '0 0 20px var(--glow-primary)',
            }}
          >
            切换
          </button>
        )}
        <button
          className="px-3 py-1.5 rounded-lg text-xs font-semibold transition-all hover:scale-105"
          style={{
            background: 'var(--bg-tertiary)',
            color: 'var(--text-primary)',
            border: '1px solid var(--border-color)',
          }}
        >
          编辑
        </button>
        {!config.is_current && !config.is_default && (
          <button
            className="px-3 py-1.5 rounded-lg text-xs font-semibold transition-all text-white hover:scale-105"
            style={{
              background: 'var(--accent-danger)',
              boxShadow: '0 0 20px var(--glow-danger)',
            }}
          >
            删除
          </button>
        )}
      </div>
    </article>
  );
}

function DetailField({ label, value }: { label: string; value: string }) {
  return (
    <div
      className="rounded-md p-2"
      style={{
        background: 'var(--bg-secondary)',
        border: '1px solid var(--border-color)',
      }}
    >
      <div
        className="text-[10px] uppercase tracking-wide font-semibold mb-0.5"
        style={{ color: 'var(--text-muted)' }}
      >
        {label}
      </div>
      <div
        className="text-xs font-mono truncate"
        style={{ color: 'var(--text-primary)' }}
        title={value}
      >
        {value}
      </div>
    </div>
  );
}

