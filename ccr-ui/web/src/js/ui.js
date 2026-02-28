import { appState } from './state.js';
import { loadSyncStatus, loadProviderStats, loadData } from './api.js'; // Imports from API
import { renderProviderStats } from './render.js';
import { setButtonLoading } from './utils.js';
import { showNotification } from './ui-notifications.js';
import { isCodexPlatformActive } from './main.js';

export function initTheme() {
    const savedTheme = localStorage.getItem('ccr-theme') || 'light';
    setTheme(savedTheme);
}

export function toggleTheme() {
    const currentTheme = document.documentElement.getAttribute('data-theme');
    const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
    localStorage.setItem('ccr-theme', newTheme);
}

function setTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    const themeIcon = document.getElementById('themeIcon');
    if (themeIcon) {
        themeIcon.textContent = theme === 'dark' ? '🌙' : '☀️';
    }
}

export function switchTab(tab, evt) {
    document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));

    if (evt && evt.target) {
        evt.target.classList.add('active');
    } else {
        const btns = Array.from(document.querySelectorAll('.tab-btn'));
        const match = btns.find(b => {
            const text = b.textContent.trim();
            if (tab === 'configs') return text.includes('配置') || text.includes('配置列表');
            if (tab === 'history') return text.includes('历史');
            if (tab === 'sync') return text.includes('同步') || text.includes('云同步');
            return false;
        });
        if (match) match.classList.add('active');
    }

    const content = document.getElementById(tab + '-tab');
    if (content) {
        content.classList.add('active');
    }

    if (tab === 'sync') {
        loadSyncStatus();
    }
}

export function openModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) modal.classList.add('show');
}

export function closeModal() {
    document.getElementById('configModal').classList.remove('show');
}

export function openAddModal() {
    appState.currentEditingConfig = null;
    document.getElementById('modalTitle').textContent = '➕ 添加配置';
    document.getElementById('modalTitle').textContent = '✏️ 编辑配置'; // Default to edit icon, will be overwritten if adding
    document.getElementById('configForm').reset();

    // Set default models
    document.getElementById('configModel').value = 'claude-opus-4-5-20251101';
    document.getElementById('configSmallModel').value = 'claude-haiku-4-5-20251001';

    if (isCodexPlatformActive()) {
        toggleCodexFieldsSection(true);
        resetCodexFields();
    } else {
        toggleCodexFieldsSection(false);
    }
    document.getElementById('configModal').classList.add('show');
}

export function toggleCodexFieldsSection(show) {
    const section = document.getElementById('codexFieldsSection');
    if (!section) return;
    section.style.display = show ? 'block' : 'none';
}

export function resetCodexFields() {
    const apiMode = document.getElementById('codexApiMode');
    if (!apiMode) return;
    apiMode.value = 'custom';
    document.getElementById('codexWireApi').value = 'responses';
    document.getElementById('codexEnvKey').value = '';
    document.getElementById('codexRequiresAuth').checked = true;
    document.getElementById('codexOrganization').value = '';
    document.getElementById('codexApprovalPolicy').value = '';
    document.getElementById('codexSandboxMode').value = '';
    document.getElementById('codexModelReasoning').value = '';
}

export function populateCodexFields(config) {
    toggleCodexFieldsSection(true);
    document.getElementById('codexApiMode').value = config.api_mode || 'custom';
    document.getElementById('codexWireApi').value = config.wire_api || 'responses';
    document.getElementById('codexEnvKey').value = config.env_key || '';
    document.getElementById('codexRequiresAuth').checked = config.requires_openai_auth !== false;
    document.getElementById('codexOrganization').value = config.organization || '';
    document.getElementById('codexApprovalPolicy').value = config.approval_policy || '';
    document.getElementById('codexSandboxMode').value = config.sandbox_mode || '';
    document.getElementById('codexModelReasoning').value = config.model_reasoning_effort || '';
}

