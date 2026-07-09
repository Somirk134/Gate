<template>
  <header class="app-header">
    <div class="header-left">
      <!-- Toggle Sidebar -->
      <button class="header-btn" type="button" title="切换侧边栏" @click="layout.toggleSidebar">
        <GIcon name="menu" :size="16" />
      </button>

      <!-- Breadcrumbs -->
      <div v-if="breadcrumbs.length" class="header-breadcrumbs">
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
      <span v-if="pageTitle" class="page-title">{{ pageTitle }}</span>
    </div>

    <div class="header-right">
      <!-- Search Placeholder -->
      <button
        class="header-btn search-btn"
        type="button"
        :title="t('common.search')"
        @click="layout.openCommandPalette">
        <GIcon name="search" :size="16" />
        <span class="search-label">{{ t('common.search') }}</span>
        <span class="search-shortcut">Ctrl K</span>
      </button>

      <!-- Quick Actions -->
      <button
        class="header-btn"
        type="button"
        :title="t('common.toggleInspector')"
        @click="layout.toggleInspector">
        <GIcon name="panel-right-open" :size="16" />
      </button>

      <button
        class="header-btn"
        type="button"
        :title="t('common.toggleTheme')"
        @click="themeStore.toggleTheme">
        <GIcon :name="themeStore.isDark ? 'sun' : 'moon'" :size="16" />
      </button>

      <div class="notification-wrap">
        <button
          class="header-btn"
          type="button"
          :aria-expanded="notificationOpen"
          :title="t('common.notifications')"
          @click="toggleNotifications">
          <GIcon name="bell" :size="16" />
          <span v-if="notificationCount > 0" class="notification-dot" />
        </button>

        <div v-if="notificationOpen" class="notification-popover">
          <div class="notification-popover__header">
            <strong>{{ t('common.notifications') }}</strong>
            <button
              v-if="notificationCount > 0"
              class="notification-clear"
              type="button"
              @click="notificationStore.clearAll">
              清空
            </button>
          </div>

          <div v-if="notificationItems.length" class="notification-list">
            <article
              v-for="item in notificationItems"
              :key="item.id"
              class="notification-item"
              :class="`is-${item.type}`">
              <span>{{ notificationTypeLabel(item.type) }}</span>
              <div>
                <strong>{{ item.title }}</strong>
                <p v-if="item.content">{{ item.content }}</p>
                <small>{{ formatNotificationTime(item.timestamp) }}</small>
              </div>
              <button
                v-if="item.closable"
                class="notification-dismiss"
                type="button"
                title="关闭通知"
                @click="notificationStore.dismiss(item.id)">
                <GIcon name="close" :size="14" />
              </button>
            </article>
          </div>

          <div v-else class="notification-empty">
            <GIcon name="bell" :size="20" />
            <span>暂无通知</span>
          </div>
        </div>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useLayoutStore, useNavigationStore, useNotificationStore, useThemeStore } from '@stores'
import GIcon from '@components/icons/GIcon.vue'

const router = useRouter()
const route = useRoute()
const layout = useLayoutStore()
const navStore = useNavigationStore()
const notificationStore = useNotificationStore()
const themeStore = useThemeStore()
const { t, locale } = useI18n()
const notificationOpen = ref(false)

const routeTitleMap: Record<string, string> = {
  dashboard: 'nav.dashboard',
  projects: 'nav.projects',
  'project-detail': 'nav.projects',
  tunnels: 'nav.tunnels',
  'tunnel-detail': 'nav.tunnels',
  servers: 'nav.servers',
  'server-detail': 'nav.servers',
  logs: 'nav.logs',
  help: 'nav.help',
  settings: 'nav.settings',
  about: 'nav.about',
}

const routeSegmentMap: Record<string, string> = {
  projects: 'nav.projects',
  tunnels: 'nav.tunnels',
  servers: 'nav.servers',
  logs: 'nav.logs',
  certificates: 'nav.certificates',
  help: 'nav.help',
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

const notificationItems = computed(() => notificationStore.notifications.slice(0, 6))
const notificationCount = computed(() => notificationStore.notifications.length)

function toggleNotifications() {
  notificationOpen.value = !notificationOpen.value
}

function notificationTypeLabel(type: string) {
  if (type === 'success') return '成功'
  if (type === 'error') return '错误'
  if (type === 'warning') return '警告'
  return '消息'
}

function formatNotificationTime(timestamp: number) {
  return new Intl.DateTimeFormat(locale.value === 'en' ? 'en-US' : 'zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(timestamp)
}
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

.header-right {
  position: relative;
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
.notification-wrap {
  position: relative;
}

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

.notification-popover {
  position: absolute;
  top: calc(100% + var(--space-2));
  right: 0;
  z-index: var(--z-popover);
  width: min(340px, calc(100vw - var(--space-6)));
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.notification-popover__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
}

.notification-popover__header strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.notification-clear,
.notification-dismiss {
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.notification-clear {
  font-size: var(--text-xs);
}

.notification-clear:hover,
.notification-dismiss:hover {
  color: var(--text-primary);
}

.notification-list {
  max-height: 320px;
  overflow-y: auto;
  padding: var(--space-2);
}

.notification-item {
  display: grid;
  grid-template-columns: 42px minmax(0, 1fr) 24px;
  align-items: start;
  gap: var(--space-2);
  padding: var(--space-2);
  border-radius: var(--radius-md);
}

.notification-item:hover {
  background: var(--bg-surface-hover);
}

.notification-item > span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.notification-item.is-success > span {
  color: var(--color-success);
}

.notification-item.is-error > span {
  color: var(--color-error);
}

.notification-item.is-warning > span {
  color: var(--color-warning);
}

.notification-item strong {
  display: block;
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-sm);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.notification-item p {
  margin-top: 2px;
  overflow-wrap: anywhere;
  color: var(--text-secondary);
  font-size: var(--text-xs);
  line-height: var(--leading-normal);
}

.notification-item small {
  display: block;
  margin-top: 2px;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.notification-dismiss {
  width: 24px;
  height: 24px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-sm);
}

.notification-empty {
  min-height: 112px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-2);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}
</style>
