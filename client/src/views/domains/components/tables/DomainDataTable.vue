<template>
  <section class="domain-table-shell">
    <header class="domain-table-toolbar">
      <GInput
        :model-value="keyword"
        prefix="search"
        :placeholder="t('domains.table.search')"
        @update:model-value="emit('update:keyword', String($event ?? ''))" />
      <div class="domain-table-toolbar__select-wrap">
        <select :value="health" class="domain-table-toolbar__select" @change="onHealthChange">
          <option value="all">{{ t('domains.table.filterHealthAll') }}</option>
          <option v-for="status in healthOptions" :key="status" :value="status">
            {{ t(`domains.health.${status}`) }}
          </option>
        </select>
        <GIcon name="chevron-down" :size="14" class="domain-table-toolbar__chevron" />
      </div>
      <div class="domain-table-toolbar__select-wrap">
        <select :value="protocol" class="domain-table-toolbar__select" @change="onProtocolChange">
          <option value="all">{{ t('domains.table.filterProtocolAll') }}</option>
          <option value="https">HTTPS</option>
          <option value="http">HTTP</option>
        </select>
        <GIcon name="chevron-down" :size="14" class="domain-table-toolbar__chevron" />
      </div>
      <div class="domain-table-toolbar__select-wrap">
        <select :value="sortBy" class="domain-table-toolbar__select" @change="onSortByChange">
          <option value="host">{{ t('domains.table.columns.host') }}</option>
          <option value="requestCount24h">{{ t('domains.table.columns.requests') }}</option>
          <option value="traffic24h">{{ t('domains.table.columns.traffic') }}</option>
          <option value="lastAccessAt">{{ t('domains.table.columns.lastAccess') }}</option>
          <option value="createdAt">{{ t('domains.table.columns.createdAt') }}</option>
        </select>
        <GIcon name="chevron-down" :size="14" class="domain-table-toolbar__chevron" />
      </div>
      <GButton variant="ghost" size="sm" icon="arrow-up-down" @click="emit('toggle-sort')">
        {{ sortDir === 'asc' ? t('domains.table.sortAsc') : t('domains.table.sortDesc') }}
      </GButton>
    </header>

    <div class="domain-table" role="table">
      <div class="domain-table__head" role="row">
        <label class="domain-table__cell domain-table__cell--check">
          <input
            type="checkbox"
            :checked="allSelected"
            @change="emit('toggle-all', ($event.target as HTMLInputElement).checked)" />
        </label>
        <span v-for="column in columns" :key="column.key" class="domain-table__cell" role="columnheader">
          {{ column.label }}
        </span>
        <span class="domain-table__cell domain-table__cell--chevron" aria-hidden="true" />
      </div>

      <button
        v-for="item in items"
        :key="item.host"
        type="button"
        class="domain-table__row"
        :class="{ active: selectedHost === item.host }"
        role="row"
        @click="emit('select', item.host)">
        <span class="domain-table__cell domain-table__cell--check" @click.stop>
          <input
            type="checkbox"
            :checked="selectedHosts.has(item.host)"
            @change="emit('toggle-host', item.host, ($event.target as HTMLInputElement).checked)" />
        </span>
        <span class="domain-table__cell domain-table__cell--host">
          <div class="domain-table__host-line">
            <strong>{{ item.host }}</strong>
            <span class="domain-pill domain-pill--protocol">{{ item.protocol.toUpperCase() }}</span>
          </div>
          <small>{{ item.url }}</small>
        </span>
        <span class="domain-table__cell">
          <span class="domain-table__binding">{{ item.tunnelName || t('domains.table.unbound') }}</span>
        </span>
        <span class="domain-table__cell">
          <span class="domain-table__binding">{{ item.projectName || t('domains.table.unbound') }}</span>
        </span>
        <span class="domain-table__cell domain-table__cell--status">
          <span class="domain-pill" :class="`is-${item.dnsStatus}`">{{ t(`domains.dns.${item.dnsStatus}`) }}</span>
          <span class="domain-pill" :class="`is-${DOMAIN_HEALTH_TONES[item.healthStatus] || 'neutral'}`">
            {{ t(`domains.health.${item.healthStatus}`) }}
          </span>
        </span>
        <span class="domain-table__cell domain-table__cell--metrics">
          <span>{{ formatRelativeTime(item.lastAccessAt) }}</span>
          <small>{{ item.requestCount24h }} · {{ formatDomainBytes(item.traffic24h) }}</small>
        </span>
        <span class="domain-table__cell domain-table__cell--chevron" aria-hidden="true">
          <GIcon name="chevron-right" :size="16" />
        </span>
      </button>

      <div v-if="!items.length" class="domain-table__empty">
        {{ t('domains.table.empty') }}
      </div>
    </div>

    <footer class="domain-table-footer">
      <span>{{ t('domains.table.total', { count: total }) }}</span>
      <div class="domain-table-footer__pager">
        <GButton variant="ghost" size="sm" :disabled="page <= 1" @click="emit('page', page - 1)">‹</GButton>
        <span>{{ page }}</span>
        <GButton variant="ghost" size="sm" :disabled="page * pageSize >= total" @click="emit('page', page + 1)">›</GButton>
      </div>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GInput from '@components/form/GInput.vue'
