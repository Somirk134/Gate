<!--
  TunnelCard - 隧道摘要卡片。
  本地/远程标签统一走 i18n，路由地址仍按真实数据渲染。
-->
<template>
  <GCard variant="plain" padding="md">
    <div class="tunnel-card">
      <div class="tunnel-card__head">
        <GIcon name="link" :size="16" class="tunnel-card__lead" />
        <span class="tunnel-card__name">{{ name }}</span>
        <GBadge :variant="protocolVariant" type="solid" size="sm" class="tunnel-card__proto">
          {{ protocol.toUpperCase() }}
        </GBadge>
        <GStatusBadge :status="status" size="sm" class="tunnel-card__status" />
        <GIconButton name="more-horizontal" size="sm" @click="emit('action', 'menu')" />
      </div>

      <div class="tunnel-card__route">
        <span class="tunnel-card__endpoint">
          <span class="tunnel-card__label">{{ t('business.tunnel.local') }}</span>
          <span class="tunnel-card__addr">127.0.0.1:{{ localPort }}</span>
        </span>
        <GIcon name="arrow-right" :size="14" class="tunnel-card__arrow" />
        <span class="tunnel-card__endpoint">
          <span class="tunnel-card__label">{{ t('business.tunnel.remote') }}</span>
          <span class="tunnel-card__addr">{{ remoteHost }}:{{ remotePort }}</span>
        </span>
      </div>

      <div v-if="traffic" class="tunnel-card__foot">
        <span class="tunnel-card__traffic">
          <GIcon name="arrow-up" :size="11" /> {{ traffic.up }}
        </span>
        <span class="tunnel-card__traffic">
          <GIcon name="arrow-down" :size="11" /> {{ traffic.down }}
        </span>
      </div>
    </div>
  </GCard>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GCard from '@components/base/GCard.vue'
import GBadge from '@components/base/GBadge.vue'
import GIcon from '@components/icons/GIcon.vue'
import GIconButton from '@components/base/GIconButton.vue'
import GStatusBadge from '@components/status/GStatusBadge.vue'

const props = defineProps<{
  name: string
  protocol: 'http' | 'https' | 'tcp' | 'udp'
  localPort: number
  remoteHost: string
  remotePort: number
  status: 'online' | 'offline' | 'connecting' | 'error' | 'warning' | 'starting'
  traffic?: { up: string; down: string }
}>()

const emit = defineEmits<{ click: []; action: [key: string] }>()
const { t } = useI18n()

const protocolVariant = computed(() => {
  switch (props.protocol) {
    case 'https':
      return 'success'
    case 'http':
      return 'info'
    case 'tcp':
      return 'primary'
    case 'udp':
      return 'warning'
    default:
      return 'neutral'
  }
})
</script>

<style scoped>
.tunnel-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.tunnel-card__head {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.tunnel-card__lead {
  color: var(--text-tertiary);
  flex-shrink: 0;
}
.tunnel-card__name {
  flex: 1;
  min-width: 0;
  font-size: var(--text-base);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tunnel-card__route {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  background: var(--bg-input);
  border-radius: var(--radius-md);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}
.tunnel-card__endpoint {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.tunnel-card__label {
  font-family: var(--font-ui);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
.tunnel-card__addr {
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tunnel-card__arrow {
  color: var(--text-tertiary);
  flex-shrink: 0;
}
.tunnel-card__foot {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}
.tunnel-card__traffic {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
}
</style>