export function openCleanModal() {
    document.getElementById('cleanModal').style.display = 'flex';
    document.getElementById('cleanPreview').style.display = 'none';
    document.getElementById('cleanDays').value = 7;
    document.getElementById('cleanDryRun').checked = true;
    updateCleanDaysDisplay();
}

export function closeCleanModal() {
    document.getElementById('cleanModal').style.display = 'none';
    document.getElementById('cleanPreview').style.display = 'none';
}

export function updateCleanDaysDisplay() {
    const days = document.getElementById('cleanDays').value;
    document.getElementById('cleanDaysDisplay').textContent = days;
}

export function openExportModal() {
    document.getElementById('exportModal').classList.add('show');
    document.getElementById('exportIncludeSecrets').checked = true;
}

export function closeExportModal() {
    document.getElementById('exportModal').classList.remove('show');
}

export function openImportModal() {
    document.getElementById('importModal').classList.add('show');
    document.getElementById('importFile').value = '';
    document.getElementById('importMode').value = 'merge';
    document.getElementById('importBackup').checked = true;
    document.getElementById('importPreview').style.display = 'none';
    // Reset file content var if exported or managed
}

export function closeImportModal() {
    document.getElementById('importModal').classList.remove('show');
}

export function openProviderStatsModal() {
    const modal = document.getElementById('providerModal');
    if (modal) {
        modal.classList.add('show');
        modal.style.animation = 'fadeIn 0.3s ease';
        const defaultRange = document.getElementById('providerRange')?.value || 'month';
        const sortSelect = document.getElementById('providerSort');
        if (sortSelect) {
            sortSelect.value = appState.providerSortMode;
        }
        loadProviderStats(defaultRange);
    }
}

export function closeProviderModal() {
    const modal = document.getElementById('providerModal');
    if (modal) {
        modal.style.animation = 'fadeOut 0.2s ease';
        setTimeout(() => {
            modal.classList.remove('show');
        }, 200);
    }
}

export function updatePlatformUI() {
    const modeIndicator = document.getElementById('modeIndicator');
    const platformNavBar = document.getElementById('platformNavBar');
    const platformStatusSection = document.getElementById('platformStatusSection');
    const currentPlatformIndicator = document.getElementById('currentPlatformIndicator');

    if (appState.platformInfo.mode === 'unified') {
        modeIndicator.textContent = 'Unified 模式';
        modeIndicator.style.color = 'var(--accent-success)';
        if (platformNavBar) platformNavBar.style.display = 'flex';
        if (platformStatusSection) platformStatusSection.style.display = 'block';
        if (currentPlatformIndicator && appState.platformInfo.currentPlatform) {
            currentPlatformIndicator.style.display = 'flex';
            document.getElementById('activePlatformName').textContent = appState.platformInfo.currentPlatform;
        }
    } else {
        modeIndicator.textContent = 'Legacy 模式';
        modeIndicator.style.color = 'var(--text-secondary)';
        if (platformNavBar) platformNavBar.style.display = 'none';
        if (platformStatusSection) platformStatusSection.style.display = 'none';
        if (currentPlatformIndicator) currentPlatformIndicator.style.display = 'none';
    }

    toggleCodexFieldsSection(isCodexPlatformActive());
}

export function updateSyncActions(data) {
    const pushBtn = document.getElementById('syncPushBtn');
    const pullBtn = document.getElementById('syncPullBtn');
    const forcePush = document.getElementById('syncForcePush');
    const forcePull = document.getElementById('syncForcePull');

    const configured = !!data.configured;
    const enabled = !!data.enabled;
    const disabled = !(configured && enabled);

    if (pushBtn) {
        pushBtn.disabled = disabled;
        pushBtn.title = disabled ? '请先启用并保存同步配置' : '上传本地配置到云端';
    }
    if (pullBtn) {
        pullBtn.disabled = disabled;
        pullBtn.title = disabled ? '请先启用并保存同步配置' : '从云端下载配置到本地';
    }
    if (forcePush) forcePush.disabled = disabled;
    if (forcePull) forcePull.disabled = disabled;
}
