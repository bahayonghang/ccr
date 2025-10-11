        let currentEditingConfig = null;
        let allConfigs = [];
        let notificationTimeout = null;

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
                const response = await fetch('/api/configs');

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();

                if (data.success) {
                    allConfigs = data.data.configs;
                    document.getElementById('currentConfigName').textContent = data.data.current_config || '-';
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

        // 渲染配置列表
        function renderConfigs() {
            const container = document.getElementById('configsList');
            if (allConfigs.length === 0) {
                container.innerHTML = '<div style="text-align: center; color: var(--text-muted); padding: 40px;">暂无配置</div>';
                return;
            }

            container.innerHTML = allConfigs.map(config => `
                <div id="config-${config.name}" class="config-card ${config.is_current ? 'active' : ''}">
                    <div class="config-header">
                        <div class="config-info">
                            <h3>${config.name}
                                ${config.is_current ? '<span class="badge badge-active">当前</span>' : ''}
                                ${config.is_default ? '<span class="badge badge-default">默认</span>' : ''}
                            </h3>
                            <div class="config-desc">${config.description || '无描述'}</div>
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
            `).join('');
        }

        // 渲染配置目录导航
        function renderConfigNav() {
            const nav = document.getElementById('configNav');
            if (allConfigs.length === 0) {
                nav.innerHTML = '<li class="config-nav-item"><span style="font-size: 12px; color: var(--text-muted);">暂无配置</span></li>';
                return;
            }

            nav.innerHTML = allConfigs.map(config => `
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
            document.getElementById('configModal').classList.add('show');
        }

        // 保存配置
        async function saveConfig(event) {
            event.preventDefault();

            const submitBtn = event.target.querySelector('button[type="submit"]');
            setButtonLoading(submitBtn, true);

            const configData = {
                name: document.getElementById('configName').value,
                description: document.getElementById('configDesc').value || null,
                base_url: document.getElementById('configBaseUrl').value,
                auth_token: document.getElementById('configAuthToken').value,
                model: document.getElementById('configModel').value || null,
                small_fast_model: document.getElementById('configSmallModel').value || null
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

        // ESC 关闭模态框
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                closeModal();
                closeCleanModal();
                closeExportModal();
                closeImportModal();
            }
        });
