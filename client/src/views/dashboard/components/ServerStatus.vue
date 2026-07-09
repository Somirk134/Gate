<!--
  ServerStatus — 服务器状态
  ------------------------------------------------------------------
  显示已连接服务器：名称/地区/IP/版本/Ping 与 CPU/Mem/Disk/Net。
-->
<template>
  <section class="dashboard-section">
    <div class="dashboard-section__head">
      <div class="dashboard-section__title">
        <GIcon name="servers" :size="16" class="dashboard-section__title-icon" />
        <span>{{ title }}</span>
        <GBadge variant="info" type="soft" size="sm">
          {{ connectedCount }} / {{ servers.length }}
        </GBadge>
      </div>
      <button class="dashboard-section__more" @click="$emit('viewAll')">
        管理
        <GIcon name="chevron-right" :size="12" />
      </button>
    </div>

    <div v-if="servers.length" class="dashboard-grid--servers">
      <GCard
        v-for="(server, i) in servers"
        :key="server.id"
        variant="plain"
        padding="md"
        class="server-status__card dashboard-card-lift"
        :class="`stagger-${(i % 6) + 1}`">
        <div class="server-status__head">
          <span class="server-status__icon" :class="`server-status__icon--${server.status}`">
            <GIcon name="server" :size="18" />
          </span>
          <div class="server-status__title-wrap">
            <span class="server-status__name">{{ server.name }}</span>
            <span class="server-status__region">
              <GIcon name="globe" :size="11" />
              {{ server.region }}
            </span>
          </div>
          <GStatusBadge :status="server.status" size="sm" />
        </div>

        <div class="server-status__info">
          <div class="server-status__info-item">
            <span class="server-status__info-label">IP</span>
            <span class="server-status__info-value">{{ server.ip }}</span>
          </div>
          <div class="server-status__info-item">
            <span class="server-status__info-label">版本</span>
            <span class="server-status__info-value">{{ server.version }}</span>
          </div>
          <div class="server-status__info-item">
            <span class="server-status__info-label">Ping</span>
            <span class="server-status__info-value" :class="pingClass(server.ping)">
              {{ server.status === 'online' ? `${server.ping}ms` : '—' }}
            </span>
          </div>
        </div>

        <div v-if="server.status === 'online'" class="server-status__resources">
          <div class="server-status__resource">
            <div class="server-status__resource-head">
              <span class="server-status__resource-label">
                <GIcon name="cpu" :size="11" /> CPU
              </span>
              <span class="server-status__resource-value">{{ server.cpu }}%</span>
            </div>
            <GProgress :value="server.cpu" :variant="resourceVariant(server.cpu)" size="sm" />
          </div>
          <div class="server-status__resource">
            <div class="server-status__resource-head">
              <span class="server-status__resource-label">
                <GIcon name="memory-stick" :size="11" /> 内存
              </span>
              <span class="server-status__resource-value">{{ server.memory }}%</span>
            </div>
            <GProgress :value="server.memory" :variant="resourceVariant(server.memory)" size="sm" />
          </div>
          <div class="server-status__resource">
            <div class="server-status__resource-head">
              <span class="server-status__resource-label">
                <GIcon name="hard-drive" :size="11" /> 磁盘
              </span>
              <span class="server-status__resource-value">{{ server.disk }}%</span>
            </div>
            <GProgress :value="server.disk" :variant="resourceVariant(server.disk)" size="sm" />
          </div>
          <div class="server-status__resource">
            <div class="server-status__resource-head">
              <span class="server-status__resource-label">
                <GIcon name="network" :size="11" /> 网络
              </span>
              <span class="server-status__resource-value">{{ server.network }}%</span>
            </div>
            <GProgress
              :value="server.network"
              :variant="resourceVariant(server.network)"
              size="sm" />
          </div>
        </div>

        <div v-else class="server-status__offline-resources">
          <GIcon name="wifi-off" :size="14" />
          <span>服务器未连接，资源数据不可用</span>
        </div>
      </GCard>
    </div>

    <GCard v-else variant="plain" padding="lg">
      <GEmptyState title="尚未连接服务器" description="添加 Gate 服务器以开始创建隧道。">
        <template #action>
          <GButton variant="primary" icon="plus" @click="$emit('addServer')"> 添加服务器 </GButton>
        </template>
      </GEmptyState>
    </GCard>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GCard from '@components/base/GCard.vue'
import GButton from '@components/base/GButton.vue'
import GBadge from '@components/base/GBadge.vue'
import GIcon from '@components/icons/GIcon.vue'
import GStatusBadge from '@components/status/GStatusBadge.vue'
import GProgress from '@components/feedback/GProgress.vue'
import GEmptyState from '@components/feedback/GEmptyState.vue'
import type { DashboardServer } from '../types'

const props = withDefaults(
  defineProps<{
    servers: DashboardServer[]
    title?: string
  }>(),
  {
    title: '服务器状态',
  },
)

defineEmits<{ viewAll: []; addServer: [] }>()

const connectedCount = computed(() => props.servers.filter((s) => s.status === 'online').length)

function pingClass(ping: number) {
  if (ping === 0) return ''
  if (ping < 100) return 'server-status__info-value--good'
  if (ping < 300) return 'server-status__info-value--warn'
  return 'server-status__info-value--bad'
}

function resourceVariant(v: number): 'success' | 'warning' | 'error' {
  if (v < 60) return 'success'
  if (v < 85) return 'warning'
  return 'error'
}
</script>

<style scoped>
.server-status__card {
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.server-status__head {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.server-status__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  background: var(--bg-surface-hover);
  color: var(--text-secondary);
  flex-shrink: 0;
}
.server-status__icon--online {
  background: var(--color-success-muted);
  color: var(--color-success);
}
.server-status__icon--offline {
  background: var(--status-offline-bg);
  color: var(--status-offline);
}
.server-status__icon--connecting {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}
.server-status__title-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.server-status__name {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
}
.server-status__region {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
.server-status__info {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: var(--space-2);
  padding: var(--space-3);
  background: var(--bg-input);
  border-radius: var(--radius-md);
}
.server-status__info-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.server-status__info-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wide);
}
.server-status__info-value {
  font-size: var(--text-sm);
  font-family: var(--font-mono);
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.server-status__info-value--good {
  color: var(--color-success);
}
.server-status__info-value--warn {
  color: var(--color-warning);
}
.server-status__info-value--bad {
  color: var(--color-error);
}
.server-status__resources {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.server-status__resource-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}
.server-status__resource-label {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
.server-status__resource-value {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
  font-family: var(--font-mono);
}
.server-status__offline-resources {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-4);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  background: var(--bg-input);
  border-radius: var(--radius-md);
}
</style>
