import { appState } from './state.js';
import '../css/style.css'; // Import CSS for Vite bundling
import { 
    loadSystemInfo, loadData, loadConfigs, switchConfig, saveConfig, deleteConfig, enableConfig, disableConfig, 
    loadSyncStatus, saveSyncConfig, executeSyncPush, executeSyncPull, previewClean, executeClean, executeExport, executeImport,
    validateConfigs, loadPlatformInfo, editConfig, loadProviderStats, setImportFileContent
} from './api.js';
import { 
    initTheme, toggleTheme, switchTab, openModal, closeModal, openAddModal, openCleanModal, closeCleanModal,
    openExportModal, closeExportModal, openImportModal, closeImportModal, openProviderStatsModal, closeProviderModal,
    toggleCodexFieldsSection, resetCodexFields, updateCleanDaysDisplay
} from './ui.js';
import { renderConfigs, renderConfigNav, renderPlatformStatus, renderPlatformNavigation } from './render.js';
import { showNotification } from './ui-notifications.js';

// Expose functions to global scope for HTML onclick attributes
window.switchConfig = switchConfig;
window.editConfig = editConfig;
window.deleteConfig = deleteConfig;
window.enableConfig = enableConfig;
window.disableConfig = disableConfig;
window.saveConfig = saveConfig;
window.validateConfigs = validateConfigs;
window.toggleTheme = toggleTheme;
window.switchTab = switchTab;
window.openModal = openModal;
window.closeModal = closeModal;
window.openAddModal = openAddModal;
window.loadSyncStatus = loadSyncStatus;
window.saveSyncConfig = saveSyncConfig;
window.executeSyncPush = executeSyncPush;
window.executeSyncPull = executeSyncPull;
window.openCleanModal = openCleanModal;
window.closeCleanModal = closeCleanModal;
window.previewClean = previewClean;
window.executeClean = executeClean;
window.updateCleanDaysDisplay = updateCleanDaysDisplay;
window.openExportModal = openExportModal;
window.closeExportModal = closeExportModal;
window.executeExport = executeExport;
window.openImportModal = openImportModal;
window.closeImportModal = closeImportModal;
window.executeImport = executeImport;
window.openProviderStatsModal = openProviderStatsModal;
window.closeProviderModal = closeProviderModal;
window.loadProviderStats = loadProviderStats;
window.retryLastOperation = function(context) {
    if (appState.lastOperation && appState.lastOperation.func) {
        console.log(`Retrying ${appState.lastOperation.context}...`);
        appState.lastOperation.func();
    } else {
        console.warn('No operation to retry');
    }
};

window.scrollToConfig = function(name, event) {
    if (event) event.preventDefault();
    const element = document.getElementById(`config-${name}`);
    if (element) {
        element.scrollIntoView({ behavior: 'smooth', block: 'start' });
        element.classList.add('active');
        setTimeout(() => element.classList.remove('active'), 2000);
        
        // Update URL hash without jumping
        history.pushState(null, null, `#config-${name}`);
        
        // Update nav active state
        document.querySelectorAll('.config-nav-link').forEach(link => link.classList.remove('active'));
        if (event && event.target) {
            event.target.closest('.config-nav-link').classList.add('active');
        }
    }
};

window.switchPlatform = async function(platformName) {
    if (platformName === appState.platformInfo.currentPlatform) return;
    
    // Check implementation status
    if (['qwen', 'iflow'].includes(platformName)) {
        showNotification(`平台 "${platformName}" 尚未实现`, 'warning', { icon: '🚧' });
        return;
    }

    const tab = document.querySelector(`.platform-tab[data-platform="${platformName}"]`);
    if (tab) {
        tab.classList.add('switching');
        setTimeout(() => tab.classList.remove('switching'), 600);
    }

    // Optimistic update
    appState.platformInfo.currentPlatform = platformName;
    renderPlatformNavigation();
    renderPlatformStatus();
    
    // Clear configs list while loading
    document.getElementById('configsList').innerHTML = '<div class="loading-state"><div class="spinner"></div><div class="loading-text">正在切换平台...</div></div>';
    document.getElementById('currentConfigName').textContent = '-';

    await loadConfigs();
    
    // Update platform UI specific fields
    toggleCodexFieldsSection(isCodexPlatformActive());
};

window.activateCurrentPlatform = async function() {
    const btn = document.getElementById('activatePlatformBtn');
    if (!btn) return;
    
    const activeTab = document.querySelector('.platform-tab.active');
    if (!activeTab) return;
    
    const platformName = activeTab.getAttribute('data-platform');
    if (!platformName) return;

    btn.disabled = true;
    btn.innerHTML = '<span class="loading-indicator"></span> 激活中...';

    try {
        const response = await fetch('/api/platform/activate', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ platform: platformName })
        });
        
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const data = await response.json();
        
        if (data.success) {
            showNotification(`✓ 已激活平台: ${platformName}`, 'success');
            await loadPlatformInfo(); // Refresh global info
            await loadConfigs();
        } else {
            showNotification(data.message || '激活失败', 'error');
        }
    } catch (error) {
        console.error('Activate platform error:', error);
        showNotification('激活平台失败', 'error');
    } finally {
        btn.disabled = false;
        btn.innerHTML = '激活此平台';
        renderPlatformNavigation(); // Re-render to hide button if needed
    }
};

// Helper for other modules
export function isCodexPlatformActive() {
    return appState.platformInfo.mode === 'unified' && appState.platformInfo.currentPlatform === 'codex';
}

// Initialize
document.addEventListener('DOMContentLoaded', async () => {
    initTheme();
    
    // Setup global event listeners for static elements that don't have onclick in HTML
    // (Most are handled by window functions called by onclick in HTML)
    
    const importInput = document.getElementById('importFile');
    if (importInput) {
        importInput.addEventListener('change', (e) => {
            const file = e.target.files[0];
            if (!file) return;

            const reader = new FileReader();
            reader.onload = (e) => {
                setImportFileContent(e.target.result);
                document.getElementById('importPreview').style.display = 'block';
                document.getElementById('importPreviewContent').textContent = e.target.result;
            };
            reader.readAsText(file);
        });
    }

    const codexApiMode = document.getElementById('codexApiMode');
    if (codexApiMode) {
        codexApiMode.addEventListener('change', (e) => {
            const isCustom = e.target.value === 'custom';
            // Logic to show/hide custom fields if needed, currently all shown
        });
    }
    
    const filterBtns = document.querySelectorAll('.type-filter-btn');
    filterBtns.forEach(btn => {
        btn.addEventListener('click', () => {
            filterBtns.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            appState.currentFilter = btn.dataset.filter;
            renderConfigs();
            renderConfigNav();
        });
    });

    const sortSelect = document.getElementById('sortSelect');
    if (sortSelect) {
        sortSelect.addEventListener('change', (e) => {
            appState.currentSort = e.target.value;
            renderConfigs();
        });
    }

    const providerSort = document.getElementById('providerSort');
    if (providerSort) {
        providerSort.addEventListener('change', (e) => {
            appState.providerSortMode = e.target.value;
            if (appState.providerStatsCache) {
                renderProviderStats(appState.providerStatsCache); // Re-render with new sort
            }
        });
    }

    // Initial Load
    await loadSystemInfo();
    const platformLoaded = await loadPlatformInfo(true);
    if (platformLoaded) {
        await loadData();
    }
    
    // Periodic refresh
    setInterval(loadSystemInfo, 5000);
});
