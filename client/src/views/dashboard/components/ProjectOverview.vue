<!--
  ProjectOverview — 最近项目概览
  ------------------------------------------------------------------
  展示最近项目卡片，支持 Pin/Favorite/快速启停/进入项目。
-->
<template>
  <section class="dashboard-section">
    <div class="dashboard-section__head">
      <div class="dashboard-section__title">
        <GIcon name="projects" :size="16" class="dashboard-section__title-icon" />
        <span>{{ title }}</span>
      </div>
      <button class="dashboard-section__more" @click="$emit('viewAll')">
        查看全部
        <GIcon name="chevron-right" :size="12" />
      </button>
    </div>

    <div class="dashboard-grid--projects">
      <GCard
        v-for="(project, i) in sortedProjects"
        :key="project.id"
        variant="interactive"
        padding="md"
        clickable
        class="project-overview__card dashboard-card-lift"
        :class="`stagger-${(i % 6) + 1}`"
        @click="$emit('open', project)"
      >
        <div class="project-overview__head">
          <span class="project-overview__icon">
            <GIcon :name="project.icon" :size="18" />
          </span>
          <div class="project-overview__title-wrap">
            <span class="project-overview__name">{{ project.name }}</span>
            <GStatusBadge :status="project.status" size="sm" />
          </div>
          <div class="project-overview__actions">
            <button
              class="project-overview__pin"
              :class="{ 'project-overview__pin--active': project.favorite }"
              title="收藏"
              @click.stop="$emit('toggleFavorite', project.id)"
            >
              <GIcon :name="project.favorite ? 'star' : 'star-off'" :size="14" />
            </button>
            <button
              class="project-overview__pin"
              :class="{ 'project-overview__pin--active': project.pinned }"
              title="固定"
              @click.stop="$emit('togglePin', project.id)"
            >
              <GIcon name="pin" :size="14" />
            </button>
          </div>
        </div>

        <p class="project-overview__desc">{{ project.description }}</p>

        <div class="project-overview__meta">
          <span class="project-overview__meta-item">
            <GIcon name="link" :size="12" />
            {{ project.tunnelCount }} 隧道
          </span>
          <span class="project-overview__meta-item project-overview__meta-item--online">
            <GStatusDot status="online" size="xs" />
            {{ project.runningCount }} 运行
          </span>
          <span class="project-overview__meta-item">
            <GIcon name="clock" :size="12" />
            {{ project.lastStartedAt }}
          </span>
        </div>

        <div class="project-overview__foot">
          <GButton
            size="sm"
            :variant="project.runningCount > 0 ? 'ghost' : 'primary'"
            :icon="project.runningCount > 0 ? 'stop' : 'play'"
            @click.stop="onToggleRun(project)"
          >
            {{ project.runningCount > 0 ? '停止' : '启动' }}
          </GButton>
          <GButton
            size="sm"
            variant="ghost"
            trailing-icon="arrow-right"
            @click.stop="$emit('open', project)"
          >
            进入
          </GButton>
        </div>
      </GCard>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GCard from "@components/base/GCard.vue"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"
import GStatusBadge from "@components/status/GStatusBadge.vue"
import GStatusDot from "@components/status/GStatusDot.vue"
import type { DashboardProject } from "../types"

const props = withDefaults(
  defineProps<{
    projects: DashboardProject[]
    title?: string
  }>(),
  {
    title: "项目概览",
  },
)

// 固定的排前面，再按收藏排，再按原始顺序
const sortedProjects = computed(() => {
  return [...props.projects].sort((a, b) => {
    if (a.pinned !== b.pinned) return a.pinned ? -1 : 1
    if (a.favorite !== b.favorite) return a.favorite ? -1 : 1
    return 0
  })
})

const emit = defineEmits<{
  open: [project: DashboardProject]
  togglePin: [id: string]
  toggleFavorite: [id: string]
  start: [project: DashboardProject]
  stop: [project: DashboardProject]
  viewAll: []
}>()

function onToggleRun(project: DashboardProject) {
  if (project.runningCount > 0) {
    emit("stop", project)
  } else {
    emit("start", project)
  }
}
</script>

<style scoped>
.project-overview__card {
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
}
.project-overview__head {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.project-overview__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
  flex-shrink: 0;
}
.project-overview__title-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.project-overview__name {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.project-overview__actions {
  display: flex;
  align-items: center;
  gap: 2px;
}
.project-overview__pin {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}
.project-overview__pin:hover {
  background: var(--bg-surface-hover);
  color: var(--text-secondary);
}
.project-overview__pin--active {
  color: var(--color-warning);
}
.project-overview__desc {
  margin-top: var(--space-3);
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  line-height: var(--leading-relaxed);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 36px;
}
.project-overview__meta {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  flex-wrap: wrap;
  margin-top: var(--space-3);
  padding-top: var(--space-3);
  border-top: 1px solid var(--border-subtle);
}
.project-overview__meta-item {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
.project-overview__meta-item--online {
  color: var(--color-success);
}
.project-overview__foot {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-3);
}
</style>
