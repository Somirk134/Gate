<template>
  <div class="dialog-layer" @keydown.esc="dialogStore.closeAll">
    <transition-group name="dialog">
      <div
        v-for="dialog in dialogStore.activeDialogs"
        :key="dialog.id"
        class="dialog-overlay"
        @click.self="dialogStore.dismissDialog(dialog.id)">
        <div
          class="dialog-container"
          :class="`type-${dialog.type}`"
          role="dialog"
          aria-modal="true"
          tabindex="-1">
          <div class="dialog-header">
            <div v-if="dialog.type === 'delete'" class="dialog-icon">
              <GIcon name="alert-triangle" :size="20" />
            </div>
            <div v-else-if="dialog.type === 'alert'" class="dialog-icon">
              <GIcon name="alert-circle" :size="20" />
            </div>
            <div v-else-if="dialog.type === 'confirm'" class="dialog-icon">
              <GIcon name="help-circle" :size="20" />
            </div>
            <div class="dialog-title">
              {{ dialog.title }}
            </div>
            <button class="dialog-close" @click="dialogStore.dismissDialog(dialog.id)">
              <GIcon name="close" :size="16" />
            </button>
          </div>
          <div v-if="dialog.content" class="dialog-body">
            <p>{{ dialog.content }}</p>
          </div>
          <div class="dialog-footer">
            <button
              class="dialog-btn dialog-btn-secondary"
              @click="dialogStore.dismissDialog(dialog.id)">
              {{ cancelLabel(dialog) }}
            </button>
            <button
              class="dialog-btn dialog-btn-primary"
              :class="{ danger: dialog.type === 'delete' }"
              @click="dialogStore.closeDialog(dialog.id, true)">
              {{ confirmLabel(dialog) }}
            </button>
          </div>
        </div>
      </div>
    </transition-group>
  </div>
</template>

<script setup lang="ts">
import { useDialogStore } from '@stores'
import type { DialogItem } from '@stores/modules/dialog'
import GIcon from '@components/icons/GIcon.vue'

const dialogStore = useDialogStore()

function confirmLabel(dialog: DialogItem): string {
  const label = dialog.props?.confirmText
  if (typeof label === 'string') return label
  switch (dialog.type) {
    case 'delete':
      return '删除'
    case 'confirm':
      return '确认'
    case 'alert':
      return '知道了'
    case 'form':
      return '提交'
    default:
      return '确认'
  }
}

function cancelLabel(dialog: DialogItem): string {
  const label = dialog.props?.cancelText
  return typeof label === 'string' ? label : '取消'
}
</script>

<style scoped>
.dialog-layer {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  pointer-events: none;
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: auto;
  animation: g-fade-in var(--duration-fast) var(--ease-out);
  backdrop-filter: blur(8px);
}

.dialog-container {
  background: var(--bg-surface-raised);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-dialog);
  box-shadow: var(--shadow-floating);
  width: 100%;
  max-width: 440px;
  margin: var(--space-4);
  animation: g-dialog-in var(--duration-standard) var(--ease-out);
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-5) var(--space-3);
}

.dialog-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-lg);
  flex-shrink: 0;
}

.type-delete .dialog-icon {
  color: var(--color-error);
  background: var(--color-error-muted);
}
.type-alert .dialog-icon {
  color: var(--color-warning);
  background: var(--color-warning-muted);
}
.type-confirm .dialog-icon {
  color: var(--color-info);
  background: var(--color-info-muted);
}

.dialog-title {
  flex: 1;
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--text-primary);
}

.dialog-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-micro);
  flex-shrink: 0;
}

.dialog-close:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

/* ── Body ── */
.dialog-body {
  padding: 0 var(--space-5) var(--space-4);
}

.dialog-body p {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

/* ── Footer ── */
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-5) var(--space-4);
  border-top: 1px solid var(--border-subtle);
}

.dialog-btn {
  height: 32px;
  padding: 0 var(--space-4);
  border-radius: var(--radius-md);
  font-size: var(--text-sm);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-micro);
  border: 1px solid transparent;
}

.dialog-btn-secondary {
  background: var(--bg-surface-hover);
  color: var(--text-secondary);
  border-color: var(--border-default);
}

.dialog-btn-secondary:hover {
  background: var(--bg-surface-active);
  color: var(--text-primary);
}

.dialog-btn-primary {
  background: var(--color-primary);
  color: var(--color-primary-fg);
}

.dialog-btn-primary:hover {
  background: var(--color-primary-hover);
}

.dialog-btn-primary.danger {
  background: var(--color-error);
  color: var(--color-error-fg);
}

.dialog-btn-primary.danger:hover {
  background: var(--color-error-hover);
}

/* ── Transitions ── */
.dialog-enter-active,
.dialog-leave-active {
  transition: all var(--duration-standard) var(--ease-out);
}

.dialog-enter-from .dialog-container,
.dialog-leave-to .dialog-container {
  opacity: 0;
  transform: scale(0.96) translateY(8px);
}

.dialog-enter-from .dialog-overlay,
.dialog-leave-to .dialog-overlay {
  opacity: 0;
}
</style>
