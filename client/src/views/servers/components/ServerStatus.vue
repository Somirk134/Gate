<!--
  ServerStatus — 统一状态徽章
  ------------------------------------------------------------------
  基于 GStatusBadge，统一映射 ServerStatus → 圆点 + 文字。
  全模块状态展示唯一入口，颜色与 Badge 统一。
-->
<template>
  <GStatusBadge
    :status="config.dotStatus"
    :label="label ?? config.label"
    :size="size"
  />
</template>

<script setup lang="ts">
import { computed } from "vue"
import GStatusBadge from "@components/status/GStatusBadge.vue"
import type { ServerStatus } from "../types"
import { SERVER_STATUS_CONFIG } from "../utils"

const props = withDefaults(
  defineProps<{
    status: ServerStatus
    label?: string
    size?: "sm" | "md"
  }>(),
  { size: "md" },
)

const config = computed(() => SERVER_STATUS_CONFIG[props.status])
</script>
