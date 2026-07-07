<template>
  <section class="projects-page">
    <GEmptyState
      title="该功能暂未实现"
      description="Project 后端持久化与 Runtime 关联尚未接入。为避免展示示例项目，本页暂不显示任何业务数据。"
    >
      <template #icon>
        <GIcon name="projects" :size="34" />
      </template>
      <template #action>
        <GButton variant="primary" icon="router" @click="router.push('/tunnels')">
          查看 Tunnel
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
import "./styles/project.css"

const router = useRouter()
</script>

<style scoped>
.projects-page {
  width: min(100%, var(--content-max-width));
  min-height: 520px;
  display: grid;
  place-items: center;
  margin: 0 auto;
}
</style>
