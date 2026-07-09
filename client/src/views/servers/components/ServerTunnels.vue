<!--
  ServerTunnels — 工作区 Tunnels 标签（Tunnel Overview）
  ------------------------------------------------------------------
  显示属于当前 Server 的 Tunnel。支持启动 / 停止 / 查看详情。全部 Mock。
-->
<template>
  <div class="server-tunnels">
    <div class="server-section__head">
      <div class="server-section__title">
        <GIcon
          name="router"
          :size="16"
          class="server-section__title-icon"
        />
        <span>Tunnel 列表</span>
        <GBadge
          variant="neutral"
          type="soft"
          size="sm"
        >
          {{ tunnels.length }}
        </GBadge>
      </div>
      <GButton
        size="sm"
        variant="ghost"
        icon="plus"
        @click="$emit('create-tunnel')"
      >
        New Tunnel
      </GButton>
    </div>

    <div
      v-if="tunnels.length"
      class="server-sublist"
    >
      <div
        v-for="tunnel in tunnels"
        :key="tunnel.id"
        class="server-sublist__item"
        @click="$emit('view-tunnel', tunnel)"
      >
        <span
          class="server-sublist__icon"
          :style="{ color: tunnel.color, background: tunnel.color + '1f' }"
        >
          <GIcon
            :name="tunnel.icon"
            :size="14"
          />
        </span>
        <div class="server-sublist__main">
          <span class="server-sublist__name">{{ tunnel.name }}</span>
          <span class="server-sublist__meta">{{ tunnel.publicAddr }} · {{ tunnel.protocol.toUpperCase() }}</span>
        </div>
        <div class="server-sublist__actions">
          <GStatusDot
            :status="tunnel.running ? 'online' : 'offline'"
            size="sm"
          />
          <GIconButton
            v-if="!tunnel.running"
            name="play"
            size="sm"
            variant="ghost"
            tooltip="启动"
            @click.stop="$emit('start-tunnel', tunnel)"
          />
          <GIconButton
            v-else
            name="stop"
            size="sm"
            variant="ghost"
            tooltip="停止"
            @click.stop="$emit('stop-tunnel', tunnel)"
          />
          <GIconButton
            name="external-link"
            size="sm"
            variant="ghost"
            tooltip="查看详情"
            @click.stop="$emit('view-tunnel', tunnel)"
          />
        </div>
      </div>
    </div>

    <div
      v-else
      class="server-logs__empty"
    >
      <GIcon
        name="router"
        :size="20"
      />
      <span>该服务器暂无 Tunnel</span>
      <GButton
        size="sm"
        variant="ghost"
        icon="plus"
        @click="$emit('create-tunnel')"
      >
        创建第一个 Tunnel
      </GButton>
    </div>

    <p class="server-connection__hint">
      <GIcon
        name="info-circle"
        :size="12"
      />
      展示绑定到当前服务器的 Tunnel。全部 Mock 数据，未来从 Tunnel 模块联动。
    </p>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"
import GBadge from "@components/base/GBadge.vue"
import GIconButton from "@components/base/GIconButton.vue"
import GStatusDot from "@components/status/GStatusDot.vue"

export interface ServerTunnelItem {
  id: string
  name: string
  protocol: string
  publicAddr: string
  running: boolean
  icon: string
  color: string
}

defineProps<{
  tunnels: ServerTunnelItem[]
}>()

defineEmits<{
  "create-tunnel": []
  "start-tunnel": [tunnel: ServerTunnelItem]
  "stop-tunnel": [tunnel: ServerTunnelItem]
  "view-tunnel": [tunnel: ServerTunnelItem]
}>()
</script>