import GIcon from '@components/icons/GIcon.vue'
import { DOMAIN_HEALTH_TONES, formatDomainBytes, formatRelativeTime } from '../../composables/domainFormat'
import type { ManagedDomainRecord } from '../../types'

const props = defineProps<{
  items: ManagedDomainRecord[]
  total: number
  page: number
  pageSize: number
  keyword: string
  health: string
  protocol: string
  sortBy: string
  sortDir: 'asc' | 'desc'
  selectedHost: string | null
  selectedHosts: Set<string>
}>()

const emit = defineEmits<{
  'update:keyword': [value: string]
  'update:health': [value: string]
  'update:protocol': [value: string]
  'update:sortBy': [value: string]
  'toggle-sort': []
  'toggle-all': [checked: boolean]
  'toggle-host': [host: string, checked: boolean]
  select: [host: string]
  page: [page: number]
}>()

const { t } = useI18n()

const healthOptions = ['healthy', 'warning', 'offline', 'expired', 'dnsError', 'certificateError', 'tunnelOffline']

const columns = computed(() => [
  { key: 'host', label: t('domains.table.columns.host') },
  { key: 'tunnel', label: t('domains.table.columns.tunnel') },
  { key: 'project', label: t('domains.table.columns.project') },
  { key: 'status', label: t('domains.table.columns.status') },
  { key: 'metrics', label: t('domains.table.columns.metrics') },
])

const allSelected = computed(
  () => props.items.length > 0 && props.items.every((item) => props.selectedHosts.has(item.host)),
)

function onHealthChange(event: Event) {
  emit('update:health', (event.target as HTMLSelectElement).value)
}

function onProtocolChange(event: Event) {
  emit('update:protocol', (event.target as HTMLSelectElement).value)
}

function onSortByChange(event: Event) {
  emit('update:sortBy', (event.target as HTMLSelectElement).value)
}
</script>

<style scoped>
.domain-table-shell {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  min-height: 0;
}

.domain-table-toolbar {
  display: grid;
  grid-template-columns: minmax(240px, 1.6fr) repeat(3, minmax(140px, 0.9fr)) auto;
  gap: var(--space-2);
  align-items: center;
}

.domain-table-toolbar__select-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.domain-table-toolbar__select {
  appearance: none;
  width: 100%;
  height: var(--control-height-md);
  padding: 0 var(--space-6) 0 var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-input);
  color: var(--text-primary);
  font-size: var(--font-size-input);
  cursor: pointer;
}

.domain-table-toolbar__select:focus {
  border-color: var(--color-border-focus);
  outline: none;
  box-shadow: var(--shadow-focus);
}

.domain-table-toolbar__chevron {
  position: absolute;
  right: var(--space-3);
  color: var(--text-tertiary);
  pointer-events: none;
}

.domain-table {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  overflow: auto;
  background: var(--bg-surface);
}

.domain-table__head,
.domain-table__row {
  display: grid;
  grid-template-columns: 42px minmax(220px, 2fr) minmax(120px, 1fr) minmax(120px, 1fr) minmax(160px, 1.1fr) minmax(140px, 0.9fr) 32px;
  align-items: center;
  gap: var(--space-3);
  min-width: 920px;
}

.domain-table__head {
  position: sticky;
  top: 0;
  z-index: 1;
  padding: var(--space-3) var(--space-4);
  background: var(--bg-surface-muted);
  border-bottom: 1px solid var(--border-subtle);
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.domain-table__row {
  width: 100%;
  padding: var(--space-3) var(--space-4);
  border: 0;
  border-bottom: 1px solid var(--border-subtle);
  background: transparent;
  text-align: left;
  cursor: pointer;
  transition: background var(--duration-fast);
}

.domain-table__row:hover,
.domain-table__row.active {
  background: var(--bg-surface-hover);
}

.domain-table__row.active .domain-table__cell--chevron {
  color: var(--color-primary);
}

.domain-table__cell {
  min-width: 0;
  font-size: var(--text-sm);
}

.domain-table__cell--host {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.domain-table__host-line {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
}

.domain-table__host-line strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.domain-table__cell--host small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.domain-table__binding {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-secondary);
}

.domain-table__cell--status {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.domain-table__cell--metrics {
  display: flex;
  flex-direction: column;
  gap: 2px;
  color: var(--text-secondary);
}

.domain-table__cell--metrics small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.domain-table__cell--chevron {
  display: grid;
  place-items: center;
  color: var(--text-tertiary);
}

.domain-pill {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  background: var(--bg-input);
  white-space: nowrap;
}

.domain-pill--protocol {
  color: var(--color-primary);
  background: var(--color-primary-muted);
}

.domain-pill.is-matched,
.domain-pill.is-success { color: var(--color-success); }
.domain-pill.is-mismatched,
.domain-pill.is-noRecord,
.domain-pill.is-error,
.domain-pill.is-warning { color: var(--color-warning); }
.domain-pill.is-offline,
.domain-pill.is-expired,
.domain-pill.is-dnsError,
.domain-pill.is-certificateError,
.domain-pill.is-tunnelOffline,
.domain-pill.is-error-tone { color: var(--color-danger); }
.domain-pill.is-neutral,
.domain-pill.is-notChecked,
.domain-pill.is-unknown { color: var(--text-secondary); }

.domain-table__empty {
  padding: var(--space-6);
  text-align: center;
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.domain-table-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.domain-table-footer__pager {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
}
</style>
