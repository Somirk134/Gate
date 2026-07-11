<template>
  <header class="app-header">
    <div class="header-left">
      <!-- Toggle Sidebar -->
      <button
        class="header-btn"
        type="button"
        :title="t('common.toggleSidebar')"
        @click="layout.toggleSidebar">
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
      <div class="runtime-chip" :class="`is-${healthStatus}`">
        <span class="runtime-chip__dot" />
        <strong>{{ runtimeStatusLabel }}</strong>
        <span>{{ runtimeSummary }}</span>
      </div>

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
        :title="t('common.toggleTheme')"
        @click="themeStore.toggleTheme">
        <GIcon :name="themeStore.isDark ? 'sun' : 'moon'" :size="16" />
      </button>

      <div
        class="notification-wrap"
        v-click-outside="() => (notificationOpen = false)">
        <button
          class="header-btn"
          type="button"
          :aria-expanded="notificationOpen"
          :title="t('common.notifications')"
          @click.stop="toggleNotifications">
          <GIcon name="bell" :size="16" />
          <span v-if="notificationCount > 0" class="notification-badge">
            {{ notificationCount > 99 ? '99+' : notificationCount }}
          </span>
        </button>

        <div v-if="notificationOpen" class="notification-popover" @click.stop>
          <div class="notification-popover__header">
            <strong>{{ t('common.notifications') }}</strong>
            <button
              v-if="notificationCount > 0"
              class="notification-clear"
              type="button"
              @click="notificationStore.clearHistory">
              {{ t('common.clear') }}
            </button>
          </div>

          <div v-if="notificationItems.length" class="notification-list">
            <article
              v-for="item in notificationItems"
              :key="item.id"
              class="notification-item"
              :class="`is-${item.type}`"
              role="button"
              tabindex="0"
              :title="t('common.viewNotificationDetail')"
              @click="openNotificationDetail(item)"
              @keydown.enter.prevent="openNotificationDetail(item)">
              <span>{{ notificationTypeLabel(item.type) }}</span>
              <div class="notification-item__body">
                <strong>{{ item.title }}</strong>
                <p v-if="item.content">{{ item.content }}</p>
                <small>{{ formatNotificationTime(item.timestamp) }}</small>
              </div>
              <button
                v-if="item.closable"
                class="notification-dismiss"
                type="button"
                :title="t('common.closeNotification')"
                @click.stop="notificationStore.dismissHistory(item.id)">
                <GIcon name="close" :size="14" />
              </button>
            </article>
          </div>

          <div v-else class="notification-empty">
            <GIcon name="bell" :size="20" />
            <span>{{ t('common.noNotifications') }}</span>
          </div>
        </div>
      </div>

      <LangSwitch />
    </div>
  </header>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useLayoutStore, useNavigationStore, useNotificationStore, useThemeStore } from '@stores'
import GIcon from '@components/icons/GIcon.vue'
import LangSwitch from '@components/common/LangSwitch.vue'
import { useMonitoringDashboard } from '@/monitoring/composables/useMonitoringDashboard'
import type { NotificationItem } from '@/stores/modules/notification'

// vClickOutside 指令
const vClickOutside = {
  mounted(el: HTMLElement, binding: { value: (...args: unknown[]) => void }) {
    const handler = (event: Event) => {
      if (!(el as HTMLElement).contains(event.target as Node)) {
        binding.value(event)
      }
    }
    ;(el as any & { _clickOutsideHandler?: EventListener })._clickOutsideHandler = handler
    document.addEventListener('mousedown', handler)
  },
  unmounted(el: HTMLElement) {
    const h = (el as any & { _clickOutsideHandler?: EventListener })._clickOutsideHandler
    if (h) document.removeEventListener('mousedown', h)
  },
}

const router = useRouter()
const route = useRoute()
const layout = useLayoutStore()
const navStore = useNavigationStore()
const notificationStore = useNotificationStore()
const themeStore = useThemeStore()
const { t, locale } = useI18n()
const notificationOpen = ref(false)
const { dashboard, healthStatus } = useMonitoringDashboard()

