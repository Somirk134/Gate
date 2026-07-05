<!--
  DashboardLoadingState — 加载骨架屏
  ------------------------------------------------------------------
  统一 Loading 风格：Card Skeleton + 行骨架。
-->
<template>
  <div class="dash-loading">
    <!-- Welcome skeleton -->
    <GSkeleton variant="rect" height="120px" rounded="var(--radius-xl)" />

    <!-- Quick actions skeleton -->
    <div class="dash-loading__row">
      <GSkeleton
        v-for="i in 6"
        :key="`qa-${i}`"
        variant="rect"
        height="96px"
        rounded="var(--radius-lg)"
      />
    </div>

    <!-- Stats skeleton -->
    <div class="dash-loading__row dash-loading__row--4">
      <GSkeleton
        v-for="i in 4"
        :key="`st-${i}`"
        variant="rect"
        height="92px"
        rounded="var(--radius-lg)"
      />
    </div>

    <!-- Projects skeleton -->
    <div class="dash-loading__row">
      <GSkeleton
        v-for="i in 3"
        :key="`pr-${i}`"
        variant="rect"
        height="140px"
        rounded="var(--radius-lg)"
      />
    </div>

    <!-- Bottom row skeleton -->
    <div class="dash-loading__bottom">
      <GSkeleton variant="rect" height="280px" rounded="var(--radius-xl)" />
      <GSkeleton variant="rect" height="280px" rounded="var(--radius-xl)" />
    </div>
  </div>
</template>

<script setup lang="ts">
import GSkeleton from "@components/feedback/GSkeleton.vue"
</script>

<style scoped>
.dash-loading {
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
  max-width: var(--content-max-width);
  margin: 0 auto;
}
.dash-loading__row {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: var(--space-3);
}
.dash-loading__row--4 {
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
}
.dash-loading__bottom {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: var(--space-4);
}
@media (max-width: 980px) {
  .dash-loading__bottom {
    grid-template-columns: 1fr;
  }
}
</style>
