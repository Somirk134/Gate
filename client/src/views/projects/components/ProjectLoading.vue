<!--
  ProjectLoading — 骨架屏加载态
  ------------------------------------------------------------------
  加载项目时统一展示 Skeleton 卡片网格。
-->
<template>
  <div class="projects-grid">
    <div
      v-for="i in count"
      :key="i"
      class="project-skeleton-card"
    >
      <div class="project-skeleton-card__head">
        <GSkeleton
          variant="circle"
          width="40px"
          height="40px"
        />
        <div class="project-skeleton-card__head-text">
          <GSkeleton
            variant="rect"
            height="14px"
            width="60%"
            rounded="4px"
          />
          <GSkeleton
            variant="rect"
            height="10px"
            width="40%"
            rounded="4px"
          />
        </div>
        <GSkeleton
          variant="circle"
          width="22px"
          height="22px"
        />
      </div>
      <GSkeleton
        variant="text"
        :rows="2"
      />
      <div class="project-skeleton-card__meta">
        <GSkeleton
          variant="rect"
          height="12px"
          width="70px"
          rounded="4px"
        />
        <GSkeleton
          variant="rect"
          height="12px"
          width="60px"
          rounded="4px"
        />
      </div>
      <div class="project-skeleton-card__foot">
        <GSkeleton
          variant="rect"
          height="24px"
          width="60px"
          rounded="6px"
        />
        <GSkeleton
          variant="rect"
          height="24px"
          width="60px"
          rounded="6px"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import GSkeleton from "@components/feedback/GSkeleton.vue"

withDefaults(defineProps<{ count?: number }>(), { count: 8 })
</script>

<style scoped>
.project-skeleton-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4);
  background: var(--bg-card);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-card);
}

.project-skeleton-card__head {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.project-skeleton-card__head-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.project-skeleton-card__meta {
  display: flex;
  gap: var(--space-3);
}

.project-skeleton-card__foot {
  display: flex;
  gap: var(--space-2);
}
</style>
