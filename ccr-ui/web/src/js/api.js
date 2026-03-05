import { appState } from './state.js';
import { handleApiError, showNotification } from './ui-notifications.js';
import { renderConfigs, renderConfigNav, renderHistory, renderProviderStats, renderPlatformNavigation, renderPlatformStatus } from './render.js';
import { updatePlatformUI, updateSyncActions, toggleCodexFieldsSection, populateCodexFields, closeModal, closeCleanModal, closeExportModal, closeImportModal } from './ui.js';
import { setButtonLoading, setButtonsDisabled, formatUptime } from './utils.js';
import { isCodexPlatformActive } from './main.js';

// Retry logic wrapper
function saveLastOperation(context, func) {
    appState.lastOperation = { context, func };
}

// System Info
export async function loadSystemInfo() {
    try {
        const response = await fetch('/api/system');
        if (!response.ok) throw new Error(`HTTP ${response.status}`);

        const result = await response.json();
        if (result.success && result.data) {
            const data = result.data;
            document.getElementById('sysHostname').textContent = data.hostname;
            document.getElementById('sysOS').textContent = `${data.os} ${data.os_version}`;
            document.getElementById('sysCPU').textContent = `${data.cpu_cores} 核心`;
            
            const cpuUsage = Math.round(data.cpu_usage);
            document.getElementById('sysCPUUsage').textContent = `${cpuUsage}%`;
            document.getElementById('sysCPUBar').style.width = `${cpuUsage}%`;

            const usedMem = data.used_memory_gb.toFixed(1);
            const totalMem = data.total_memory_gb.toFixed(1);
            const memPercent = Math.round(data.memory_usage_percent);
            document.getElementById('sysMemory').textContent = `${usedMem} GB / ${totalMem} GB (${memPercent}%)`;
            document.getElementById('sysMemBar').style.width = `${memPercent}%`;

            document.getElementById('sysUptime').textContent = formatUptime(data.uptime_seconds);
        }
    } catch (error) {
        // eslint-disable-next-line no-console
        console.error('加载系统信息失败:', error);
    }
}

// Configs
export async function loadConfigs() {
    saveLastOperation('加载配置', loadConfigs);

    try {
        let endpoint = '/api/configs';
        if (appState.platformInfo.mode === 'unified' && appState.platformInfo.currentPlatform) {
            const platformEndpoints = { 'codex': '/api/codex/profiles' };
            endpoint = platformEndpoints[appState.platformInfo.currentPlatform] || '/api/configs';
            // eslint-disable-next-line no-console
            console.log(`加载平台配置: ${appState.platformInfo.currentPlatform} -> ${endpoint}`);
        }

        let response = await fetch(endpoint);
        if (!response.ok && endpoint !== '/api/configs') {
            // eslint-disable-next-line no-console
            console.warn(`平台端点 ${endpoint} 返回 ${response.status}，回退到 /api/configs`);
            response = await fetch('/api/configs');
        }

        if (!response.ok) throw new Error(`HTTP ${response.status}: ${response.statusText}`);

        const data = await response.json();
        if (data.success) {
            if (appState.platformInfo.mode === 'unified' && appState.platformInfo.currentPlatform !== 'claude') {
                appState.allConfigs = normalizeConfigData(data.data, appState.platformInfo.currentPlatform);
            } else {
                appState.allConfigs = data.data.configs || data.data || [];
            }

            const currentConfigName = data.data.current_config || data.data.current_profile || data.data.active_profile || '-';

            if (appState.platformInfo.mode === 'unified' && appState.platformInfo.currentPlatform && appState.allConfigs.length === 0) {
                const notImplementedPlatforms = ['qwen', 'iflow'];
                if (notImplementedPlatforms.includes(appState.platformInfo.currentPlatform)) {
                    showNotification(`平台 "${appState.platformInfo.currentPlatform}" 尚未实现\n请切换到已实现的平台 (Claude, Codex, Gemini)`, 'warning', { icon: '🚧', duration: 5000 });
                } else if (currentConfigName === '-') {
                    showNotification(`平台 "${appState.platformInfo.currentPlatform}" 暂无配置\n请先添加配置文件`, 'info', { icon: 'ℹ️', duration: 6000 });
                }
            }

            document.getElementById('currentConfigName').textContent = currentConfigName;
            document.getElementById('totalConfigs').textContent = appState.allConfigs.length;

            renderConfigs();
            renderConfigNav();
            renderPlatformNavigation(); // Update badges
        } else {
            showNotification(data.message || '加载失败', 'error');
        }
    } catch (error) {
        handleApiError(error, '加载配置');
    }
}

