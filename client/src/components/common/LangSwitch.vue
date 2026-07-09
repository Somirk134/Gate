<template>
  <div class="lang-switch">
    <button
      v-for="l in locales"
      :key="l.value"
      class="lang-btn"
      :class="{ active: locale === l.value }"
      @click="setLocale(l.value)">
      {{ t(l.labelKey) }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useLocaleSwitcher } from '@composables/useLocaleSwitcher'

const { locale, locales, setLocale } = useLocaleSwitcher()
const { t } = useI18n()
</script>

<style scoped>
.lang-switch {
  display: flex;
  gap: 2px;
  padding: 2px;
  background: var(--bg-input);
  border-radius: var(--radius-md);
}
.lang-btn {
  padding: 2px 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-ui);
  font-size: var(--text-xs);
  font-weight: 500;
  border-radius: 4px;
  cursor: pointer;
  transition: all var(--duration-micro) var(--ease-out);
}
.lang-btn:hover {
  color: var(--text-secondary);
}
.lang-btn.active {
  background: var(--bg-surface);
  color: var(--text-primary);
  box-shadow: var(--shadow-xs);
}
</style>
