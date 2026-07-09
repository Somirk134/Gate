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
      <dt>{{ t('settings.legacy.currentValue') }}</dt>
      <dd>{{ formatSettingValue(currentValue) }}</dd>
      <dt>{{ t('settings.legacy.defaultValue') }}</dt>
      <dd>{{ formatSettingValue(context.item.defaultValue) }}</dd>
      <dt>{{ t('settings.legacy.recommendedValue') }}</dt>
      <dd>{{ formatSettingValue(context.item.recommendedValue ?? context.item.defaultValue) }}</dd>
      <dt>{{ t('settings.legacy.restart') }}</dt>
      <dd>
        {{
          context.item.restartRequired
            ? t('settings.legacy.needed')
            : t('settings.legacy.notNeeded')
        }}
      </dd>
      <dt>{{ t('settings.legacy.status') }}</dt>
      <dd>{{ modified ? t('settings.legacy.modified') : t('settings.legacy.default') }}</dd>
    </dl>

    <div v-if="context.item.validation" class="setting-description__section">
      <h3>{{ t('settings.legacy.validationRules') }}</h3>
      <p>{{ validationText }}</p>
    </div>

    <div v-if="error" class="setting-description__section setting-description__section--error">
      <h3>{{ t('settings.legacy.validationError') }}</h3>
      <p>{{ error }}</p>
    </div>

    <div class="setting-description__links">
      <GButton size="sm" variant="secondary" icon="external-link" :disabled="!context.item.helpUrl">
        {{ t('settings.legacy.help') }}
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import type { SettingContext, SettingValue } from '../types'
import { formatSettingValue } from '../utils'

const props = defineProps<{
  context: SettingContext | null
  currentValue: SettingValue
  modified: boolean
  error?: string
}>()

const { t } = useI18n()

const validationText = computed(() => {
  const validation = props.context?.item.validation
  if (!validation) return t('settings.legacy.noValidationRules')

  const parts = [
    validation.required ? t('settings.legacy.required') : '',
    typeof validation.min === 'number' ? t('settings.legacy.min', { value: validation.min }) : '',
    typeof validation.max === 'number' ? t('settings.legacy.max', { value: validation.max }) : '',
    validation.pattern ? t('settings.legacy.pattern') : '',
  ].filter(Boolean)

  return parts.join(' / ') || t('settings.legacy.noValidationRules')
})
</script>
