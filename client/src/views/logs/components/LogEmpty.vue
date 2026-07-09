<template>
  <div class="log-empty">
    <div class="log-empty__icon">
      <GIcon name="terminal" :size="44" />
    </div>
    <h2>{{ t('logs.emptyTitle') }}</h2>
    <p>{{ t('logs.emptyDescription') }}</p>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'

const { t } = useI18n()
</script>
