<!--
  TunnelOverview — 工作区 Overview 标签
  ------------------------------------------------------------------
  展示隧道完整信息：连接状态 / 公网地址 / 本地地址 / 协议 /
  创建时间 / 更新时间 / 备注。地址支持点击复制。
-->
<template>
  <div class="tunnel-overview">
    <div class="tunnel-info-grid">
      <!-- 连接信息 -->
      <div class="tunnel-info-card">
        <div class="tunnel-info-card__title">
          <GIcon name="link" :size="12" />
          {{ t('tunnel.overview.connectionInfo') }}
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.status') }}</span>
          <TunnelStatus :status="tunnel.status" size="sm" />
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.protocol') }}</span>
          <TunnelBadge :protocol="tunnel.protocol" size="sm" />
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.publicAddress') }}</span>
          <span
            class="tunnel-info-row__value mono copy"
            :title="t('tunnel.overview.copyTitle', { value: tunnel.publicAddr })"
            @click="copy(tunnel.publicAddr)">
            {{ tunnel.publicAddr }}
          </span>
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.localAddress') }}</span>
          <span class="tunnel-info-row__value mono"
            >{{ tunnel.localHost }}:{{ tunnel.localPort }}</span
          >
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.remotePort') }}</span>
          <span class="tunnel-info-row__value mono">{{ tunnel.remotePort }}</span>
        </div>
      </div>

      <!-- 归属信息 -->
      <div class="tunnel-info-card">
        <div class="tunnel-info-card__title">
          <GIcon name="package" :size="12" />
          {{ t('tunnel.overview.ownershipInfo') }}
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.server') }}</span>
          <span class="tunnel-info-row__value">{{ tunnel.serverName }}</span>
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.project') }}</span>
          <span class="tunnel-info-row__value">{{ tunnel.projectName }}</span>
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.autoStart') }}</span>
          <span class="tunnel-info-row__value">
            <GIcon
              :name="tunnel.autoStart ? 'check' : 'close'"
              :size="12"
              :class="tunnel.autoStart ? 'on' : 'off'" />
            {{ tunnel.autoStart ? t('tunnel.overview.enabled') : t('tunnel.overview.disabled') }}
          </span>
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.tags') }}</span>
          <span v-if="tunnel.tags.length" class="tunnel-info-row__value">
            <TunnelTag v-for="tag in tunnel.tags" :key="tag" :name="tag" />
          </span>
          <span v-else class="tunnel-info-row__value">—</span>
        </div>
      </div>

      <!-- 时间信息 -->
      <div class="tunnel-info-card">
        <div class="tunnel-info-card__title">
          <GIcon name="clock" :size="12" />
          {{ t('tunnel.overview.timeInfo') }}
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.lastStarted') }}</span>
          <span class="tunnel-info-row__value">{{ tunnel.lastStartedAt }}</span>
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.createdAt') }}</span>
          <span class="tunnel-info-row__value mono">{{ formatDateTime(tunnel.createdAt) }}</span>
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.updatedAt') }}</span>
          <span class="tunnel-info-row__value mono">{{ formatDateTime(tunnel.updatedAt) }}</span>
        </div>
        <div class="tunnel-info-row">
          <span class="tunnel-info-row__label">{{ t('tunnel.overview.uptime') }}</span>
          <span class="tunnel-info-row__value mono">{{
            formatDuration(tunnel.statistics.uptime, t)
          }}</span>
        </div>
      </div>

      <!-- 备注 -->
      <div v-if="tunnel.remark" class="tunnel-info-card">
        <div class="tunnel-info-card__title">
          <GIcon name="file-text" :size="12" />
          {{ t('tunnel.overview.remark') }}
        </div>
        <p class="tunnel-overview__remark">
          {{ tunnel.remark }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import TunnelStatus from './TunnelStatus.vue'
import TunnelBadge from './TunnelBadge.vue'
import TunnelTag from './TunnelTag.vue'
import type { Tunnel } from '../types'
import { formatDateTime, formatDuration } from '../utils'
import { useFeedback } from '@composables/useFeedback'

defineProps<{ tunnel: Tunnel }>()

const { toast } = useFeedback()
const { t } = useI18n()

function copy(text: string) {
  navigator.clipboard?.writeText(text).then(
    () => toast.success(t('common.copiedWithValue', { value: text })),
    () => toast.error(t('common.copyFailed')),
  )
}
</script>

<style scoped>
.tunnel-info-row__value :deep(.on) {
  color: var(--color-success);
}
.tunnel-info-row__value :deep(.off) {
  color: var(--text-tertiary);
}

.tunnel-overview__remark {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-surface-hover);
  border-radius: var(--radius-md);
}
</style>
