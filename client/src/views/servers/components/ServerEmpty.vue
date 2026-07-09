<!--
  ServerEmpty — 空状态 / 欢迎页
  ------------------------------------------------------------------
  没有服务器时显示欢迎页面，引导添加第一个服务器。
-->
<template>
  <div class="server-empty">
    <div class="server-empty__illustration">
      <GIcon name="servers" :size="48" />
    </div>
    <h2 class="server-empty__title">{{ t('server.empty') }}</h2>
    <p class="server-empty__desc">{{ t('server.emptyDesc') }}</p>
    <p class="server-empty__hint">
      <GIcon name="info-circle" :size="12" />
      {{ t('server.emptyGuide') }}
    </p>
    <div class="server-empty__action">
      <GButton variant="primary" icon="plus" @click="$emit('create')">
        {{ t('server.addServer') }}
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'

const { t } = useI18n()

defineEmits<{ create: [] }>()
</script>
