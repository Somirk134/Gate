<!--
  ProjectSearch — 项目搜索框
  ------------------------------------------------------------------
  基于 GSearchInput，封装 placeholder 与事件透传。
-->
<template>
  <GSearchInput
    :model-value="modelValue"
    :placeholder="placeholder"
    size="md"
    class="projects-toolbar__search"
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
    placeholder: "搜索项目名称、标签、服务器…",
  },
)

defineEmits<{ "update:modelValue": [value: string] }>()
</script>
