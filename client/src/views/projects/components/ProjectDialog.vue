<!--
  ProjectDialog — 创建/编辑项目对话框
  ------------------------------------------------------------------
  统一处理创建与编辑两种模式。全部字段实时校验。
  字段：名称 / 图标 / 颜色 / 描述 / 默认服务器 / 自动启动 / 标签 / 备注
-->
<template>
  <Transition name="dialog-fade">
    <div
      v-if="visible"
      class="project-dialog__overlay"
      @click.self="handleClose"
    >
      <Transition
        name="dialog-pop"
        appear
      >
        <div
          v-if="visible"
          class="project-dialog"
          @click.stop
        >
          <!-- 头部 -->
          <header class="project-dialog__header">
            <div class="project-dialog__title-wrap">
              <span
                class="project-dialog__icon"
                :style="previewStyle"
              >
                <GIcon
                  :name="form.icon"
                  :size="20"
                />
              </span>
              <div>
                <h3 class="project-dialog__title">
                  {{ isEdit ? "编辑项目" : "创建项目" }}
                </h3>
                <p class="project-dialog__subtitle">
                  {{ isEdit ? "修改项目信息，自动保存" : "配置项目基础信息" }}
                </p>
              </div>
            </div>
            <GIconButton
              name="close"
              variant="ghost"
              size="sm"
              @click="handleClose"
            />
          </header>

          <!-- 主体表单 -->
          <div class="project-dialog__body">
            <!-- 名称 -->
            <GFormField
              :error="errors.name"
              required
            >
              <template #label>
                项目名称
              </template>
              <GInput
                v-model="form.name"
                placeholder="例如：My API Service"
                :state="errors.name ? 'error' : 'normal'"
                :maxlength="40"
                clearable
                @update:model-value="validateField('name')"
              />
            </GFormField>

            <!-- 颜色 -->
            <GFormField>
              <template #label>
                项目颜色
              </template>
              <ProjectColorPicker v-model="form.color" />
            </GFormField>

            <!-- 图标 -->
            <GFormField>
              <template #label>
                项目图标
              </template>
              <ProjectIconPicker v-model="form.icon" />
            </GFormField>

            <!-- 描述 -->
            <GFormField :error="errors.description">
              <template #label>
                描述
              </template>
              <GTextarea
                v-model="form.description"
                placeholder="简要描述项目用途…"
                :rows="2"
                :maxlength="120"
                resizable
                @update:model-value="validateField('description')"
              />
            </GFormField>

            <!-- 默认服务器 -->
            <GFormField>
              <template #label>
                默认服务器
              </template>
              <div class="project-dialog__select-wrap">
                <select
                  v-model="form.serverName"
                  class="project-dialog__select"
                >
                  <option
                    v-for="s in serverNames"
                    :key="s"
                    :value="s"
                  >
                    {{ s }}
                  </option>
                </select>
                <GIcon
                  name="chevron-down"
                  :size="14"
                  class="project-dialog__select-chevron"
                />
              </div>
            </GFormField>

            <!-- 自动启动 -->
            <div class="project-dialog__row">
              <div class="project-dialog__row-text">
                <span class="project-dialog__row-label">自动启动</span>
                <span class="project-dialog__row-hint">应用启动时自动运行该项目所有 Tunnel</span>
              </div>
              <button
                type="button"
                class="project-toggle"
                :class="{ 'project-toggle--on': form.autoStart }"
                @click="form.autoStart = !form.autoStart"
              >
                <span class="project-toggle__thumb" />
              </button>
            </div>

            <!-- 标签 -->
            <GFormField>
              <template #label>
                标签
              </template>
              <div
                class="project-tag-input"
                :class="{ 'project-tag-input--focused': tagFocused }"
              >
                <ProjectTag
                  v-for="tag in form.tags"
                  :key="tag"
                  :name="tag"
                  removable
                  @remove="removeTag"
                />
                <input
                  v-model="tagInput"
                  class="project-tag-input__field"
                  placeholder="输入标签后回车"
                  @focus="tagFocused = true"
                  @blur="onTagBlur"
                  @keydown.enter.prevent="addTag"
                  @keydown.backspace="onBackspace"
                >
              </div>
              <div class="project-tag-suggest">
                <button
                  v-for="tag in suggestedTags"
                  :key="tag.name"
                  type="button"
                  class="project-tag-suggest__chip"
                  :style="{ color: tag.color }"
                  @click="addSuggestedTag(tag.name)"
                >
                  <GIcon
                    name="plus"
                    :size="10"
                  />
                  {{ tag.name }}
                </button>
              </div>
            </GFormField>

            <!-- 备注 -->
            <GFormField>
              <template #label>
                备注
              </template>
              <GTextarea
                v-model="form.remark"
                placeholder="内部备注，仅自己可见…"
                :rows="2"
                :maxlength="200"
                resizable
              />
            </GFormField>
          </div>

          <!-- 底部 -->
          <footer class="project-dialog__footer">
            <GButton
              variant="ghost"
              @click="handleClose"
            >
              取消
            </GButton>
            <GButton
              variant="primary"
              :icon="isEdit ? 'save' : 'plus'"
              :loading="submitting"
              :disabled="!isValid"
              @click="handleSubmit"
            >
              {{ isEdit ? "保存" : "创建项目" }}
            </GButton>
          </footer>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"
