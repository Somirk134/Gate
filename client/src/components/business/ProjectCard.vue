<!--
  ProjectCard — 项目卡片（业务组件模板）
  ------------------------------------------------------------------
  用途：项目列表中展示单个项目概览。
  属业务组件：基于 GCard + GStatusBadge 组合，仅负责展示结构，
  不包含任何数据获取/状态管理逻辑（数据由父级传入）。

  Props:
    name        项目名
    icon        项目图标名
    description 描述
    tunnelCount 隧道数
    onlineCount 在线数
    lastActive  最近活跃
    status      整体状态

  Events:
    click       点击整卡
    action      点击操作按钮（payload: action key）

  复用：GCard / GIcon / GStatusBadge / GButton
-->
<template>
  <GCard variant="interactive" padding="md" clickable @click="emit('click')">
    <div class="project-card">
      <div class="project-card__head">
        <span class="project-card__icon">
          <GIcon :name="icon" :size="18" />
        </span>
        <div class="project-card__title-wrap">
          <span class="project-card__name">{{ name }}</span>
          <GStatusBadge v-if="status" :status="status" size="sm" />
        </div>
        <GIconButton name="more-horizontal" size="sm" @click.stop="emit('action', 'menu')" />
      </div>

      <p v-if="description" class="project-card__desc">{{ description }}</p>

      <div class="project-card__meta">
        <span class="project-card__meta-item">
          <GIcon name="link" :size="12" />
          {{ tunnelCount }} 隧道
        </span>
        <span class="project-card__meta-item project-card__meta-item--online">
          <GStatusDot status="online" size="xs" />
          {{ onlineCount }} 在线
        </span>
        <span class="project-card__meta-item project-card__meta-item--time">
          <GIcon name="clock" :size="12" />
          {{ lastActive }}
        </span>
      </div>
    </div>
  </GCard>
</template>

<script setup lang="ts">
import GCard from "@components/base/GCard.vue"
import GIcon from "@components/icons/GIcon.vue"
import GIconButton from "@components/base/GIconButton.vue"
import GStatusBadge from "@components/status/GStatusBadge.vue"
import GStatusDot from "@components/status/GStatusDot.vue"

withDefaults(
  defineProps<{
    name: string
    icon?: string
    description?: string
    tunnelCount?: number
    onlineCount?: number
    lastActive?: string
    status?: "online" | "offline" | "connecting" | "error" | "warning" | "maintenance"
  }>(),
  {
    icon: "package",
    tunnelCount: 0,
    onlineCount: 0,
    lastActive: "",
  },
)

const emit = defineEmits<{
  click: []
  action: [key: string]
}>()
</script>

<style scoped>
.project-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}
.project-card__head {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.project-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
  flex-shrink: 0;
}
.project-card__title-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.project-card__name {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.project-card__desc {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  line-height: var(--leading-relaxed);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.project-card__meta {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  flex-wrap: wrap;
}
.project-card__meta-item {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
.project-card__meta-item--online { color: var(--color-success); }
</style>