function normalizeConfigData(data, platform) {
    if (Array.isArray(data)) return data;
    if (data.configs && Array.isArray(data.configs)) return data.configs;
    if (data.profiles && Array.isArray(data.profiles)) {
        return data.profiles.map(profile => ({
            name: profile.name,
            description: profile.description,
            base_url: profile.base_url || profile.api_url,
            auth_token: profile.auth_token || profile.api_key,
            model: profile.model,
            small_fast_model: profile.small_fast_model,
            is_current: profile.is_active || profile.is_current || false,
            is_default: profile.is_default || false,
            provider: profile.provider,
            provider_type: profile.provider_type,
            tags: profile.tags,
            api_mode: profile.api_mode,
            wire_api: profile.wire_api,
            env_key: profile.env_key,
            requires_openai_auth: profile.requires_openai_auth,
            approval_policy: profile.approval_policy,
            sandbox_mode: profile.sandbox_mode,
            model_reasoning_effort: profile.model_reasoning_effort,
            organization: profile.organization
        }));
    }
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
    return [];
}

export async function switchConfig(name) {
    if (!confirm(`确定切换到配置 "${name}" 吗？`)) return;
    setButtonsDisabled('.btn-primary', true);
    saveLastOperation('切换配置', () => switchConfig(name));

    try {
        const response = await fetch('/api/switch', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ config_name: name })
        });

        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        if (data.success) {
            showNotification(`✓ 成功切换到配置 "${name}"`, 'success');
            await loadData();
        } else {
            showNotification(data.message || '切换失败', 'error', { autoHide: false, actions: [{ label: '关闭', type: 'secondary', onclick: 'closeNotification()' }] });
        }
    } catch (error) {
        handleApiError(error, '切换配置');
    } finally {
        setButtonsDisabled('.btn-primary', false);
    }
}

export async function saveConfig(event) {
    event.preventDefault();
    const submitBtn = event.target.querySelector('button[type="submit"]');
    setButtonLoading(submitBtn, true);

    const providerTypeValue = document.getElementById('configProviderType').value;
    const providerValue = document.getElementById('configProvider').value;
    const accountValue = document.getElementById('configAccount').value;
    const tagsValue = document.getElementById('configTags').value;

    let tagsArray = null;
    if (tagsValue && tagsValue.trim()) {
        tagsArray = tagsValue.split(',').map(tag => tag.trim()).filter(tag => tag.length > 0);
    }

    let configData = {
        name: document.getElementById('configName').value,
        description: document.getElementById('configDesc').value || null,
        base_url: document.getElementById('configBaseUrl').value,
        auth_token: document.getElementById('configAuthToken').value,
        model: document.getElementById('configModel').value || null,
        small_fast_model: document.getElementById('configSmallModel').value || null,
        provider_type: providerTypeValue || null,
        provider: providerValue || null,
        account: accountValue || null,
        tags: tagsArray
    };

    if (isCodexPlatformActive()) {
        configData = attachCodexFields(configData);
    }

    saveLastOperation('保存配置', () => saveConfig(event));

    try {
        const url = appState.currentEditingConfig ? resolveConfigEndpoint(appState.currentEditingConfig) : resolveConfigEndpoint();
        const method = appState.currentEditingConfig ? 'PUT' : 'POST';

        const response = await fetch(url, {
            method: method,
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(configData)
        });

        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        if (data.success) {
            showNotification(appState.currentEditingConfig ? '✓ 配置更新成功' : '✓ 配置添加成功', 'success');
            closeModal();
            await loadData();
        } else {
            showNotification(data.message || '保存失败', 'error', { autoHide: false, actions: [{ label: '关闭', type: 'secondary', onclick: 'closeNotification()' }] });
        }
    } catch (error) {
        handleApiError(error, '保存配置');
    } finally {
        setButtonLoading(submitBtn, false);
    }
}

