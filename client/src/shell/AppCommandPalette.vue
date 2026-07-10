<template>
  <div
    ref="overlayRef"
    class="command-palette-overlay"
    @click.self="close"
    @keydown.esc.prevent="close"
    tabindex="-1">
    <div class="command-palette" @click.stop>
      <div class="command-palette-input">
        <GIcon name="search" :size="16" />
        <input
          ref="inputRef"
          v-model="query"
          type="text"
          :placeholder="t('commands.palette.placeholder')"
          @keydown.down.prevent="selectNext"
          @keydown.up.prevent="selectPrev"
          @keydown.enter.prevent="executeSelected"
          @keydown.esc.prevent="close" />
        <span class="shortcut-hint">ESC</span>
      </div>

      <div class="command-palette-results">
        <div v-if="filteredCommands.length === 0" class="no-results">
          <GIcon name="search" size="lg" />
          <p>{{ t('commands.palette.noResults') }}</p>
        </div>

        <div
          v-for="(cmd, i) in filteredCommands"
          :key="cmd.id"
          class="command-item"
          :class="{ selected: i === selectedIndex }"
          @click="executeCommand(cmd.id)"
          @mouseenter="selectedIndex = i">
          <div class="command-icon">
            <GIcon :name="cmd.icon || 'circle'" :size="16" />
          </div>
          <div class="command-info">
            <div class="command-title">
              {{ cmd.title }}
            </div>
            <div class="command-subtitle">
              {{ cmd.subtitle }}
            </div>
          </div>
          <div class="command-meta">
            <span v-if="cmd.shortcut" class="command-shortcut">{{ cmd.shortcut }}</span>
            <span class="command-category">{{ cmd.category }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCommandPalette } from '@composables/useCommandPalette'
import GIcon from '@components/icons/GIcon.vue'

const {
  query,
  selectedIndex,
  filteredCommands,
  selectNext,
  selectPrev,
  executeSelected,
  executeCommand,
  close,
} = useCommandPalette()

const inputRef = ref<HTMLInputElement | null>(null)
const overlayRef = ref<HTMLElement | null>(null)
const { t } = useI18n()

onMounted(() => {
  nextTick(() => {
    inputRef.value?.focus()
    overlayRef.value?.focus()
  })
})
</script>

<style scoped>
.command-palette-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  background: var(--color-overlay);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
  animation: fadeIn var(--duration-fast) var(--ease-out);
}

.command-palette {
  width: 640px;
  max-width: 90vw;
  background: var(--bg-surface-raised);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-floating);
  overflow: hidden;
  animation: scaleIn var(--duration-fast) var(--ease-out);
}

.command-palette-input {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
}

.command-palette-input input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--text-primary);
  font-size: var(--text-md);
  font-family: var(--font-ui);
}

.command-palette-input input::placeholder {
  color: var(--text-tertiary);
}

.shortcut-hint {
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  color: var(--text-tertiary);
  background: var(--bg-surface-hover);
  padding: 2px 6px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-subtle);
}

/* ── Results ── */
.command-palette-results {
  max-height: 400px;
  overflow-y: auto;
  padding: var(--space-2);
}

.command-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
  color: var(--text-secondary);
}

.command-item:hover,
.command-item.selected {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.command-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  flex-shrink: 0;
  color: var(--text-tertiary);
}

.command-item:hover .command-icon,
.command-item.selected .command-icon {
  color: var(--color-primary);
  background: var(--color-primary-muted);
}

.command-info {
  flex: 1;
  min-width: 0;
}

.command-title {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-primary);
}

.command-subtitle {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-top: 2px;
}

.command-meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

.command-shortcut {
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  color: var(--text-tertiary);
  background: var(--bg-surface-hover);
  padding: 1px 4px;
  border-radius: var(--radius-xs);
  border: 1px solid var(--border-subtle);
}

.command-category {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: capitalize;
}

/* ── No Results ── */
.no-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-8);
  gap: var(--space-3);
  color: var(--text-tertiary);
}

.no-results p {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes scaleIn {
  from {
    opacity: 0;
    transform: scale(0.96) translateY(-8px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}
</style>
