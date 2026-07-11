<template>
  <aside class="domain-drawer" :class="{ open: visible }">
    <div class="domain-drawer__scrim" @click="emit('close')" />
    <section class="domain-drawer__panel">
      <header class="domain-drawer__header">
        <div class="domain-drawer__header-copy">
          <p>{{ t('domains.drawer.title') }}</p>
          <h2>{{ summary?.host || '—' }}</h2>
        </div>
        <GButton variant="ghost" size="sm" icon="close" @click="emit('close')" />
      </header>

      <div v-if="loading" class="domain-drawer__scroll">
        <GSkeleton v-for="index in 4" :key="index" height="72px" />
      </div>

      <div v-else-if="detail" class="domain-drawer__scroll">
        <article class="domain-drawer__card">
          <h3>{{ t('domains.drawer.actions') }}</h3>
          <div class="domain-action-grid">
            <button
              type="button"
              class="domain-action is-primary"
              :disabled="!summary?.url"
              @click="emit('copy-url', summary?.url || '')">
              <span class="domain-action__icon"><GIcon name="copy" :size="16" /></span>
              <span class="domain-action__label">{{ t('domains.drawer.copyUrl') }}</span>
            </button>
            <button
              type="button"
              class="domain-action is-info"
              :disabled="!summary?.url"
              @click="emit('open-url', summary?.url || '')">
              <span class="domain-action__icon"><GIcon name="external-link" :size="16" /></span>
              <span class="domain-action__label">{{ t('domains.drawer.openUrl') }}</span>
            </button>
            <button
              type="button"
              class="domain-action is-success"
              :disabled="!summary?.host"
              @click="emit('check-dns', summary?.host || '')">
              <span class="domain-action__icon"><GIcon name="refresh" :size="16" /></span>
              <span class="domain-action__label">{{ t('domains.row.checkDns') }}</span>
            </button>
            <button
              type="button"
              class="domain-action is-warning"
              :disabled="!summary?.host"
              @click="emit('bind-tunnel', summary?.host || '')">
              <span class="domain-action__icon"><GIcon name="router" :size="16" /></span>
              <span class="domain-action__label">{{ t('domains.row.bindTunnel') }}</span>
            </button>
            <button
              type="button"
              class="domain-action is-certificate"
              :disabled="!summary?.host"
              @click="emit('bind-certificate', summary?.host || '')">
              <span class="domain-action__icon"><GIcon name="shield-check" :size="16" /></span>
              <span class="domain-action__label">{{ t('domains.row.bindCertificate') }}</span>
            </button>
            <button
              type="button"
              class="domain-action is-neutral"
              :disabled="!summary?.host"
              @click="emit('view-logs', summary?.host || '')">
              <span class="domain-action__icon"><GIcon name="logs" :size="16" /></span>
              <span class="domain-action__label">{{ t('domains.drawer.viewLogs') }}</span>
            </button>
            <button
              type="button"
              class="domain-action is-danger domain-action--full"
              :disabled="!summary?.host"
              @click="emit('delete', summary?.host || '')">
              <span class="domain-action__icon"><GIcon name="trash" :size="16" /></span>
              <span class="domain-action__label">{{ t('common.delete') }}</span>
            </button>
          </div>
        </article>

        <article class="domain-drawer__card">
          <h3>{{ t('domains.drawer.basic') }}</h3>
          <dl class="domain-drawer__dl">
            <div><dt>{{ t('domains.drawer.host') }}</dt><dd>{{ summary?.host }}</dd></div>
            <div><dt>{{ t('domains.drawer.alias') }}</dt><dd>{{ summary?.aliases?.join(', ') || '-' }}</dd></div>
            <div><dt>{{ t('domains.drawer.path') }}</dt><dd>{{ summary?.path }}</dd></div>
            <div><dt>{{ t('domains.drawer.server') }}</dt><dd>{{ summary?.serverName || '-' }}</dd></div>
            <div><dt>{{ t('domains.drawer.tunnel') }}</dt><dd>{{ summary?.tunnelName || '-' }}</dd></div>
            <div><dt>{{ t('domains.drawer.project') }}</dt><dd>{{ summary?.projectName || '-' }}</dd></div>
          </dl>
        </article>

        <article class="domain-drawer__card">
          <div class="domain-drawer__card-head">
            <h3>{{ t('domains.drawer.https') }}</h3>
            <GButton variant="ghost" size="sm" icon="refresh" :loading="dnsRefreshing" @click="refreshDns">
              {{ t('domains.drawer.refreshDns') }}
            </GButton>
          </div>
          <dl v-if="summary?.certificate" class="domain-drawer__dl">
            <div><dt>TLS</dt><dd>{{ summary.certificate.tlsVersion || 'TLS' }}</dd></div>
            <div><dt>{{ t('domains.drawer.certificate') }}</dt><dd>{{ summary.certificate.domain }}</dd></div>
            <div><dt>Issuer</dt><dd>{{ summary.certificate.issuer }}</dd></div>
            <div><dt>SAN</dt><dd>{{ summary.certificate.san.join(', ') || '-' }}</dd></div>
            <div><dt>{{ t('domains.drawer.expire') }}</dt><dd>{{ summary.certificate.daysRemaining }}d</dd></div>
            <div><dt>{{ t('domains.drawer.autoRenew') }}</dt><dd>{{ summary.certificate.autoRenewalEnabled ? t('common.enabled') : t('common.disabled') }}</dd></div>
          </dl>
          <p v-else class="is-empty">{{ t('domains.drawer.noCertificate') }}</p>
          <div class="domain-drawer__inline-actions">
            <GButton variant="secondary" size="sm" icon="shield-check" @click="emit('open-certificates')">
              {{ t('domains.drawer.changeCertificate') }}
            </GButton>
            <GButton variant="secondary" size="sm" icon="refresh" @click="emit('renew-now')">
              {{ t('domains.drawer.renewNow') }}
            </GButton>
            <GButton variant="secondary" size="sm" icon="upload" @click="emit('redeploy')">
              {{ t('domains.drawer.redeploy') }}
            </GButton>
          </div>
        </article>

        <article class="domain-drawer__card">
          <div class="domain-drawer__card-head">
            <h3>DNS</h3>
            <GButton variant="ghost" size="sm" icon="copy" @click="copyDnsRecords">
              {{ t('domains.drawer.copyDns') }}
            </GButton>
          </div>
          <p class="domain-drawer__dns-status" :class="`is-${detail.dns.status}`">
            {{ t(`domains.dns.${detail.dns.status}`) }}
          </p>
          <div v-for="record in detail.dns.records" :key="record.type" class="domain-dns-record">
            <strong>{{ record.type }}</strong>
            <span>{{ record.values.join(', ') || '-' }}</span>
            <small>TTL {{ record.ttl }}</small>
          </div>
        </article>

        <article class="domain-drawer__card">
          <h3>{{ t('domains.drawer.runtime') }}</h3>
          <div class="domain-runtime-grid">
            <div><span>{{ t('domains.drawer.requests') }}</span><strong>{{ detail.runtime.httpRequests }}</strong></div>
            <div><span>{{ t('domains.drawer.traffic') }}</span><strong>{{ formatDomainBytes(detail.runtime.trafficBytes) }}</strong></div>
            <div><span>{{ t('domains.drawer.latency') }}</span><strong>{{ detail.runtime.latencyMs.toFixed(1) }}ms</strong></div>
            <div><span>{{ t('domains.drawer.errorRate') }}</span><strong>{{ (detail.runtime.errorRate * 100).toFixed(1) }}%</strong></div>
            <div><span>{{ t('domains.drawer.connections') }}</span><strong>{{ detail.runtime.currentConnections }}</strong></div>
            <div><span>{{ t('domains.drawer.peakConnections') }}</span><strong>{{ detail.runtime.peakConnections }}</strong></div>
          </div>
          <RuntimeTrendChart
            :title="t('domains.drawer.latency')"
            :eyebrow="t('domains.drawer.runtime')"
            :series="latencySeries" />
        </article>

        <article class="domain-drawer__card domain-drawer__card--logs">
          <h3>{{ t('domains.drawer.logs') }}</h3>
          <p class="domain-drawer__log-kind">{{ t('domains.drawer.accessLogs') }}</p>
          <div class="domain-log-list">
            <div v-for="(log, index) in detail.logs.access.slice(0, 8)" :key="`access-${index}`" class="domain-log-line">
              <span>{{ log.level }}</span>
              <p>{{ log.message }}</p>
            </div>
            <p v-if="!detail.logs.access.length" class="is-empty">{{ t('domains.drawer.noLogs') }}</p>
          </div>
          <p class="domain-drawer__log-kind">{{ t('domains.drawer.errorLogs') }}</p>
          <div class="domain-log-list">
            <div v-for="(log, index) in detail.logs.error.slice(0, 8)" :key="`error-${index}`" class="domain-log-line is-error">
              <span>{{ log.level }}</span>
              <p>{{ log.message }}</p>
            </div>
            <p v-if="!detail.logs.error.length" class="is-empty">{{ t('domains.drawer.noLogs') }}</p>
          </div>
        </article>
      </div>
    </section>
  </aside>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import GSkeleton from '@components/feedback/GSkeleton.vue'
