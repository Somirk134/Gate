<!--
  GPageContainer — 页面内容容器
  ------------------------------------------------------------------
  用途：所有页面最外层包裹器，统一最大宽度与内边距。
  Props:
    width  normal(1200) | narrow(800) | wide(1600) | full
    padded 是否内边距
-->
<template>
  <div class="g-page-container" :class="[`g-page-container--${width}`, { 'g-page-container--padded': padded }]">
    <slot />
  </div>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    width?: "normal" | "narrow" | "wide" | "full"
    padded?: boolean
  }>(),
  {
    width: "normal",
    padded: true,
  },
)
</script>

<style scoped>
.g-page-container {
  width: 100%;
  margin: 0 auto;
}
.g-page-container--padded {
  padding: var(--page-padding);
}
.g-page-container--normal { max-width: var(--content-max-width); }
.g-page-container--narrow { max-width: 800px; }
.g-page-container--wide   { max-width: 1600px; }
.g-page-container--full   { max-width: 100%; }
</style>
