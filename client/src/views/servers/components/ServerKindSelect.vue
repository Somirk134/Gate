<!--
  ServerKindSelect — 服务器类型选择器
  ------------------------------------------------------------------
  网格选择服务器类型。V1 启用 personal/cloud/nas/company/docker，
  kubernetes 预留。颜色随类型变化。
-->
<template>
  <div class="server-kind-grid">
    <button
      v-for="preset in KIND_PRESETS"
      :key="preset.key"
      type="button"
      class="server-kind-option"
      :class="{
        'server-kind-option--active': modelValue === preset.key,
        'server-kind-option--disabled': preset.availability !== 'enabled',
      }"
      :style="{ '--server-color': preset.color }"
      :disabled="preset.availability !== 'enabled'"
      @click="preset.availability === 'enabled' && $emit('update:modelValue', preset.key)"
    >
      <div class="server-kind-option__head">
        <span
          class="server-kind-option__icon"
          :style="{ background: preset.color + '1f', color: preset.color }"
        >
          <GIcon :name="preset.icon" :size="14" />
        </span>
        <span class="server-kind-option__name">{{ preset.label }}</span>
      </div>
      <span class="server-kind-option__desc">{{ preset.description }}</span>
      <GBadge
        v-if="preset.availability !== 'enabled'"
        variant="neutral"
        type="soft"
        size="sm"
        class="server-kind-option__badge"
      >
        {{ preset.availability === "soon" ? "即将" : "计划" }}
      </GBadge>
    </button>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import GBadge from "@components/base/GBadge.vue"
import type { ServerKind } from "../types"
import { KIND_PRESETS } from "../utils"

defineProps<{ modelValue: ServerKind }>()
defineEmits<{ "update:modelValue": [value: ServerKind] }>()
</script>
