<!--
  ProjectDeleteDialog — 删除确认对话框（危险操作）
  ------------------------------------------------------------------
  二次确认：必须输入项目名称才能删除。
-->
<template>
  <Transition name="dialog-fade">
    <div
      v-if="visible"
      class="project-delete__overlay"
      @click.self="handleClose"
    >
      <Transition
        name="dialog-pop"
        appear
      >
        <div
          v-if="visible"
          class="project-delete"
          @click.stop
        >
          <header class="project-delete__header">
            <span class="project-delete__icon">
              <GIcon
                name="alert-triangle"
                :size="22"
              />
            </span>
            <h3 class="project-delete__title">
              删除项目
            </h3>
          </header>

          <div class="project-delete__body">
            <p class="project-delete__warning">
              你即将删除项目
              <strong class="project-delete__name">「{{ project?.name }}」</strong>
            </p>
            <ul class="project-delete__list">
              <li>该项目下的 {{ project?.tunnelCount ?? 0 }} 个 Tunnel 将全部停止并移除</li>
              <li>所有流量统计与运行记录将被清除</li>
              <li>此操作<b>不可撤销</b></li>
            </ul>

            <div class="project-delete__confirm-box">
              <p class="project-delete__confirm-text">
                请输入项目名称 <code>{{ project?.name }}</code> 以确认：
              </p>
              <GInput
                v-model="confirmText"
                placeholder="输入项目名称"
                :state="confirmText && confirmText !== project?.name ? 'error' : 'normal'"
                clearable
              />
            </div>
          </div>

          <footer class="project-delete__footer">
            <GButton
              variant="ghost"
              @click="handleClose"
            >
              取消
            </GButton>
            <GButton
              variant="danger"
              icon="trash"
              :loading="deleting"
              :disabled="confirmText !== project?.name"
              @click="handleDelete"
            >
              确认删除
            </GButton>
          </footer>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"
import GInput from "@components/form/GInput.vue"
import type { Project } from "../types"

const props = defineProps<{
  visible: boolean
  project: Project | null
}>()

const emit = defineEmits<{
  "update:visible": [value: boolean]
  confirm: [project: Project]
}>()

const confirmText = ref("")
const deleting = ref(false)

watch(
  () => props.visible,
  (v) => {
    if (v) {
      confirmText.value = ""
      deleting.value = false
    }
  },
)

function handleClose() {
  emit("update:visible", false)
}

function handleDelete() {
  if (confirmText.value !== props.project?.name) return
  deleting.value = false
  if (props.project) emit("confirm", props.project)
  emit("update:visible", false)
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
  content: "•";
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
  transition: transform var(--duration-slow) var(--ease-spring),
    opacity var(--duration-base) var(--ease-out);
}
.dialog-pop-leave-active {
  transition: transform var(--duration-fast) var(--ease-in),
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
