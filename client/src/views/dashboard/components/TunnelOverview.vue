<!--
  TunnelOverview — 运行中的隧道
  ------------------------------------------------------------------
  展示当前运行中的 Tunnel，含协议/状态/端口/速度/连接数。
-->
<template>
  <section class="dashboard-section">
    <div class="dashboard-section__head">
      <div class="dashboard-section__title">
        <GIcon name="link" :size="16" class="dashboard-section__title-icon" />
        <span>{{ title }}</span>
        <GBadge variant="success" type="soft" size="sm">{{ tunnels.length }}</GBadge>
      </div>
      <button class="dashboard-section__more" @click="$emit('viewAll')">
        查看全部
        <GIcon name="chevron-right" :size="12" />
      </button>
    </div>

    <div v-if="tunnels.length" class="dashboard-grid--tunnels">
      <GCard
        v-for="(tunnel, i) in tunnels"
        :key="tunnel.id"
        variant="plain"
        padding="md"
        class="tunnel-overview__card dashboard-card-lift"
        :class="`stagger-${(i % 6) + 1}`"
      >
        <div class="tunnel-overview__head">
          <GIcon name="link" :size="15" class="tunnel-overview__lead" />
          <span class="tunnel-overview__name">{{ tunnel.name }}</span>
          <GBadge :variant="protocolVariant(tunnel.protocol)" type="solid" size="sm">
            {{ tunnel.protocol.toUpperCase() }}
          </GBadge>
          <GStatusBadge :status="tunnel.status" size="sm" class="tunnel-overview__status" />
        </div>

        <div class="tunnel-overview__route">
          <span class="tunnel-overview__endpoint">
            <span class="tunnel-overview__label">本地</span>
            <span class="tunnel-overview__addr">127.0.0.1:{{ tunnel.localPort }}</span>
          </span>
          <GIcon name="arrow-right" :size="13" class="tunnel-overview__arrow" />
          <span class="tunnel-overview__endpoint">
            <span class="tunnel-overview__label">公网</span>
            <span class="tunnel-overview__addr">{{ tunnel.publicHost }}:{{ tunnel.publicPort }}</span>
          </span>
        </div>

        <div class="tunnel-overview__metrics">
          <div class="tunnel-overview__metric">
            <GIcon name="arrow-up" :size="11" class="tunnel-overview__metric-icon tunnel-overview__metric-icon--up" />
            <span class="tunnel-overview__metric-value">{{ formatSpeed(tunnel.uploadSpeed) }}</span>
          </div>
          <div class="tunnel-overview__metric">
            <GIcon name="arrow-down" :size="11" class="tunnel-overview__metric-icon tunnel-overview__metric-icon--down" />
            <span class="tunnel-overview__metric-value">{{ formatSpeed(tunnel.downloadSpeed) }}</span>
          </div>
          <div class="tunnel-overview__metric">
            <GIcon name="users" :size="11" class="tunnel-overview__metric-icon" />
            <span class="tunnel-overview__metric-value">{{ tunnel.connections }}</span>
          </div>
        </div>

        <div class="tunnel-overview__foot">
          <GButton
            v-if="tunnel.status === 'online' || tunnel.status === 'connecting'"
            size="sm"
            variant="ghost"
            icon="stop"
            @click="$emit('stop', tunnel.id)"
          >
            停止
          </GButton>
          <GButton
            v-else
            size="sm"
            variant="primary"
            icon="play"
            @click="$emit('start', tunnel.id)"
          >
            启动
          </GButton>
          <GButton
            size="sm"
            variant="ghost"
            trailing-icon="chevron-right"
            @click="$emit('detail', tunnel)"
          >
            详情
          </GButton>
        </div>
      </GCard>
    </div>

    <GCard v-else variant="plain" padding="lg">
      <GEmptyState
        title="暂无运行中的隧道"
        description="启动项目中的隧道后，将在此处实时展示。"
      />
    </GCard>
  </section>
</template>

<script setup lang="ts">
import GCard from "@components/base/GCard.vue"
import GButton from "@components/base/GButton.vue"
import GBadge from "@components/base/GBadge.vue"
import GIcon from "@components/icons/GIcon.vue"
import GStatusBadge from "@components/status/GStatusBadge.vue"
import GEmptyState from "@components/feedback/GEmptyState.vue"
import type { DashboardTunnel, Protocol } from "../types"

withDefaults(
  defineProps<{
    tunnels: DashboardTunnel[]
    title?: string
  }>(),
  {
    title: "运行中的隧道",
  },
)

defineEmits<{
  start: [id: string]
  stop: [id: string]
  detail: [tunnel: DashboardTunnel]
  viewAll: []
}>()

function protocolVariant(p: Protocol) {
  switch (p) {
    case "https": return "success"
    case "http": return "info"
    case "tcp": return "primary"
    case "udp": return "warning"
    default: return "neutral"
  }
}

function formatSpeed(kbps: number): string {
  if (kbps < 1) return "0 KB/s"
  if (kbps < 1024) return `${kbps.toFixed(1)} KB/s`
  return `${(kbps / 1024).toFixed(2)} MB/s`
}
</script>

<style scoped>
.tunnel-overview__card {
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.tunnel-overview__head {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.tunnel-overview__lead {
  color: var(--text-tertiary);
  flex-shrink: 0;
}
.tunnel-overview__name {
  flex: 1;
  min-width: 0;
  font-size: var(--text-base);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tunnel-overview__route {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  background: var(--bg-input);
  border-radius: var(--radius-md);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}
.tunnel-overview__endpoint {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}
.tunnel-overview__label {
  font-family: var(--font-ui);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
.tunnel-overview__addr {
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tunnel-overview__arrow {
  color: var(--text-tertiary);
  flex-shrink: 0;
}
.tunnel-overview__metrics {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}
.tunnel-overview__metric {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.tunnel-overview__metric-icon {
  color: var(--text-tertiary);
}
.tunnel-overview__metric-icon--up {
  color: var(--color-success);
}
.tunnel-overview__metric-icon--down {
  color: var(--color-primary);
}
.tunnel-overview__metric-value {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
  font-family: var(--font-mono);
}
.tunnel-overview__foot {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: auto;
}
</style>
