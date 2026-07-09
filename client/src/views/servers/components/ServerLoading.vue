<!--
  ServerLoading — 骨架屏加载态
  ------------------------------------------------------------------
  加载服务器时统一展示 Skeleton 行 / 卡片。
-->
<template>
  <div class="server-list__items">
    <div
      v-for="i in count"
      :key="i"
      class="server-skeleton-row"
    >
      <GSkeleton
        variant="rect"
        width="3px"
        height="28px"
        rounded="2px"
      />
      <GSkeleton
        variant="rect"
        width="28px"
        height="28px"
        rounded="6px"
      />
      <div
        class="server-skeleton-row__text"
        style="flex:1"
      >
        <GSkeleton
          variant="rect"
          height="12px"
          width="70%"
          rounded="4px"
        />
        <GSkeleton
          variant="rect"
          height="10px"
          width="50%"
          rounded="4px"
        />
      </div>
      <GSkeleton
        variant="circle"
        width="8px"
        height="8px"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import GSkeleton from "@components/feedback/GSkeleton.vue"

withDefaults(defineProps<{ count?: number }>(), { count: 8 })
</script>

<style scoped>
.server-skeleton-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
}

.server-skeleton-row__text {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
</style>