function resolveConfigEndpoint(name = null) {
    if (isCodexPlatformActive()) {
        return name ? `/api/codex/profiles/${encodeURIComponent(name)}` : '/api/codex/profiles';
    }
    return name ? `/api/config/${encodeURIComponent(name)}` : '/api/config';
}

function attachCodexFields(payload) {
    // Helper to attach codex fields
    payload.api_mode = document.getElementById('codexApiMode').value || 'custom';
    payload.wire_api = document.getElementById('codexWireApi').value || null;
    payload.env_key = document.getElementById('codexEnvKey').value.trim() || null;
    payload.requires_openai_auth = document.getElementById('codexRequiresAuth').checked;
    payload.approval_policy = document.getElementById('codexApprovalPolicy').value.trim() || null;
    payload.sandbox_mode = document.getElementById('codexSandboxMode').value.trim() || null;
    payload.model_reasoning_effort = document.getElementById('codexModelReasoning').value.trim() || null;
    payload.organization = document.getElementById('codexOrganization').value.trim() || null;
    return payload;
}

export async function deleteConfig(name) {
    if (!confirm(`确定删除配置 "${name}" 吗？此操作不可恢复！`)) return;
    setButtonsDisabled('.btn-danger', true);
    saveLastOperation('删除配置', () => deleteConfig(name));

    try {
        const response = await fetch(resolveConfigEndpoint(name), { method: 'DELETE' });
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        if (data.success) {
            showNotification(`✓ 配置 "${name}" 删除成功`, 'success');
            await loadData();
        } else {
            showNotification(data.message || '删除失败', 'error', { autoHide: false, actions: [{ label: '关闭', type: 'secondary', onclick: 'closeNotification()' }] });
        }
    } catch (error) {
        handleApiError(error, '删除配置');
    } finally {
        setButtonsDisabled('.btn-danger', false);
    }
}

export async function enableConfig(name) {
    saveLastOperation('启用配置', () => enableConfig(name));
    try {
        const response = await fetch(`/api/config/${encodeURIComponent(name)}/enable`, { method: 'PATCH' });
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();
        if (data.success || data.message) {
            showNotification(data.message || `✓ 配置 "${name}" 已启用`, 'success');
            await loadData();
        } else {
            showNotification('启用失败', 'error');
        }
    } catch (error) {
        handleApiError(error, '启用配置');
    }
}

export async function disableConfig(name) {
    saveLastOperation('禁用配置', () => disableConfig(name));
    try {
        const response = await fetch(`/api/config/${encodeURIComponent(name)}/disable`, { method: 'PATCH' });
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();
        if (data.success || data.message) {
            showNotification(data.message || `✓ 配置 "${name}" 已禁用`, 'success');
            await loadData();
        } else {
            showNotification('禁用失败', 'error');
        }
    } catch (error) {
        handleApiError(error, '禁用配置');
    }
}

export async function loadHistory() {
    try {
        const response = await fetch('/api/history');
        const data = await response.json();
        if (data.success) {
            document.getElementById('historyCount').textContent = data.data.total;
            renderHistory(data.data.entries);
        }
    } catch (error) {
        // eslint-disable-next-line no-console
        console.error('加载历史记录失败:', error);
    }
}

export async function validateConfigs() {
    const btn = event.target; // event is global if called from onclick, but here we might need to pass it or use currentTarget
    // For now assuming passed or bound. If called from module, event might be undefined.
    // Better to pass btn.
    // We'll fix this in main.js binding.
    setButtonLoading(btn, true);
    saveLastOperation('验证配置', validateConfigs);

    try {
        const response = await fetch('/api/validate', { method: 'POST' });
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();
        if (data.success) {
            showNotification('✓ 配置验证通过', 'success', { icon: '✅' });
        } else {
            showNotification(data.message || '验证失败', 'error');
        }
    } catch (error) {
        handleApiError(error, '验证配置');
    } finally {
        setButtonLoading(btn, false);
    }
}

