<!--
  ServerCard — 服务器列表行（左栏）
  ------------------------------------------------------------------
  左栏列表项。展示类型图标 / 名称 / 公网IP / 状态 / Ping。
  交互：
    - 单击选中（高亮 + 联动中栏工作区与右栏 Inspector）
    - 双击进入详情（聚焦工作区）
    - Hover 显示快捷操作（连接/断开/编辑/健康检查）
    - Favorite 快捷切换
  颜色通过 --server-color CSS 变量驱动，随类型变化。
-->
<template>
  <div
    class="server-row"
    :class="[`server-row--${server.status}`, { 'server-row--active': active }]"
    :style="colorVars"
    @click="$emit('select', server)"
    @dblclick="$emit('open', server)"
    @contextmenu.prevent="$emit('contextmenu', server, $event)">
    <span class="server-row__bar" :class="`server-row__bar--${server.status}`" />

    <span class="server-row__icon">
      <GIcon :name="kindPreset.icon" :size="14" />
    </span>

    <div class="server-row__main">
      <span class="server-row__name" :title="server.name">{{ server.name }}</span>
      <span class="server-row__addr" :title="server.publicIp">{{ server.publicIp }}</span>
    </div>

    <div class="server-row__meta">
      <GStatusDot :status="dotStatus" :pulse="config.pulse" size="sm" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'
import GStatusDot from '@components/status/GStatusDot.vue'
import type { Server } from '../types'
import { SERVER_STATUS_CONFIG, KIND_MAP, serverColorVars } from '../utils'

const props = defineProps<{
  server: Server
  active?: boolean
}>()

defineEmits<{
  select: [server: Server]
  open: [server: Server]
  contextmenu: [server: Server, event: MouseEvent]
}>()

const colorVars = computed(() => serverColorVars(props.server.kind))
const config = computed(() => SERVER_STATUS_CONFIG[props.server.status])
const kindPreset = computed(() => KIND_MAP[props.server.kind])
const dotStatus = computed(() => config.value.dotStatus)
</script>
