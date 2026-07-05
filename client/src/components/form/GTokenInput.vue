<!--
  GTokenInput — Token / 密钥输入框
  ------------------------------------------------------------------
  用途：认证 Token、API Key、密钥输入。默认掩码，可显隐，可一键复制，
  等宽字体便于核对。前置 key 图标。
-->
<template>
  <GInput
    :model-value="modelValue"
    :type="visible ? 'text' : 'password'"
    :size="size"
    :placeholder="placeholder ?? '粘贴或输入 Token'"
    :state="state"
    :disabled="disabled"
    prefix="key"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <template #suffix>
      <button
        type="button"
        class="g-token__action"
        :title="visible ? '隐藏' : '显示'"
        @click="visible = !visible"
      >
        <GIcon :name="visible ? 'eye-off' : 'eye'" :size="14" />
      </button>
      <button
        type="button"
        class="g-token__action"
        :title="copied ? '已复制' : '复制'"
        @click="copy"
      >
        <GIcon :name="copied ? 'check' : 'copy'" :size="14" />
      </button>
    </template>
  </GInput>
</template>

<script setup lang="ts">
import { ref } from "vue"
import GInput from "./GInput.vue"
import GIcon from "@components/icons/GIcon.vue"

const props = withDefaults(
  defineProps<{
    modelValue?: string
    size?: "sm" | "md" | "lg"
    placeholder?: string
    disabled?: boolean
    state?: "normal" | "error" | "success"
  }>(),
  {
    size: "md",
    disabled: false,
    state: "normal",
  },
)

const emit = defineEmits<{
  "update:modelValue": [value: string]
  copy: [value: string]
}>()

const visible = ref(false)
const copied = ref(false)

async function copy() {
  if (!props.modelValue) return
  try {
    await navigator.clipboard.writeText(props.modelValue)
  } catch {
    /* 业务层可自行处理 */
  }
  emit("copy", props.modelValue)
  copied.value = true
  setTimeout(() => (copied.value = false), 1500)
}
</script>

<style scoped>
:deep(.g-input__field) {
  font-family: var(--font-mono);
  letter-spacing: 0.04em;
}
.g-token__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 2px;
  border-radius: var(--radius-sm);
  margin-right: var(--space-1);
  transition: color var(--duration-fast) var(--ease-out), background-color var(--duration-fast) var(--ease-out);
}
.g-token__action:hover {
  color: var(--text-primary);
  background: var(--bg-surface-hover);
}
</style>
