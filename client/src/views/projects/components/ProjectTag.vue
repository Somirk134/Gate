<!--
  ProjectTag — 项目标签 chip
  ------------------------------------------------------------------
  展示单个标签，支持点击移除（编辑态）与展示态（纯展示）。
-->
<template>
  <span class="project-tag" :class="{ 'project-tag--removable': removable }" :style="tagStyle">
    <span class="project-tag__label">{{ displayName }}</span>
    <button
      v-if="removable"
      class="project-tag__remove"
      type="button"
      @click.stop="$emit('remove', name)">
      <GIcon name="close" :size="10" />
    </button>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'

const props = withDefaults(
  defineProps<{
    name: string
    color?: string
    removable?: boolean
  }>(),
  {
    color: '',
    removable: false,
  },
)

defineEmits<{ remove: [name: string] }>()

const { t, te } = useI18n()

const displayName = computed(() => {
  const key = `project.tags.${props.name}`
  return te(key) ? t(key) : props.name
})

const tagStyle = computed(() => {
  if (!props.color) return {}
  return {
    background: `${props.color}1f`,
    color: props.color,
    borderColor: `${props.color}40`,
  }
})
</script>

<style scoped>
.project-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 20px;
  padding: 0 var(--space-2);
  background: var(--bg-surface-hover);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-tag);
  font-family: var(--font-ui);
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
  color: var(--text-secondary);
  white-space: nowrap;
  vertical-align: middle;
}

.project-tag--removable {
  cursor: default;
}

.project-tag__remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  border: none;
  background: transparent;
  color: inherit;
  opacity: 0.6;
  border-radius: var(--radius-full);
  cursor: pointer;
  transition:
    opacity var(--duration-fast) var(--ease-out),
    background-color var(--duration-fast) var(--ease-out);
}

.project-tag__remove:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.12);
}
</style>