import GIconButton from "@components/base/GIconButton.vue"
import GInput from "@components/form/GInput.vue"
import GTextarea from "@components/form/GTextarea.vue"
import GFormField from "@components/form/GFormField.vue"
import ProjectColorPicker from "./ProjectColorPicker.vue"
import ProjectIconPicker from "./ProjectIconPicker.vue"
import ProjectTag from "./ProjectTag.vue"
import type { Project, ProjectColor, ProjectFormData } from "../types"
import { PROJECT_TAGS, projectColorVars } from "../utils"

const props = defineProps<{
  visible: boolean
  /** 传入项目则为编辑模式 */
  project?: Project | null
  serverNames: string[]
}>()

const emit = defineEmits<{
  "update:visible": [value: boolean]
  submit: [form: ProjectFormData, isEdit: boolean]
}>()

const isEdit = computed(() => !!props.project)
const submitting = ref(false)
const tagFocused = ref(false)
const tagInput = ref("")

const form = reactive<ProjectFormData>({
  name: "",
  icon: "package",
  color: "blue",
  description: "",
  serverName: props.serverNames[0] ?? "",
  autoStart: false,
  tags: [],
  remark: "",
})

const errors = reactive<{ name?: string; description?: string }>({})

// 预览样式（头部图标）
const previewStyle = computed(() => {
  const vars = projectColorVars(form.color as ProjectColor)
  return {
    background: vars["--project-color-muted"],
    color: vars["--project-color"],
  }
})

const suggestedTags = computed(() =>
  PROJECT_TAGS.filter((t) => !form.tags.includes(t.name)).slice(0, 6),
)

const isValid = computed(() => form.name.trim().length >= 2 && !errors.name)

// 初始化 / 重置表单
watch(
  () => props.visible,
  (v) => {
    if (v) {
      if (props.project) {
        form.name = props.project.name
        form.icon = props.project.icon
        form.color = props.project.color
        form.description = props.project.description
        form.serverName = props.project.serverName
        form.autoStart = props.project.autoStart
        form.tags = [...props.project.tags]
        form.remark = props.project.remark ?? ""
      } else {
        resetForm()
      }
      errors.name = undefined
      errors.description = undefined
    }
  },
  { immediate: true },
)

function resetForm() {
  form.name = ""
  form.icon = "package"
  form.color = "blue"
  form.description = ""
  form.serverName = props.serverNames[0] ?? ""
  form.autoStart = false
  form.tags = []
  form.remark = ""
  tagInput.value = ""
}

function validateField(field: "name" | "description") {
  if (field === "name") {
    const v = form.name.trim()
    if (v.length === 0) errors.name = "项目名称不能为空"
    else if (v.length < 2) errors.name = "名称至少 2 个字符"
    else if (v.length > 40) errors.name = "名称不能超过 40 个字符"
    else errors.name = undefined
  }
  if (field === "description") {
    if (form.description.length > 120) errors.description = "描述不能超过 120 个字符"
    else errors.description = undefined
  }
}

function addTag() {
  const v = tagInput.value.trim()
  if (v && !form.tags.includes(v)) {
    form.tags.push(v)
  }
  tagInput.value = ""
}

function addSuggestedTag(name: string) {
  if (!form.tags.includes(name)) form.tags.push(name)
}

function removeTag(name: string) {
  const idx = form.tags.indexOf(name)
  if (idx !== -1) form.tags.splice(idx, 1)
}

function onBackspace() {
  if (tagInput.value === "" && form.tags.length > 0) {
    form.tags.pop()
  }
}

function onTagBlur() {
  tagFocused.value = false
  if (tagInput.value.trim()) addTag()
}

function handleClose() {
  emit("update:visible", false)
}

function handleSubmit() {
  validateField("name")
  if (!isValid.value) return
  submitting.value = false
  emit("submit", { ...form, tags: [...form.tags] }, isEdit.value)
  emit("update:visible", false)
}
</script>

<style scoped>
.project-dialog__overlay {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.project-dialog {
  width: 520px;
  max-width: calc(100vw - 48px);
  max-height: calc(100vh - 64px);
  display: flex;
  flex-direction: column;
  background: var(--bg-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-dialog);
  box-shadow: var(--shadow-floating);
  overflow: hidden;
}

/* ── 头部 ── */
.project-dialog__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: var(--space-5) var(--space-5) var(--space-4);
  border-bottom: 1px solid var(--color-border-subtle);
}

.project-dialog__title-wrap {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.project-dialog__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.project-dialog__title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
}

.project-dialog__subtitle {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-top: 2px;
}

/* ── 主体 ── */
.project-dialog__body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-4) var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

/* ── 行（label + toggle） ── */
.project-dialog__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.project-dialog__row-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.project-dialog__row-label {
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--text-secondary);
}

.project-dialog__row-hint {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

/* ── 下拉选择 ── */
.project-dialog__select-wrap {
  position: relative;
  display: inline-flex;
  align-items: center;
  width: 100%;
}

.project-dialog__select {
  appearance: none;
  width: 100%;
  height: var(--control-height-md);
  padding: 0 var(--space-6) 0 var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-input);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--font-size-input);
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-out);
}

.project-dialog__select:hover {
  border-color: var(--color-border-strong);
}

.project-dialog__select:focus {
  border-color: var(--color-border-focus);
  outline: none;
  box-shadow: var(--shadow-focus);
}

.project-dialog__select-chevron {
  position: absolute;
  right: var(--space-3);
  color: var(--text-tertiary);
  pointer-events: none;
}

/* ── 底部 ── */
.project-dialog__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-4) var(--space-5) var(--space-5);
  border-top: 1px solid var(--color-border-subtle);
}

/* ── 过渡 ── */
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
