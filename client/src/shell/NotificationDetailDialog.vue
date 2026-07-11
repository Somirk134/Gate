<template>
  <Teleport to="body">
    <Transition name="notification-detail">
      <div
        v-if="item"
        class="notification-detail-overlay"
        @click.self="notificationStore.closeDetail"
        @keydown.esc="notificationStore.closeDetail">
        <div
          class="notification-detail-dialog"
          :class="`is-${item.type}`"
          role="dialog"
          aria-modal="true"
          :aria-label="t('common.notificationDetail')">
          <div class="notification-detail-dialog__header">
            <div class="notification-detail-dialog__badge">
              <GIcon :name="iconForType(item.type)" :size="18" />
              <span>{{ notificationTypeLabel(item.type) }}</span>
            </div>
            <button
              class="notification-detail-dialog__close"
              type="button"
              :title="t('common.closeNotification')"
              @click="notificationStore.closeDetail">
              <GIcon name="close" :size="16" />
            </button>
          </div>

          <div class="notification-detail-dialog__body">
            <h3>{{ item.title }}</h3>
            <p v-if="item.content" class="notification-detail-dialog__content">{{ item.content }}</p>
            <time class="notification-detail-dialog__time">{{ formatTime(item.timestamp) }}</time>
          </div>

          <div class="notification-detail-dialog__footer">
            <button
              class="notification-detail-dialog__btn notification-detail-dialog__btn--secondary"
              type="button"
              @click="copyFullMessage">
              <GIcon name="clipboard" :size="14" />
              {{ t('common.copyContent') }}
            </button>
            <button
              class="notification-detail-dialog__btn notification-detail-dialog__btn--primary"
              type="button"
              @click="notificationStore.closeDetail">
              {{ t('dialog.ok') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useNotificationStore } from '@stores'
import type { NotificationType } from '@/stores/modules/notification'
import GIcon from '@components/icons/GIcon.vue'
import { useFeedback } from '@/composables/useFeedback'

const notificationStore = useNotificationStore()
const { t, locale } = useI18n()
const { toast } = useFeedback()

const item = computed(() => notificationStore.detailItem)

function notificationTypeLabel(type: NotificationType) {
  return t(`common.notificationType.${type}`)
}

function iconForType(type: NotificationType): string {
  switch (type) {
    case 'success':
      return 'check-circle'
    case 'error':
      return 'alert-circle'
    case 'warning':
      return 'alert-triangle'
    default:
      return 'info-circle'
  }
}

function formatTime(timestamp: number) {
  return new Intl.DateTimeFormat(locale.value === 'en-US' ? 'en-US' : 'zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(timestamp)
}

function fullMessageText() {
  if (!item.value) return ''
  if (item.value.content) {
    return `${item.value.title}\n\n${item.value.content}`
  }
  return item.value.title
}

async function copyFullMessage() {
  try {
    await navigator.clipboard.writeText(fullMessageText())
    toast.success(t('common.copiedWithValue', { value: t('common.copyContent') }))
  } catch {
    toast.error(t('common.copyFailed'))
  }
}
</script>

<style scoped>
.notification-detail-overlay {
  position: fixed;
  inset: 0;
  z-index: calc(var(--z-modal) + 1);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-4);
  background: var(--color-overlay);
  backdrop-filter: blur(8px);
}

.notification-detail-dialog {
  width: min(520px, 100%);
  max-height: min(80vh, 640px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-dialog);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.notification-detail-dialog__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-5) var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
}

.notification-detail-dialog__badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  min-height: 32px;
  padding: 0 var(--space-3);
  border-radius: var(--radius-full);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
}

.notification-detail-dialog.is-success .notification-detail-dialog__badge {
  color: var(--color-success);
  background: var(--color-success-muted);
}

.notification-detail-dialog.is-error .notification-detail-dialog__badge {
  color: var(--color-error);
  background: var(--color-error-muted);
}

.notification-detail-dialog.is-warning .notification-detail-dialog__badge {
  color: var(--color-warning);
  background: var(--color-warning-muted);
}

.notification-detail-dialog.is-info .notification-detail-dialog__badge {
  color: var(--color-info);
  background: var(--color-info-muted);
}

.notification-detail-dialog__close {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.notification-detail-dialog__close:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.notification-detail-dialog__body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: var(--space-4) var(--space-5);
}

.notification-detail-dialog__body h3 {
  margin: 0;
  color: var(--text-primary);
  font-size: var(--text-base);
  font-weight: var(--weight-semibold);
  line-height: var(--leading-relaxed);
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.notification-detail-dialog__content {
  margin: var(--space-3) 0 0;
  color: var(--text-secondary);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
  overflow-wrap: anywhere;
  white-space: pre-wrap;
}

.notification-detail-dialog__time {
  display: block;
  margin-top: var(--space-4);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.notification-detail-dialog__footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-5) var(--space-4);
  border-top: 1px solid var(--border-subtle);
}

.notification-detail-dialog__btn {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  height: 32px;
  padding: 0 var(--space-4);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  font-weight: 500;
  cursor: pointer;
}

.notification-detail-dialog__btn--secondary {
  background: var(--bg-surface-hover);
  color: var(--text-secondary);
  border-color: var(--border-default);
}

.notification-detail-dialog__btn--secondary:hover {
  color: var(--text-primary);
  background: var(--bg-surface-active);
}

.notification-detail-dialog__btn--primary {
  background: var(--color-primary);
  color: var(--color-primary-fg);
}

.notification-detail-dialog__btn--primary:hover {
  background: var(--color-primary-hover);
}

.notification-detail-enter-active,
.notification-detail-leave-active {
  transition: opacity var(--duration-standard) var(--ease-out);
}

.notification-detail-enter-active .notification-detail-dialog,
.notification-detail-leave-active .notification-detail-dialog {
  transition:
    opacity var(--duration-standard) var(--ease-out),
    transform var(--duration-standard) var(--ease-out);
}

.notification-detail-enter-from,
.notification-detail-leave-to {
  opacity: 0;
}

.notification-detail-enter-from .notification-detail-dialog,
.notification-detail-leave-to .notification-detail-dialog {
  opacity: 0;
  transform: scale(0.96) translateY(8px);
}
</style>
