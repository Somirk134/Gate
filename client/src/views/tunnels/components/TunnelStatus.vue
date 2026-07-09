<!--
  TunnelStatus — 统一状态徽章
  ------------------------------------------------------------------
  基于 GStatusBadge，统一映射 TunnelStatus → 圆点 + 文字。
  全模块状态展示唯一入口，颜色与 Badge 统一。
-->
<template>
  <GStatusBadge
    :status="config.dotStatus"
    :label="label ?? t(`tunnel.statusLabels.${status}`)"
    :size="size" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GStatusBadge from '@components/status/GStatusBadge.vue'
import type { TunnelStatus } from '../types'
import { TUNNEL_STATUS_CONFIG } from '../utils'

const props = withDefaults(
  defineProps<{
    status: TunnelStatus
    label?: string
    size?: 'sm' | 'md'
  }>(),
  { size: 'md' },
)

const { t } = useI18n()
const config = computed(() => TUNNEL_STATUS_CONFIG[props.status])
</script>