import RuntimeTrendChart from '@components/runtime/RuntimeTrendChart.vue'
import { domainService } from '../../services/domain.service'
import { formatDomainBytes } from '../../composables/domainFormat'
import type { DomainDetailResponse } from '../../types'

const props = defineProps<{
  visible: boolean
  loading?: boolean
  detail: DomainDetailResponse | null
}>()

const emit = defineEmits<{
  close: []
  'copy-url': [url: string]
  'open-url': [url: string]
  'view-logs': [host: string]
  'check-dns': [host: string]
  'bind-tunnel': [host: string]
  'bind-certificate': [host: string]
  delete: [host: string]
  'open-certificates': []
  'renew-now': []
  redeploy: []
  'dns-updated': []
}>()

const { t } = useI18n()
const dnsRefreshing = ref(false)

const summary = computed(() => props.detail?.summary ?? null)
const latencySeries = computed(() => [
  {
    name: t('domains.drawer.latency'),
    color: 'var(--color-primary)',
    values: (props.detail?.runtime.latencyTrend ?? [])
      .map((value) => (typeof value === 'number' ? value : Number(value) || 0))
      .slice(-24),
  },
])

async function refreshDns() {
  if (!summary.value?.host) return
  dnsRefreshing.value = true
  try {
    await domainService.checkDns(summary.value.host)
    emit('dns-updated')
  } finally {
    dnsRefreshing.value = false
  }
}

