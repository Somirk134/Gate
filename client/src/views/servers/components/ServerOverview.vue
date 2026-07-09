<!--
  ServerOverview — 工作区 Overview 标签
  ------------------------------------------------------------------
  展示服务器基本信息：Hostname / OS / Architecture / Rust Version /
  Server Version / Install Time / Last Online / Last Heartbeat。
-->
<template>
  <div class="server-overview">
    <div class="server-info-grid">
      <!-- 服务器信息 -->
      <div class="server-info-card">
        <div class="server-info-card__title">
          <GIcon name="servers" :size="12" />
          {{ t('server.overview.serverInfo') }}
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.status') }}</span>
          <ServerStatus :status="server.status" size="sm" />
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.type') }}</span>
          <ServerBadge :kind="server.kind" size="sm" />
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.publicIp') }}</span>
          <span
            class="server-info-row__value mono copy"
            :title="t('server.overview.copyTitle', { value: server.publicIp })"
            @click="copy(server.publicIp)">
            {{ server.publicIp }}
          </span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.region') }}</span>
          <span class="server-info-row__value">{{ server.region }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.version') }}</span>
          <span class="server-info-row__value mono">{{ server.version }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.connectionMethod') }}</span>
          <span class="server-info-row__value mono">{{
            server.connectionMethod.toUpperCase()
          }}</span>
        </div>
      </div>

      <!-- 系统信息 -->
      <div class="server-info-card">
        <div class="server-info-card__title">
          <GIcon name="cpu" :size="12" />
          {{ t('server.overview.systemInfo') }}
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.hostname') }}</span>
          <span class="server-info-row__value mono">{{ server.overview.hostname }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.os') }}</span>
          <span class="server-info-row__value">{{ server.overview.os }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.arch') }}</span>
          <span class="server-info-row__value mono">{{ server.overview.arch }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.rustVersion') }}</span>
          <span class="server-info-row__value mono">{{ server.overview.rustVersion }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.serverVersion') }}</span>
          <span class="server-info-row__value mono">{{ server.overview.serverVersion }}</span>
        </div>
      </div>

      <!-- 时间信息 -->
      <div class="server-info-card">
        <div class="server-info-card__title">
          <GIcon name="clock" :size="12" />
          {{ t('server.overview.timeInfo') }}
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.installTime') }}</span>
          <span class="server-info-row__value mono">{{
            formatDateTime(server.overview.installTime)
          }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.lastOnline') }}</span>
          <span class="server-info-row__value">{{ server.overview.lastOnline }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.lastHeartbeat') }}</span>
          <span class="server-info-row__value">{{ server.overview.lastHeartbeat }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.lastConnected') }}</span>
          <span class="server-info-row__value">{{ server.lastConnectedAt }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">{{ t('server.overview.createdAt') }}</span>
          <span class="server-info-row__value mono">{{ formatDateTime(server.createdAt) }}</span>
        </div>
      </div>

      <!-- 备注 -->
      <div v-if="server.settings.remark" class="server-info-card">
        <div class="server-info-card__title">
          <GIcon name="file-text" :size="12" />
          {{ t('server.overview.remark') }}
        </div>
        <p class="server-overview__remark">
          {{ server.settings.remark }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import ServerStatus from './ServerStatus.vue'
import ServerBadge from './ServerBadge.vue'
import type { Server } from '../types'
import { formatDateTime } from '../utils'
import { useFeedback } from '@composables/useFeedback'

defineProps<{ server: Server }>()

const { toast } = useFeedback()
const { t } = useI18n()

function copy(text: string) {
  navigator.clipboard?.writeText(text).then(
    () => toast.success(t('common.copiedWithValue', { value: text })),
    () => toast.error(t('common.copyFailed')),
  )
}
</script>
