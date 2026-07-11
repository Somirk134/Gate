<template>
  <main class="app-content">
    <router-view v-slot="{ Component, route }">
      <transition name="page" mode="out-in" appear>
        <div :key="String(route.name ?? route.path)" class="app-route-view">
          <keep-alive v-if="route.meta.keepAlive" :include="cachedViews">
            <component :is="Component" />
          </keep-alive>
          <component v-else :is="Component" />
        </div>
      </transition>
    </router-view>

    <div v-if="!routeHasComponent" class="content-empty">
      <GEmptyState :title="t('shell.empty.title')" :description="t('shell.empty.description')" />
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import GEmptyState from '@components/feedback/GEmptyState.vue'

const route = useRoute()
const { t } = useI18n()

// keep-alive include 匹配的是组件 name，需与各页面 defineOptions 保持一致。
const cachedViews = [
  'dashboard',
  'projects',
  'tunnels',
  'http-tunnels',
  'servers',
  'settings',
]

const routeHasComponent = computed(() => {
  return route.matched.length > 0
})
</script>

<style scoped>
.app-content {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: var(--space-6);
  position: relative;
  box-sizing: border-box;
}

.app-route-view {
  min-height: 0;
}

.content-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

/* 页面过渡 */
.page-enter-active,
.page-leave-active {
  transition:
    opacity var(--duration-standard) var(--ease-out),
    transform var(--duration-standard) var(--ease-out);
}

.page-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
