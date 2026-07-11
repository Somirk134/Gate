<template>
  <aside
    class="app-sidebar"
    data-sidebar="active-shell-sidebar"
    :class="{ collapsed: layout.sidebarCollapsed, hovered: layout.sidebarHovered }"
    @mouseenter="layout.hoverSidebar(true)"
    @mouseleave="layout.hoverSidebar(false)">
    <!-- Header: Logo -->
    <div class="sidebar-header">
      <div class="sidebar-logo" @click="layout.toggleSidebar">
        <div class="logo-icon">
          <img :src="appLogoUrl" alt="" />
        </div>
        <span v-show="!isCollapsed" class="logo-text">Gate</span>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="sidebar-nav">
      <div
        v-for="item in navItems"
        :key="item.path"
        class="nav-item"
        :class="{ active: isActive(item.path) }"
        :data-onboarding-target="item.tourTarget"
        :title="isCollapsed ? item.label : undefined"
        @click="navigate(item.path)">
        <div class="nav-icon">
          <GIcon :name="item.icon" :size="18" />
        </div>
        <span v-show="!isCollapsed" class="nav-label">{{ item.label }}</span>
        <span v-show="!isCollapsed && item.shortcut" class="nav-shortcut">{{ item.shortcut }}</span>
      </div>
    </nav>

    <!-- Footer -->
    <div class="sidebar-footer">
      <button
        class="onboarding-link"
        type="button"
        :title="t('common.reopenOnboarding')"
        @click="openOnboarding">
        <GIcon name="sparkles" :size="14" />
        <span v-show="!isCollapsed">{{ t('common.onboarding') }}</span>
      </button>
      <div v-show="!isCollapsed" class="version-info">
        <span class="version-text">v{{ version }}</span>
      </div>
      <a
        v-show="!isCollapsed"
        class="github-link"
        :href="GITHUB_REPOSITORY_URL"
        target="_blank"
        rel="noopener noreferrer">
        <GIcon name="external-link" :size="14" />
        <span>GitHub</span>
      </a>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useLayoutStore } from '@stores'
import GIcon from '@components/icons/GIcon.vue'
import { APP_VERSION, GITHUB_REPOSITORY_URL } from '@/constants'
import appLogoUrl from '@repo-assets/logo/logo-ui.png'

const router = useRouter()
const route = useRoute()
const layout = useLayoutStore()
const { t } = useI18n()

const isCollapsed = computed(() => layout.sidebarCollapsed && !layout.sidebarHovered)

const navItems = computed(() => [
  {
    path: '/',
    label: t('nav.dashboard'),
    icon: 'dashboard',
    shortcut: '',
    tourTarget: 'dashboard',
  },
  {
    path: '/projects',
    label: t('nav.projects'),
    icon: 'projects',
    shortcut: '',
    tourTarget: 'projects',
  },
  {
    path: '/tunnels',
    label: t('nav.tunnels'),
    icon: 'router',
    shortcut: '',
    tourTarget: 'tunnels',
  },
  {
    path: '/servers',
    label: t('nav.servers'),
    icon: 'servers',
    shortcut: '',
    tourTarget: 'servers',
  },
  {
    path: '/domains',
    label: t('nav.domains'),
    icon: 'globe',
    shortcut: '',
    tourTarget: 'domains',
  },
  {
    path: '/certificates',
    label: t('nav.certificates'),
    icon: 'shield-check',
    shortcut: '',
    tourTarget: 'certificates',
  },
  { path: '/logs', label: t('nav.logs'), icon: 'logs', shortcut: '', tourTarget: 'logs' },
  {
    path: '/help',
    label: t('nav.help'),
    icon: 'help',
    shortcut: '',
    tourTarget: 'help',
  },
  {
    path: '/settings',
    label: t('nav.settings'),
    icon: 'settings',
    shortcut: '',
    tourTarget: 'settings',
  },
  { path: '/about', label: t('nav.about'), icon: 'about', shortcut: '', tourTarget: 'about' },
])

const version = APP_VERSION

function isActive(path: string) {
  if (path === '/') return route.path === '/'
  return route.path.startsWith(path)
}

function navigate(path: string) {
  router.push(path)
}

function openOnboarding() {
  window.dispatchEvent(new CustomEvent('gate:onboarding:open', { detail: { restart: false } }))
}
</script>

<style scoped>
.app-sidebar {
  display: flex;
  flex-direction: column;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-subtle);
  user-select: none;
  overflow: hidden;
  transition: width var(--duration-standard) var(--ease-out);
}

.sidebar-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.sidebar-logo {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  cursor: pointer;
  padding: var(--space-1) 0;
}

.logo-icon {
  width: 28px;
  height: 28px;
  flex-shrink: 0;
  display: grid;
  place-items: center;
}

.logo-icon img {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: contain;
  filter: drop-shadow(0 4px 10px rgba(56, 189, 248, 0.18));
}

.logo-text {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: var(--tracking-tight);
  white-space: nowrap;
}

/* ── Navigation ── */
.sidebar-nav {
  flex: 1;
  padding: var(--space-2);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
  color: var(--text-secondary);
  position: relative;
}

.nav-item:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.nav-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 16px;
  background: var(--color-primary);
  border-radius: 0 var(--radius-full) var(--radius-full) 0;
}

.nav-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.nav-label {
  font-size: var(--text-sm);
  font-weight: 500;
  white-space: nowrap;
  flex: 1;
}

.nav-shortcut {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  white-space: nowrap;
}

/* ── Footer ── */
.sidebar-footer {
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  flex-shrink: 0;
}

.onboarding-link {
  width: 100%;
  min-height: 30px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 0 var(--space-2);
  cursor: pointer;
}

.onboarding-link:hover {
  border-color: var(--color-primary);
  color: var(--text-primary);
}

.version-info {
  display: flex;
  align-items: center;
}

.version-text {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
}

.github-link {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-decoration: none;
  transition: color var(--duration-fast);
}

.github-link:hover {
  color: var(--text-secondary);
}

/* ── Collapsed State ── */
.app-sidebar.collapsed {
  width: var(--sidebar-collapsed-width);
}

.app-sidebar.collapsed.hovered {
  position: fixed;
  top: 0;
  left: 0;
  bottom: 0;
  width: var(--sidebar-width);
  z-index: var(--z-fixed);
  box-shadow: 4px 0 24px rgba(0, 0, 0, 0.28);
}

.app-sidebar.collapsed:not(.hovered) .nav-item {
  justify-content: center;
  width: 36px;
  height: 36px;
  margin-inline: auto;
  padding: 0;
}

.app-sidebar.collapsed:not(.hovered) .nav-item.active::before {
  display: none;
}

.app-sidebar.collapsed:not(.hovered) .sidebar-logo {
  justify-content: center;
}

.app-sidebar.collapsed:not(.hovered) .sidebar-header {
  padding-inline: var(--space-2);
}

.app-sidebar.collapsed:not(.hovered) .sidebar-footer {
  padding-inline: var(--space-2);
  align-items: center;
}

.app-sidebar.collapsed:not(.hovered) .onboarding-link {
  width: 36px;
  height: 36px;
  margin-inline: auto;
  justify-content: center;
  padding: 0;
}
</style>
