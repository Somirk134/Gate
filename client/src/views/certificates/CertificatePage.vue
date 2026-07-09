<template>
  <main class="certificate-page">
    <header class="certificate-page__header">
      <div>
        <span class="certificate-page__eyebrow">TLS 存储</span>
        <h1>证书</h1>
      </div>
      <button class="certificate-page__icon-button" type="button" title="刷新" @click="refresh">
        <GIcon name="refresh" :size="16" />
      </button>
    </header>

    <GEmptyState
      v-if="!loading && certificates.length === 0"
      title="暂无证书"
      :description="`存储目录：${storeRoot || '尚未创建'}`">
      <template #icon>
        <GIcon name="shield-check" :size="32" />
      </template>
      <template #action>
        <button class="certificate-page__button" type="button" @click="refresh">
          <GIcon name="refresh" :size="14" />
          刷新
        </button>
      </template>
    </GEmptyState>

    <section v-else class="certificate-page__table" aria-label="证书列表">
      <div class="certificate-page__table-head">
        <span>域名</span>
        <span>签发者</span>
        <span>签发时间</span>
        <span>到期时间</span>
        <span>天数</span>
        <span>状态</span>
        <span>续期</span>
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
        <span class="certificate-page__status" :class="`is-${certificate.status}`">
          {{ statusLabel(certificate.status) }}
        </span>
        <span class="certificate-page__renewal">{{
          renewalLabel(certificate.autoRenewalStatus)
        }}</span>
        <div class="certificate-page__actions">
          <button type="button" title="查看详情" @click="select(certificate.domain)">
            <GIcon name="eye" :size="15" />
          </button>
          <button type="button" title="导出 PEM" @click="exportPem(certificate.domain)">
            <GIcon name="download" :size="15" />
          </button>
          <button type="button" title="复制证书信息" @click="copyInfo(certificate)">
            <GIcon name="copy" :size="15" />
          </button>
        </div>
      </article>
    </section>

    <aside v-if="selected" class="certificate-page__detail" aria-label="证书详情">
      <header>
        <div>
          <span>证书详情</span>
          <h2>{{ selected.summary.domain }}</h2>
        </div>
        <button type="button" title="关闭" @click="selected = null">
          <GIcon name="close" :size="16" />
        </button>
      </header>
      <dl>
        <dt>签发者</dt>
        <dd>{{ selected.summary.issuer }}</dd>
        <dt>序列号</dt>
        <dd>{{ selected.summary.serialNumber || '-' }}</dd>
        <dt>算法</dt>
        <dd>{{ selected.summary.algorithm }}</dd>
        <dt>指纹</dt>
        <dd class="mono">
          {{ selected.summary.fingerprintSha256 }}
        </dd>
        <dt>SAN</dt>
        <dd>{{ selected.summary.san.join(', ') || '-' }}</dd>
        <dt>证书路径</dt>
        <dd class="mono">
          {{ selected.summary.certificatePath }}
        </dd>
      </dl>
      <pre>{{ selected.certificatePem }}</pre>
    </aside>

    <p v-if="loading" class="certificate-page__loading">正在加载证书...</p>
    <p v-if="error" class="certificate-page__error">
      {{ error }}
    </p>
  </main>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import GEmptyState from '@/components/feedback/GEmptyState.vue'
import GIcon from '@/components/icons/GIcon.vue'
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
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat(undefined, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(new Date(value))
}

function statusLabel(status: CertificateStatus) {
  return {
    pending: '待处理',
    active: '有效',
    expiringSoon: '即将过期',
    expired: '已过期',
    revoked: '已吊销',
    deleted: '已删除',
    failed: '失败',
    unknown: '未知',
  }[status]
}

function renewalLabel(status: AutoRenewalStatus) {
  return {
    scheduled: '已计划',
    due: '待续期',
    notScheduled: '未计划',
    expired: '已过期',
  }[status]
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