const routeTitleMap: Record<string, string> = {
  dashboard: 'nav.dashboard',
  projects: 'nav.projects',
  'project-detail': 'nav.projectDetail',
  tunnels: 'nav.tunnels',
  'http-tunnels': 'nav.httpTunnels',
  'tunnel-detail': 'nav.tunnels',
  servers: 'nav.servers',
  'server-detail': 'nav.serverDetail',
  logs: 'nav.logs',
  certificates: 'nav.certificates',
  help: 'nav.help',
  settings: 'nav.settings',
  about: 'nav.about',
  'not-found': 'nav.notFound',
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
  return navStore.pageTitle ? t(navStore.pageTitle) : ''
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

const notificationItems = computed(() => notificationStore.recentHistory)
const notificationCount = computed(() => notificationStore.history.length)
const runtimeStatusLabel = computed(() => t(`dashboard.healthStatus.${healthStatus.value}`))
const runtimeSummary = computed(
  () =>
    `${dashboard.value.overview.runningTunnel}/${dashboard.value.overview.tunnelCount} Tunnel · ${formatSpeed(
      dashboard.value.statistics.traffic.uploadSpeedBps +
        dashboard.value.statistics.traffic.downloadSpeedBps,
    )}`,
)

function toggleNotifications() {
  notificationOpen.value = !notificationOpen.value
}

function openNotificationDetail(item: NotificationItem) {
  notificationStore.showDetail(item)
  notificationOpen.value = false
}

function notificationTypeLabel(type: string) {
  if (type === 'success') return t('common.notificationType.success')
  if (type === 'error') return t('common.notificationType.error')
  if (type === 'warning') return t('common.notificationType.warning')
  return t('common.notificationType.info')
}

function formatNotificationTime(timestamp: number) {
  return new Intl.DateTimeFormat(locale.value === 'en-US' ? 'en-US' : 'zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(timestamp)
}

function formatSpeed(bytesPerSecond: number): string {
  if (!Number.isFinite(bytesPerSecond) || bytesPerSecond <= 0) return '0 B/s'
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  const index = Math.min(units.length - 1, Math.floor(Math.log(bytesPerSecond) / Math.log(1024)))
  const value = bytesPerSecond / 1024 ** index
  return `${value.toFixed(value >= 10 || index === 0 ? 0 : 1)} ${units[index]}`
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

.runtime-chip {
  height: 28px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  max-width: 320px;
  padding: 0 var(--space-2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.runtime-chip__dot {
  width: 7px;
  height: 7px;
  border-radius: var(--radius-full);
  background: var(--status-online);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--status-online) 16%, transparent);
}

.runtime-chip.is-warning .runtime-chip__dot {
  background: var(--status-warning);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--status-warning) 16%, transparent);
}

.runtime-chip.is-critical .runtime-chip__dot,
.runtime-chip.is-offline .runtime-chip__dot {
  background: var(--status-error);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--status-error) 16%, transparent);
}

.runtime-chip strong,
.runtime-chip span:last-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.runtime-chip strong {
  color: var(--text-primary);
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

/* ── Notification Badge ── */
.notification-wrap {
  position: relative;
}

.notification-badge {
  position: absolute;
  top: -4px;
  right: -6px;
  min-width: 16px;
  height: 16px;
  padding: 0 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-error);
  color: #fff;
  font-size: 10px;
  font-weight: 600;
  line-height: 1;
  border-radius: var(--radius-full);
  border: 2px solid var(--bg-toolbar);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-error) 24%, transparent);
  pointer-events: none;
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
  cursor: pointer;
}

.notification-item:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--color-primary) 48%, transparent);
  outline-offset: 1px;
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
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
  overflow: hidden;
  margin-top: 2px;
  color: var(--text-secondary);
  font-size: var(--text-xs);
  line-height: var(--leading-normal);
}

.notification-item__body {
  min-width: 0;
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

@media (max-width: 1180px) {
  .runtime-chip {
    max-width: 180px;
  }

  .runtime-chip span:last-child {
    display: none;
  }
}

@media (max-width: 860px) {
  .runtime-chip,
  .header-center {
    display: none;
  }
}
</style>
