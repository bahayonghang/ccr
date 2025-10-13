'use client';

import type { ConfigItem } from '@/lib/types';

interface RightSidebarProps {
  configs: ConfigItem[];
  currentFilter: string;
  onConfigClick: (name: string) => void;
}

export default function RightSidebar({ configs, currentFilter, onConfigClick }: RightSidebarProps) {
  // 根据当前筛选器过滤配置
  let filtered = configs;
  if (currentFilter === 'official_relay') {
    filtered = configs.filter(
      (c) => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay'
    );
  } else if (currentFilter === 'third_party_model') {
    filtered = configs.filter(
      (c) => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model'
    );
  } else if (currentFilter === 'uncategorized') {
    filtered = configs.filter((c) => !c.provider_type);
  }

  return (
    <aside
      className="rounded-xl p-5 h-fit sticky top-5 max-h-[calc(100vh-120px)] overflow-y-auto glass-effect"
      style={{
        border: '1px solid var(--border-color)',
        boxShadow: 'var(--shadow-small)',
      }}
    >
      <h2
        className="text-xs font-semibold uppercase tracking-wider mb-4"
        style={{ color: 'var(--text-secondary)' }}
      >
        配置目录
      </h2>
      <nav aria-label="配置导航">
        <ul className="space-y-2">
          {filtered.length === 0 ? (
            <li className="text-xs text-center py-2" style={{ color: 'var(--text-muted)' }}>
              当前分类下暂无配置
            </li>
          ) : (
            filtered.map((config) => (
              <li key={config.name}>
                <button
                  onClick={() => onConfigClick(config.name)}
                  className="w-full text-left px-3 py-2 rounded-lg text-sm font-medium transition-all relative overflow-hidden group hover:scale-105"
                  style={{
                    color: 'var(--text-secondary)',
                  }}
                  aria-label={`跳转到配置 ${config.name}`}
                >
                  {/* 左侧指示器 */}
                  <span
                    className="absolute left-0 top-0 w-0.5 h-full transition-transform origin-center scale-y-0 group-hover:scale-y-100"
                    style={{ background: 'var(--accent-primary)' }}
                    aria-hidden="true"
                  />

                  <span className="flex items-center space-x-2 group-hover:translate-x-1 transition-transform">
                    <span
                      className={`w-2 h-2 rounded-full ${
                        config.is_current
                          ? 'animate-pulse'
                          : config.is_default
                          ? ''
                          : 'opacity-0'
                      }`}
                      style={{
                        background: config.is_current
                          ? 'var(--accent-success)'
                          : config.is_default
                          ? 'var(--accent-warning)'
                          : 'transparent',
                        boxShadow: config.is_current ? '0 0 8px var(--glow-success)' : undefined,
                      }}
                      aria-label={config.is_current ? '当前配置' : config.is_default ? '默认配置' : undefined}
                    />
                    <span className="truncate">{config.name}</span>
                  </span>
                </button>
              </li>
            ))
          )}
        </ul>
      </nav>
    </aside>
  );
}

