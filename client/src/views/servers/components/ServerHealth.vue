<!--
  ServerHealth — 工作区 Health 标签
  ------------------------------------------------------------------
  Health Report。检查：Server Online / API Available / Token Valid /
  Tunnel Service / Disk / Memory / Clock / Version。
-->
<template>
  <div class="server-health">
    <!-- 健康概要 -->
    <div class="server-health-summary">
      <div class="server-health-summary__score">
        <svg width="84" height="84" viewBox="0 0 84 84">
          <circle
            cx="42"
            cy="42"
            :r="ringRadius"
            stroke="var(--bg-surface-hover)"
            :stroke-width="ringStroke"
            fill="none" />
          <circle
            cx="42"
            cy="42"
            :r="ringRadius"
            :stroke="scoreColor"
            :stroke-width="ringStroke"
            :stroke-dasharray="ringCircumference"
            :stroke-dashoffset="ringCircumference * (1 - server.health.score / 100)"
            stroke-linecap="round"
            fill="none"
            transform="rotate(-90 42 42)"
            class="server-statistics__ring-bar" />
        </svg>
        <span class="server-health-summary__score-value">{{ server.health.score }}</span>
      </div>
      <div class="server-health-summary__text">
        <span class="server-health-summary__label" :style="{ color: scoreColor }">
          {{ overallLabel }}
        </span>
        <span class="server-health-summary__desc">
          {{
            t('server.health.summary', {
              pass: passCount,
              total: server.health.items.length,
              warn: warnCount,
              fail: failCount,
            })
          }}
        </span>
        <span class="server-health-summary__desc">
          {{
            t('server.health.lastChecked', {
              time: formatDateTime(new Date(server.health.checkedAt).toISOString()),
            })
          }}
        </span>
      </div>
      <GButton
        variant="primary"
        size="sm"
        icon="activity"
        :loading="checking"
        @click="$emit('recheck')">
        {{ checking ? t('server.health.checking') : t('server.health.recheck') }}
      </GButton>
    </div>

    <!-- 检查项列表 -->
    <div class="server-health-list">
      <div
        v-for="item in server.health.items"
        :key="item.key"
        class="server-health-item"
        :class="`server-health-item--${item.status}`">
        <span class="server-health-item__icon" :style="iconStyle(item.status)">
          <GIcon :name="statusIcon(item.status)" :size="16" />
        </span>
        <div class="server-health-item__body">
          <span class="server-health-item__label">{{ item.label }}</span>
          <span class="server-health-item__message">{{ item.message }}</span>
        </div>
        <div class="server-health-item__meta">
          <span v-if="item.latency > 0" class="server-health-item__latency"
            >{{ item.latency }}ms</span
          >
          <GBadge :variant="badgeVariant(item.status)" type="soft" size="sm">
            {{ statusLabel(item.status) }}
          </GBadge>
        </div>
      </div>
    </div>

    <p class="server-connection__hint">
      <GIcon name="info-circle" :size="12" />
      {{ t('server.health.hint') }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GBadge from '@components/base/GBadge.vue'
import type { HealthItemStatus, Server } from '../types'
import { healthColor } from '../utils'
import { useServerHealth } from '../composables/useServerHealth'
import { toRef } from 'vue'
import { formatDateTime } from '../utils'

const props = defineProps<{ server: Server }>()

defineEmits<{ recheck: [] }>()

const { t } = useI18n()
const { checking, passCount, warnCount, failCount } = useServerHealth(toRef(props, 'server'))

const ringStroke = 7
const ringRadius = (84 - ringStroke) / 2
const ringCircumference = 2 * Math.PI * ringRadius

const scoreColor = computed(() => healthColor(props.server.health.overall))

const overallLabel = computed(() => {
  switch (props.server.health.overall) {
    case 'healthy':
      return t('server.health.overall.healthy')
    case 'warning':
      return t('server.health.overall.warning')
    case 'critical':
      return t('server.health.overall.critical')
    default:
      return t('server.health.overall.unknown')
  }
})

function statusIcon(status: HealthItemStatus): string {
  switch (status) {
    case 'pass':
      return 'check-circle'
    case 'warn':
      return 'alert-triangle'
    case 'fail':
      return 'alert-circle'
    case 'pending':
      return 'loader'
  }
}

function statusLabel(status: HealthItemStatus): string {
  switch (status) {
    case 'pass':
      return t('server.health.itemStatus.pass')
    case 'warn':
      return t('server.health.itemStatus.warn')
    case 'fail':
      return t('server.health.itemStatus.fail')
    case 'pending':
      return t('server.health.itemStatus.pending')
  }
}

function badgeVariant(status: HealthItemStatus): 'success' | 'warning' | 'error' | 'neutral' {
  switch (status) {
    case 'pass':
      return 'success'
    case 'warn':
      return 'warning'
    case 'fail':
      return 'error'
    case 'pending':
      return 'neutral'
  }
}

function iconStyle(status: HealthItemStatus): Record<string, string> {
  const colors: Record<HealthItemStatus, string> = {
    pass: '#22C55E',
    warn: '#F59E0B',
    fail: '#EF4444',
    pending: '#6B6B72',
  }
  const c = colors[status]
  return { color: c, background: `${c}1f` }
}
</script>
