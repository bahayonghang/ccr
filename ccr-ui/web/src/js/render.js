import { appState } from './state.js';
import { escapeHtml } from './utils.js';

// Helper to check platform active state
function isCodexActive() {
    return appState.platformInfo.mode === 'unified' && appState.platformInfo.currentPlatform === 'codex';
}

export function renderConfigs() {
    const container = document.getElementById('configsList');
    if (appState.allConfigs.length === 0) {
        container.innerHTML = '<div style="text-align: center; color: var(--text-muted); padding: 40px;">暂无配置</div>';
        return;
    }

    let filtered = appState.allConfigs;
    if (appState.currentFilter === 'official_relay') {
        filtered = appState.allConfigs.filter(c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay');
    } else if (appState.currentFilter === 'third_party_model') {
        filtered = appState.allConfigs.filter(c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model');
    } else if (appState.currentFilter === 'uncategorized') {
        filtered = appState.allConfigs.filter(c => !c.provider_type);
    }

    if (filtered.length === 0) {
        container.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding: 40px;">当前分类下暂无配置</div>`;
        return;
    }

    if (appState.currentSort === 'usage_count') {
        filtered = filtered.slice().sort((a, b) => (b.usage_count || 0) - (a.usage_count || 0));
    } else if (appState.currentSort === 'recent') {
        filtered = filtered.slice().sort((a, b) => {
            if (a.is_current) return -1;
            if (b.is_current) return 1;
            return (b.usage_count || 0) - (a.usage_count || 0);
        });
    } else {
        filtered = filtered.slice().sort((a, b) => a.name.localeCompare(b.name));
    }

    container.innerHTML = filtered.map((config, index) => {
        let providerTypeBadge = '';
        if (config.provider_type) {
            const typeMap = {
                'OfficialRelay': { text: '🔄 官方中转', class: 'official-relay' },
                'official_relay': { text: '🔄 官方中转', class: 'official-relay' },
                'ThirdPartyModel': { text: '🤖 第三方模型', class: 'third-party-model' },
                'third_party_model': { text: '🤖 第三方模型', class: 'third-party-model' }
            };
            const type = typeMap[config.provider_type];
            if (type) {
                providerTypeBadge = `<span class="config-type-badge ${type.class}">${type.text}</span>`;
            }
        }

        let tagsHtml = '';
        if (config.tags && config.tags.length > 0) {
            tagsHtml = `
                <div class="config-tags">
                    ${config.tags.map(tag => `<span class="config-tag">${tag}</span>`).join('')}
                </div>
            `;
        }

        const enabledBadge = (config.enabled === false)
            ? '<span class="badge badge-disabled" style="background: var(--accent-danger); color: white;">已禁用</span>'
            : '';

        return `
            <div id="config-${config.name}" class="config-card ${config.is_current ? 'active' : ''} ${config.enabled === false ? 'disabled-config' : ''} fade-in" style="animation-delay: ${index * 0.05}s; ${config.enabled === false ? 'opacity: 0.6;' : ''}">
                <div class="config-header">
                    <div class="config-info">
                        <h3 class="config-title">
                            ${providerTypeBadge}
                            <span class="config-name">${config.name}</span>
                            ${config.is_current ? '<span class="badge badge-active">当前</span>' : ''}
                            ${config.is_default ? '<span class="badge badge-default">默认</span>' : ''}
                            ${enabledBadge}
                        </h3>
                        <div class="config-description">
                            <span class="desc-icon">📝</span>
                            <span class="desc-text">${config.description || '无描述'}</span>
                        </div>
                        ${config.provider ? `
                        <div class="config-meta">
                            <div class="meta-item">
                                <span class="meta-icon">🏢</span>
                                <span class="meta-label">提供商:</span>
                                <span class="meta-value provider-name">${config.provider}</span>
                            </div>
                            ${config.account ? `
                            <div class="meta-item">
                                <span class="meta-icon">👤</span>
                                <span class="meta-label">账号:</span>
                                <span class="meta-value account-name">${config.account}</span>
                            </div>
                            ` : ''}
                            <div class="meta-item">
                                <span class="meta-icon">📊</span>
                                <span class="meta-label">使用次数:</span>
                                <span class="meta-value usage-count" style="font-weight: 500; color: var(--accent-primary);">${config.usage_count || 0}</span>
                            </div>
                        </div>
                        ` : `
                        <div class="config-meta">
                            <div class="meta-item">
                                <span class="meta-icon">📊</span>
                                <span class="meta-label">使用次数:</span>
                                <span class="meta-value usage-count" style="font-weight: 500; color: var(--accent-primary);">${config.usage_count || 0}</span>
                            </div>
                        </div>
                        `}
                        ${tagsHtml}
                    </div>
                    <div class="config-actions-top">
                        ${!config.is_current ? `
                        <button class="btn btn-primary btn-action-top" onclick="window.switchConfig('${config.name}')" title="切换到此配置" ${config.enabled === false ? 'disabled' : ''}>
                            <span class="btn-icon">⚡</span>
                            <span class="btn-text">切换</span>
                        </button>
                        ` : ''}
                        <button class="btn btn-secondary btn-action-top" onclick="window.editConfig('${config.name}')" title="编辑配置">
                            <span class="btn-icon">✏️</span>
                            <span class="btn-text">编辑</span>
                        </button>
                        ${config.enabled === false ? `
                        <button class="btn btn-success btn-action-top" onclick="window.enableConfig('${config.name}')" title="启用配置" style="background: var(--accent-success);">
                            <span class="btn-icon">✓</span>
                            <span class="btn-text">启用</span>
                        </button>
                        ` : `
                        <button class="btn btn-warning btn-action-top" onclick="window.disableConfig('${config.name}')" title="禁用配置" style="background: var(--accent-warning);">
                            <span class="btn-icon">◯</span>
                            <span class="btn-text">禁用</span>
                        </button>
                        `}
                        ${!config.is_current && !config.is_default ? `
                        <button class="btn btn-danger btn-action-top" onclick="window.deleteConfig('${config.name}')" title="删除配置">
                            <span class="btn-icon">🗑️</span>
                            <span class="btn-text">删除</span>
                        </button>
                        ` : ''}
                    </div>
                </div>
                <div class="config-details">
                    <div class="config-field">
                        <div class="field-label">Base URL</div>
                        <div class="field-value">${config.base_url || '-'}</div>
                    </div>
                    <div class="config-field">
                        <div class="field-label">Auth Token</div>
                        <div class="field-value">${config.auth_token || '-'}</div>
                    </div>
                    ${config.model ? `
                    <div class="config-field">
                        <div class="field-label">Model</div>
                        <div class="field-value">${config.model}</div>
                    </div>
                    ` : ''}
                    ${config.small_fast_model ? `
                    <div class="config-field">
                        <div class="field-label">Small Fast Model</div>
                        <div class="field-value">${config.small_fast_model}</div>
                    </div>
                    ` : ''}
                    ${isCodexActive() ? renderCodexMeta(config) : ''}
                </div>
            </div>
        `;
    }).join('');
}

function renderCodexMeta(config) {
    const metaEntries = [];
    if (config.api_mode) metaEntries.push({ label: 'API 模式', value: config.api_mode === 'github' ? 'GitHub 官方' : '自定义 API' });
    if (config.wire_api) metaEntries.push({ label: 'Wire API', value: config.wire_api });
    if (config.env_key) metaEntries.push({ label: 'Env Key', value: config.env_key });
    if (typeof config.requires_openai_auth === 'boolean') metaEntries.push({ label: '需要登录', value: config.requires_openai_auth ? '是' : '否' });
    if (config.organization) metaEntries.push({ label: '组织', value: config.organization });
    if (config.approval_policy) metaEntries.push({ label: '审批策略', value: config.approval_policy });
    if (config.sandbox_mode) metaEntries.push({ label: '沙盒模式', value: config.sandbox_mode });
    if (config.model_reasoning_effort) metaEntries.push({ label: '推理强度', value: config.model_reasoning_effort });

    if (metaEntries.length === 0) return '';

    return `
        <div class="codex-meta-grid">
            ${metaEntries.map(item => `
                <div class="codex-meta-item">
                    <div class="field-label">${item.label}</div>
                    <div class="field-value">${item.value}</div>
                </div>
            `).join('')}
        </div>
    `;
}

export function renderConfigNav() {
    const nav = document.getElementById('configNav');
    if (appState.allConfigs.length === 0) {
        nav.innerHTML = '<li class="config-nav-item"><span style="font-size: 12px; color: var(--text-muted);">暂无配置</span></li>';
        return;
    }

    let filtered = appState.allConfigs;
    if (appState.currentFilter === 'official_relay') {
        filtered = appState.allConfigs.filter(c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay');
    } else if (appState.currentFilter === 'third_party_model') {
        filtered = appState.allConfigs.filter(c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model');
    } else if (appState.currentFilter === 'uncategorized') {
        filtered = appState.allConfigs.filter(c => !c.provider_type);
    }

    if (filtered.length === 0) {
        nav.innerHTML = '<li class="config-nav-item"><span style="font-size: 12px; color: var(--text-muted);">当前分类下暂无配置</span></li>';
        return;
    }

    nav.innerHTML = filtered.map(config => `
        <li class="config-nav-item">
            <a href="#config-${config.name}" class="config-nav-link" onclick="window.scrollToConfig('${config.name}', event)">
                <span class="config-nav-badge ${config.is_current ? 'current' : config.is_default ? 'default' : ''}"></span>
                ${config.name}
            </a>
        </li>
    `).join('');
}

export function renderHistory(entries) {
    const container = document.getElementById('historyList');
    if (entries.length === 0) {
        container.innerHTML = '<div style="text-align: center; color: var(--text-muted); padding: 40px;">暂无历史记录</div>';
        return;
    }

    container.innerHTML = entries.map(entry => `
        <div class="history-item">
            <div class="history-header">
                <span class="history-op">${entry.operation}</span>
                <span class="history-time">${new Date(entry.timestamp).toLocaleString()}</span>
            </div>
            <div class="history-details">
                操作者: ${entry.actor}<br>
                ${entry.from_config && entry.to_config ? `从 ${entry.from_config} 切换到 ${entry.to_config}` : ''}
            </div>
        </div>
    `).join('');
}

export function renderPlatformNavigation() {
    if (appState.platformInfo.mode !== 'unified') return;

    const platforms = appState.platformInfo.availablePlatforms.filter(p => !['qwen', 'iflow'].includes(p.name));

    platforms.forEach(platform => {
        const badge = document.getElementById(`badge-${platform.name}`);
        if (badge) {
            const profileCount = platform.current_profile ? 1 : 0;
            badge.textContent = profileCount.toString();
            if (platform.is_current && appState.allConfigs.length > 0) {
                badge.textContent = appState.allConfigs.length.toString();
            }
        }

        const tab = document.querySelector(`.platform-tab[data-platform="${platform.name}"]`);
        if (tab) {
            if (platform.is_current) {
                tab.classList.add('active');
            } else {
                tab.classList.remove('active');
            }
        }
    });

    const activateBtn = document.getElementById('activatePlatformBtn');
    if (activateBtn) {
        const activeTab = document.querySelector('.platform-tab.active');
        if (activeTab) {
            const selectedPlatform = activeTab.getAttribute('data-platform');
            const currentPlatform = platforms.find(p => p.is_current);
            if (currentPlatform && currentPlatform.name !== selectedPlatform) {
                activateBtn.style.display = 'inline-block';
            } else {
                activateBtn.style.display = 'none';
            }
        }
    }
}

export function renderPlatformStatus() {
    if (appState.platformInfo.mode !== 'unified') return;

    const statusList = document.getElementById('platformStatusList');
    if (!statusList) return;

    const platforms = appState.platformInfo.availablePlatforms.filter(p => !['qwen', 'iflow'].includes(p.name));

    if (platforms.length === 0) {
        statusList.innerHTML = '<div style="text-align: center; color: var(--text-muted); padding: 20px; font-size: 12px;">暂无平台</div>';
        return;
    }

    const platformIcons = { 'claude': '🤖', 'codex': '💻', 'gemini': '✨' };

    statusList.innerHTML = platforms.map(platform => {
        const icon = platformIcons[platform.name] || '📦';
        const badgeClass = platform.enabled ? '' : 'inactive';
        const badgeText = platform.enabled ? '已启用' : '已禁用';

        return `
            <div class="platform-status-item ${platform.is_current ? 'active' : ''}"
                 onclick="window.switchPlatform('${platform.name}')">
                <div class="platform-status-name">
                    <span class="platform-status-icon">${icon}</span>
                    <span>${platform.name}</span>
                </div>
                <span class="platform-status-badge ${badgeClass}">${badgeText}</span>
            </div>
        `;
    }).join('');
}

export function renderProviderStats(providerMap) {
    const body = document.getElementById('providerStatsBody');
    if (!body) return;

    let entries = Object.entries(providerMap);
    if (appState.providerSortMode === 'count_asc') {
        entries.sort((a, b) => a[1] - b[1]);
    } else if (appState.providerSortMode === 'name_asc') {
        entries.sort((a, b) => a[0].localeCompare(b[0]));
    } else {
        entries.sort((a, b) => b[1] - a[1]);
    }

    if (entries.length === 0) {
        body.innerHTML = `
            <div class="empty-state">
                <div style="font-size: 32px; margin-bottom: 12px;">📊</div>
                <div>暂无提供商使用数据</div>
                <div style="margin-top: 8px; font-size: 12px; color: var(--text-muted);">
                    开始使用 AI API 后，这里将显示提供商统计
                </div>
            </div>
        `;
        return;
    }

    const maxCount = Math.max(...entries.map(([, count]) => count));
    const totalCount = entries.reduce((sum, [, count]) => sum + count, 0);
    const yTickPercents = [0, 25, 50, 75, 100];
    const yTickValues = yTickPercents.map(p => Math.round((maxCount * p) / 100));

    let sortLabel = '使用次数（从高到低）';
    if (appState.providerSortMode === 'count_asc') sortLabel = '使用次数（从低到高）';
    else if (appState.providerSortMode === 'name_asc') sortLabel = '供应商名称（A → Z）';

    body.innerHTML = `
        <div class="provider-chart-container">
            <div class="provider-chart-summary">
                共 ${entries.length} 个提供商 · 总调用 ${totalCount} 次
                <div class="provider-chart-desc">
                    Y 轴：使用次数（单位：次） · X 轴：提供商 · 当前排序：${sortLabel}
                </div>
            </div>
            <div class="provider-chart">
                <div class="provider-chart-y-grid">
                    ${yTickPercents.map((percent, idx) => `
                        <div class="y-grid-line" style="bottom:${percent}%;">
                            <span class="y-grid-label">${yTickValues[idx]}</span>
                        </div>
                    `).join('')}
                </div>
                <div class="provider-chart-y-axis-line"></div>
                <div class="provider-chart-x-axis-line"></div>
                <div class="provider-chart-bars">
                    ${entries.map(([provider, count], index) => {
                        const ratio = maxCount > 0 ? count / maxCount : 0;
                        const barHeight = ratio > 0 ? Math.max(ratio * 100, 8) : 0;
                        const displayName = provider || 'unknown';
                        const safeName = escapeHtml(displayName);
                        const percentage = maxCount > 0 ? (ratio * 100).toFixed(1) : 0;
                        const colorClass = `bar-color-${index % 5}`;

                        return `
                            <div class="provider-bar-column" style="animation: configFadeIn 0.4s ease ${index * 0.05}s backwards;">
                                <div class="provider-bar-vertical-bg">
                                    <div class="provider-bar-vertical-fill ${colorClass}" style="height:${barHeight}%;" title="${safeName}: ${count} 次，占最高值的 ${percentage}%"></div>
                                </div>
                                <div class="provider-bar-value">${count} 次</div>
                                <div class="provider-bar-label" title="${safeName}">${safeName}</div>
                            </div>
                        `;
                    }).join('')}
                </div>
            </div>
            <div class="provider-chart-x-axis-label">X 轴：提供商</div>
        </div>
    `;
}
