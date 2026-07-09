<template>
  <article
    class="setting-item"
    :class="{
      'setting-item--selected': selected,
      'setting-item--modified': modified,
      'setting-item--disabled': disabled,
    }"
    tabindex="0"
    @click="emit('select')"
    @focusin="emit('select')"
    @mouseenter="emit('select')">
    <div class="setting-item__body">
      <div class="setting-item__title">
        <span>
          <template
            v-for="(part, index) in labelParts"
            :key="`${index}-${part.text}-${part.highlighted}`">
            <mark v-if="part.highlighted">{{ part.text }}</mark>
            <template v-else>{{ part.text }}</template>
          </template>
        </span>
        <span v-if="modified" class="setting-item__dot" :aria-label="t('settings.legacy.modified')" />
        <span v-if="item.restartRequired" class="setting-badge setting-badge--warning">
          {{ t('settings.legacy.restartRequired') }}
        </span>
        <span v-if="item.status === 'reserved'" class="setting-badge">
          {{ t('settings.legacy.reserved') }}
        </span>
      </div>

      <p class="setting-item__description">
        <template
          v-for="(part, index) in descriptionParts"
          :key="`${index}-${part.text}-${part.highlighted}`">
          <mark v-if="part.highlighted">{{ part.text }}</mark>
          <template v-else>
            {{ part.text }}
          </template>
        </template>
      </p>

      <p v-if="error" class="setting-item__error">
        {{ error }}
      </p>
    </div>

    <div class="setting-item__control" @click.stop>
      <SettingSwitch
        v-if="control.type === 'switch'"
        :model-value="booleanValue"
        :disabled="disabled"
        @update:model-value="emitValue" />

      <SettingSelect
        v-else-if="control.type === 'select'"
        :model-value="primitiveValue"
        :options="selectOptions"
        :disabled="disabled"
        @update:model-value="emitValue" />

      <SettingInput
        v-else-if="control.type === 'input'"
        :model-value="stringValue"
        :placeholder="inputPlaceholder"
        :input-type="inputType"
        :disabled="disabled"
        @update:model-value="emitValue" />

      <label v-else-if="control.type === 'number'" class="setting-number">
        <input
          class="setting-control setting-number__input"
          type="number"
          :value="numberValue"
          :min="numberMin"
          :max="numberMax"
          :step="numberStep"
          :disabled="disabled"
          @input="handleNumberInput" />
        <span v-if="controlUnit" class="setting-number__unit">{{ controlUnit }}</span>
      </label>

      <div v-else-if="control.type === 'slider'" class="setting-slider">
        <input
          type="range"
          :value="numberValue"
          :min="numberMin"
          :max="numberMax"
          :step="numberStep"
          :disabled="disabled"
          @input="handleNumberInput" />
        <span>{{ numberValue }}{{ controlUnit }}</span>
      </div>

      <label v-else-if="control.type === 'checkbox'" class="setting-checkbox">
        <input
          type="checkbox"
          :checked="booleanValue"
          :disabled="disabled"
          @change="handleCheckboxChange" />
        <span />
      </label>

      <div v-else-if="control.type === 'radio'" class="setting-radio">
        <button
          v-for="option in radioOptions"
          :key="String(option.value)"
          type="button"
          :class="{ active: primitiveValue === option.value }"
          :disabled="disabled || option.disabled"
          @click="emitValue(option.value)">
          {{ option.label }}
        </button>
      </div>

      <div v-else-if="control.type === 'color'" class="setting-color">
        <button
          v-for="swatch in colorSwatches"
          :key="swatch"
          type="button"
          :style="{ backgroundColor: swatch }"
          :class="{ active: value === swatch }"
          :disabled="disabled"
          @click="emitValue(swatch)" />
      </div>

      <div v-else-if="control.type === 'folder'" class="setting-folder">
        <span>{{ stringValue || folderPlaceholder }}</span>
        <GButton size="sm" icon="projects" :disabled="folderDisabled">
          {{ t('settings.legacy.browse') }}
        </GButton>
      </div>

      <div v-else-if="control.type === 'shortcut'" class="setting-shortcut">
        <kbd v-for="segment in shortcutSegments" :key="segment">{{ segment }}</kbd>
        <GButton size="sm" variant="ghost" icon="edit" disabled>
          {{ t('settings.legacy.edit') }}
        </GButton>
      </div>

      <div
        v-else-if="control.type === 'readonly'"
        class="setting-readonly"
        :class="`setting-readonly--${readonlyVariant}`">
        {{ valueLabel }}
      </div>

      <div v-else-if="control.type === 'action'" class="setting-actions">
        <GButton
          v-for="action in actions"
          :key="action.id"
          size="sm"
          :variant="action.variant ?? 'secondary'"
          :icon="action.icon"
          :disabled="action.disabled"
          :loading="actionStatuses[action.id] === 'running'"
          @click="emit('run-action', action.id)">
          {{ actionStatuses[action.id] === 'done' ? t('settings.legacy.done') : action.label }}
        </GButton>
      </div>
    </div>

    <SettingReset v-if="canReset" :modified="modified" @reset="emit('reset')" />
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import type {
  SettingAction,
  SettingActionStatus,
  SettingItem,
  SettingOption,
  SettingPrimitive,
  SettingValue,
} from '../types'
import { formatSettingValue, highlightText } from '../utils'
import SettingInput from './SettingInput.vue'
import SettingReset from './SettingReset.vue'
import SettingSelect from './SettingSelect.vue'
import SettingSwitch from './SettingSwitch.vue'

