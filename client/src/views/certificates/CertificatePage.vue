<template>
  <main class="certificate-page">
    <header class="certificate-page__header">
      <div>
        <span class="certificate-page__eyebrow">{{ t('certificate.eyebrow') }}</span>
        <h1>{{ t('certificate.title') }}</h1>
      </div>
      <button
        class="certificate-page__icon-button"
        type="button"
        :title="t('certificate.refresh')"
        @click="refresh">
        <GIcon name="refresh" :size="16" />
      </button>
    </header>

    <section v-if="loading && certificates.length === 0" class="certificate-page__skeleton">
      <span v-for="index in 6" :key="index" />
    </section>

    <GEmptyState
      v-else-if="!loading && certificates.length === 0"
      :title="t('certificate.emptyTitle')"
      :description="
        t('certificate.emptyDescription', { path: storeRoot || t('certificate.storeNotCreated') })
      ">
      <template #icon>
        <GIcon name="shield-check" :size="32" />
      </template>
      <template #action>
        <button class="certificate-page__button" type="button" @click="refresh">
          <GIcon name="refresh" :size="14" />
          {{ t('certificate.refresh') }}
        </button>
      </template>
    </GEmptyState>

    <section v-else class="certificate-page__table" :aria-label="t('certificate.listAria')">
      <div class="certificate-page__table-head">
        <span>{{ t('certificate.columns.domain') }}</span>
        <span>{{ t('certificate.columns.issuer') }}</span>
        <span>{{ t('certificate.columns.issuedAt') }}</span>
        <span>{{ t('certificate.columns.expiresAt') }}</span>
        <span>{{ t('certificate.columns.days') }}</span>
        <span>{{ t('certificate.columns.status') }}</span>
        <span>{{ t('certificate.columns.renewal') }}</span>
        <span />
      </div>
      <article
        v-for="certificate in certificates"
        :key="certificate.domain"
        class="certificate-page__row">
        <strong>{{ certificate.domain }}</strong>
        <span :title="certificate.issuer">{{ certificate.issuer }}</span>
        <span>{{ formatDate(certificate.createTime) }}</span>
        <span>{{ formatDate(certificate.expireTime) }}</span>
        <span>{{ certificate.daysRemaining }}</span>
        <span
          class="certificate-page__status"
          :class="`is-${certificate.status}`"
          :title="certificate.lastError || undefined">
          {{ statusLabel(certificate.status) }}
        </span>
        <span class="certificate-page__renewal" :title="certificate.lastError || undefined">
          {{ renewalLabel(certificate.autoRenewalStatus) }}
        </span>
        <div class="certificate-page__actions">
          <button
            type="button"
            :title="t('certificate.viewDetails')"
            @click="select(certificate.domain)">
            <GIcon name="eye" :size="15" />
          </button>
          <button
            type="button"
            :title="t('certificate.exportPem')"
            :disabled="!certificate.hasCertificatePem"
            @click="exportPem(certificate.domain)">
            <GIcon name="download" :size="15" />
          </button>
          <button type="button" :title="t('certificate.copyInfo')" @click="copyInfo(certificate)">
            <GIcon name="copy" :size="15" />
          </button>
        </div>
      </article>
    </section>

    <aside
      v-if="selected"
      class="certificate-page__detail"
      :aria-label="t('certificate.detailAria')">
      <header>
        <div>
          <span>{{ t('certificate.detailTitle') }}</span>
          <h2>{{ selected.summary.domain }}</h2>
        </div>
        <button type="button" :title="t('certificate.close')" @click="selected = null">
          <GIcon name="close" :size="16" />
        </button>
      </header>
      <dl>
        <dt>{{ t('certificate.fields.issuer') }}</dt>
        <dd>{{ selected.summary.issuer }}</dd>
        <dt>{{ t('certificate.fields.serialNumber') }}</dt>
        <dd>{{ selected.summary.serialNumber || '-' }}</dd>
        <dt>{{ t('certificate.fields.algorithm') }}</dt>
        <dd>{{ selected.summary.algorithm }}</dd>
        <dt>{{ t('certificate.fields.fingerprint') }}</dt>
        <dd class="mono">
          {{ selected.summary.fingerprintSha256 }}
        </dd>
        <dt>SAN</dt>
        <dd>{{ selected.summary.san.join(', ') || '-' }}</dd>
        <dt>{{ t('certificate.fields.certificatePath') }}</dt>
        <dd class="mono">
          {{ selected.summary.certificatePath }}
        </dd>
        <dt v-if="selected.summary.lastError">{{ t('certificate.fields.error') }}</dt>
        <dd v-if="selected.summary.lastError" class="certificate-page__inline-error">
          {{ selected.summary.lastError }}
        </dd>
      </dl>
      <pre v-if="selected.certificatePem">{{ selected.certificatePem }}</pre>
      <p v-else class="certificate-page__empty-pem">{{ t('certificate.noPem') }}</p>
    </aside>

    <p v-if="loading && certificates.length > 0" class="certificate-page__loading">
      {{ t('certificate.loading') }}
    </p>
    <p v-if="error" class="certificate-page__error">
      {{ error }}
    </p>
  </main>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import GEmptyState from '@/components/feedback/GEmptyState.vue'
import GIcon from '@/components/icons/GIcon.vue'
import { useFeedback } from '@/composables/useFeedback'
import { certificateService } from './service'
import type {
  CertificateDetailResponse,
  CertificateStatus,
  CertificateSummary,
  AutoRenewalStatus,
} from './types'