function copyDnsRecords() {
  if (!props.detail?.dns.records.length) return
  const text = props.detail.dns.records
    .map((record) => `${record.type}\t${record.values.join(', ')}\tTTL ${record.ttl}`)
    .join('\n')
  void navigator.clipboard.writeText(text)
}
</script>

<style scoped>
.domain-drawer {
  position: fixed;
  inset: 0;
  z-index: 80;
  pointer-events: none;
}

.domain-drawer.open {
  pointer-events: auto;
}

.domain-drawer__scrim {
  position: absolute;
  inset: 0;
  background: rgba(8, 12, 20, 0.42);
  opacity: 0;
  transition: opacity var(--duration-standard);
}

.domain-drawer.open .domain-drawer__scrim {
  opacity: 1;
}

.domain-drawer__panel {
  position: absolute;
  top: 0;
  right: 0;
  width: min(520px, 100vw);
  height: 100%;
  background: var(--bg-surface);
  border-left: 1px solid var(--border-subtle);
  transform: translateX(100%);
  transition: transform var(--duration-standard) var(--ease-out);
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.domain-drawer.open .domain-drawer__panel {
  transform: translateX(0);
}

.domain-drawer__header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-3);
  flex-shrink: 0;
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
  background: var(--bg-surface);
}

.domain-drawer__header-copy {
  min-width: 0;
}

