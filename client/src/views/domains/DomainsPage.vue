<template>
  <section class="domains-page">
    <header class="domains-hero">
      <div>
        <p class="domains-hero__eyebrow">{{ t('domains.eyebrow') }}</p>
        <h1>{{ t('domains.title') }}</h1>
        <span>{{ t('domains.subtitle') }}</span>
      </div>
      <div class="domains-hero__actions">
        <GButton variant="secondary" icon="refresh" :loading="store.loading" @click="refreshAll">
          {{ t('domains.refresh') }}
        </GButton>
        <GButton variant="secondary" icon="download" :disabled="!store.hasItems" @click="exportSelected">
          {{ t('domains.export') }}
        </GButton>
        <GButton variant="primary" icon="plus" :disabled="!store.isRuntimeAvailable" @click="wizardVisible = true">
          {{ t('domains.create') }}
        </GButton>
      </div>
    </header>

    <GCard v-if="!store.isRuntimeAvailable" variant="plain" padding="lg">
      <GErrorState :title="t('domains.emptyRuntimeTitle')" :message="t('domains.emptyRuntimeMessage')" />
    </GCard>

    <template v-else>
      <DomainStatsCards :stats="store.stats" :loading="store.loading" />

      <div class="domains-batch" v-if="selectedHosts.size">
        <span>{{ t('domains.batch.selected', { count: selectedHosts.size }) }}</span>
        <GButton variant="ghost" size="sm" @click="runBatch('checkDns')">{{ t('domains.batch.checkDns') }}</GButton>
        <GButton variant="ghost" size="sm" @click="runBatch('enable')">{{ t('domains.batch.enable') }}</GButton>
        <GButton variant="ghost" size="sm" @click="runBatch('disable')">{{ t('domains.batch.disable') }}</GButton>
        <GButton variant="ghost" size="sm" @click="batchRedeploy">{{ t('domains.batch.redeploy') }}</GButton>
        <GButton variant="ghost" size="sm" @click="batchRenew">{{ t('domains.batch.renew') }}</GButton>
        <GButton variant="ghost" size="sm" @click="runBatch('delete')">{{ t('domains.batch.delete') }}</GButton>
        <GButton variant="ghost" size="sm" @click="exportSelected">{{ t('domains.batch.export') }}</GButton>
      </div>

      <DomainDataTable
        :items="store.items"
        :total="store.total"
        :page="store.query.page || 1"
        :page-size="store.query.pageSize || 20"
        :keyword="keyword"
        :health="healthFilter"
        :protocol="protocolFilter"
        :sort-by="store.query.sortBy || 'host'"
        :sort-dir="store.query.sortDir || 'asc'"
        :selected-host="store.selectedHost"
        :selected-hosts="selectedHosts"
        @update:keyword="onKeyword"
        @update:health="onHealth"
        @update:protocol="onProtocol"
        @update:sort-by="onSortBy"
        @toggle-sort="toggleSort"
        @toggle-all="toggleAll"
        @toggle-host="toggleHost"
        @select="openDetail"
        @page="onPage" />

      <div v-if="!store.loading && !store.hasItems" class="domains-empty">
        <GIcon name="globe" :size="36" />
        <h2>{{ t('domains.emptyTitle') }}</h2>
        <p>{{ t('domains.emptyDescription') }}</p>
        <GButton variant="primary" icon="plus" @click="wizardVisible = true">{{ t('domains.create') }}</GButton>
      </div>
    </template>

    <DomainDetailDrawer
      :visible="drawerVisible"
      :loading="store.detailLoading"
      :detail="store.detail"
      @close="closeDrawer"
      @copy-url="copyUrl"
      @open-url="openUrl"
      @view-logs="openLogs"
      @check-dns="checkDns"
      @bind-tunnel="bindTunnel"
      @bind-certificate="bindCertificate"
      @delete="removeDomain"
      @open-certificates="router.push('/certificates')"
      @renew-now="renewCertificate"
      @redeploy="redeployCertificate"
      @dns-updated="reloadDetail" />

    <DomainCreateWizard
      v-model:visible="wizardVisible"
      :servers="servers"
      :tunnels="tunnels"
      :projects="projects"
      :existing-hosts="existingHosts"
      @submitted="refreshAll" />
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GCard from '@components/base/GCard.vue'
import GErrorState from '@components/feedback/GErrorState.vue'
import GIcon from '@components/icons/GIcon.vue'
import { certificateService } from '@views/certificates/service'
import { useProjectStore } from '@views/projects/store/project'
import { useServerStore } from '@views/servers'
import { useTunnelStore } from '@views/tunnels/store/tunnel'
import DomainStatsCards from './components/cards/DomainStatsCards.vue'
import DomainDetailDrawer from './components/drawers/DomainDetailDrawer.vue'
import DomainDataTable from './components/tables/DomainDataTable.vue'
import DomainCreateWizard from './components/wizard/DomainCreateWizard.vue'
import { useDomains } from './hooks/useDomains'
import { domainService } from './services/domain.service'
import { useFeedback } from '@composables/useFeedback'

const { t } = useI18n()
const { notify } = useFeedback()
const router = useRouter()
const route = useRoute()
const store = useDomains()
const projectStore = useProjectStore()
const serverStore = useServerStore()
const tunnelStore = useTunnelStore()

