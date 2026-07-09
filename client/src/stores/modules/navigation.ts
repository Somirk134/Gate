import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { RouteLocationNormalized } from 'vue-router'

export interface BreadcrumbItem {
  label: string
  path?: string
  icon?: string
}

export interface NavItem {
  name: string
  path: string
  label: string
  icon: string
  meta?: Record<string, any>
}

export const useNavigationStore = defineStore('navigation', () => {
  // === State ===
  const breadcrumbs = ref<BreadcrumbItem[]>([])
  const currentRoute = ref<RouteLocationNormalized | null>(null)
  const pageTitle = ref('')
  const pageIcon = ref('')

  // === Getters ===
  const activeRouteName = computed(() => currentRoute.value?.name as string | undefined)
  const activeRoutePath = computed(() => currentRoute.value?.path ?? '')

  // === Actions ===
  function setBreadcrumbs(items: BreadcrumbItem[]) {
    breadcrumbs.value = items
  }

  function pushBreadcrumb(item: BreadcrumbItem) {
    breadcrumbs.value.push(item)
  }

  function setCurrentRoute(route: RouteLocationNormalized) {
    currentRoute.value = route
    pageTitle.value = (route.meta?.titleKey as string) || ''
    pageIcon.value = (route.meta?.icon as string) || ''
  }

  function setPageTitle(title: string) {
    pageTitle.value = title
  }

  function setPageIcon(icon: string) {
    pageIcon.value = icon
  }

  return {
    breadcrumbs,
    currentRoute,
    pageTitle,
    pageIcon,
    activeRouteName,
    activeRoutePath,
    setBreadcrumbs,
    pushBreadcrumb,
    setCurrentRoute,
    setPageTitle,
    setPageIcon,
  }
})
