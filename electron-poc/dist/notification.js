"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.notifications = exports.NotificationManager = void 0;
require("./notification.css");
class NotificationManager {
    constructor() {
        this.container = null;
        this.notifications = new Map();
        this.initContainer();
    }
    static getInstance() {
        if (!NotificationManager.instance) {
            NotificationManager.instance = new NotificationManager();
        }
        return NotificationManager.instance;
    }
    initContainer() {
        if (!this.container) {
            this.container = document.createElement('div');
            this.container.className = 'notification-container';
            document.body.appendChild(this.container);
        }
    }
    show(options) {
        var _a;
        const id = Date.now().toString();
        const { title, message, type = 'info', duration = 3000, closeable = true } = options;
        const notification = document.createElement('div');
        notification.className = `notification ${type}`;
        const iconMap = {
            success: '✓',
            error: '✕',
            warning: '⚠',
            info: 'ℹ',
            loading: '↻'
        };
        notification.innerHTML = `
      <span class="notification-icon codicon ${type === 'loading' ? '' : `codicon-${type === 'success' ? 'check' : type === 'error' ? 'error' : 'info'}`}">
        ${type === 'loading' ? '↻' : ''}
      </span>
      <div class="notification-content">
        ${title ? `<div class="notification-title">${title}</div>` : ''}
        <div class="notification-message">${message}</div>
      </div>
      ${closeable ? '<button class="notification-close codicon codicon-close"></button>' : ''}
    `;
        if (closeable) {
            const closeBtn = notification.querySelector('.notification-close');
            closeBtn === null || closeBtn === void 0 ? void 0 : closeBtn.addEventListener('click', () => this.hide(id));
        }
        (_a = this.container) === null || _a === void 0 ? void 0 : _a.appendChild(notification);
        this.notifications.set(id, notification);
        // Auto-hide after duration (unless it's persistent or loading)
        if (duration > 0 && type !== 'loading') {
            setTimeout(() => this.hide(id), duration);
        }
        return id;
    }
    update(id, options) {
        const notification = this.notifications.get(id);
        if (!notification)
            return;
        const { message, type, title } = options;
        if (type) {
            notification.className = `notification ${type}`;
        }
        const contentEl = notification.querySelector('.notification-content');
        if (contentEl && (message || title !== undefined)) {
            contentEl.innerHTML = `
        ${title ? `<div class="notification-title">${title}</div>` : ''}
        <div class="notification-message">${message || ''}</div>
      `;
        }
        // If updating from loading to success/error, auto-hide
        if (type && type !== 'loading' && options.duration !== 0) {
            setTimeout(() => this.hide(id), options.duration || 3000);
        }
    }
    hide(id) {
        const notification = this.notifications.get(id);
        if (!notification)
            return;
        notification.classList.add('hiding');
        setTimeout(() => {
            notification.remove();
            this.notifications.delete(id);
        }, 300);
    }
    hideAll() {
        this.notifications.forEach((_, id) => this.hide(id));
    }
}
exports.NotificationManager = NotificationManager;
// Export singleton instance
exports.notifications = NotificationManager.getInstance();
//# sourceMappingURL=notification.js.map