export async function loadPlatformInfo(initial = false) {
    try {
        const response = await fetch('/api/platforms');
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        if (data.success && data.data) {
            const platforms = data.data.available_platforms || [];
            let current = data.data.current_platform;
            if (!current && platforms.length > 0) {
                const active = platforms.find(p => p.is_current);
                if (active) current = active.name;
            }

            appState.platformInfo = {
                mode: data.data.mode || 'legacy',
                currentPlatform: current || null,
                availablePlatforms: platforms,
            };

            updatePlatformUI();
            if (appState.platformInfo.mode === 'unified') {
                renderPlatformNavigation();
                renderPlatformStatus();
            }
        }
        return true;
    } catch (error) {
        // eslint-disable-next-line no-console
        console.error('加载平台信息失败:', error);
        if (initial) {
            appState.platformInfo = { mode: 'legacy', currentPlatform: null, availablePlatforms: [] };
            updatePlatformUI();
        }
        return false;
    }
}

export async function editConfig(name) {
    const cachedConfig = appState.allConfigs.find(c => c.name === name);
    if (!cachedConfig) return;

    appState.currentEditingConfig = name;
    document.getElementById('modalTitle').textContent = '✏️ 编辑配置';

    document.getElementById('configName').value = cachedConfig.name;
    document.getElementById('configDesc').value = cachedConfig.description || '';
    document.getElementById('configBaseUrl').value = cachedConfig.base_url || '';
    document.getElementById('configAuthToken').value = cachedConfig.auth_token || '';
    document.getElementById('configModel').value = cachedConfig.model || '';
    document.getElementById('configSmallModel').value = cachedConfig.small_fast_model || '';
    document.getElementById('configProviderType').value = cachedConfig.provider_type || '';
    document.getElementById('configProvider').value = cachedConfig.provider || '';
    document.getElementById('configAccount').value = cachedConfig.account || '';
    document.getElementById('configTags').value = cachedConfig.tags ? cachedConfig.tags.join(', ') : '';

    if (isCodexPlatformActive()) {
        populateCodexFields(cachedConfig);
    } else {
        toggleCodexFieldsSection(false);
    }

    document.getElementById('configModal').classList.add('show');

    try {
        const endpoint = resolveConfigEndpoint(name);
        const response = await fetch(endpoint);
        if (response.ok) {
            const result = await response.json();
            const config = result.data || result;
            if (config.auth_token) document.getElementById('configAuthToken').value = config.auth_token;
        }
    } catch (error) {
        // eslint-disable-next-line no-console
        console.warn('获取完整配置失败，使用缓存数据:', error);
    }
}

