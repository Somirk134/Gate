<template>
  <GIconButton
    class="setting-reset"
    name="refresh"
    size="sm"
    :disabled="!modified"
    :tooltip="modified ? t('settings.legacy.resetSetting') : t('settings.legacy.currentDefault')"
    @click="emit('reset')" />
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIconButton from '@components/base/GIconButton.vue'

defineProps<{
  modified: boolean
}>()

const emit = defineEmits<{
  reset: []
}>()

const { t } = useI18n()
</script>
