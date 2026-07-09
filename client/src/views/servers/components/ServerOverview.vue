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
          服务器信息
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">状态</span>
          <ServerStatus :status="server.status" size="sm" />
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">类型</span>
          <ServerBadge :kind="server.kind" size="sm" />
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">公网 IP</span>
          <span
            class="server-info-row__value mono copy"
            :title="`点击复制 ${server.publicIp}`"
            @click="copy(server.publicIp)">
            {{ server.publicIp }}
          </span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">地区</span>
          <span class="server-info-row__value">{{ server.region }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">版本</span>
          <span class="server-info-row__value mono">{{ server.version }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">连接方式</span>
          <span class="server-info-row__value mono">{{
            server.connectionMethod.toUpperCase()
          }}</span>
        </div>
      </div>

      <!-- 系统信息 -->
      <div class="server-info-card">
        <div class="server-info-card__title">
          <GIcon name="cpu" :size="12" />
          系统信息
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">主机名</span>
          <span class="server-info-row__value mono">{{ server.overview.hostname }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">操作系统</span>
          <span class="server-info-row__value">{{ server.overview.os }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">架构</span>
          <span class="server-info-row__value mono">{{ server.overview.arch }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">Rust 版本</span>
          <span class="server-info-row__value mono">{{ server.overview.rustVersion }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">服务端版本</span>
          <span class="server-info-row__value mono">{{ server.overview.serverVersion }}</span>
        </div>
      </div>

      <!-- 时间信息 -->
      <div class="server-info-card">
        <div class="server-info-card__title">
          <GIcon name="clock" :size="12" />
          时间信息
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">安装时间</span>
          <span class="server-info-row__value mono">{{
            formatDateTime(server.overview.installTime)
          }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">最后在线</span>
          <span class="server-info-row__value">{{ server.overview.lastOnline }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">最后心跳</span>
          <span class="server-info-row__value">{{ server.overview.lastHeartbeat }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">最后连接</span>
          <span class="server-info-row__value">{{ server.lastConnectedAt }}</span>
        </div>
        <div class="server-info-row">
          <span class="server-info-row__label">创建时间</span>
          <span class="server-info-row__value mono">{{ formatDateTime(server.createdAt) }}</span>
        </div>
      </div>

      <!-- 备注 -->
      <div v-if="server.settings.remark" class="server-info-card">
        <div class="server-info-card__title">
          <GIcon name="file-text" :size="12" />
          备注
        </div>
        <p class="server-overview__remark">
          {{ server.settings.remark }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import GIcon from '@components/icons/GIcon.vue'
import ServerStatus from './ServerStatus.vue'
import ServerBadge from './ServerBadge.vue'
import type { Server } from '../types'
import { formatDateTime } from '../utils'
import { useFeedback } from '@composables/useFeedback'

defineProps<{ server: Server }>()

const { toast } = useFeedback()

function copy(text: string) {
  navigator.clipboard?.writeText(text).then(
    () => toast.success(`已复制：${text}`),
    () => toast.error('复制失败'),
  )
}
</script>
