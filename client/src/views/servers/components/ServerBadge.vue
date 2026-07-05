<!--
  ServerBadge — 类型徽章
  ------------------------------------------------------------------
  统一展示服务器类型，颜色随类型变化。未来类型以"即将"标记。
-->
<template>
  <span
    class="server-badge"
    :class="[`server-badge--${kind}`, `server-badge--${size}`]"
    :style="{ '--badge-color': color }"
  >
    <GIcon :name="icon" :size="size === 'sm' ? 11 : 13" />
    <span class="server-badge__label">{{ label }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import type { ServerKind } from "../types"
import { KIND_MAP } from "../utils"

const props = withDefaults(
  defineProps<{
    kind: ServerKind
    size?: "sm" | "md"
  }>(),
  { size: "sm" },
)

const preset = computed(() => KIND_MAP[props.kind])
const color = computed(() => preset.value.color)
const label = computed(() => preset.value.label)
const icon = computed(() => preset.value.icon)
</script>
