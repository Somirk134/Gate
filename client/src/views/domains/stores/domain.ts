import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { domainService } from '../services/domain.service'
import type {
  DomainDetailResponse,
  DomainListQuery,
  DomainStats,
  DomainTopologyResponse,
  ManagedDomainRecord,
} from '../types'

export const useDomainStore = defineStore('domain-center', () => {
  const items = ref<ManagedDomainRecord[]>([])
  const total = ref(0)
  const stats = ref<DomainStats | null>(null)
  const topology = ref<DomainTopologyResponse | null>(null)
  const selectedHost = ref<string | null>(null)
  const detail = ref<DomainDetailResponse | null>(null)
  const loading = ref(false)
  const detailLoading = ref(false)
  const error = ref('')
  const query = ref<DomainListQuery>({
    page: 1,
    pageSize: 20,
    sortBy: 'host',
    sortDir: 'asc',
  })

  const isRuntimeAvailable = computed(() => isTauri())
  const hasItems = computed(() => items.value.length > 0)
  const selected = computed(() =>
    items.value.find((item) => item.host === selectedHost.value) ?? null,
  )

  async function loadList(options: { silent?: boolean } = {}) {
    if (!isRuntimeAvailable.value) {
      items.value = []
      total.value = 0
      return
    }
    if (!options.silent) {
      loading.value = true
      error.value = ''
    }
    try {
      const response = await domainService.list(query.value)
      items.value = response.items
      total.value = response.total
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      throw err
    } finally {
      loading.value = false
    }
  }

  async function loadStats(options: { silent?: boolean } = {}) {
    if (!isRuntimeAvailable.value) {
      stats.value = null
      return
    }
    try {
      stats.value = await domainService.stats()
    } catch (err) {
      if (!options.silent) {
        error.value = err instanceof Error ? err.message : String(err)
      }
      throw err
    }
  }

  async function loadTopology() {
    if (!isRuntimeAvailable.value) {
      topology.value = null
      return
    }
    topology.value = await domainService.topology()
  }

  async function loadDetail(host: string) {
    if (!isRuntimeAvailable.value) return
    detailLoading.value = true
    try {
      detail.value = await domainService.detail(host)
      selectedHost.value = host
    } finally {
      detailLoading.value = false
    }
  }

  async function refresh() {
    await Promise.all([loadList({ silent: true }), loadStats({ silent: true })])
  }

  function setQuery(patch: Partial<DomainListQuery>) {
    query.value = { ...query.value, ...patch }
  }

  function clearSelection() {
    selectedHost.value = null
    detail.value = null
  }

  return {
    items,
    total,
    stats,
    topology,
    selectedHost,
    selected,
    detail,
    loading,
    detailLoading,
    error,
    query,
    isRuntimeAvailable,
    hasItems,
    loadList,
    loadStats,
    loadTopology,
    loadDetail,
    refresh,
    setQuery,
    clearSelection,
  }
})
