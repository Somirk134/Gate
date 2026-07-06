<template>
    <div class="notification-layer">
        <transition-group name="notification" tag="div" class="notification-list">
            <div
                v-for="notif in notificationStore.activeNotifications"
                :key="notif.id"
                class="notification-item"
                :class="`type-${notif.type}`"
                @click="notificationStore.dismiss(notif.id)"
            >
                <div class="notification-icon">
                    <GIcon :name="iconForType(notif.type)" :size="16" />
                </div>
                <div class="notification-content">
                    <div class="notification-title">{{ notif.title }}</div>
                    <div v-if="notif.content" class="notification-body">{{ notif.content }}</div>
                </div>
                <button class="notification-close" @click.stop="notificationStore.dismiss(notif.id)">
                    <GIcon name="close" :size="12" />
                </button>
            </div>
        </transition-group>
    </div>
</template>

<script setup lang="ts">
import { useNotificationStore } from "@stores"
import type { NotificationType } from "@/types/shell"
import GIcon from "@components/icons/GIcon.vue"

const notificationStore = useNotificationStore()

function iconForType(type: NotificationType): string {
    switch (type) {
        case 'success': return 'check-circle'
        case 'error': return 'alert-circle'
        case 'warning': return 'alert-triangle'
        case 'info': return 'info-circle'
        default: return 'info-circle'
    }
}
</script>

<style scoped>
.notification-layer {
    position: fixed;
    top: var(--space-4);
    right: var(--space-4);
    z-index: var(--z-toast);
    pointer-events: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    width: min(380px, calc(100vw - 32px));
}

.notification-list {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
}

.notification-item {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    min-height: 58px;
    padding: var(--space-3);
    background: color-mix(in srgb, var(--bg-surface-raised) 92%, transparent);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-popup);
    backdrop-filter: blur(14px);
    cursor: pointer;
    transition: all var(--duration-fast) var(--ease-out);
    animation: g-toast-in var(--duration-base) var(--ease-out);
}

.notification-item:hover {
    border-color: var(--border-strong);
    transform: translateX(-2px);
    transition: all var(--duration-fast) var(--ease-out);
}

.notification-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: var(--radius-md);
    flex-shrink: 0;
}

.notification-item.type-success .notification-icon {
    color: var(--color-success);
    background: var(--color-success-muted);
}
.notification-item.type-error .notification-icon {
    color: var(--color-error);
    background: var(--color-error-muted);
}
.notification-item.type-warning .notification-icon {
    color: var(--color-warning);
    background: var(--color-warning-muted);
}
.notification-item.type-info .notification-icon {
    color: var(--color-info);
    background: var(--color-info-muted);
}

.notification-content {
    flex: 1;
    min-width: 0;
}

.notification-title {
    font-size: var(--text-sm);
    font-weight: var(--weight-semibold);
    color: var(--text-primary);
}

.notification-body {
    font-size: var(--text-xs);
    color: var(--text-secondary);
    margin-top: 2px;
    line-height: var(--leading-relaxed);
}

.notification-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: none;
    background: transparent;
    color: var(--text-tertiary);
    border-radius: var(--radius-sm);
    cursor: pointer;
    flex-shrink: 0;
    transition: all var(--duration-micro);
}

.notification-close:hover {
    background: var(--bg-surface-hover);
    color: var(--text-primary);
}

/* ── Transitions ── */
.notification-enter-active,
.notification-leave-active {
    transition: all var(--duration-standard) var(--ease-out);
}

.notification-enter-from {
    opacity: 0;
    transform: translateX(20px);
}

.notification-leave-to {
    opacity: 0;
    transform: translateX(20px) scale(0.95);
}
</style>
