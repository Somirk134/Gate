<!--
  ServerHeader — 工作区顶部详情头部
  ------------------------------------------------------------------
  显示：类型图标 / 名称 / 状态 / 收藏 / 地区 / 公网IP / 版本 / 连接方式
  指标条：隧道 / 项目 / 流量 / 连接 / Ping / 运行时间
  按钮：连接 / 断开 / 重启 / 健康检查 / 编辑 / 删除
-->
<template>
  <div class="server-detail-header" :style="colorVars">
    <div class="server-detail-header__left">
      <span class="server-detail-header__icon">
        <GIcon :name="kindPreset.icon" :size="20" />
      </span>
      <div class="server-detail-header__info">
        <div class="server-detail-header__title-row">
          <h2 class="server-detail-header__name" :title="server.name">
            {{ server.name }}
          </h2>
          <ServerStatus :status="server.status" size="md" />
          <button
            class="server-header__quick"
            :class="{ 'server-header__quick--active': server.favorite }"
            :title="t('project.card.favorite')"
            @click="$emit('toggle-favorite', server.id)">
            <GIcon :name="server.favorite ? 'star' : 'star-off'" :size="15" />
          </button>
        </div>
        <div class="server-detail-header__addr">
          {{ server.publicIp }} · {{ server.region }} · {{ server.version }}
        </div>
      </div>
    </div>

    <div class="server-detail-header__actions">
      <GButton
        v-if="!isOnline"
        size="sm"
        variant="primary"
        icon="plug"
        :disabled="isTransition"
        @click="$emit('connect')">
        {{ t('server.actions.connect') }}
      </GButton>
      <GButton v-else size="sm" variant="secondary" icon="stop" @click="$emit('disconnect')">
        {{ t('server.actions.disconnect') }}
      </GButton>
      <GButton
        size="sm"
        variant="ghost"
        icon="refresh"
        :disabled="isTransition || !isOnline"
        @click="$emit('restart')">
        {{ t('server.reconnect') }}
      </GButton>
      <GIconButton
        name="activity"
        size="sm"
        variant="ghost"
        :tooltip="t('server.header.healthCheck')"
        @click="$emit('check-health')" />
      <GIconButton name="edit" size="sm" variant="ghost" :tooltip="t('common.edit')" @click="$emit('edit')" />
      <GIconButton name="trash" size="sm" variant="ghost" :tooltip="t('common.delete')" @click="$emit('delete')" />
    </div>
  </div>

  <!-- 指标条 -->
  <div class="server-metric-bar">
    <div class="server-metric-bar__item">
      <GIcon name="router" :size="13" />
      <span class="server-metric-bar__label">{{ t('server.metrics.tunnels') }}</span>
      <span class="server-metric-bar__value">{{ server.statistics.tunnelCount }}</span>
    </div>
    <span class="server-metric-bar__sep" />
    <div class="server-metric-bar__item">
      <GIcon name="package" :size="13" />
      <span class="server-metric-bar__label">{{ t('server.metrics.projects') }}</span>
      <span class="server-metric-bar__value">{{ server.statistics.projectCount }}</span>
    </div>
    <span class="server-metric-bar__sep" />
    <div class="server-metric-bar__item">
      <GIcon name="cloud" :size="13" />
      <span class="server-metric-bar__label">{{ t('common.traffic') }}</span>
      <span class="server-metric-bar__value">{{ trafficLabel }}</span>
    </div>
    <span class="server-metric-bar__sep" />
    <div class="server-metric-bar__item">
      <GIcon name="link" :size="13" />
      <span class="server-metric-bar__label">{{ t('server.metrics.connections') }}</span>
      <span class="server-metric-bar__value">{{ server.monitor.connections.active }}</span>
    </div>
    <span class="server-metric-bar__sep" />
    <div class="server-metric-bar__item">
      <GIcon name="gauge" :size="13" />
      <span class="server-metric-bar__label">Ping</span>
      <span class="server-metric-bar__value">{{ isOnline ? `${server.ping}ms` : '—' }}</span>
    </div>
    <span class="server-metric-bar__sep" />
    <div class="server-metric-bar__item">
      <GIcon name="clock" :size="13" />
      <span class="server-metric-bar__label">{{ t('server.metrics.uptime') }}</span>
      <span class="server-metric-bar__value">{{ uptimeLabel }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GIconButton from '@components/base/GIconButton.vue'
import ServerStatus from './ServerStatus.vue'
import type { Server } from '../types'
import {
  KIND_MAP,
  serverColorVars,
  formatBytes,
  formatDuration,
  isOnlineStatus,
  isTransitionStatus,
} from '../utils'

const props = defineProps<{ server: Server }>()
const { t } = useI18n()

defineEmits<{
  connect: []
  disconnect: []
  restart: []
  'check-health': []
  edit: []
  delete: []
  'toggle-favorite': [id: string]
}>()

const colorVars = computed(() => serverColorVars(props.server.kind))
const kindPreset = computed(() => KIND_MAP[props.server.kind])
const isOnline = computed(() => isOnlineStatus(props.server.status))
const isTransition = computed(() => isTransitionStatus(props.server.status))
const trafficLabel = computed(() =>
  formatBytes(props.server.traffic.totalUpload + props.server.traffic.totalDownload),
)
const uptimeLabel = computed(() => formatDuration(props.server.statistics.uptime, t))
</script>
