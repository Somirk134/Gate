<template>
  <div v-if="context" class="setting-description">
    <div class="setting-description__heading">
      <span>{{ context.category.label }} / {{ context.group.label }}</span>
      <h2>{{ context.item.label }}</h2>
      <code>{{ context.item.key }}</code>
    </div>

    <p class="setting-description__text">
      {{ context.item.description }}
    </p>

    <dl class="setting-description__meta">
      <dt>当前值</dt>
      <dd>{{ formatSettingValue(currentValue) }}</dd>
      <dt>默认值</dt>
      <dd>{{ formatSettingValue(context.item.defaultValue) }}</dd>
      <dt>推荐值</dt>
      <dd>{{ formatSettingValue(context.item.recommendedValue ?? context.item.defaultValue) }}</dd>
      <dt>重启</dt>
      <dd>{{ context.item.restartRequired ? '需要' : '不需要' }}</dd>
      <dt>状态</dt>
      <dd>{{ modified ? '已修改' : '默认' }}</dd>
    </dl>

    <div v-if="context.item.validation" class="setting-description__section">
      <h3>校验规则</h3>
      <p>{{ validationText }}</p>
    </div>

    <div v-if="error" class="setting-description__section setting-description__section--error">
      <h3>校验错误</h3>
      <p>{{ error }}</p>
    </div>

    <div class="setting-description__links">
      <GButton size="sm" variant="secondary" icon="external-link" :disabled="!context.item.helpUrl">
        帮助
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GButton from '@components/base/GButton.vue'
import type { SettingContext, SettingValue } from '../types'
import { formatSettingValue } from '../utils'

const props = defineProps<{
  context: SettingContext | null
  currentValue: SettingValue
  modified: boolean
  error?: string
}>()

const validationText = computed(() => {
  const validation = props.context?.item.validation
  if (!validation) return '无校验规则。'

  const parts = [
    validation.required ? '必填' : '',
    typeof validation.min === 'number' ? `最小 ${validation.min}` : '',
    typeof validation.max === 'number' ? `最大 ${validation.max}` : '',
    validation.pattern ? '格式校验' : '',
  ].filter(Boolean)

  return parts.join(' / ') || '无校验规则。'
})
</script>