const wizardVisible = ref(false)
const drawerVisible = ref(false)
const selectedHosts = ref(new Set<string>())
const keyword = ref('')
const healthFilter = ref('all')
const protocolFilter = ref('all')

const servers = computed(() => serverStore.servers)
const tunnels = computed(() => tunnelStore.tunnels)
const projects = computed(() =>
  projectStore.projects.map((project) => ({ id: project.id, name: project.name })),
)
const existingHosts = computed(() => store.items.map((item) => item.host))

onMounted(async () => {
  await Promise.all([
    serverStore.load?.().catch(() => undefined),
    tunnelStore.load({ silent: true }).catch(() => undefined),
    projectStore.load?.().catch(() => undefined),
  ])
  const host = typeof route.query.host === 'string' ? route.query.host : null
  if (host) {
    await openDetail(host)
  }
})

watch([keyword, healthFilter, protocolFilter], async () => {
  store.setQuery({
    keyword: keyword.value,
    health: healthFilter.value === 'all' ? undefined : healthFilter.value,
    protocol: protocolFilter.value === 'all' ? undefined : protocolFilter.value,
    page: 1,
  })
  await store.loadList()
})

async function refreshAll() {
  await store.refresh()
  await store.loadList()
}

async function openDetail(host: string) {
  drawerVisible.value = true
  await store.loadDetail(host)
  await router.replace({ query: { ...route.query, host } })
}

function closeDrawer() {
  drawerVisible.value = false
  store.clearSelection()
  const query = { ...route.query }
  delete query.host
  void router.replace({ query })
}

async function reloadDetail() {
  if (!store.selectedHost) return
  await store.loadDetail(store.selectedHost)
}

function onKeyword(value: string) {
  keyword.value = value
}

function onHealth(value: string) {
  healthFilter.value = value
}

function onProtocol(value: string) {
  protocolFilter.value = value
}

function onSortBy(value: string) {
  store.setQuery({ sortBy: value })
  void store.loadList()
}

function toggleSort() {
  store.setQuery({ sortDir: store.query.sortDir === 'asc' ? 'desc' : 'asc' })
  void store.loadList()
}

function onPage(page: number) {
  store.setQuery({ page })
  void store.loadList()
}

function toggleAll(checked: boolean) {
  selectedHosts.value = checked ? new Set(store.items.map((item) => item.host)) : new Set()
}

function toggleHost(host: string, checked: boolean) {
  const next = new Set(selectedHosts.value)
  if (checked) next.add(host)
  else next.delete(host)
  selectedHosts.value = next
}

async function batchRedeploy() {
  for (const host of selectedHosts.value) {
    await certificateService.redeploy(host).catch(() => undefined)
  }
  await refreshAll()
}

async function batchRenew() {
  for (const host of selectedHosts.value) {
    await certificateService.renewNow(host).catch(() => undefined)
  }
  await refreshAll()
}

async function checkDns(host: string) {
  await domainService.checkDns(host)
  await refreshAll()
  if (store.selectedHost === host) {
    await reloadDetail()
  }
}

function bindTunnel(host: string) {
  void router.push({ path: '/tunnels', query: { bindHost: host } })
}

function bindCertificate(host: string) {
  void router.push({ path: '/certificates', query: { domain: host } })
}

async function redeployCertificate() {
  if (!store.selectedHost) return
  await certificateService.redeploy(store.selectedHost)
  await reloadDetail()
}

async function runBatch(action: string) {
  await domainService.batch({ hosts: [...selectedHosts.value], action })
  selectedHosts.value = new Set()
  await refreshAll()
}

async function removeDomain(host: string) {
  await domainService.delete(host)
  if (store.selectedHost === host) {
    closeDrawer()
  }
  await refreshAll()
}

function copyUrl(url: string) {
  void navigator.clipboard.writeText(url)
  notify.success(t('common.copiedWithValue', { value: url }))
}

function openUrl(url: string) {
  window.open(url, '_blank', 'noopener,noreferrer')
}

function openLogs(host: string) {
  void router.push({ path: '/logs', query: { host } })
}

async function exportSelected() {
  const hosts = selectedHosts.value.size ? [...selectedHosts.value] : store.items.map((item) => item.host)
  const records = store.items.filter((item) => hosts.includes(item.host))
  const blob = new Blob([JSON.stringify(records, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = 'gate-domains.json'
  anchor.click()
  URL.revokeObjectURL(url)
}

async function renewCertificate() {
  if (!store.selectedHost) return
  await certificateService.renewNow(store.selectedHost)
  await reloadDetail()
}
</script>

<style scoped>
.domains-page {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  min-height: 0;
}

.domains-hero {
  display: flex;
  justify-content: space-between;
  gap: var(--space-4);
  align-items: flex-start;
}

.domains-hero__eyebrow {
  margin: 0;
  color: var(--color-primary);
  font-size: var(--text-xs);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.domains-hero h1 {
  margin: 4px 0;
}

.domains-hero span {
  color: var(--text-secondary);
}

.domains-hero__actions {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.domains-batch {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
  background: var(--bg-surface-muted);
}

.domains-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-8);
  text-align: center;
  color: var(--text-secondary);
}
</style>
