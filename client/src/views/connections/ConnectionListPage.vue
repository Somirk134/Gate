<template>
  <n-space vertical>
    <n-h1>{{ t('connections.title') }}</n-h1>
    <n-data-table :columns="columns" :data="connections" :loading="loading">
      <template #empty>
        <div class="connections-empty">
          <GIcon name="link" :size="28" />
          <strong>暂无活动连接</strong>
          <span>启动 Tunnel 并产生访问流量后，连接会显示在这里。</span>
          <GButton variant="primary" icon="plus" @click="router.push('/tunnels?create=1')">
            创建第一个 Tunnel
          </GButton>
        </div>
      </template>
    </n-data-table>
  </n-space>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'

const { t } = useI18n()
const router = useRouter()

const loading = ref(false)
const connections = ref([])

const columns = computed(() => [
  { title: t('common.details'), key: 'id' },
  { title: t('connections.client'), key: 'clientId' },
  { title: t('connections.remoteAddr'), key: 'remoteAddr' },
  { title: t('connections.connectedAt'), key: 'connectedAt' },
])
</script>

<style scoped>
.connections-empty {
  min-height: 220px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  color: var(--text-tertiary);
  text-align: center;
}

.connections-empty strong {
  color: var(--text-primary);
  font-size: var(--text-md);
}

.connections-empty span {
  max-width: 360px;
  line-height: var(--leading-relaxed);
}
</style>
