<template>
  <section class="project-detail-page">
    <GEmptyState
      title="该功能暂未实现"
      description="Project 详情需要真实 Project 存储与 Tunnel 关联数据。当前版本不使用示例数据填充详情页。"
    >
      <template #icon>
        <GIcon name="projects" :size="34" />
      </template>
      <template #action>
        <GButton variant="primary" icon="arrow-left" @click="router.push('/projects')">
          返回 Projects
        </GButton>
      </template>
    </GEmptyState>
  </section>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router"
import GButton from "@components/base/GButton.vue"
import GEmptyState from "@components/feedback/GEmptyState.vue"
import GIcon from "@components/icons/GIcon.vue"

const router = useRouter()
</script>

<style scoped>
.project-detail-page {
  width: min(100%, var(--content-max-width));
  min-height: 520px;
  display: grid;
  place-items: center;
  margin: 0 auto;
}
</style>
