        let currentEditingConfig = null;
        let allConfigs = [];
        let notificationTimeout = null;
        let currentFilter = 'all'; // 当前过滤类型

        // ===== 工具函数 =====

        // 按钮状态管理
        function setButtonLoading(button, loading = true) {
            if (loading) {
                button.disabled = true;
                button.classList.add('loading');
                button.dataset.originalText = button.innerHTML;
            } else {
                button.disabled = false;
                button.classList.remove('loading');
                if (button.dataset.originalText) {
                    button.innerHTML = button.dataset.originalText;
                }
            }
        }

        // 通过选择器批量禁用/启用按钮
        function setButtonsDisabled(selector, disabled = true) {
            document.querySelectorAll(selector).forEach(btn => {
                btn.disabled = disabled;
                if (disabled) {
                    btn.classList.add('disabled');
                } else {
                    btn.classList.remove('disabled');
                }
            });
        }

        // 增强的通知系统
        function showNotification(message, type = 'success', options = {}) {
            const notification = document.getElementById('notification');

            // 清除之前的定时器
            if (notificationTimeout) {
                clearTimeout(notificationTimeout);
            }

            // 图标映射
            const icons = {
                success: '✓',
                error: '✗',
                warning: '⚠',
                info: 'ℹ'
            };

            const icon = options.icon || icons[type] || 'ℹ';

            // 构建通知内容
            let content = `
                <div class="notification-content">
                    <span class="notification-icon">${icon}</span>
                    <div class="notification-text">${message}</div>
                </div>
            `;

            // 添加操作按钮（如重试）
            if (options.actions && options.actions.length > 0) {
                content += '<div class="notification-actions">';
                options.actions.forEach(action => {
                    content += `<button class="notification-btn notification-btn-${action.type || 'secondary'}" onclick="${action.onclick}">${action.label}</button>`;
                });
                content += '</div>';
            }

            notification.innerHTML = content;
            notification.className = `notification ${type} show`;

            // 自动隐藏（除非指定不自动隐藏）
            if (options.autoHide !== false) {
                notificationTimeout = setTimeout(() => {
                    notification.classList.remove('show');
                }, options.duration || 4000);
            }
        }

        // 错误处理辅助函数
        function handleApiError(error, context = '') {
            console.error(`API Error in ${context}:`, error);

            let message = '';
            let retry = null;

            if (error.message && error.message.includes('Failed to fetch')) {
                message = `网络连接失败 ${context ? `(${context})` : ''}\n请检查网络连接或服务器状态`;
                retry = context;
            } else if (error.message) {
                message = `操作失败: ${error.message}`;
            } else {
                message = `未知错误 ${context ? `(${context})` : ''}`;
            }

            const options = {
                autoHide: false,
                actions: [
                    {
                        label: '关闭',
                        type: 'secondary',
                        onclick: 'closeNotification()'
                    }
                ]
            };

            // 如果有重试上下文，添加重试按钮
            if (retry) {
                options.actions.unshift({
                    label: '重试',
                    type: 'primary',
                    onclick: `retryLastOperation('${retry}')`
                });
            }

            showNotification(message, 'error', options);
        }

        // 关闭通知
        function closeNotification() {
            const notification = document.getElementById('notification');
            notification.classList.remove('show');
        }

        // 重试逻辑（存储最后的操作供重试）
        let lastOperation = null;

        function retryLastOperation(context) {
            closeNotification();
            if (lastOperation && lastOperation.context === context) {
                lastOperation.func();
            }
        }

        // 主题管理
        function initTheme() {
            // 从 localStorage 读取保存的主题，默认为 light
            const savedTheme = localStorage.getItem('ccr-theme') || 'light';
            setTheme(savedTheme);
        }

        function toggleTheme() {
            const currentTheme = document.documentElement.getAttribute('data-theme');
            const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
            setTheme(newTheme);
            // 保存到 localStorage
            localStorage.setItem('ccr-theme', newTheme);
        }

        function setTheme(theme) {
            document.documentElement.setAttribute('data-theme', theme);
            const themeIcon = document.getElementById('themeIcon');
            if (themeIcon) {
                themeIcon.textContent = theme === 'dark' ? '🌙' : '☀️';
            }
        }

        // 页面加载时初始化
        document.addEventListener('DOMContentLoaded', () => {
            initTheme();
            loadData();
            loadSystemInfo();
            // 每 5 秒更新一次系统信息
            setInterval(loadSystemInfo, 5000);
        });

        // 加载所有数据
        async function loadData() {
            await loadConfigs();
            await loadHistory();
        }

        // 加载系统信息
        async function loadSystemInfo() {
            try {
                const response = await fetch('/api/system');
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}`);
                }
                
                const result = await response.json();
                if (result.success && result.data) {
                    const data = result.data;
                    
                    // 更新主机名
                    document.getElementById('sysHostname').textContent = data.hostname;
                    
                    // 更新系统信息
                    document.getElementById('sysOS').textContent = `${data.os} ${data.os_version}`;
                    
                    // 更新 CPU 信息
                    document.getElementById('sysCPU').textContent = `${data.cpu_cores} 核心`;
                    
                    // 更新 CPU 使用率
                    const cpuUsage = Math.round(data.cpu_usage);
                    document.getElementById('sysCPUUsage').textContent = `${cpuUsage}%`;
                    document.getElementById('sysCPUBar').style.width = `${cpuUsage}%`;
                    
                    // 更新内存信息
                    const usedMem = data.used_memory_gb.toFixed(1);
                    const totalMem = data.total_memory_gb.toFixed(1);
                    const memPercent = Math.round(data.memory_usage_percent);
                    document.getElementById('sysMemory').textContent = `${usedMem} GB / ${totalMem} GB (${memPercent}%)`;
                    document.getElementById('sysMemBar').style.width = `${memPercent}%`;
                    
                    // 更新运行时间
                    document.getElementById('sysUptime').textContent = formatUptime(data.uptime_seconds);
                }
            } catch (error) {
                console.error('加载系统信息失败:', error);
                // 静默失败，不影响其他功能
            }
        }
        
        // 格式化运行时间
        function formatUptime(seconds) {
            const days = Math.floor(seconds / 86400);
            const hours = Math.floor((seconds % 86400) / 3600);
            const minutes = Math.floor((seconds % 3600) / 60);
            
            if (days > 0) {
                return `${days}天 ${hours}时`;
            } else if (hours > 0) {
                return `${hours}时 ${minutes}分`;
            } else {
                return `${minutes}分钟`;
            }
        }

        // 加载配置列表
        async function loadConfigs() {
            // 保存操作以供重试
            lastOperation = { context: '加载配置', func: loadConfigs };

            try {
                // 🆕 根据平台模式和当前平台选择正确的 API 端点
                let endpoint = '/api/configs';

                if (platformInfo.mode === 'unified' && platformInfo.currentPlatform) {
                    // Unified 模式下，根据当前平台调用不同的端点
                    const platformEndpoints = {
                        'claude': '/api/configs',           // Claude 使用默认端点
                        'codex': '/api/codex/profiles',     // Codex 使用 profiles 端点
                        'gemini': '/api/gemini/config',     // Gemini 使用 config 端点
                        'qwen': '/api/qwen/config',         // Qwen 使用 config 端点
                        'iflow': '/api/iflow/configs'       // iFlow 使用 configs 端点
                    };

                    endpoint = platformEndpoints[platformInfo.currentPlatform] || '/api/configs';

                    console.log(`加载平台配置: ${platformInfo.currentPlatform} -> ${endpoint}`);
                }

                const response = await fetch(endpoint);

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    // 🆕 处理不同平台的响应格式
                    if (platformInfo.mode === 'unified' && platformInfo.currentPlatform !== 'claude') {
                        // 非 Claude 平台可能返回不同的数据结构
                        // 需要统一转换为标准格式
                        allConfigs = normalizeConfigData(data.data, platformInfo.currentPlatform);
                    } else {
                        // Legacy 模式或 Claude 平台，使用原有格式
                        allConfigs = data.data.configs || data.data || [];
                    }

                    // 更新 UI 显示
                    const currentConfig = data.data.current_config ||
                                        data.data.current_profile ||
                                        data.data.active_profile ||
                                        '-';
                    document.getElementById('currentConfigName').textContent = currentConfig;
                    document.getElementById('totalConfigs').textContent = allConfigs.length;

                    renderConfigs();
                    renderConfigNav();
                } else {
                    showNotification(data.message || '加载失败', 'error');
                }
            } catch (error) {
                handleApiError(error, '加载配置');
            }
        }

        // 🆕 标准化不同平台的配置数据格式
        function normalizeConfigData(data, platform) {
            // 如果已经是数组格式，直接返回
            if (Array.isArray(data)) {
                return data;
            }

            // 如果有 configs 字段，返回它
            if (data.configs && Array.isArray(data.configs)) {
                return data.configs;
            }

            // 如果有 profiles 字段（Codex），转换为标准格式
            if (data.profiles && Array.isArray(data.profiles)) {
                return data.profiles.map(profile => ({
                    name: profile.name,
                    description: profile.description,
                    base_url: profile.base_url || profile.api_url,
                    auth_token: profile.auth_token || profile.api_key,
                    model: profile.model,
                    small_fast_model: profile.small_fast_model,
                    is_current: profile.is_active || profile.is_current,
                    is_default: profile.is_default,
                    provider: profile.provider,
                    provider_type: profile.provider_type
                }));
            }

            // 如果是单个配置对象（Gemini/Qwen），转换为数组
            if (data.api_key || data.auth_token) {
                return [{
                    name: platform,
                    description: `${platform} Configuration`,
                    base_url: data.base_url || data.api_url,
                    auth_token: data.auth_token || data.api_key,
                    model: data.model,
                    small_fast_model: data.small_fast_model,
                    is_current: true,
                    is_default: true
                }];
            }

            // 默认返回空数组
            console.warn('无法解析配置数据格式:', data);
            return [];
        }

        // 🆕 过滤配置列表
        function filterConfigsByType(type) {
            currentFilter = type;

            // 更新按钮激活状态
            document.querySelectorAll('.type-filter-btn').forEach(btn => {
                btn.classList.remove('active');
                if (btn.getAttribute('data-type') === type) {
                    btn.classList.add('active');
                }
            });

            // 重新渲染配置列表和导航
            renderConfigs();
            renderConfigNav();
        }

        // 渲染配置列表
        function renderConfigs() {
            const container = document.getElementById('configsList');
            if (allConfigs.length === 0) {
                container.innerHTML = '<div style="text-align: center; color: var(--text-muted); padding: 40px;">暂无配置</div>';
                return;
            }

            // 🆕 根据 currentFilter 过滤配置
            let filtered = allConfigs;
            if (currentFilter === 'official_relay') {
                filtered = allConfigs.filter(c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay');
            } else if (currentFilter === 'third_party_model') {
                filtered = allConfigs.filter(c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model');
            } else if (currentFilter === 'uncategorized') {
                filtered = allConfigs.filter(c => !c.provider_type);
            }
            // else: 'all' - 显示全部配置

            if (filtered.length === 0) {
                container.innerHTML = `<div style="text-align: center; color: var(--text-muted); padding: 40px;">当前分类下暂无配置</div>`;
                return;
            }

            container.innerHTML = filtered.map((config, index) => {
                // 🆕 生成提供商类型徽章
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

                // 🆕 生成标签列表
                let tagsHtml = '';
                if (config.tags && config.tags.length > 0) {
                    tagsHtml = `
                        <div class="config-tags">
                            ${config.tags.map(tag => `<span class="config-tag">${tag}</span>`).join('')}
                        </div>
                    `;
                }

                return `
                <div id="config-${config.name}" class="config-card ${config.is_current ? 'active' : ''} fade-in" style="animation-delay: ${index * 0.05}s">
                    <div class="config-header">
                        <div class="config-info">
                            <h3 class="config-title">
                                ${providerTypeBadge}
                                <span class="config-name">${config.name}</span>
                                ${config.is_current ? '<span class="badge badge-active">当前</span>' : ''}
                                ${config.is_default ? '<span class="badge badge-default">默认</span>' : ''}
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
                            </div>
                            ` : ''}
                            ${tagsHtml}
                        </div>
                    </div>
                    <div class="config-details">
                        <div class="config-field">
                            <div class="field-label">Base URL</div>
                            <div class="field-value">${config.base_url}</div>
                        </div>
                        <div class="config-field">
                            <div class="field-label">Auth Token</div>
                            <div class="field-value">${config.auth_token}</div>
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
                    </div>
                    <div class="config-actions">
                        ${!config.is_current ? `
                        <button class="btn btn-primary btn-small" onclick="switchConfig('${config.name}')">
                            切换
                        </button>
                        ` : ''}
                        <button class="btn btn-secondary btn-small" onclick="editConfig('${config.name}')">
                            编辑
                        </button>
                        ${!config.is_current && !config.is_default ? `
                        <button class="btn btn-danger btn-small" onclick="deleteConfig('${config.name}')">
                            删除
                        </button>
                        ` : ''}
                    </div>
                </div>
            `;
            }).join('');
        }

        // 渲染配置目录导航
        function renderConfigNav() {
            const nav = document.getElementById('configNav');
            if (allConfigs.length === 0) {
                nav.innerHTML = '<li class="config-nav-item"><span style="font-size: 12px; color: var(--text-muted);">暂无配置</span></li>';
                return;
            }

            // 🆕 根据 currentFilter 过滤配置
            let filtered = allConfigs;
            if (currentFilter === 'official_relay') {
                filtered = allConfigs.filter(c => c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay');
            } else if (currentFilter === 'third_party_model') {
                filtered = allConfigs.filter(c => c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model');
            } else if (currentFilter === 'uncategorized') {
                filtered = allConfigs.filter(c => !c.provider_type);
            }
            // else: 'all' - 显示全部配置

            if (filtered.length === 0) {
                nav.innerHTML = '<li class="config-nav-item"><span style="font-size: 12px; color: var(--text-muted);">当前分类下暂无配置</span></li>';
                return;
            }

            nav.innerHTML = filtered.map(config => `
                <li class="config-nav-item">
                    <a href="#config-${config.name}" class="config-nav-link" onclick="scrollToConfig('${config.name}', event)">
                        <span class="config-nav-badge ${config.is_current ? 'current' : config.is_default ? 'default' : ''}"></span>
                        ${config.name}
                    </a>
                </li>
            `).join('');
        }

        // 滚动到指定配置
        function scrollToConfig(name, event) {
            event.preventDefault();
            const element = document.getElementById(`config-${name}`);
            if (element) {
                element.scrollIntoView({ behavior: 'smooth', block: 'center' });

                // 更新导航激活状态
                document.querySelectorAll('.config-nav-link').forEach(link => {
                    link.classList.remove('active');
                });
                event.target.classList.add('active');

                // 高亮配置卡片
                element.style.transform = 'scale(1.02)';
                setTimeout(() => {
                    element.style.transform = '';
                }, 300);
            }
        }

        // 切换配置
        async function switchConfig(name) {
            if (!confirm(`确定切换到配置 "${name}" 吗？`)) return;

            // 禁用所有切换按钮
            setButtonsDisabled('.btn-primary', true);

            // 保存操作以供重试
            lastOperation = { context: '切换配置', func: () => switchConfig(name) };

            try {
                const response = await fetch('/api/switch', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ config_name: name })
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    showNotification(`✓ 成功切换到配置 "${name}"`, 'success');
                    await loadData();
                } else {
                    showNotification(data.message || '切换失败', 'error', {
                        autoHide: false,
                        actions: [{
                            label: '关闭',
                            type: 'secondary',
                            onclick: 'closeNotification()'
                        }]
                    });
                }
            } catch (error) {
                handleApiError(error, '切换配置');
            } finally {
                // 恢复按钮状态
                setButtonsDisabled('.btn-primary', false);
            }
        }

        // 打开添加模态框
        function openAddModal() {
            currentEditingConfig = null;
            document.getElementById('modalTitle').textContent = '添加配置';
            document.getElementById('configForm').reset();
            document.getElementById('configModal').classList.add('show');
        }

        // 编辑配置
        function editConfig(name) {
            const config = allConfigs.find(c => c.name === name);
            if (!config) return;

            currentEditingConfig = name;
            document.getElementById('modalTitle').textContent = '编辑配置';
            document.getElementById('configName').value = config.name;
            document.getElementById('configDesc').value = config.description || '';
            document.getElementById('configBaseUrl').value = config.base_url;
            document.getElementById('configAuthToken').value = config.auth_token;
            document.getElementById('configModel').value = config.model || '';
            document.getElementById('configSmallModel').value = config.small_fast_model || '';

            // 🆕 填充分类字段
            document.getElementById('configProviderType').value = config.provider_type || '';
            document.getElementById('configProvider').value = config.provider || '';
            document.getElementById('configAccount').value = config.account || '';
            document.getElementById('configTags').value = config.tags ? config.tags.join(', ') : '';

            document.getElementById('configModal').classList.add('show');
        }

        // 保存配置
        async function saveConfig(event) {
            event.preventDefault();

            const submitBtn = event.target.querySelector('button[type="submit"]');
            setButtonLoading(submitBtn, true);

            // 🆕 获取分类字段值
            const providerTypeValue = document.getElementById('configProviderType').value;
            const providerValue = document.getElementById('configProvider').value;
            const accountValue = document.getElementById('configAccount').value;
            const tagsValue = document.getElementById('configTags').value;

            // 🆕 处理标签（逗号分隔转数组）
            let tagsArray = null;
            if (tagsValue && tagsValue.trim()) {
                tagsArray = tagsValue.split(',').map(tag => tag.trim()).filter(tag => tag.length > 0);
            }

            const configData = {
                name: document.getElementById('configName').value,
                description: document.getElementById('configDesc').value || null,
                base_url: document.getElementById('configBaseUrl').value,
                auth_token: document.getElementById('configAuthToken').value,
                model: document.getElementById('configModel').value || null,
                small_fast_model: document.getElementById('configSmallModel').value || null,
                // 🆕 分类字段
                provider_type: providerTypeValue || null,
                provider: providerValue || null,
                account: accountValue || null,
                tags: tagsArray
            };

            // 保存操作以供重试
            lastOperation = { context: '保存配置', func: () => saveConfig(event) };

            try {
                const url = currentEditingConfig ? `/api/config/${currentEditingConfig}` : '/api/config';
                const method = currentEditingConfig ? 'PUT' : 'POST';

                const response = await fetch(url, {
                    method: method,
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(configData)
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    showNotification(currentEditingConfig ? '✓ 配置更新成功' : '✓ 配置添加成功', 'success');
                    closeModal();
                    await loadData();
                } else {
                    showNotification(data.message || '保存失败', 'error', {
                        autoHide: false,
                        actions: [{
                            label: '关闭',
                            type: 'secondary',
                            onclick: 'closeNotification()'
                        }]
                    });
                }
            } catch (error) {
                handleApiError(error, '保存配置');
            } finally {
                setButtonLoading(submitBtn, false);
            }
        }

        // 删除配置
        async function deleteConfig(name) {
            if (!confirm(`确定删除配置 "${name}" 吗？此操作不可恢复！`)) return;

            // 禁用所有删除按钮
            setButtonsDisabled('.btn-danger', true);

            // 保存操作以供重试
            lastOperation = { context: '删除配置', func: () => deleteConfig(name) };

            try {
                const response = await fetch(`/api/config/${name}`, {
                    method: 'DELETE'
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    showNotification(`✓ 配置 "${name}" 删除成功`, 'success');
                    await loadData();
                } else {
                    showNotification(data.message || '删除失败', 'error', {
                        autoHide: false,
                        actions: [{
                            label: '关闭',
                            type: 'secondary',
                            onclick: 'closeNotification()'
                        }]
                    });
                }
            } catch (error) {
                handleApiError(error, '删除配置');
            } finally {
                // 恢复按钮状态
                setButtonsDisabled('.btn-danger', false);
            }
        }

        // 加载历史记录
        async function loadHistory() {
            try {
                const response = await fetch('/api/history');
                const data = await response.json();

                if (data.success) {
                    document.getElementById('historyCount').textContent = data.data.total;
                    renderHistory(data.data.entries);
                }
            } catch (error) {
                console.error('加载历史记录失败:', error);
            }
        }

        // 渲染历史记录
        function renderHistory(entries) {
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

        // 验证配置
        async function validateConfigs() {
            const btn = event.target;
            setButtonLoading(btn, true);

            // 保存操作以供重试
            lastOperation = { context: '验证配置', func: validateConfigs };

            try {
                const response = await fetch('/api/validate', { method: 'POST' });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    showNotification('✓ 配置验证通过', 'success', { icon: '✅' });
                } else {
                    showNotification(data.message || '验证失败', 'error', {
                        autoHide: false,
                        actions: [{
                            label: '关闭',
                            type: 'secondary',
                            onclick: 'closeNotification()'
                        }]
                    });
                }
            } catch (error) {
                handleApiError(error, '验证配置');
            } finally {
                setButtonLoading(btn, false);
            }
        }

        // 切换标签页
        function switchTab(tab) {
            document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));

            event.target.classList.add('active');
            document.getElementById(tab + '-tab').classList.add('active');
        }

        // 关闭模态框
        function closeModal() {
            document.getElementById('configModal').classList.remove('show');
        }

        // 显示通知
        function showNotification(message, type = 'success') {
            const notification = document.getElementById('notification');
            notification.textContent = message;
            notification.className = `notification ${type} show`;

            setTimeout(() => {
                notification.classList.remove('show');
            }, 3000);
        }

        // 清理备份相关函数
        function openCleanModal() {
            document.getElementById('cleanModal').style.display = 'flex';
            document.getElementById('cleanPreview').style.display = 'none';
            document.getElementById('cleanDays').value = 7;
            document.getElementById('cleanDryRun').checked = true;
            updateCleanDaysDisplay();
        }

        function closeCleanModal() {
            document.getElementById('cleanModal').style.display = 'none';
            document.getElementById('cleanPreview').style.display = 'none';
        }

        function updateCleanDaysDisplay() {
            const days = document.getElementById('cleanDays').value;
            document.getElementById('cleanDaysDisplay').textContent = days;
        }

        async function previewClean() {
            const days = parseInt(document.getElementById('cleanDays').value);
            const btn = event.target;
            setButtonLoading(btn, true);

            // 保存操作以供重试
            lastOperation = { context: '预览清理', func: previewClean };

            try {
                const response = await fetch('/api/clean', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ days, dry_run: true })
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    const result = data.data;
                    const preview = document.getElementById('cleanPreview');
                    const content = document.getElementById('cleanPreviewContent');

                    if (result.deleted_count === 0) {
                        content.innerHTML = `
                            <div style="color: var(--accent-success); margin-bottom: 8px;">✓ 没有需要清理的文件</div>
                            <div style="font-size: 13px; color: var(--text-muted);">所有备份都在保留期内</div>
                        `;
                    } else {
                        content.innerHTML = `
                            <div style="margin-bottom: 8px;">
                                <span style="color: var(--accent-warning);">⚠</span>
                                <span>将删除 ${result.deleted_count} 个文件</span>
                            </div>
                            <div style="margin-bottom: 8px;">
                                <span style="color: var(--accent-primary);">ℹ</span>
                                <span>保留 ${result.skipped_count} 个文件</span>
                            </div>
                            <div style="margin-bottom: 8px;">
                                <span style="color: var(--accent-success);">💾</span>
                                <span>将释放 ${result.total_size_mb.toFixed(2)} MB</span>
                            </div>
                        `;
                    }

                    preview.style.display = 'block';
                } else {
                    showNotification(data.message || '预览失败', 'error', {
                        autoHide: false,
                        actions: [{
                            label: '关闭',
                            type: 'secondary',
                            onclick: 'closeNotification()'
                        }]
                    });
                }
            } catch (error) {
                handleApiError(error, '预览清理');
            } finally {
                setButtonLoading(btn, false);
            }
        }

        async function executeClean() {
            const days = parseInt(document.getElementById('cleanDays').value);
            const dryRun = document.getElementById('cleanDryRun').checked;
            const btn = document.getElementById('cleanExecuteBtn');

            if (dryRun) {
                showNotification('请取消勾选"模拟运行"以执行实际清理', 'warning', { icon: '⚠' });
                return;
            }

            if (!confirm(`确定清理 ${days} 天前的备份文件吗？此操作不可恢复！`)) {
                return;
            }

            setButtonLoading(btn, true);

            // 保存操作以供重试
            lastOperation = { context: '执行清理', func: executeClean };

            try {
                const response = await fetch('/api/clean', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ days, dry_run: false })
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    const result = data.data;
                    if (result.deleted_count > 0) {
                        showNotification(`✓ 已删除 ${result.deleted_count} 个文件，释放 ${result.total_size_mb.toFixed(2)} MB`, 'success', { icon: '🗑️' });
                    } else {
                        showNotification('✓ 没有需要清理的文件', 'success');
                    }
                    closeCleanModal();
                } else {
                    showNotification(data.message || '清理失败', 'error', {
                        autoHide: false,
                        actions: [{
                            label: '关闭',
                            type: 'secondary',
                            onclick: 'closeNotification()'
                        }]
                    });
                }
            } catch (error) {
                handleApiError(error, '执行清理');
            } finally {
                setButtonLoading(btn, false);
            }
        }

        // 导出配置相关函数
        function openExportModal() {
            document.getElementById('exportModal').classList.add('show');
            document.getElementById('exportIncludeSecrets').checked = true;
        }

        function closeExportModal() {
            document.getElementById('exportModal').classList.remove('show');
        }

        async function executeExport() {
            const includeSecrets = document.getElementById('exportIncludeSecrets').checked;
            const btn = event.target;
            setButtonLoading(btn, true);

            // 保存操作以供重试
            lastOperation = { context: '导出配置', func: executeExport };

            try {
                const response = await fetch('/api/export', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ include_secrets: includeSecrets })
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    // 创建下载链接
                    const blob = new Blob([data.data.content], { type: 'text/plain;charset=utf-8' });
                    const url = window.URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = data.data.filename;
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    window.URL.revokeObjectURL(url);

                    showNotification('✓ 配置导出成功', 'success', { icon: '📥' });
                    closeExportModal();
                } else {
                    showNotification(data.message || '导出失败', 'error', {
                        autoHide: false,
                        actions: [{
                            label: '关闭',
                            type: 'secondary',
                            onclick: 'closeNotification()'
                        }]
                    });
                }
            } catch (error) {
                handleApiError(error, '导出配置');
            } finally {
                setButtonLoading(btn, false);
            }
        }

        // 导入配置相关函数
        let importFileContent = null;

        function openImportModal() {
            document.getElementById('importModal').classList.add('show');
            document.getElementById('importFile').value = '';
            document.getElementById('importMode').value = 'merge';
            document.getElementById('importBackup').checked = true;
            document.getElementById('importPreview').style.display = 'none';
            importFileContent = null;
        }

        function closeImportModal() {
            document.getElementById('importModal').classList.remove('show');
        }

        // 监听文件选择
        document.addEventListener('DOMContentLoaded', () => {
            const fileInput = document.getElementById('importFile');
            if (fileInput) {
                fileInput.addEventListener('change', (e) => {
                    const file = e.target.files[0];
                    if (file) {
                        const reader = new FileReader();
                        reader.onload = (event) => {
                            importFileContent = event.target.result;
                            
                            // 显示预览
                            const preview = document.getElementById('importPreview');
                            const content = document.getElementById('importPreviewContent');
                            
                            // 尝试解析配置信息
                            try {
                                const lines = importFileContent.split('\n');
                                const sectionCount = (importFileContent.match(/\[sections\./g) || []).length;
                                const firstLines = lines.slice(0, 5).join('\n');
                                
                                content.innerHTML = `
                                    <div style="margin-bottom: 8px;">
                                        <span style="color: var(--accent-primary);">📄</span> 
                                        <span>文件名: ${file.name}</span>
                                    </div>
                                    <div style="margin-bottom: 8px;">
                                        <span style="color: var(--accent-success);">📊</span> 
                                        <span>检测到 ${sectionCount} 个配置节</span>
                                    </div>
                                    <div style="margin-bottom: 8px;">
                                        <span style="color: var(--text-muted);">前几行内容:</span>
                                    </div>
                                    <pre style="background: var(--bg-secondary); padding: 10px; border-radius: 6px; font-size: 11px; overflow-x: auto; color: var(--text-primary);">${firstLines}...</pre>
                                `;
                                preview.style.display = 'block';
                            } catch (error) {
                                content.innerHTML = `
                                    <div style="color: var(--accent-danger);">
                                        <span>⚠️</span> 文件格式可能有问题
                                    </div>
                                `;
                                preview.style.display = 'block';
                            }
                        };
                        reader.readAsText(file);
                    }
                });
            }
        });

        async function executeImport() {
            if (!importFileContent) {
                showNotification('请先选择配置文件', 'error', { icon: '⚠' });
                return;
            }

            const mode = document.getElementById('importMode').value;
            const backup = document.getElementById('importBackup').checked;
            const btn = document.getElementById('importExecuteBtn');
            setButtonLoading(btn, true);

            if (mode === 'replace' && !confirm('替换模式将完全覆盖现有配置，确定继续吗？')) {
                setButtonLoading(btn, false);
                return;
            }

            // 保存操作以供重试
            lastOperation = { context: '导入配置', func: executeImport };

            try {
                const response = await fetch('/api/import', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        content: importFileContent,
                        mode: mode,
                        backup: backup
                    })
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    const result = data.data;
                    let message = '✓ 配置导入成功\n';
                    if (result.added > 0) message += `新增: ${result.added} 个\n`;
                    if (result.updated > 0) message += `更新: ${result.updated} 个\n`;
                    if (result.skipped > 0) message += `跳过: ${result.skipped} 个`;

                    showNotification(message, 'success', { icon: '📤' });
                    closeImportModal();
                    await loadData();
                } else {
                    showNotification(data.message || '导入失败', 'error', {
                        autoHide: false,
                        actions: [{
                            label: '关闭',
                            type: 'secondary',
                            onclick: 'closeNotification()'
                        }]
                    });
                }
            } catch (error) {
                handleApiError(error, '导入配置');
            } finally {
                setButtonLoading(btn, false);
            }
        }

        // ESC 关闭模态框 & 🆕 键盘快捷键支持
        document.addEventListener('keydown', (e) => {
            // ESC 关闭所有模态框
            if (e.key === 'Escape') {
                closeModal();
                closeCleanModal();
                closeExportModal();
                closeImportModal();
                return;
            }

            // 🆕 Ctrl+1-5: 快速切换平台 (仅在 Unified 模式下)
            if (e.ctrlKey && platformInfo.mode === 'unified') {
                const platformMap = {
                    '1': 'claude',
                    '2': 'codex',
                    '3': 'gemini',
                    '4': 'qwen',
                    '5': 'iflow'
                };

                const platformName = platformMap[e.key];
                if (platformName) {
                    e.preventDefault(); // 防止浏览器默认行为

                    // 检查平台是否存在
                    const platform = platformInfo.availablePlatforms.find(p => p.name === platformName);
                    if (platform) {
                        switchPlatform(platformName);

                        // 显示快捷键提示
                        showNotification(
                            `快捷键切换: Ctrl+${e.key} → ${platformName}`,
                            'success',
                            { icon: '⌨️', duration: 1500 }
                        );
                    }
                }
            }

            // 🆕 Ctrl+Enter: 激活当前选中的平台
            if (e.ctrlKey && e.key === 'Enter' && platformInfo.mode === 'unified') {
                const activateBtn = document.getElementById('activatePlatformBtn');
                if (activateBtn && activateBtn.style.display !== 'none') {
                    e.preventDefault();
                    activateCurrentPlatform();
                }
            }

            // 🆕 Ctrl+R: 刷新数据
            if (e.ctrlKey && e.key === 'r') {
                e.preventDefault();
                loadData();
                showNotification('数据已刷新', 'success', { icon: '🔄', duration: 1500 });
            }
        });

        // ===== 🆕 多平台支持 (Multi-Platform Support) =====

        let platformInfo = {
            mode: 'legacy', // 'legacy' or 'unified'
            currentPlatform: null,
            availablePlatforms: []
        };

        // 在 DOMContentLoaded 中添加平台信息加载
        document.addEventListener('DOMContentLoaded', () => {
            loadPlatformInfo();
        });

        // 加载平台信息
        async function loadPlatformInfo() {
            try {
                const response = await fetch('/api/platforms');

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success && data.data) {
                    platformInfo = {
                        mode: data.data.mode,
                        currentPlatform: data.data.current_platform,
                        availablePlatforms: data.data.available_platforms || []
                    };

                    // 更新 UI
                    updatePlatformUI();

                    // 如果是 Unified 模式，渲染平台导航栏和状态
                    if (platformInfo.mode === 'unified') {
                        renderPlatformNavigation();
                        renderPlatformStatus();
                    }
                }
            } catch (error) {
                console.error('加载平台信息失败:', error);
                // 静默失败，使用默认的 Legacy 模式
                platformInfo = { mode: 'legacy', currentPlatform: null, availablePlatforms: [] };
                updatePlatformUI();
            }
        }

        // 更新平台相关的 UI 显示
        function updatePlatformUI() {
            const modeIndicator = document.getElementById('modeIndicator');
            const platformNavBar = document.getElementById('platformNavBar');
            const platformStatusSection = document.getElementById('platformStatusSection');
            const currentPlatformIndicator = document.getElementById('currentPlatformIndicator');

            if (platformInfo.mode === 'unified') {
                // Unified 模式
                modeIndicator.textContent = 'Unified 模式';
                modeIndicator.style.color = 'var(--accent-success)';

                // 显示平台导航栏
                if (platformNavBar) {
                    platformNavBar.style.display = 'flex';
                }

                // 显示平台状态区域
                if (platformStatusSection) {
                    platformStatusSection.style.display = 'block';
                }

                // 显示当前平台指示器
                if (currentPlatformIndicator && platformInfo.currentPlatform) {
                    currentPlatformIndicator.style.display = 'flex';
                    document.getElementById('activePlatformName').textContent = platformInfo.currentPlatform;
                }
            } else {
                // Legacy 模式
                modeIndicator.textContent = 'Legacy 模式';
                modeIndicator.style.color = 'var(--text-secondary)';

                // 隐藏平台导航栏
                if (platformNavBar) {
                    platformNavBar.style.display = 'none';
                }

                // 隐藏平台状态区域
                if (platformStatusSection) {
                    platformStatusSection.style.display = 'none';
                }

                // 隐藏当前平台指示器
                if (currentPlatformIndicator) {
                    currentPlatformIndicator.style.display = 'none';
                }
            }
        }

        // 渲染平台导航栏
        function renderPlatformNavigation() {
            if (platformInfo.mode !== 'unified') return;

            const platforms = platformInfo.availablePlatforms;

            // 更新每个平台标签的徽章数量
            platforms.forEach(platform => {
                const badge = document.getElementById(`badge-${platform.name}`);
                if (badge) {
                    // 🆕 显示平台的当前 profile（如果有的话）
                    // 在 Unified 模式下，每个平台可能有多个 profile
                    // 这里显示 profile 数量或配置数量
                    const profileCount = platform.current_profile ? 1 : 0;
                    badge.textContent = profileCount.toString();

                    // 🆕 如果是当前平台，显示当前配置的数量
                    if (platform.is_current && allConfigs.length > 0) {
                        badge.textContent = allConfigs.length.toString();
                    }
                }

                // 更新激活状态
                const tab = document.querySelector(`.platform-tab[data-platform="${platform.name}"]`);
                if (tab) {
                    if (platform.is_current) {
                        tab.classList.add('active');
                    } else {
                        tab.classList.remove('active');
                    }
                }
            });

            // 显示/隐藏激活按钮
            const activateBtn = document.getElementById('activatePlatformBtn');
            if (activateBtn) {
                // 如果当前选中的标签不是激活的平台，显示激活按钮
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

        // 渲染平台状态列表
        function renderPlatformStatus() {
            if (platformInfo.mode !== 'unified') return;

            const statusList = document.getElementById('platformStatusList');
            if (!statusList) return;

            const platforms = platformInfo.availablePlatforms;

            if (platforms.length === 0) {
                statusList.innerHTML = '<div style="text-align: center; color: var(--text-muted); padding: 20px; font-size: 12px;">暂无平台</div>';
                return;
            }

            // 平台图标映射
            const platformIcons = {
                'claude': '🤖',
                'codex': '💻',
                'gemini': '✨',
                'qwen': '🌟',
                'iflow': '⚡'
            };

            statusList.innerHTML = platforms.map(platform => {
                const icon = platformIcons[platform.name] || '📦';
                const badgeClass = platform.enabled ? '' : 'inactive';
                const badgeText = platform.enabled ? '已启用' : '已禁用';

                return `
                    <div class="platform-status-item ${platform.is_current ? 'active' : ''}"
                         onclick="switchPlatform('${platform.name}')">
                        <div class="platform-status-name">
                            <span class="platform-status-icon">${icon}</span>
                            <span>${platform.name}</span>
                        </div>
                        <span class="platform-status-badge ${badgeClass}">${badgeText}</span>
                    </div>
                `;
            }).join('');
        }

        // 切换平台 (在导航栏点击平台标签时调用)
        async function switchPlatform(platformName) {
            if (platformInfo.mode !== 'unified') {
                showNotification('平台切换仅在 Unified 模式下可用', 'error');
                return;
            }

            // 🆕 添加切换动画
            const clickedTab = document.querySelector(`.platform-tab[data-platform="${platformName}"]`);
            if (clickedTab) {
                clickedTab.classList.add('switching');
                setTimeout(() => clickedTab.classList.remove('switching'), 600);
            }

            // 更新 UI 选中状态
            document.querySelectorAll('.platform-tab').forEach(tab => {
                tab.classList.remove('active');
                if (tab.getAttribute('data-platform') === platformName) {
                    tab.classList.add('active');
                }
            });

            // 显示/隐藏激活按钮
            const currentPlatform = platformInfo.availablePlatforms.find(p => p.is_current);
            const activateBtn = document.getElementById('activatePlatformBtn');

            if (activateBtn) {
                if (currentPlatform && currentPlatform.name !== platformName) {
                    activateBtn.style.display = 'inline-block';
                    activateBtn.setAttribute('data-platform', platformName);
                } else {
                    activateBtn.style.display = 'none';
                }
            }

            // 加载该平台的配置数据
            // 在 Unified 模式下，切换平台标签时仅更新 UI
            // 实际的配置加载在激活平台后进行
            // 这里可以显示一个提示信息
            const platform = platformInfo.availablePlatforms.find(p => p.name === platformName);
            if (platform && !platform.is_current) {
                showNotification(
                    `已选择平台 "${platformName}"\n点击 "✓ 激活此平台" 按钮以切换到该平台`,
                    'info',
                    { icon: '💡', duration: 3000 }
                );
            }

            // 更新平台状态列表的选中状态
            renderPlatformStatus();
        }

        // 激活当前选中的平台
        async function activateCurrentPlatform() {
            const activeTab = document.querySelector('.platform-tab.active');
            if (!activeTab) {
                showNotification('请先选择要激活的平台', 'error');
                return;
            }

            const platformName = activeTab.getAttribute('data-platform');

            if (!confirm(`确定将 "${platformName}" 设置为激活平台吗？`)) {
                return;
            }

            const btn = document.getElementById('activatePlatformBtn');
            setButtonLoading(btn, true);

            // 🆕 添加加载指示器到按钮
            const originalText = btn.innerHTML;
            btn.innerHTML = '激活中... <span class="loading-indicator"></span>';

            try {
                const response = await fetch('/api/platforms/switch', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ platform_name: platformName })
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    // 🆕 成功激活后的视觉反馈
                    showNotification(`✓ 已激活平台: ${platformName}`, 'success');

                    // 🆕 平台标签激活动画
                    if (activeTab) {
                        activeTab.classList.add('activated');
                        activeTab.classList.add('success-pulse');
                        setTimeout(() => {
                            activeTab.classList.remove('activated');
                            activeTab.classList.remove('success-pulse');
                        }, 800);
                    }

                    // 🆕 平台状态项激活动画
                    const statusItems = document.querySelectorAll('.platform-status-item');
                    statusItems.forEach(item => {
                        if (item.textContent.includes(platformName)) {
                            item.classList.add('activating');
                            setTimeout(() => item.classList.remove('activating'), 500);
                        }
                    });

                    // 重新加载平台信息
                    await loadPlatformInfo();

                    // 重新加载配置数据
                    await loadData();
                } else {
                    showNotification(data.message || '激活失败', 'error', {
                        autoHide: false,
                        actions: [{
                            label: '关闭',
                            type: 'secondary',
                            onclick: 'closeNotification()'
                        }]
                    });
                }
            } catch (error) {
                handleApiError(error, '激活平台');
            } finally {
                setButtonLoading(btn, false);
                btn.innerHTML = originalText;
            }
        }
