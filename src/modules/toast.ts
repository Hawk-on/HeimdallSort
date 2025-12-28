export type ToastType = 'success' | 'error' | 'info' | 'warning';

export class ToastManager {
    private container: HTMLElement;

    constructor() {
        this.container = document.createElement('div');
        this.container.id = 'toast-container';
        this.container.style.cssText = `
        position: fixed;
        bottom: 24px;
        right: 24px;
        display: flex;
        flex-direction: column;
        gap: 12px;
        z-index: 9999;
        pointer-events: none;
    `;
        document.body.appendChild(this.container);
    }

    public show(message: string, type: ToastType = 'info', duration: number = 3000) {
        const toast = document.createElement('div');
        toast.className = `toast toast-${type}`;

        // Add icon based on type
        let icon = '';
        switch (type) {
            case 'success': icon = '✅'; break;
            case 'error': icon = '❌'; break;
            case 'warning': icon = '⚠️'; break;
            default: icon = 'ℹ️';
        }

        toast.innerHTML = `
        <span class="toast-icon">${icon}</span>
        <span class="toast-message">${message}</span>
    `;

        // Basic Styles (Injecting here for simplicity, or move to css)
        toast.style.cssText = `
        background: rgba(18, 18, 26, 0.95);
        color: white;
        padding: 12px 16px;
        border-radius: 8px;
        border: 1px solid rgba(255,255,255,0.1);
        border-left: 4px solid ${this.getColor(type)};
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
        display: flex;
        align-items: center;
        gap: 12px;
        min-width: 300px;
        transform: translateX(100%);
        opacity: 0;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        pointer-events: auto;
        backdrop-filter: blur(8px);
        font-family: var(--font-family-base, sans-serif);
        font-size: 0.9rem;
    `;

        this.container.appendChild(toast);

        // Animate in
        requestAnimationFrame(() => {
            toast.style.transform = 'translateX(0)';
            toast.style.opacity = '1';
        });

        // Auto remove
        setTimeout(() => {
            toast.style.transform = 'translateX(100%)';
            toast.style.opacity = '0';
            setTimeout(() => {
                if (toast.parentElement) this.container.removeChild(toast);
            }, 300);
        }, duration);
    }

    private getColor(type: ToastType): string {
        switch (type) {
            case 'success': return '#00e676';
            case 'error': return '#ff1744';
            case 'warning': return '#ffea00';
            case 'info': return '#7c4dff';
            default: return '#ccc';
        }
    }
}

export const toast = new ToastManager();