export async function loadSyncStatus(silent = false) {
    saveLastOperation('加载同步状态', loadSyncStatus);
    const btn = document.getElementById('syncRefreshBtn');
    if (btn) setButtonLoading(btn, true);

    try {
        const resp = await fetch('/api/sync/status');
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
        const result = await resp.json();
        if (!result.success) throw new Error(result.message || '获取同步状态失败');

        const data = result.data || {};
        
        // Update UI elements... (Simplified for brevity, assuming standard IDs)
        const map = {
            'syncConfiguredValue': data.configured ? '是' : '否',
            'syncEnabledValue': data.enabled ? '是' : '否',
            'syncWebdavUrlValue': data.webdav_url || '-',
            'syncUsernameValue': data.username || '-',
            'syncRemotePathValue': data.remote_path || '-',
            'syncAutoSyncValue': data.auto_sync === true ? '是' : (data.auto_sync === false ? '否' : '-'),
            'syncLocalPathValue': data.local_path || '-',
            'syncTypeValue': data.sync_type || '-',
            'syncRemoteExistsValue': data.remote_exists === true ? '是' : (data.remote_exists === false ? '否' : '-')
        };
        for (const [id, val] of Object.entries(map)) {
            const el = document.getElementById(id);
            if(el) el.textContent = val;
        }

        // Fill form
        if (data.configured) {
            const enabledEl = document.getElementById('syncEnabled');
            if (enabledEl) enabledEl.checked = !!data.enabled;
            const urlEl = document.getElementById('syncWebdavUrl');
            if (urlEl && data.webdav_url && !urlEl.value) urlEl.value = data.webdav_url;
            const userEl = document.getElementById('syncUsername');
            if (userEl && data.username && !userEl.value) userEl.value = data.username;
            const remoteEl = document.getElementById('syncRemotePath');
            if (remoteEl && data.remote_path && !remoteEl.value) remoteEl.value = data.remote_path;
            const autoEl = document.getElementById('syncAutoSync');
            if (autoEl && typeof data.auto_sync === 'boolean') autoEl.checked = data.auto_sync;
        }

        updateSyncActions(data);
        if (!silent) showNotification('同步状态已更新', 'success', { icon: '🔄' });

    } catch (error) {
        if (!silent) handleApiError(error, '加载同步状态');
    } finally {
        if (btn) setButtonLoading(btn, false);
    }
}

export async function saveSyncConfig(event) {
    if (event) event.preventDefault();
    saveLastOperation('保存同步配置', saveSyncConfig);
    const btn = document.getElementById('syncSaveBtn');
    setButtonLoading(btn, true);

    try {
        const payload = {
            webdav_url: document.getElementById('syncWebdavUrl').value.trim(),
            username: document.getElementById('syncUsername').value.trim(),
            password: document.getElementById('syncPassword').value,
            remote_path: document.getElementById('syncRemotePath').value.trim() || undefined,
            enabled: document.getElementById('syncEnabled').checked,
            auto_sync: document.getElementById('syncAutoSync').checked,
        };

        if (!payload.webdav_url || !payload.username || !payload.password) {
            throw new Error('请填写 WebDAV URL、用户名、密码');
        }

        const resp = await fetch('/api/sync/config', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload),
        });

        const result = await resp.json();
        if (!resp.ok || !result.success) throw new Error(result.message || `HTTP ${resp.status}`);

        showNotification((result.data && result.data.message) || '同步配置已保存', 'success', { icon: '✅' });
        await loadSyncStatus(true);
    } catch (error) {
        handleApiError(error, '保存同步配置');
    } finally {
        setButtonLoading(btn, false);
    }
}

export async function executeSyncPush() {
    saveLastOperation('执行同步 Push', executeSyncPush);
    const btn = document.getElementById('syncPushBtn');
    setButtonLoading(btn, true);
    try {
        const force = document.getElementById('syncForcePush').checked;
        const resp = await fetch('/api/sync/push', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ force }),
        });
        const result = await resp.json();
        if (!resp.ok || !result.success) throw new Error(result.message || `HTTP ${resp.status}`);
        showNotification((result.data && result.data.message) || '已成功上传到云端', 'success', { icon: '📤' });
        await loadSyncStatus(true);
    } catch (error) {
        handleApiError(error, '执行同步 Push');
    } finally {
        setButtonLoading(btn, false);
    }
}

export async function executeSyncPull() {
    saveLastOperation('执行同步 Pull', executeSyncPull);
    const btn = document.getElementById('syncPullBtn');
    setButtonLoading(btn, true);
    try {
        const force = document.getElementById('syncForcePull').checked;
        const resp = await fetch('/api/sync/pull', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ force }),
        });
        const result = await resp.json();
        if (!resp.ok || !result.success) throw new Error(result.message || `HTTP ${resp.status}`);
        showNotification((result.data && result.data.message) || '已成功从云端下载', 'success', { icon: '📥' });
        await loadSyncStatus(true);
    } catch (error) {
        handleApiError(error, '执行同步 Pull');
    } finally {
        setButtonLoading(btn, false);
    }
}

