<!--
  ProjectEmpty — 空状态 / 欢迎页
  ------------------------------------------------------------------
  没有项目时显示欢迎页面，引导创建第一个项目。
-->
<template>
  <div class="project-empty">
    <div class="project-empty__illustration">
      <GIcon name="projects" :size="48" />
    </div>
    <h2 class="project-empty__title">{{ t('project.empty') }}</h2>
    <p class="project-empty__desc">{{ t('project.emptyDesc') }}</p>
    <p class="project-empty__hint">
      <GIcon name="info-circle" :size="12" />
      {{ t('project.emptyHint') }}
    </p>
    <div class="project-empty__action">
      <GButton variant="primary" icon="plus" @click="$emit('create')">
        {{ t('project.createNow') }}
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'

const { t } = useI18n()

defineEmits<{ create: [] }>()
</script>

<style scoped>
.project-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-16) var(--space-6);
  text-align: center;
  animation: g-fade-in var(--duration-slow) var(--ease-out) both;
}

.project-empty__illustration {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 88px;
  height: 88px;
  margin-bottom: var(--space-5);
  border-radius: var(--radius-3xl);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.project-empty__title {
  font-size: var(--text-xl);
  font-weight: var(--weight-bold);
  color: var(--text-primary);
  margin-bottom: var(--space-2);
}

.project-empty__desc {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  max-width: 360px;
  line-height: var(--leading-relaxed);
  margin-bottom: var(--space-2);
}

.project-empty__hint {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-bottom: var(--space-5);
}

.project-empty__action {
  margin-top: var(--space-2);
}
</style>
