<template>
  <header class="app-header">
    <div class="header-left">
      <!-- Toggle Sidebar -->
      <button class="header-btn" title="切换侧边栏" @click="layout.toggleSidebar">
        <GIcon name="menu" :size="16" />
      </button>

      <!-- Breadcrumbs -->
      <div class="header-breadcrumbs" v-if="breadcrumbs.length">
        <template v-for="(crumb, i) in breadcrumbs" :key="i">
          <span v-if="i > 0" class="breadcrumb-sep">/</span>
          <span
            class="breadcrumb-item"
            :class="{ active: i === breadcrumbs.length - 1 }"
            @click="crumb.path && router.push(crumb.path)">
            {{ crumb.label }}
          </span>
        </template>
      </div>
    </div>

    <div class="header-center">
      <!-- Page Title (optional) -->
      <span class="page-title" v-if="pageTitle">{{ pageTitle }}</span>
    </div>

    <div class="header-right">
      <!-- Search Placeholder -->
      <button class="header-btn search-btn" @click="layout.openCommandPalette">
        <GIcon name="search" :size="16" />
        <span class="search-label">{{ t('common.search') }}</span>
        <span class="search-shortcut">Ctrl K</span>
      </button>

      <!-- Quick Actions -->
      <button
        class="header-btn"
        :title="t('common.toggleInspector')"
        @click="layout.toggleInspector">
        <GIcon name="panel-right-open" :size="16" />
      </button>

      <button class="header-btn" :title="t('common.toggleTheme')" @click="themeStore.toggleTheme">
        <GIcon :name="themeStore.isDark ? 'sun' : 'moon'" :size="16" />
      </button>

      <!-- Notification Placeholder -->
      <button class="header-btn" :title="t('common.notifications')">
        <GIcon name="bell" :size="16" />
        <span class="notification-dot"></span>
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useLayoutStore, useNavigationStore, useThemeStore } from '@stores'
import GIcon from '@components/icons/GIcon.vue'

const router = useRouter()
const route = useRoute()
const layout = useLayoutStore()
const navStore = useNavigationStore()
const themeStore = useThemeStore()
const { t } = useI18n()

const routeTitleMap: Record<string, string> = {
  dashboard: 'nav.dashboard',
  projects: 'nav.projects',
  'project-detail': 'nav.projects',
  tunnels: 'nav.tunnels',
  'tunnel-detail': 'nav.tunnels',
  servers: 'nav.servers',
  'server-detail': 'nav.servers',
  logs: 'nav.logs',
  settings: 'nav.settings',
  about: 'nav.about',
}

const routeSegmentMap: Record<string, string> = {
  projects: 'nav.projects',
  tunnels: 'nav.tunnels',
  servers: 'nav.servers',
  logs: 'nav.logs',
  certificates: 'nav.certificates',
  diagnostics: 'nav.diagnostics',
  feedback: 'nav.feedback',
  settings: 'nav.settings',
  about: 'nav.about',
}

const pageTitle = computed(() => {
  const routeName = typeof route.name === 'string' ? route.name : ''
  const titleKey = routeTitleMap[routeName]
  if (titleKey) return t(titleKey)
  return navStore.pageTitle
})

const breadcrumbs = computed(() => {
  const parts = route.path.split('/').filter(Boolean)
  if (parts.length === 0) return [{ label: t('nav.dashboard'), path: '/' }]
  return parts.map((p, i) => {
    const path = '/' + parts.slice(0, i + 1).join('/')
    const labelKey = routeSegmentMap[p]
    return { label: labelKey ? t(labelKey) : p, path }
  })
})
</script>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--toolbar-height);
  padding: 0 var(--space-3);
  background: var(--bg-toolbar);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  gap: var(--space-3);
}

.header-left,
.header-right {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.header-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.page-title {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-primary);
}

.header-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  height: 28px;
  padding: 0 var(--space-2);
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-micro) var(--ease-out);
  font-size: var(--text-sm);
  position: relative;
}

.header-btn:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.search-btn {
  width: 180px;
  justify-content: space-between;
  border: 1px solid var(--border-default);
  background: var(--bg-input);
}

.search-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.search-shortcut {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  background: var(--bg-surface-hover);
  padding: 1px 4px;
  border-radius: var(--radius-xs);
}

/* ── Breadcrumbs ── */
.header-breadcrumbs {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  margin-left: var(--space-3);
  font-size: var(--text-sm);
}

.breadcrumb-sep {
  color: var(--text-tertiary);
}

.breadcrumb-item {
  color: var(--text-muted);
  cursor: pointer;
  transition: color var(--duration-micro);
}

.breadcrumb-item:hover {
  color: var(--text-secondary);
}

.breadcrumb-item.active {
  color: var(--text-primary);
  font-weight: 500;
  cursor: default;
}

/* ── Notification Dot ── */
.notification-dot {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 6px;
  height: 6px;
  background: var(--color-error);
  border-radius: var(--radius-full);
  border: 2px solid var(--bg-toolbar);
}
</style>
