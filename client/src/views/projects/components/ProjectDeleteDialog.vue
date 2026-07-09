<!--
  ProjectDeleteDialog — 删除确认对话框（危险操作）
  ------------------------------------------------------------------
  二次确认：必须输入项目名称才能删除。
-->
<template>
  <Transition name="dialog-fade">
    <div v-if="visible" class="project-delete__overlay">
      <Transition name="dialog-pop" appear>
        <div v-if="visible" class="project-delete" @click.stop>
          <header class="project-delete__header">
            <span class="project-delete__icon">
              <GIcon name="alert-triangle" :size="22" />
            </span>
            <h3 class="project-delete__title">{{ t('project.deleteDialog.title') }}</h3>
          </header>

          <div class="project-delete__body">
            <p class="project-delete__warning">
              {{ t('project.deleteDialog.warning') }}
              <strong class="project-delete__name">「{{ project?.name }}」</strong>
            </p>
            <ul class="project-delete__list">
              <li>
                {{ t('project.deleteDialog.tunnelRefs', { count: project?.tunnelCount ?? 0 }) }}
              </li>
              <li>
                {{ t('project.deleteDialog.domainRefs', { count: project?.domainCount ?? 0 }) }}
              </li>
              <li>
                {{
                  t('project.deleteDialog.certificateRefs', {
                    count: project?.certificateCount ?? 0,
                  })
                }}
              </li>
            </ul>

            <div class="project-delete__mode">
              <button
                type="button"
                :class="{ active: mode === 'projectOnly' }"
                @click="mode = 'projectOnly'">
                <strong>{{ t('project.deleteDialog.projectOnly') }}</strong>
                <span>{{ t('project.deleteDialog.projectOnlyDesc') }}</span>
              </button>
              <button
                type="button"
                :class="{ active: mode === 'cascadeResources' }"
                @click="mode = 'cascadeResources'">
                <strong>{{ t('project.deleteDialog.cascadeResources') }}</strong>
                <span>{{ t('project.deleteDialog.cascadeResourcesDesc') }}</span>
              </button>
            </div>

            <div class="project-delete__confirm-box">
              <p class="project-delete__confirm-text">
                {{ t('project.deleteDialog.confirmText', { name: project?.name ?? '' }) }}
              </p>
              <GInput
                v-model="confirmText"
                :placeholder="t('project.deleteDialog.placeholder')"
                :state="confirmText && confirmText !== project?.name ? 'error' : 'normal'"
                clearable />
            </div>
          </div>

          <footer class="project-delete__footer">
            <GButton variant="ghost" @click="handleClose"> {{ t('common.cancel') }} </GButton>
            <GButton
              variant="danger"
              icon="trash"
              :loading="deleting"
              :disabled="confirmText !== project?.name"
              @click="handleDelete">
              {{
                mode === 'projectOnly'
                  ? t('project.deleteDialog.title')
                  : t('project.deleteDialog.cascadeResources')
              }}
            </GButton>
          </footer>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GInput from '@components/form/GInput.vue'
import type { Project, ProjectDeleteMode } from '../types'

const props = defineProps<{
  visible: boolean
  project: Project | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  confirm: [project: Project, mode: ProjectDeleteMode]
}>()

const confirmText = ref('')
const deleting = ref(false)
const mode = ref<ProjectDeleteMode>('projectOnly')
const { t } = useI18n()

watch(
  () => props.visible,
  (v) => {
    if (v) {
      confirmText.value = ''
      deleting.value = false
      mode.value = 'projectOnly'
    }
  },
)

function handleClose() {
  emit('update:visible', false)
}

function handleDelete() {
  if (confirmText.value !== props.project?.name) return
  deleting.value = false
  if (props.project) emit('confirm', props.project, mode.value)
  emit('update:visible', false)
}
</script>

<style scoped>
.project-delete__overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.project-delete {
  width: 440px;
  max-width: calc(100vw - 48px);
  background: var(--bg-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-dialog);
  box-shadow: var(--shadow-floating);
  overflow: hidden;
}

.project-delete__header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-5) var(--space-5) var(--space-4);
  border-bottom: 1px solid var(--color-border-subtle);
}

.project-delete__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
  color: var(--color-error);
  flex-shrink: 0;
}

.project-delete__title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
}

.project-delete__body {
  padding: var(--space-4) var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.project-delete__warning {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.project-delete__name {
  color: var(--text-primary);
}

.project-delete__list {
  list-style: none;
  padding: var(--space-3);
  background: var(--color-error-muted);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.project-delete__list li {
  position: relative;
  padding-left: var(--space-4);
  font-size: var(--text-xs);
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.project-delete__list li::before {
  content: '•';
  position: absolute;
  left: var(--space-1);
  color: var(--color-error);
}

.project-delete__list li b {
  color: var(--color-error);
}

.project-delete__confirm-box {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.project-delete__mode {
  display: grid;
  gap: var(--space-2);
}

.project-delete__mode button {
  display: grid;
  gap: 2px;
  min-height: 58px;
  padding: var(--space-3);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  text-align: left;
  cursor: pointer;
}

.project-delete__mode button:hover,
.project-delete__mode button.active {
  border-color: var(--color-error);
  background: var(--color-error-muted);
}

.project-delete__mode strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.project-delete__mode span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: var(--leading-normal);
}

.project-delete__confirm-text {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.project-delete__confirm-text code {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  background: var(--bg-surface-hover);
  padding: 1px var(--space-1);
  border-radius: var(--radius-xs);
  color: var(--text-primary);
}

.project-delete__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-5) var(--space-5);
  border-top: 1px solid var(--color-border-subtle);
}

.dialog-fade-enter-active,
.dialog-fade-leave-active {
  transition: opacity var(--duration-base) var(--ease-out);
}
.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}

.dialog-pop-enter-active {
  transition:
    transform var(--duration-slow) var(--ease-spring),
    opacity var(--duration-base) var(--ease-out);
}
.dialog-pop-leave-active {
  transition:
    transform var(--duration-fast) var(--ease-in),
    opacity var(--duration-fast) var(--ease-in);
}
.dialog-pop-enter-from {
  transform: scale(0.94) translateY(-8px);
  opacity: 0;
}
.dialog-pop-leave-to {
  transform: scale(0.96);
  opacity: 0;
}
</style>
