<!--
  TunnelBadge — 协议徽章
  ------------------------------------------------------------------
  统一展示隧道协议，颜色随协议变化。未来协议以"即将/计划"标记。
-->
<template>
  <span
    class="tunnel-badge"
    :class="[`tunnel-badge--${protocol}`, `tunnel-badge--${size}`]"
    :style="{ '--badge-color': color }">
    <GIcon :name="icon" :size="size === 'sm' ? 11 : 13" />
    <span class="tunnel-badge__label">{{ label }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'
import type { TunnelProtocol } from '../types'
import { PROTOCOL_MAP } from '../utils'

const props = withDefaults(
  defineProps<{
    protocol: TunnelProtocol
    size?: 'sm' | 'md'
  }>(),
  { size: 'sm' },
)

const preset = computed(() => PROTOCOL_MAP[props.protocol])
const color = computed(() => preset.value.color)
const label = computed(() => preset.value.label)
const icon = computed(() => preset.value.icon)
</script>

<style scoped>
.tunnel-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: var(--font-ui);
  font-weight: var(--weight-semibold);
  line-height: 1;
  white-space: nowrap;
  border-radius: var(--radius-badge);
  background: color-mix(in srgb, var(--badge-color) 16%, transparent);
  color: var(--badge-color);
  border: 1px solid color-mix(in srgb, var(--badge-color) 30%, transparent);
}

.tunnel-badge--sm {
  font-size: var(--text-xs);
  padding: 2px var(--space-2);
  height: 18px;
}

.tunnel-badge--md {
  font-size: var(--text-sm);
  padding: 3px var(--space-2);
  height: 22px;
}
</style>
