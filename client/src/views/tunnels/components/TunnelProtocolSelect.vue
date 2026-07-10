<!--
  TunnelProtocolSelect - 协议选择器
  仅展示 Runtime 当前支持的 HTTP / TCP / HTTPS。
-->
<template>
  <div class="tunnel-protocol-grid">
    <button
      v-for="p in protocols"
      :key="p.key"
      type="button"
      class="tunnel-protocol-option"
      :class="{ 'tunnel-protocol-option--active': modelValue === p.key }"
      :style="{ '--proto-color': p.color }"
      @click="emit('update:modelValue', p.key)">
      <span class="tunnel-protocol-option__head">
        <span
          class="tunnel-protocol-option__icon"
          :style="{ background: `${p.color}22`, color: p.color }">
          <GIcon :name="p.icon" :size="14" />
        </span>
        <span class="tunnel-protocol-option__name">{{ p.label }}</span>
      </span>
      <span class="tunnel-protocol-option__desc">{{ p.description }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { TunnelProtocol } from '../types'
import { PROTOCOL_PRESETS } from '../utils'

defineProps<{
  modelValue: TunnelProtocol
}>()

const emit = defineEmits<{
  'update:modelValue': [value: TunnelProtocol]
}>()

const { t } = useI18n()
const protocols = computed(() =>
  PROTOCOL_PRESETS.map((protocol) => ({
    ...protocol,
    description: t(`tunnel.protocolDescriptions.${protocol.description}`),
  })),
)
</script>
