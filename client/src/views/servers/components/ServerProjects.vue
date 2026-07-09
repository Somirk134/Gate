<!--
  ServerProjects — 工作区 Projects 标签（Project Overview）
  ------------------------------------------------------------------
  显示属于当前 Server 的 Project。点击进入。
-->
<template>
  <div class="server-projects">
    <div class="server-section__head">
      <div class="server-section__title">
        <GIcon name="package" :size="16" class="server-section__title-icon" />
        <span>{{ t('server.projects.title') }}</span>
        <GBadge variant="neutral" type="soft" size="sm">
          {{ projects.length }}
        </GBadge>
      </div>
    </div>

    <div v-if="projects.length" class="server-sublist">
      <div
        v-for="project in projects"
        :key="project.id"
        class="server-sublist__item"
        @click="$emit('view-project', project)">
        <span
          class="server-sublist__icon"
          :style="{ color: project.color, background: project.color + '1f' }">
          <GIcon name="package" :size="14" />
        </span>
        <div class="server-sublist__main">
          <span class="server-sublist__name">{{ project.name }}</span>
          <span class="server-sublist__meta"
            >{{ t('server.projects.projectMeta', { count: project.tunnelCount, remark: project.remark || '—' }) }}</span
          >
        </div>
        <div class="server-sublist__actions">
          <GIconButton
            name="external-link"
            size="sm"
            variant="ghost"
            :tooltip="t('server.projects.enterProject')"
            @click.stop="$emit('view-project', project)" />
        </div>
      </div>
    </div>

    <div v-else class="server-logs__empty">
      <GIcon name="package" :size="20" />
      <span>{{ t('server.projects.empty') }}</span>
    </div>

    <p class="server-connection__hint">
      <GIcon name="info-circle" :size="12" />
      {{ t('server.projects.hint') }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GBadge from '@components/base/GBadge.vue'
import GIconButton from '@components/base/GIconButton.vue'

export interface ServerProjectItem {
  id: string
  name: string
  tunnelCount: number
  remark: string
  color: string
}

defineProps<{
  projects: ServerProjectItem[]
}>()

defineEmits<{
  'view-project': [project: ServerProjectItem]
}>()

const { t } = useI18n()
</script>
