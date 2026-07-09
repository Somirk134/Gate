<!--
  TunnelProtocolSelect — 协议选择器
  ------------------------------------------------------------------
  V1 启用 HTTP / TCP，未来 HTTPS / UDP / P2P 以"即将/计划"标记预留。
  网格卡片形态，非传统下拉。
-->
<template>
  <div class="tunnel-protocol-grid">
    <button
      v-for="p in protocols"
      :key="p.key"
      type="button"
      class="tunnel-protocol-option"
      :class="{
        'tunnel-protocol-option--active': modelValue === p.key,
        'tunnel-protocol-option--disabled': p.availability !== 'enabled',
      }"
      :style="{ '--proto-color': p.color }"
      :disabled="p.availability !== 'enabled'"
      @click="onSelect(p.key, p.availability)">
      <span class="tunnel-protocol-option__head">
        <span
          class="tunnel-protocol-option__icon"
          :style="{ background: `${p.color}22`, color: p.color }">
          <GIcon :name="p.icon" :size="14" />
        </span>
        <span class="tunnel-protocol-option__name">{{ p.label }}</span>
      </span>
      <span class="tunnel-protocol-option__desc">{{ p.description }}</span>
      <GBadge
        v-if="p.availability !== 'enabled'"
        class="tunnel-protocol-option__badge"
        :variant="p.availability === 'soon' ? 'info' : 'neutral'"
        type="soft"
        size="sm">
        {{ p.availability === 'soon' ? '即将' : '计划' }}
      </GBadge>
    </button>
  </div>
</template>

<script setup lang="ts">
import GIcon from '@components/icons/GIcon.vue'
import GBadge from '@components/base/GBadge.vue'
import type { ProtocolAvailability, TunnelProtocol } from '../types'
import { PROTOCOL_PRESETS } from '../utils'

defineProps<{
  modelValue: TunnelProtocol
}>()

const emit = defineEmits<{
  'update:modelValue': [value: TunnelProtocol]
}>()

const protocols = PROTOCOL_PRESETS

function onSelect(key: TunnelProtocol, availability: ProtocolAvailability) {
  if (availability !== 'enabled') return
  emit('update:modelValue', key)
}
</script>
