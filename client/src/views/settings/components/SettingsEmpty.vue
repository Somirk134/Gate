<template>
  <div class="settings-empty">
    <GIcon name="search" :size="34" />
    <h2>没有找到设置</h2>
    <p v-if="query">没有匹配“{{ query }}”的结果。</p>
    <p v-else>当前分类暂无设置项。</p>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"

withDefaults(
  defineProps<{
    query?: string
  }>(),
  {
    query: "",
  },
)
</script>
