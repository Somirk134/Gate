<template>
  <div class="lang-switch" v-click-outside="() => (open = false)">
    <button
      class="lang-trigger"
      type="button"
      :aria-expanded="open"
      @click.prevent="open = !open">
      <GIcon name="globe" :size="14" />
      <span>{{ currentLabel }}</span>
      <GIcon :name="open ? 'chevron-up' : 'chevron-down'" :size="12" />
    </button>

    <Transition name="lang-dropdown">
      <ul v-if="open" class="lang-dropdown">
        <li
          v-for="l in locales"
          :key="l.value"
          class="lang-option"
          :class="{ active: locale === l.value }"
          role="option"
          :aria-selected="locale === l.value"
          @click="setLocale(l.value); open = false">
          {{ t(l.labelKey) }}
          <GIcon v-if="locale === l.value" name="check" :size="12" />
        </li>
      </ul>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useLocaleSwitcher } from '@composables/useLocaleSwitcher'
import GIcon from '@components/icons/GIcon.vue'

const { locale, locales, setLocale } = useLocaleSwitcher()
const { t } = useI18n()
const open = ref(false)

const currentLabel = computed(() => {
  const current = locales.find((l) => l.value === locale.value)
  return current ? t(current.labelKey) : locale.value
})
</script>

<script lang="ts">
// vClickOutside directive — lightweight inline version
export default {
  directives: {
    clickOutside: {
      mounted(el: HTMLElement, binding: { value: (...args: unknown[]) => void }) {
        const handler = (event: Event) => {
          if (!(el as HTMLElement).contains(event.target as Node)) {
            binding.value(event)
          }
        }
        ;(el as any & { _clickOutsideHandler?: EventListener })._clickOutsideHandler = handler
        document.addEventListener('mousedown', handler)
      },
      unmounted(el: HTMLElement) {
        const h = (el as any & { _clickOutsideHandler?: EventListener })._clickOutsideHandler
        if (h) document.removeEventListener('mousedown', h)
      },
    },
  },
}
</script>

<style scoped>
.lang-switch {
  position: relative;
}

.lang-trigger {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 28px;
  padding: 0 8px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--duration-micro) var(--ease-out);
  font-family: var(--font-ui);
}

.lang-trigger:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
  border-color: var(--border-strong);
}

.lang-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  z-index: var(--z-popover);
  min-width: 120px;
  margin: 0;
  padding: 4px;
  list-style: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.lang-option {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: 6px 10px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
  transition: all var(--duration-micro) var(--ease-out);
}

.lang-option:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.lang-option.active {
  color: var(--color-primary);
  background: var(--color-primary-muted);
}

/* ── Transition ── */
.lang-dropdown-enter-active,
.lang-dropdown-leave-active {
  transition: all var(--duration-fast) var(--ease-out);
}
.lang-dropdown-enter-from,
.lang-dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.96);
}
</style>