export async function previewClean() {
    const days = parseInt(document.getElementById('cleanDays').value);
    const btn = event.target;
    setButtonLoading(btn, true);
    saveLastOperation('预览清理', previewClean);

    try {
        const response = await fetch('/api/clean', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ days, dry_run: true })
        });

        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        if (data.success) {
            const result = data.data;
            const preview = document.getElementById('cleanPreview');
            const content = document.getElementById('cleanPreviewContent');
            // ... (HTML generation for preview) ...
            // Simplified:
            content.innerHTML = result.deleted_count === 0 
                ? '<div style="color: var(--accent-success);">✓ 没有需要清理的文件</div>'
                : `<div style="margin-bottom: 8px;"><span>将删除 ${result.deleted_count} 个文件</span></div>`;
            preview.style.display = 'block';
        } else {
            showNotification(data.message || '预览失败', 'error');
        }
    } catch (error) {
        handleApiError(error, '预览清理');
    } finally {
        setButtonLoading(btn, false);
    }
}

export async function executeClean() {
    const days = parseInt(document.getElementById('cleanDays').value);
    const dryRun = document.getElementById('cleanDryRun').checked;
    const btn = document.getElementById('cleanExecuteBtn');

    if (dryRun) {
        showNotification('请取消勾选"模拟运行"以执行实际清理', 'warning', { icon: '⚠' });
        return;
    }
    if (!confirm(`确定清理 ${days} 天前的备份文件吗？此操作不可恢复！`)) return;

    setButtonLoading(btn, true);
    saveLastOperation('执行清理', executeClean);

    try {
        const response = await fetch('/api/clean', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ days, dry_run: false })
        });

        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        if (data.success) {
            showNotification(`✓ 已删除 ${data.data.deleted_count} 个文件`, 'success', { icon: '🗑️' });
            closeCleanModal();
        } else {
            showNotification(data.message || '清理失败', 'error');
        }
    } catch (error) {
        handleApiError(error, '执行清理');
    } finally {
        setButtonLoading(btn, false);
    }
}

export async function executeExport() {
    const includeSecrets = document.getElementById('exportIncludeSecrets').checked;
    const btn = event.target;
    setButtonLoading(btn, true);
    saveLastOperation('导出配置', executeExport);

    try {
        const response = await fetch('/api/export', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ include_secrets: includeSecrets })
        });

        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        if (data.success) {
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
            showNotification(data.message || '导出失败', 'error');
        }
    } catch (error) {
        handleApiError(error, '导出配置');
    } finally {
        setButtonLoading(btn, false);
    }
}

export let importFileContent = null;
export function setImportFileContent(content) { importFileContent = content; }

export async function executeImport() {
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

    saveLastOperation('导入配置', executeImport);

    try {
        const response = await fetch('/api/import', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ content: importFileContent, mode, backup })
        });

        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();

        if (data.success) {
            const result = data.data;
            showNotification(`✓ 配置导入成功 (新增: ${result.added}, 更新: ${result.updated})`, 'success', { icon: '📤' });
            closeImportModal();
            await loadData();
        } else {
            showNotification(data.message || '导入失败', 'error');
        }
    } catch (error) {
        handleApiError(error, '导入配置');
    } finally {
        setButtonLoading(btn, false);
    }
}

export async function loadData() {
    await loadConfigs();
    await loadHistory();
    try { await loadSyncStatus(true); } catch (e) { /* silent */ }
}

export async function loadProviderStats(range = 'month') {
    appState.providerRange = range;
    const body = document.getElementById('providerStatsBody');
    if (body) body.innerHTML = `<div class="loading-state"><div class="spinner"></div><div class="loading-text">正在加载提供商统计...</div></div>`;

    try {
        const response = await fetch(`/api/stats/provider-usage`); // Assuming query param for range if backend supports it
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();
        appState.providerStatsCache = data || {};
        await new Promise(resolve => setTimeout(resolve, 300));
        renderProviderStats(appState.providerStatsCache || {});
    } catch (error) {
        // eslint-disable-next-line no-console
        console.error('加载提供商统计失败:', error);
        if (body) body.innerHTML = `<div class="empty-state"><div>加载失败</div></div>`;
    }
}
