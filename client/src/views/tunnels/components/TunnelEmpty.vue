<!--
  TunnelEmpty — 空状态 / 欢迎页
  ------------------------------------------------------------------
  没有隧道时显示欢迎页面，引导创建第一个隧道。
-->
<template>
  <div class="tunnel-empty">
    <div class="tunnel-empty__illustration">
      <GIcon name="router" :size="48" />
    </div>
    <h2 class="tunnel-empty__title">{{ t('tunnel.emptyZeroTitle') }}</h2>
    <p class="tunnel-empty__desc">{{ t('tunnel.emptyZeroDesc') }}</p>
    <p class="tunnel-empty__hint">
      <GIcon name="info-circle" :size="12" />
      {{ t('tunnel.emptyZeroHint') }}
    </p>
    <div class="tunnel-empty__action">
      <GButton variant="primary" icon="plus" @click="$emit('create')">
        {{ t('tunnel.create') }}
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'

defineEmits<{ create: [] }>()
const { t } = useI18n()
</script>