const props = withDefaults(
  defineProps<{
    item: SettingItem
    value: SettingValue
    error?: string
    selected?: boolean
    modified?: boolean
    highlightQuery?: string
    actionStatuses?: Record<string, SettingActionStatus>
  }>(),
  {
    error: undefined,
    selected: false,
    modified: false,
    highlightQuery: '',
    actionStatuses: () => ({}),
  },
)

const emit = defineEmits<{
  'update:value': [value: SettingValue]
  select: []
  reset: []
  'run-action': [actionId: string]
}>()

const { t } = useI18n()
const control = computed(() => props.item.control)
const disabled = computed(
  () =>
    ('disabled' in control.value && control.value.disabled === true) ||
    props.item.readonly === true,
)
const canReset = computed(
  () =>
    !props.item.readonly && control.value.type !== 'action' && control.value.type !== 'readonly',
)
const labelParts = computed(() => highlightText(props.item.label, props.highlightQuery))
const descriptionParts = computed(() => highlightText(props.item.description, props.highlightQuery))
const booleanValue = computed(() => props.value === true)
const stringValue = computed(() => (typeof props.value === 'string' ? props.value : ''))
const numberValue = computed(() =>
  typeof props.value === 'number' ? props.value : Number(props.value ?? 0),
)
const primitiveValue = computed<SettingPrimitive>(() => {
  if (
    typeof props.value === 'string' ||
    typeof props.value === 'number' ||
    typeof props.value === 'boolean'
  )
    return props.value
  return ''
})
const valueLabel = computed(() => formatSettingValue(props.value))
const selectOptions = computed<SettingOption[]>(() =>
  control.value.type === 'select' ? control.value.options : [],
)
const radioOptions = computed<SettingOption[]>(() =>
  control.value.type === 'radio' ? control.value.options : [],
)
const colorSwatches = computed(() =>
  control.value.type === 'color' ? (control.value.swatches ?? []) : [],
)
const actions = computed<SettingAction[]>(() =>
  control.value.type === 'action' ? control.value.actions : [],
)
const inputPlaceholder = computed(() =>
  control.value.type === 'input' ? (control.value.placeholder ?? '') : '',
)
const inputType = computed(() =>
  control.value.type === 'input' ? (control.value.inputType ?? 'text') : 'text',
)
const folderPlaceholder = computed(() =>
  control.value.type === 'folder'
    ? (control.value.placeholder ?? t('settings.legacy.folderPlaceholder'))
    : t('settings.legacy.folderPlaceholder'),
)
const folderDisabled = computed(() => disabled.value || control.value.type === 'folder')
const readonlyVariant = computed(() =>
  control.value.type === 'readonly' ? (control.value.variant ?? 'text') : 'text',
)
const controlUnit = computed(() => ('unit' in control.value ? (control.value.unit ?? '') : ''))
const numberMin = computed(() => ('min' in control.value ? control.value.min : undefined))
const numberMax = computed(() => ('max' in control.value ? control.value.max : undefined))
const numberStep = computed(() => ('step' in control.value ? (control.value.step ?? 1) : 1))
const shortcutSegments = computed(() => stringValue.value.split('+').filter(Boolean))

function emitValue(value: SettingValue) {
  emit('update:value', value)
}

function handleNumberInput(event: Event) {
  const nextValue = Number((event.target as HTMLInputElement).value)
  if (!Number.isNaN(nextValue)) emitValue(nextValue)
}

function handleCheckboxChange(event: Event) {
  emitValue((event.target as HTMLInputElement).checked)
}
</script>
