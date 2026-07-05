<!--
  TunnelCard — Mock Tunnel 卡片
  ------------------------------------------------------------------
  详情页 Tunnel 区域占位卡片，不开发真实 Tunnel 逻辑。
-->
<template>
  <div
    class="tunnel-card"
    :class="[`tunnel-card--${tunnel.status}`, { 'tunnel-card--selected': selected }]"
    @click="$emit('select', tunnel)"
  >
    <div class="tunnel-card__bar" />
    <div class="tunnel-card__body">
      <div class="tunnel-card__top">
        <span class="tunnel-card__status" :class="`tunnel-card__status--${tunnel.status}`">
          <span class="tunnel-card__dot" />
          {{ statusLabel }}
        </span>
        <GIconButton name="more-horizontal" size="sm" variant="ghost" @click.stop />
      </div>
      <h3 class="tunnel-card__name">{{ tunnel.name }}</h3>
      <div class="tunnel-card__protocol">
        <GBadge variant="primary" type="soft" size="sm">{{ tunnel.protocol.toUpperCase() }}</GBadge>
        <span class="tunnel-card__addr">{{ tunnel.localAddr }} → {{ tunnel.remoteAddr }}</span>
      </div>
      <div class="tunnel-card__public">{{ tunnel.publicAddr }}</div>
      <div class="tunnel-card__metrics">
        <span class="tunnel-card__metric">
          <GIcon name="arrow-down" :size="11" /> {{ tunnel.downSpeed }}
        </span>
        <span class="tunnel-card__metric">
          <GIcon name="arrow-up" :size="11" /> {{ tunnel.upSpeed }}
        </span>
        <span class="tunnel-card__metric">
          <GIcon name="link" :size="11" /> {{ tunnel.connections }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import GIconButton from "@components/base/GIconButton.vue"
import GBadge from "@components/base/GBadge.vue"
import type { MockTunnel } from "../types"

const props = defineProps<{
  tunnel: MockTunnel
  selected?: boolean
}>()

defineEmits<{ select: [tunnel: MockTunnel] }>()

const statusLabel = computed(() => {
  const map: Record<string, string> = {
    online: "在线",
    offline: "离线",
    starting: "启动中",
    error: "错误",
  }
  return map[props.tunnel.status] ?? "未知"
})
</script>

<style scoped>
.tunnel-card {
  display: flex;
  background: var(--bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  overflow: hidden;
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-out),
    box-shadow var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out);
}

.tunnel-card:hover {
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-xs);
  transform: translateY(-1px);
}

.tunnel-card--selected {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 1px var(--color-primary);
}

.tunnel-card__bar {
  width: 3px;
  flex-shrink: 0;
}

.tunnel-card--online .tunnel-card__bar { background: var(--color-success); }
.tunnel-card--offline .tunnel-card__bar { background: var(--status-offline); }
.tunnel-card--starting .tunnel-card__bar { background: var(--status-starting); }
.tunnel-card--error .tunnel-card__bar { background: var(--color-error); }

.tunnel-card__body {
  flex: 1;
  padding: var(--space-3) var(--space-4);
  min-width: 0;
}

.tunnel-card__top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-2);
}

.tunnel-card__status {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 1px var(--space-2);
  border-radius: var(--radius-sm);
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
}

.tunnel-card__status--online { background: var(--color-success-muted); color: var(--color-success); }
.tunnel-card__status--offline { background: var(--status-offline-bg); color: var(--status-offline); }
.tunnel-card__status--starting { background: var(--status-starting-bg); color: var(--status-starting); }
.tunnel-card__status--error { background: var(--color-error-muted); color: var(--color-error); }

.tunnel-card__dot {
  width: 5px;
  height: 5px;
  border-radius: var(--radius-full);
  background: currentColor;
}

.tunnel-card__name {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  margin-bottom: var(--space-1);
}

.tunnel-card__protocol {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-1);
}

.tunnel-card__addr {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  font-family: var(--font-mono);
}

.tunnel-card__public {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  margin-bottom: var(--space-2);
}

.tunnel-card__metrics {
  display: flex;
  gap: var(--space-3);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
}

.tunnel-card__metric {
  display: inline-flex;
  align-items: center;
  gap: 3px;
}
</style>
