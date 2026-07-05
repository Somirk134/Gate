<!--
  GInput — 统一文本输入框
  ------------------------------------------------------------------
  用途：项目内所有文本输入必须基于 GInput，统一边框/聚焦/尺寸/状态。
  专用输入（Password/Port/Host/Token/Search）均复用本组件视觉规范。

  Props:
    modelValue  v-model 值
    size        sm(28) | md(32) | lg(36)
    placeholder 占位符
    disabled / readonly
    state       normal | error | success（边框反馈）
    clearable   是否显示清除按钮
    prefix/suffix icon 名称（或用 slot 自定义）

  Slots:
    prefix  前置内容（图标/文字）
    suffix  后置内容

  Events: update:modelValue, enter, focus, blur, clear
-->
<template>
  <div
    class="g-input"
    :class="[
      `g-input--${size}`,
      `g-input--${state}`,
      { 'g-input--disabled': disabled, 'g-input--focused': focused, 'g-input--readonly': readonly },
    ]"
  >
    <span v-if="prefix" class="g-input__prefix-icon">
      <GIcon :name="prefix" :size="iconSize" />
    </span>
    <span v-else-if="$slots.prefix" class="g-input__prefix-slot"><slot name="prefix" /></span>

    <input
      class="g-input__field"
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      :maxlength="maxlength"
      :autocomplete="autocomplete"
      :spellcheck="spellcheck"
      @input="onInput"
      @keydown.enter="emit('enter', $event)"
      @focus="onFocus"
      @blur="onBlur"
    />

    <span v-if="clearable && modelValue && !disabled" class="g-input__clear" @click="clear">
      <GIcon name="close" :size="iconSize - 2" />
    </span>

    <span v-if="suffix" class="g-input__suffix-icon">
      <GIcon :name="suffix" :size="iconSize" />
    </span>
    <span v-else-if="$slots.suffix" class="g-input__suffix-slot"><slot name="suffix" /></span>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"

const props = withDefaults(
  defineProps<{
    modelValue?: string | number
    type?: string
    size?: "sm" | "md" | "lg"
    placeholder?: string
    disabled?: boolean
    readonly?: boolean
    state?: "normal" | "error" | "success"
    clearable?: boolean
    prefix?: string
    suffix?: string
    maxlength?: number
    autocomplete?: string
    spellcheck?: boolean
  }>(),
  {
    type: "text",
    size: "md",
    disabled: false,
    readonly: false,
    state: "normal",
    clearable: false,
    spellcheck: false,
  },
)

const emit = defineEmits<{
  "update:modelValue": [value: string]
  enter: [event: KeyboardEvent]
  focus: [event: FocusEvent]
  blur: [event: FocusEvent]
  clear: []
}>()

const focused = ref(false)
const iconSize = computed(() => (props.size === "sm" ? 14 : props.size === "lg" ? 18 : 16))

function onInput(e: Event) {
  emit("update:modelValue", (e.target as HTMLInputElement).value)
}
function onFocus(e: FocusEvent) {
  focused.value = true
  emit("focus", e)
}
function onBlur(e: FocusEvent) {
  focused.value = false
  emit("blur", e)
}
function clear() {
  emit("update:modelValue", "")
  emit("clear")
}
</script>

<style scoped>
.g-input {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  background: var(--bg-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-input);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--font-size-input);
  transition: border-color var(--duration-fast) var(--ease-out),
    box-shadow var(--duration-fast) var(--ease-out),
    background-color var(--duration-fast) var(--ease-out);
}

.g-input--sm { height: var(--control-height-sm); font-size: var(--text-sm); }
.g-input--md { height: var(--control-height-md); }
.g-input--lg { height: var(--control-height-lg); font-size: var(--text-md); }

.g-input__field {
  flex: 1;
  min-width: 0;
  height: 100%;
  background: transparent;
  border: none;
  outline: none;
  color: inherit;
  font-family: inherit;
  font-size: inherit;
  padding: 0 var(--space-3);
}
.g-input__field::placeholder {
  color: var(--text-tertiary);
}

/* prefix/suffix inside */
.g-input__prefix-icon,
.g-input__suffix-icon {
  display: inline-flex;
  align-items: center;
  color: var(--text-tertiary);
  flex-shrink: 0;
}
.g-input__prefix-icon { padding-left: var(--space-3); }
.g-input__suffix-icon { padding-right: var(--space-3); }

.g-input__prefix-slot { display: inline-flex; align-items: center; padding-left: var(--space-2); flex-shrink: 0; }
.g-input__suffix-slot { display: inline-flex; align-items: center; padding-right: var(--space-2); flex-shrink: 0; }

.g-input__clear {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--text-tertiary);
  padding: 2px;
  border-radius: var(--radius-full);
  margin-right: var(--space-2);
  transition: color var(--duration-fast) var(--ease-out), background-color var(--duration-fast) var(--ease-out);
}
.g-input__clear:hover { color: var(--text-primary); background: var(--bg-surface-hover); }

/* ── States ── */
.g-input--focused {
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}
.g-input--error {
  border-color: var(--color-error);
}
.g-input--error.g-input--focused {
  box-shadow: 0 0 0 3px var(--color-error-muted);
}
.g-input--success {
  border-color: var(--color-success);
}

/* ── Disabled / Readonly ── */
.g-input--disabled {
  opacity: 0.55;
  cursor: not-allowed;
  background: var(--bg-surface);
}
.g-input--disabled .g-input__field { cursor: not-allowed; }
.g-input--readonly .g-input__field { cursor: default; }
</style>
