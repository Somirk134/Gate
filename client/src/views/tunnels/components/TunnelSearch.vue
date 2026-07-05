<!--
  TunnelSearch — 隧道搜索框
  ------------------------------------------------------------------
  基于 GSearchInput，封装 placeholder 与事件透传。
-->
<template>
  <GSearchInput
    :model-value="modelValue"
    :placeholder="placeholder"
    size="sm"
    @update:model-value="$emit('update:modelValue', $event)"
  />
</template>

<script setup lang="ts">
import GSearchInput from "@components/form/GSearchInput.vue"

withDefaults(
  defineProps<{
    modelValue: string
    placeholder?: string
  }>(),
  {
    placeholder: "搜索名称、协议、端口、项目、服务器…",
  },
)

defineEmits<{ "update:modelValue": [value: string] }>()
</script>