.domain-drawer__header p {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.domain-drawer__header h2 {
  margin: 4px 0 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.domain-drawer__scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.domain-drawer__card {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--space-3);
  background: var(--bg-surface-muted);
  isolation: isolate;
}

.domain-drawer__card h3 {
  margin: 0 0 var(--space-3);
  font-size: var(--text-sm);
}

.domain-drawer__card-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.domain-drawer__card-head h3 {
  margin: 0;
}

.domain-action-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-2);
}

.domain-action {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
  padding: 10px 12px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  color: var(--text-primary);
  font-size: var(--text-sm);
  text-align: left;
  cursor: pointer;
  transition:
    border-color var(--duration-fast) var(--ease-out),
    background var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out);
}

.domain-action:hover:not(:disabled) {
  border-color: var(--color-border-strong);
  background: var(--bg-surface-hover);
}

.domain-action:active:not(:disabled) {
  transform: scale(0.98);
}

.domain-action:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.domain-action--full {
  grid-column: 1 / -1;
}

.domain-action__icon {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.domain-action__label {
  min-width: 0;
  line-height: 1.3;
}

.domain-action.is-primary .domain-action__icon {
  color: var(--color-primary);
  background: color-mix(in srgb, var(--color-primary) 14%, transparent);
}

.domain-action.is-info .domain-action__icon {
  color: #38bdf8;
  background: color-mix(in srgb, #38bdf8 14%, transparent);
}

.domain-action.is-success .domain-action__icon {
  color: var(--color-success);
  background: color-mix(in srgb, var(--color-success) 14%, transparent);
}

.domain-action.is-warning .domain-action__icon {
  color: var(--color-warning);
  background: color-mix(in srgb, var(--color-warning) 14%, transparent);
}

.domain-action.is-certificate .domain-action__icon {
  color: #a78bfa;
  background: color-mix(in srgb, #a78bfa 14%, transparent);
}

.domain-action.is-neutral .domain-action__icon {
  color: var(--text-secondary);
  background: var(--bg-input);
}

.domain-action.is-danger {
  border-color: color-mix(in srgb, var(--color-danger) 28%, var(--border-subtle));
  background: color-mix(in srgb, var(--color-danger) 6%, var(--bg-surface));
}

.domain-action.is-danger .domain-action__icon {
  color: var(--color-danger);
  background: color-mix(in srgb, var(--color-danger) 14%, transparent);
}

.domain-action.is-danger:hover:not(:disabled) {
  border-color: color-mix(in srgb, var(--color-danger) 45%, var(--border-subtle));
  background: color-mix(in srgb, var(--color-danger) 10%, var(--bg-surface));
}

.domain-drawer__dl {
  display: grid;
  gap: 8px;
  margin: 0;
}

.domain-drawer__dl div {
  display: grid;
  grid-template-columns: 108px 1fr;
  gap: var(--space-2);
}

.domain-drawer__dl dt {
  color: var(--text-secondary);
}

.domain-drawer__dl dd {
  margin: 0;
  word-break: break-word;
}

.domain-drawer__inline-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-3);
}

.domain-drawer__dns-status.is-matched { color: var(--color-success); }
.domain-drawer__dns-status.is-mismatched,
.domain-drawer__dns-status.is-error,
.domain-drawer__dns-status.is-noRecord { color: var(--color-danger); }

.domain-dns-record {
  display: grid;
  gap: 2px;
  padding: 8px 0;
  border-top: 1px solid var(--border-subtle);
}

.domain-dns-record span {
  word-break: break-all;
}

.domain-runtime-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.domain-runtime-grid div {
  padding: var(--space-2);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.domain-runtime-grid span {
  display: block;
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.domain-log-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 180px;
  overflow: auto;
}

.domain-log-line {
  display: grid;
  grid-template-columns: 64px 1fr;
  gap: var(--space-2);
  font-size: var(--text-xs);
}

.domain-log-line p {
  margin: 0;
  color: var(--text-secondary);
  word-break: break-word;
}

.domain-drawer__log-kind {
  margin: var(--space-2) 0 6px;
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.domain-log-line.is-error span {
  color: var(--color-danger);
}

.is-empty {
  color: var(--text-tertiary);
}
</style>