const certificates = ref<CertificateSummary[]>([])
const selected = ref<CertificateDetailResponse | null>(null)
const loading = ref(false)
const error = ref('')
const storeRoot = ref('')
const { t, locale } = useI18n()
const { toast } = useFeedback()

onMounted(() => {
  void refresh()
})

async function refresh() {
  loading.value = true
  error.value = ''
  try {
    const response = await certificateService.list()
    certificates.value = response.certificates
    storeRoot.value = response.storeRoot
  } catch (source) {
    error.value = source instanceof Error ? source.message : String(source)
  } finally {
    loading.value = false
  }
}

async function select(domain: string) {
  error.value = ''
  try {
    selected.value = await certificateService.detail(domain)
  } catch (source) {
    error.value = source instanceof Error ? source.message : String(source)
  }
}

async function exportPem(domain: string) {
  const pem = await certificateService.exportPem(domain)
  const blob = new Blob([pem], { type: 'application/x-pem-file' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = `${safeFileName(domain)}.pem`
  document.body.appendChild(anchor)
  anchor.click()
  anchor.remove()
  URL.revokeObjectURL(url)
}

async function copyInfo(certificate: CertificateSummary) {
  await navigator.clipboard.writeText(JSON.stringify(certificate, null, 2))
  toast.success(t('certificate.copied'))
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat(locale.value === 'en-US' ? 'en-US' : 'zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(new Date(value))
}

function statusLabel(status: CertificateStatus) {
  return t(`certificate.status.${status}`)
}

function renewalLabel(status: AutoRenewalStatus) {
  return t(`certificate.renewal.${status}`)
}

function safeFileName(value: string) {
  return value.replace(/[^a-z0-9.-]+/gi, '_').replace(/^_+|_+$/g, '') || 'certificate'
}
</script>

<style scoped>
.certificate-page {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  height: 100%;
  padding: var(--page-padding);
  overflow: auto;
}

.certificate-page__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
}

.certificate-page__eyebrow {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  text-transform: uppercase;
}

.certificate-page__header h1 {
  margin-top: var(--space-1);
  color: var(--text-primary);
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
}

.certificate-page__icon-button,
.certificate-page__actions button,
.certificate-page__detail header button,
.certificate-page__button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 32px;
  height: 32px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
}

.certificate-page__button {
  gap: var(--space-2);
  padding: 0 var(--space-3);
}

.certificate-page__actions button:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.certificate-page__skeleton {
  display: grid;
  gap: var(--space-2);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.certificate-page__skeleton span {
  height: 44px;
  border-radius: var(--radius-sm);
  background: linear-gradient(
    90deg,
    var(--bg-input),
    var(--bg-surface-hover),
    var(--bg-input)
  );
  background-size: 200% 100%;
  animation: certificate-skeleton 1.3s ease-in-out infinite;
}

.certificate-page__table {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.certificate-page__table-head,
.certificate-page__row {
  display: grid;
  grid-template-columns:
    minmax(140px, 1.3fr) minmax(160px, 1.5fr)
    110px 110px 64px 118px 120px 112px;
  align-items: center;
  gap: var(--space-3);
  min-height: 44px;
  padding: 0 var(--space-4);
}

.certificate-page__table-head {
  color: var(--text-tertiary);
  background: var(--bg-toolbar);
  font-size: var(--text-xs);
}

.certificate-page__row {
  border-top: 1px solid var(--border-subtle);
  color: var(--text-secondary);
}

.certificate-page__row strong,
.certificate-page__detail h2 {
  color: var(--text-primary);
  font-weight: var(--weight-semibold);
}

.certificate-page__row > span,
.certificate-page__row > strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.certificate-page__status {
  width: fit-content;
  padding: 3px 8px;
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
}

.certificate-page__status.is-active {
  color: var(--color-success);
  background: var(--color-success-muted);
}

.certificate-page__status.is-expiringSoon,
.certificate-page__status.is-pending {
  color: var(--color-warning);
  background: var(--color-warning-muted);
}

.certificate-page__status.is-expired,
.certificate-page__status.is-failed,
.certificate-page__status.is-revoked {
  color: var(--color-error);
  background: var(--color-error-muted);
}

.certificate-page__renewal {
  color: var(--text-tertiary);
}

.certificate-page__actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-1);
}

.certificate-page__detail {
  display: grid;
  gap: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  padding: var(--space-4);
  background: var(--bg-surface);
}

.certificate-page__detail header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.certificate-page__detail header span,
.certificate-page__detail dt {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.certificate-page__detail dl {
  display: grid;
  grid-template-columns: 120px minmax(0, 1fr);
  gap: var(--space-2) var(--space-4);
}

.certificate-page__detail dd {
  color: var(--text-secondary);
  overflow-wrap: anywhere;
}

.certificate-page__detail pre {
  max-height: 260px;
  overflow: auto;
  padding: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  line-height: 1.5;
}

.mono {
  font-family: var(--font-mono);
}

.certificate-page__loading,
.certificate-page__error {
  color: var(--text-tertiary);
}

.certificate-page__error {
  color: var(--color-error);
}

.certificate-page__inline-error,
.certificate-page__empty-pem {
  color: var(--color-error);
}

@keyframes certificate-skeleton {
  from {
    background-position: 200% 0;
  }

  to {
    background-position: -200% 0;
  }
}

@media (max-width: 980px) {
  .certificate-page__table {
    overflow-x: auto;
  }

  .certificate-page__table-head,
  .certificate-page__row {
    min-width: 960px;
  }
}
</style>
