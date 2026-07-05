<!--
  TunnelCard — 隧道列表行（左栏）
  ------------------------------------------------------------------
  左栏列表项。展示协议图标 / 名称 / 公网地址 / 状态 / 实时速度。
  交互：
    - 单击选中（高亮 + 联动中栏工作区与右栏 Inspector）
    - 双击进入详情（聚焦工作区）
    - Hover 显示快捷操作（启动/停止/编辑/更多）
    - Pin / Favorite 快捷切换
  颜色通过 --tunnel-color CSS 变量驱动，随协议变化。
-->
<template>
  <div
    class="tunnel-row"
    :class="[
      `tunnel-row--${tunnel.status}`,
      { 'tunnel-row--active': active },
    ]"
    :style="colorVars"
    @click="$emit('select', tunnel)"
    @dblclick="$emit('open', tunnel)"
    @contextmenu.prevent="$emit('contextmenu', tunnel, $event)"
  >
    <span class="tunnel-row__bar" :class="`tunnel-row__bar--${tunnel.status}`" />

    <span class="tunnel-row__icon">
      <GIcon :name="protocolPreset.icon" :size="14" />
    </span>

    <div class="tunnel-row__main">
      <span class="tunnel-row__name" :title="tunnel.name">{{ tunnel.name }}</span>
      <span class="tunnel-row__addr" :title="tunnel.publicAddr">{{ tunnel.publicAddr }}</span>
    </div>

    <div class="tunnel-row__meta">
      <GStatusDot
        :status="dotStatus"
        :pulse="config.pulse"
        size="sm"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import GStatusDot from "@components/status/GStatusDot.vue"
import type { Tunnel } from "../types"
import { TUNNEL_STATUS_CONFIG, PROTOCOL_MAP, tunnelColorVars } from "../utils"

const props = defineProps<{
  tunnel: Tunnel
  active?: boolean
}>()

defineEmits<{
  select: [tunnel: Tunnel]
  open: [tunnel: Tunnel]
  contextmenu: [tunnel: Tunnel, event: MouseEvent]
}>()

const colorVars = computed(() => tunnelColorVars(props.tunnel.protocol))
const config = computed(() => TUNNEL_STATUS_CONFIG[props.tunnel.status])
const protocolPreset = computed(() => PROTOCOL_MAP[props.tunnel.protocol])
const dotStatus = computed(() => config.value.dotStatus)
</script>

<style scoped>
.tunnel-row__bar {
  background: var(--status-offline);
}

.tunnel-row__bar--running { background: var(--status-online); }
.tunnel-row__bar--starting { background: var(--status-starting); }
.tunnel-row__bar--connecting { background: var(--status-connecting); }
.tunnel-row__bar--restarting { background: var(--status-connecting); }
.tunnel-row__bar--stopping { background: var(--status-warning); }
.tunnel-row__bar--error { background: var(--status-error); }
.tunnel-row__bar--disconnected { background: var(--status-warning); }
.tunnel-row__bar--stopped { background: var(--status-offline); }
.tunnel-row__bar--offline { background: var(--status-offline); }
</style>
