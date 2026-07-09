<!--
  TunnelConnection — 工作区 Connection 标签
  ------------------------------------------------------------------
  展示当前连接（Mock）。未来支持实时连接。
  字段：Client IP / Region / Duration / Status / Protocol
-->
<template>
  <div class="tunnel-connection">
    <div class="tunnel-section__head">
      <div class="tunnel-section__title">
        <GIcon
          name="link"
          :size="16"
          class="tunnel-section__title-icon"
        />
        <span>当前连接</span>
        <GBadge
          variant="primary"
          type="soft"
          size="sm"
        >
          {{ tunnel.connections.length }}
        </GBadge>
      </div>
      <GButton
        size="sm"
        variant="ghost"
        icon="refresh"
        @click="$emit('refresh')"
      >
        刷新
      </GButton>
    </div>

    <div
      v-if="tunnel.connections.length === 0"
      class="tunnel-connection__empty"
    >
      <GIcon
        name="wifi-off"
        :size="28"
      />
      <span>暂无活动连接</span>
    </div>

    <div
      v-else
      class="tunnel-conn-table"
    >
      <div class="tunnel-conn-row tunnel-conn-row--head">
        <span class="tunnel-conn-row__cell">Client IP</span>
        <span class="tunnel-conn-row__cell">Region</span>
        <span class="tunnel-conn-row__cell">Duration</span>
        <span class="tunnel-conn-row__cell">Protocol</span>
        <span class="tunnel-conn-row__cell">Status</span>
      </div>
      <div
        v-for="conn in tunnel.connections"
        :key="conn.id"
        class="tunnel-conn-row"
      >
        <span class="tunnel-conn-row__cell mono">{{ conn.clientIp }}</span>
        <span class="tunnel-conn-row__cell">
          <GIcon
            name="globe"
            :size="11"
          />
          {{ conn.region }}
        </span>
        <span class="tunnel-conn-row__cell mono">{{ formatDuration(conn.duration) }}</span>
        <span class="tunnel-conn-row__cell">
          <TunnelBadge
            :protocol="conn.protocol"
            size="sm"
          />
        </span>
        <span class="tunnel-conn-row__cell">
          <GStatusDot
            :status="connStatus(conn.status)"
            size="xs"
          />
          <span :style="{ color: connColor(conn.status) }">{{ connLabel(conn.status) }}</span>
        </span>
      </div>
    </div>

    <p class="tunnel-connection__hint">
      <GIcon
        name="info-circle"
        :size="12"
      />
      当前为 Mock 数据。未来将支持实时连接监控。
    </p>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"
import GBadge from "@components/base/GBadge.vue"
import GStatusDot from "@components/status/GStatusDot.vue"
import TunnelBadge from "./TunnelBadge.vue"
import type { Tunnel, TunnelConnection } from "../types"
import { formatDuration } from "../utils"

defineProps<{ tunnel: Tunnel }>()

defineEmits<{ refresh: [] }>()

function connStatus(s: TunnelConnection["status"]): "online" | "warning" | "offline" {
  if (s === "active") return "online"
  if (s === "idle") return "warning"
  return "offline"
}

function connLabel(s: TunnelConnection["status"]): string {
  if (s === "active") return "活跃"
  if (s === "idle") return "空闲"
  return "已关闭"
}

function connColor(s: TunnelConnection["status"]): string {
  if (s === "active") return "var(--color-success)"
  if (s === "idle") return "var(--color-warning)"
  return "var(--text-tertiary)"
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
