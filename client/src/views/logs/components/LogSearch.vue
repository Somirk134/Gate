<template>
  <label class="log-search">
    <GIcon name="search" :size="14" />
    <input
      :value="modelValue"
      type="search"
      :placeholder="t('logs.search')"
      spellcheck="false"
      @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)" />
    <button
      v-if="modelValue"
      type="button"
      class="log-search__clear"
      :title="t('logs.clearSearch')"
      @click="$emit('update:modelValue', '')">
      <GIcon name="close" :size="13" />
    </button>
  </label>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'

defineProps<{ modelValue: string }>()
defineEmits<{ 'update:modelValue': [value: string] }>()

const { t } = useI18n()
</script>
