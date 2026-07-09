<!--
  TunnelConnection — 工作区 Connection 标签
  ------------------------------------------------------------------
  展示 Runtime 返回的当前连接列表。
  字段：Client IP / Region / Duration / Status / Protocol
-->
<template>
  <div class="tunnel-connection">
    <div class="tunnel-section__head">
      <div class="tunnel-section__title">
        <GIcon name="link" :size="16" class="tunnel-section__title-icon" />
        <span>{{ t('tunnel.connection.title') }}</span>
        <GBadge variant="primary" type="soft" size="sm">
          {{ tunnel.connections.length }}
        </GBadge>
      </div>
      <GButton size="sm" variant="ghost" icon="refresh" @click="$emit('refresh')">
        {{ t('common.refresh') }}
      </GButton>
    </div>

    <div v-if="tunnel.connections.length === 0" class="tunnel-connection__empty">
      <GIcon name="wifi-off" :size="28" />
      <span>{{ t('tunnel.connection.empty') }}</span>
    </div>

    <div v-else class="tunnel-conn-table">
      <div class="tunnel-conn-row tunnel-conn-row--head">
        <span class="tunnel-conn-row__cell">{{ t('tunnel.connection.clientIp') }}</span>
        <span class="tunnel-conn-row__cell">{{ t('tunnel.connection.region') }}</span>
        <span class="tunnel-conn-row__cell">{{ t('tunnel.connection.duration') }}</span>
        <span class="tunnel-conn-row__cell">{{ t('tunnel.connection.protocol') }}</span>
        <span class="tunnel-conn-row__cell">{{ t('tunnel.connection.status') }}</span>
      </div>
      <div v-for="conn in tunnel.connections" :key="conn.id" class="tunnel-conn-row">
        <span class="tunnel-conn-row__cell mono">{{ conn.clientIp }}</span>
        <span class="tunnel-conn-row__cell">
          <GIcon name="globe" :size="11" />
          {{ conn.region }}
        </span>
        <span class="tunnel-conn-row__cell mono">{{ formatDuration(conn.duration, t) }}</span>
        <span class="tunnel-conn-row__cell">
          <TunnelBadge :protocol="conn.protocol" size="sm" />
        </span>
        <span class="tunnel-conn-row__cell">
          <GStatusDot :status="connStatus(conn.status)" size="xs" />
          <span :style="{ color: connColor(conn.status) }">{{ connLabel(conn.status) }}</span>
        </span>
      </div>
    </div>

    <p class="tunnel-connection__hint">
      <GIcon name="info-circle" :size="12" />
      {{ t('tunnel.connection.hint') }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GBadge from '@components/base/GBadge.vue'
import GStatusDot from '@components/status/GStatusDot.vue'
import TunnelBadge from './TunnelBadge.vue'
import type { Tunnel, TunnelConnection } from '../types'
import { formatDuration } from '../utils'

defineProps<{ tunnel: Tunnel }>()

defineEmits<{ refresh: [] }>()
const { t } = useI18n()

function connStatus(s: TunnelConnection['status']): 'online' | 'warning' | 'offline' {
  if (s === 'active') return 'online'
  if (s === 'idle') return 'warning'
  return 'offline'
}

function connLabel(s: TunnelConnection['status']): string {
  if (s === 'active') return t('tunnel.connection.statusActive')
  if (s === 'idle') return t('tunnel.connection.statusIdle')
  return t('tunnel.connection.statusClosed')
}

function connColor(s: TunnelConnection['status']): string {
  if (s === 'active') return 'var(--color-success)'
  if (s === 'idle') return 'var(--color-warning)'
  return 'var(--text-tertiary)'
}
</script>

<style scoped>
.tunnel-conn-row__cell {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.tunnel-connection__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-10);
  color: var(--text-tertiary);
}

.tunnel-connection__hint {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  margin-top: var(--space-3);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
</style>
