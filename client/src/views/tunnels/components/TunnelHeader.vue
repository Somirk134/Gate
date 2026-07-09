<!--
  TunnelHeader — 工作区顶部详情头部
  ------------------------------------------------------------------
  显示：协议图标 / 名称 / 公网地址 / 状态 / 收藏 / 固定
  指标条：服务器 / 项目 / 协议 / 流量 / 连接 / 运行时间
  按钮：启动 / 停止 / 重启 / 克隆 / 导出 / 删除
-->
<template>
  <div class="tunnel-detail-header" :style="colorVars">
    <div class="tunnel-detail-header__left">
      <span class="tunnel-detail-header__icon">
        <GIcon :name="protocolPreset.icon" :size="20" />
      </span>
      <div class="tunnel-detail-header__info">
        <div class="tunnel-detail-header__title-row">
          <h2 class="tunnel-detail-header__name" :title="tunnel.name">
            {{ tunnel.name }}
          </h2>
          <TunnelStatus :status="tunnel.status" size="md" />
          <button
            class="tunnel-header__quick"
            :class="{ 'tunnel-header__quick--active': tunnel.favorite }"
            :title="t('project.card.favorite')"
            @click="$emit('toggle-favorite', tunnel.id)">
            <GIcon :name="tunnel.favorite ? 'star' : 'star-off'" :size="15" />
          </button>
          <button
            class="tunnel-header__quick"
            :class="{ 'tunnel-header__quick--pinned': tunnel.pinned }"
            :title="t('project.card.pin')"
            @click="$emit('toggle-pin', tunnel.id)">
            <GIcon name="pin" :size="15" />
          </button>
        </div>
        <div class="tunnel-detail-header__addr">
          {{ tunnel.localHost }}:{{ tunnel.localPort }} → {{ tunnel.publicAddr }}
        </div>
      </div>
    </div>

    <div class="tunnel-detail-header__actions">
      <GButton
        v-if="!isRunning"
        size="sm"
        variant="primary"
        icon="play"
        :disabled="isTransition"
        @click="$emit('start')">
        {{ t('tunnel.start') }}
      </GButton>
      <GButton
        v-else
        size="sm"
        variant="secondary"
        icon="stop"
        :disabled="isTransition"
        @click="$emit('stop')">
        {{ t('tunnel.stop') }}
      </GButton>
      <GButton
        size="sm"
        variant="ghost"
        icon="refresh"
        :disabled="isTransition"
        @click="$emit('restart')">
        {{ t('tunnel.restart') }}
      </GButton>
      <GIconButton
        name="copy"
        size="sm"
        variant="ghost"
        :tooltip="t('tunnel.header.clone')"
        @click="$emit('clone')" />
      <GIconButton
        name="download"
        size="sm"
        variant="ghost"
        :tooltip="t('tunnel.header.exportConfig')"
        @click="$emit('export')" />
      <GIconButton
        name="trash"
        size="sm"
        variant="ghost"
        :tooltip="t('common.delete')"
        @click="$emit('delete')" />
    </div>
  </div>

  <!-- 指标条 -->
  <div class="tunnel-metric-bar">
    <div class="tunnel-metric-bar__item">
      <GIcon name="servers" :size="13" />
      <span class="tunnel-metric-bar__label">{{ t('tunnel.overview.server') }}</span>
      <span class="tunnel-metric-bar__value">{{ tunnel.serverName }}</span>
    </div>
    <span class="tunnel-metric-bar__sep" />
    <div class="tunnel-metric-bar__item">
      <GIcon name="package" :size="13" />
      <span class="tunnel-metric-bar__label">{{ t('tunnel.overview.project') }}</span>
      <span class="tunnel-metric-bar__value">{{ tunnel.projectName }}</span>
    </div>
    <span class="tunnel-metric-bar__sep" />
    <div class="tunnel-metric-bar__item">
      <GIcon name="cloud" :size="13" />
      <span class="tunnel-metric-bar__label">{{ t('common.traffic') }}</span>
      <span class="tunnel-metric-bar__value">{{ trafficLabel }}</span>
    </div>
    <span class="tunnel-metric-bar__sep" />
    <div class="tunnel-metric-bar__item">
      <GIcon name="link" :size="13" />
      <span class="tunnel-metric-bar__label">{{ t('tunnel.connections') }}</span>
      <span class="tunnel-metric-bar__value">{{ tunnel.statistics.connections }}</span>
    </div>
    <span class="tunnel-metric-bar__sep" />
    <div class="tunnel-metric-bar__item">
      <GIcon name="clock" :size="13" />
      <span class="tunnel-metric-bar__label">{{ t('tunnel.detail.uptime') }}</span>
      <span class="tunnel-metric-bar__value">{{ uptimeLabel }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GIconButton from '@components/base/GIconButton.vue'
import TunnelStatus from './TunnelStatus.vue'
import type { Tunnel } from '../types'
import {
  PROTOCOL_MAP,
  tunnelColorVars,
  formatBytes,
  formatDuration,
  isRunningStatus,
  isTransitionStatus,
} from '../utils'

const props = defineProps<{ tunnel: Tunnel }>()
const { t } = useI18n()

defineEmits<{
  start: []
  stop: []
  restart: []
  clone: []
  export: []
  delete: []
  'toggle-pin': [id: string]
  'toggle-favorite': [id: string]
}>()

const colorVars = computed(() => tunnelColorVars(props.tunnel.protocol))
const protocolPreset = computed(() => PROTOCOL_MAP[props.tunnel.protocol])
const isRunning = computed(() => isRunningStatus(props.tunnel.status))
const isTransition = computed(() => isTransitionStatus(props.tunnel.status))
const trafficLabel = computed(() =>
  formatBytes(props.tunnel.traffic.totalUpload + props.tunnel.traffic.totalDownload),
)
const uptimeLabel = computed(() => formatDuration(props.tunnel.statistics.uptime, t))
</script>

<style scoped>
.tunnel-header__quick {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.tunnel-header__quick:hover {
  background: var(--bg-surface-hover);
  color: var(--text-secondary);
}

.tunnel-header__quick--active {
  color: var(--color-warning);
}

.tunnel-header__quick--pinned {
  color: var(--color-primary);
}
</style>
