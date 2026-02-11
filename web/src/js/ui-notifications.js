import { appState } from './state.js';

export function showNotification(message, type = 'success', options = {}) {
    const notification = document.getElementById('notification');
    
    if (appState.notificationTimeout) {
        clearTimeout(appState.notificationTimeout);
    }

    const icons = {
        success: '✓',
        error: '✗',
        warning: '⚠',
        info: 'ℹ'
    };

    const icon = options.icon || icons[type] || 'ℹ';

    let content = `
        <div class="notification-content">
            <span class="notification-icon">${icon}</span>
            <div class="notification-text">${message}</div>
        </div>
    `;

    if (options.actions && options.actions.length > 0) {
        content += '<div class="notification-actions">';
        options.actions.forEach(action => {
            // Note: onclick string won't work well with modules unless functions are global
            // We'll need to handle this. For now, we assume simple actions or global bindings.
            // Better to use event delegation or direct DOM creation.
            // Simplified for string injection:
            content += `<button class="notification-btn notification-btn-${action.type || 'secondary'}" data-action="${action.onclick}">${action.label}</button>`;
        });
        content += '</div>';
    }

    notification.innerHTML = content;
    notification.className = `notification ${type} show`;

    // Bind clicks for actions
    const btns = notification.querySelectorAll('.notification-btn');
    btns.forEach(btn => {
        btn.onclick = () => {
            const action = btn.getAttribute('data-action');
            if (action === 'closeNotification()') {
                closeNotification();
            } else if (action.startsWith('retryLastOperation')) {
                // Extract context? Or just call retry
                // retryLastOperation is imported or global? 
                // We'll fix this later.
                if (window.retryLastOperation) window.retryLastOperation();
            }
        };
    });

    if (options.autoHide !== false) {
        appState.notificationTimeout = setTimeout(() => {
            notification.classList.remove('show');
        }, options.duration || 4000);
    }
}

export function closeNotification() {
    const notification = document.getElementById('notification');
    notification.classList.remove('show');
}

export function handleApiError(error, context = '') {
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

    if (retry) {
        options.actions.unshift({
            label: '重试',
            type: 'primary',
            onclick: `retryLastOperation('${retry}')`
        });
    }

    showNotification(message, 'error', options);
}
