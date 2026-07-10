<template>
  <section class="cert-panel cert-panel--history">
    <!-- 面板头部：标题 + 统计摘要 + 折叠/刷新 -->
    <div class="cert-panel__heading">
      <div class="history-heading__left">
        <h3>
          <GIcon name="history" :size="16" />
          {{ t('certificate.history.title') }}
        </h3>
        <!-- 行内统计摘要 -->
        <div v-if="records.length > 0" class="history-chips">
          <span class="history-chip">{{ summary.total }} {{ t('certificate.history.summary.totalLabel') }}</span>
          <span v-if="summary.verifying > 0" class="history-chip is-verifying">
            <GIcon name="loader" :size="10" spin />
            {{ summary.verifying }} {{ t('certificate.history.summary.verifyingLabel') }}
          </span>
          <span class="history-chip is-success">{{ summary.issued }} {{ t('certificate.history.summary.issuedLabel') }}</span>
          <span v-if="summary.failed > 0" class="history-chip is-error">{{ summary.failed }} {{ t('certificate.history.summary.failedLabel') }}</span>
        </div>
      </div>
      <div class="history-heading__right">
        <GButton variant="ghost" size="sm" icon="refresh" :loading="loading" @click="fetchHistory" />
        <button type="button" class="collapse-btn" :class="{ collapsed: !expanded }" @click="expanded = !expanded">
          <GIcon name="chevron-down" :size="14" />
        </button>
      </div>
    </div>

    <!-- 可折叠内容区 -->
    <Transition name="collapse">
      <div v-show="expanded" class="history-body">
        <!-- 加载骨架 -->
        <div v-if="loading && records.length === 0" class="history-skeleton">
          <GIcon name="loader" :size="18" spin />
          <span>{{ t('certificate.history.loading') }}</span>
        </div>

        <div v-else-if="historyError" class="history-empty">
          <GIcon name="alert-circle" :size="28" />
          <p>{{ t('certificate.history.loadFailed') }}</p>
          <small>{{ historyError }}</small>
        </div>

        <!-- 空状态 -->
        <div v-else-if="records.length === 0" class="history-empty">
          <GIcon name="clock" :size="28" />
          <p>{{ t('certificate.history.empty') }}</p>
          <small>{{ t('certificate.history.emptyHint') }}</small>
        </div>

        <!-- 记录列表 -->
        <div v-else class="history-list">
          <article
            v-for="record in filteredRecords"
            :key="record.id"
            class="history-row"
            :class="[`status-${record.status}`, { expanded: selectedId === record.id }]"
          >
            <!-- 主行 -->
            <div class="history-row__main" @click="toggleDetail(record.id)">
              <!-- 状态指示条 -->
              <span class="history-row__indicator" />

              <!-- 状态徽标 -->
              <span class="history-badge" :class="`is-${record.status}`">
                <GIcon v-if="record.status === 'verifying'" name="loader" :size="11" spin />
                <GIcon v-else-if="record.status === 'issued'" name="check" :size="11" />
                <GIcon v-else-if="record.status === 'failed'" name="x" :size="11" />
                <GIcon v-else name="clock" :size="11" />
              </span>

              <!-- 信息 -->
              <div class="history-row__info">
                <strong>{{ record.domain }}</strong>
                <span class="history-meta">
                  <GIcon name="mail" :size="11" />{{ record.email }}
                  <span class="meta-divider">·</span>
                  <span class="challenge-tag">{{ challengeLabel(record.challengeType) }}</span>
                      <span v-if="record.staging" class="staging-tag">S</span>
                    </span>
                  </div>

                  <!-- 时间 -->
                  <time class="history-row__time">{{ formatTime(record.updatedAt) }}</time>

                  <!-- 操作（hover 显示） -->
                  <div class="history-row__ops" @click.stop>
                    <button
                      v-if="record.status === 'verifying' || record.status === 'failed'"
                      type="button"
                      class="op-btn"
                      :title="t('certificate.history.retry')"
                      @click="handleRetry(record)">
                      <GIcon name="refresh" :size="13" />
                    </button>
                    <button
                      v-if="record.status === 'issued' && record.certificateAvailable"
                      type="button"
                      class="op-btn"
                      :title="t('certificate.history.download')"
                      @click="handleDownload(record)">
                      <GIcon name="download" :size="13" />
                    </button>
                    <button
                      type="button"
                      class="op-btn op-btn--danger"
                      :title="t('certificate.history.delete')"
                      @click="handleDelete(record)">
                      <GIcon name="trash" :size="13" />
                    </button>
                  </div>
                </div>

                <!-- 展开详情 -->
                <Transition name="slide-down">
                  <div v-if="selectedId === record.id" class="history-row__detail">
                    <div class="detail-cols">
                      <div class="detail-col">
                        <dt>{{ t('certificate.history.detail.domain') }}</dt>
                        <dd>{{ record.domain }}</dd>
                      </div>
                      <div class="detail-col">
                        <dt>{{ t('certificate.history.detail.email') }}</dt>
                        <dd>{{ record.email }}</dd>
                      </div>
                      <div class="detail-col">
                        <dt>{{ t('certificate.history.detail.challengeType') }}</dt>
                        <dd>{{ challengeLabel(record.challengeType) }}</dd>
                      </div>
                      <div class="detail-col">
                        <dt>{{ t('certificate.history.detail.status') }}</dt>
                        <dd><span class="history-badge sm" :class="`is-${record.status}`">{{ statusLabel(record.status) }}</span></dd>
                      </div>
                      <div class="detail-col">
                        <dt>{{ t('certificate.history.detail.createdAt') }}</dt>
                        <dd>{{ formatFullTime(record.createdAt) }}</dd>
                      </div>
                      <div class="detail-col">
                        <dt>{{ t('certificate.history.detail.retryCount') }}</dt>
                        <dd>{{ record.retryCount }}</dd>
                      </div>

                      <!-- 已签发：签发信息 -->
                      <template v-if="record.status === 'issued'">
                        <div v-if="record.issuer" class="detail-col">
                          <dt>{{ t('certificate.history.detail.issuer') }}</dt>
                          <dd>{{ record.issuer }}</dd>
                        </div>
                        <div v-if="record.expireTime" class="detail-col">
                          <dt>{{ t('certificate.history.detail.expireTime') }}</dt>
                          <dd :class="{ 'text-warn': (record.daysRemaining ?? 0) <= 30 }">
                            {{ formatFullTime(new Date(record.expireTime).getTime()) }}
                            <span v-if="record.daysRemaining != null" class="days-hint">
                              ({{ record.daysRemaining > 0 ? `${record.daysRemaining}d` : 'expired' }})
                            </span>
                          </dd>
                        </div>
                      </template>
                    </div>

                    <!-- 错误信息 -->
                    <div v-if="record.error" class="detail-error">
                      <dt>{{ t('certificate.history.detail.error') }}</dt>
                      <dd>{{ record.error }}</dd>
                    </div>

                    <!-- 证书 PEM 下载区 -->
                    <div v-if="record.status === 'issued' && certInfo && certInfoForId === record.id" class="cert-pem-block">
                      <div class="pem-file">
                        <GIcon name="file-code" :size="16" />
                        <div class="pem-info">
                          <strong>certificate.pem</strong>
                          <small v-if="certInfo.algorithm">{{ certInfo.algorithm }}</small>
                        </div>
                        <GButton variant="secondary" size="sm" icon="copy" @click="copyPem(certInfo.certificatePem)">
                          {{ t('certificate.copyPem') }}
                        </GButton>
                      </div>
                    </div>
                  </div>
                </Transition>
              </article>
            </div>
          </div>
        </Transition>
      </section>
    </template>

    <script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { listen } from '@tauri-apps/api/event'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import { useFeedback } from '@composables/useFeedback'
import { certificateService } from '../service'
import type {
  AcmeApplicationRecord,
  AcmeHistoryResponse,
  AcmeCertificateInfo,
  AcmeRecordStatus,
} from '../types'

const props = defineProps<{
  refreshTrigger?: number
}>()

const emit = defineEmits<{
  (e: 'record-updated'): void
}>()

const { t } = useI18n()
const { toast, confirmDanger } = useFeedback()

/* ── 状态 ── */
const records = ref<AcmeApplicationRecord[]>([])
const loading = ref(false)
const historyError = ref('')
const selectedId = ref<string | null>(null)
const retryingId = ref<string | null>(null)
const downloadingId = ref<string | null>(null)
const certInfo = ref<AcmeCertificateInfo | null>(null)
const certInfoForId = ref<string | null>(null)
const expanded = ref(true)

const summary = computed(() => {
  const total = records.value.length
  const verifying = records.value.filter((r) => r.status === 'verifying').length
  const issued = records.value.filter((r) => r.status === 'issued').length
  const failed = records.value.filter((r) => r.status === 'failed').length
  return { total, verifying, issued, failed }
})

/** 按时间倒序 */
const filteredRecords = computed(() =>
  [...records.value].sort((a, b) => b.updatedAt - a.updatedAt),
)

let unlisten: (() => void) | null = null

/* ── 方法 ── */
async function fetchHistory() {
  loading.value = true
  historyError.value = ''
  try {
    const res: AcmeHistoryResponse = await certificateService.history()
    records.value = res.records || []
  } catch (error) {
    historyError.value =
      error instanceof Error ? error.message : t('certificate.history.loadFailed')
  } finally {
    loading.value = false
  }
}

function statusLabel(status: AcmeRecordStatus): string {
  const map: Record<AcmeRecordStatus, string> = {
    pending: t('certificate.history.status.pending'),
    verifying: t('certificate.history.status.verifying'),
    issued: t('certificate.history.status.issued'),
    failed: t('certificate.history.status.failed'),
    expired: t('certificate.history.status.expired'),
  }
  return map[status] || status
}

function challengeLabel(type: string): string {
  return type === 'dns01'
    ? t('certificate.wizard.dns01')
    : t('certificate.wizard.http01')
}

function formatTime(ts: number): string {
  if (!ts) return '-'
  const d = new Date(ts)
  const now = new Date()
  const diff = now.getTime() - d.getTime()
  if (diff < 60_000) return t('certificate.history.time.justNow')
  if (diff < 3600_000) return `${Math.floor(diff / 60000)}m`
  if (diff < 86400_000) return `${Math.floor(diff / 3600000)}h`
  return `${d.getMonth() + 1}/${d.getDate()} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}

function formatFullTime(ts: number): string {
  if (!ts) return '-'
  return new Date(ts).toLocaleString()
}

async function toggleDetail(id: string) {
  if (selectedId.value === id) {
    selectedId.value = null
    certInfo.value = null
    certInfoForId.value = null
    return
  }

  selectedId.value = id
  const record = records.value.find((r) => r.id === id)
  if (record?.status === 'issued' && record.certificateAvailable) {
    try {
      const res = await certificateService.recordDetail(id)
      certInfo.value = res.certificateInfo
      certInfoForId.value = id
    } catch (error) {
      certInfo.value = null
      toast.error(
        error instanceof Error ? error.message : t('certificate.history.detailLoadFailed'),
      )
    }
  }
}

async function handleRetry(record: AcmeApplicationRecord) {
  retryingId.value = record.id
  try {
    await certificateService.retryApplication(record.id)
    toast.info(t('certificate.history.notifications.retryStarted', { domain: record.domain }))
    await fetchHistory()
    emit('record-updated')
  } catch (e: any) {
    toast.error(e?.message || t('certificate.history.notifications.retryFailed'))
  } finally {
    retryingId.value = null
  }
}

async function handleDownload(record: AcmeApplicationRecord) {
  downloadingId.value = record.id
  try {
    const res = await certificateService.recordDetail(record.id)
    if (res.certificateInfo?.certificatePem) {
      copyPem(res.certificateInfo.certificatePem)
      toast.success(t('certificate.history.notifications.downloaded'))
    } else {
      toast.warning(t('certificate.history.notifications.certNotAvailable'))
    }
  } catch (e: any) {
    toast.error(e?.message || t('certificate.history.notifications.downloadFailed'))
  } finally {
    downloadingId.value = null
  }
}

function copyPem(pem: string) {
  navigator.clipboard.writeText(pem).then(() => {
    toast.success(t('certificate.pemCopied'))
  })
}

async function handleDelete(record: AcmeApplicationRecord) {
  await confirmDanger({
    title: t('certificate.history.deleteConfirmTitle'),
    content: t('certificate.history.deleteConfirmContent', { domain: record.domain }),
    onConfirm: async () => {
      try {
        await certificateService.deleteRecord(record.id)
        toast.success(t('certificate.history.notifications.deleted'))
        if (selectedId.value === record.id) {
          selectedId.value = null
          certInfo.value = null
        }
        await fetchHistory()
        emit('record-updated')
      } catch (e: any) {
        toast.error(e?.message || t('certificate.history.notifications.deleteFailed'))
      }
    },
  })
}

/** 监听 acme-verify:result 事件自动刷新 */
async function setupEventListener() {
  unlisten = await listen<{ success: boolean; error?: string; recordId?: string }>(
    'acme-verify:result',
    (event) => {
      const payload = event.payload
      if (payload.success) {
        toast.success(
          t('certificate.history.notifications.verifySuccess', {
            domain: (payload as any).data?.domain ?? '',
          }),
        )
      } else {
        toast.error(payload.error || t('certificate.wizard.dnsStep.verifyFailed'))
      }
      void fetchHistory()
      emit('record-updated')
      if (payload.recordId && selectedId.value === payload.recordId) {
        selectedId.value = null
        certInfo.value = null
      }
    },
  )
}

/* ── 生命周期 ── */
onMounted(() => {
  fetchHistory()
  setupEventListener()
})

onUnmounted(() => {
  unlisten?.()
})

watch(() => props.refreshTrigger, () => {
  void fetchHistory()
})
</script>

<style scoped>
/* ═══════════════ 面板容器 — 与 .cert-panel 完全一致 ═══════════════ */
.cert-panel--history {
  flex-shrink: 0;
}

/* ── 头部 ── */
.history-heading__left {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  min-width: 0;
}

.history-heading__left h3 {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin: 0;
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  white-space: nowrap;
}

/* 统计芯片 */
.history-chips {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.history-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 1px 10px;
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  font-variant-numeric: tabular-nums;
  color: var(--text-tertiary);
  background: var(--bg-input);
}
.history-chip.is-verifying {
  color: var(--color-warning);
  background: var(--color-warning-muted);
  animation: pulse-soft 2s ease-in-out infinite;
}
.history-chip.is-success {
  color: var(--color-success);
  background: var(--color-success-muted);
}
.history-chip.is-error {
  color: var(--color-error);
  background: var(--color-error-muted);
}

@keyframes pulse-soft {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.55; }
}

.history-heading__right {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  flex-shrink: 0;
}

.collapse-btn {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
  transition: transform 0.2s ease, background 0.15s;
}
.collapse-btn:hover {
  background: var(--bg-surface-hover);
}
.collapse-btn.collapsed {
  transform: rotate(-90deg);
}

/* ── 内容区 ── */
.history-body {
  min-height: 0;
}

/* 加载 / 空状态 */
.history-skeleton,
.history-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-8) var(--space-4);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}
.history-empty small {
  color: var(--text-quaternary, var(--text-tertiary));
  font-size: var(--text-xs);
}

/* ── 列表 ── */
.history-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

/* ── 行卡片 ── */
.history-row {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
  overflow: hidden;
}
.history-row:hover {
  border-color: var(--border-default);
  box-shadow: var(--shadow-xs);
}
.history-row.status-verifying {
  border-left: 3px solid var(--color-warning);
}
.history-row.status-issued {
  border-left: 3px solid var(--color-success);
}
.history-row.status-failed {
  border-left: 3px solid var(--color-error);
}
.history-row.status-pending {
  border-left: 3px solid var(--text-quaternary, var(--border-subtle));
}

.history-row__main {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  cursor: pointer;
  user-select: none;
}
.history-row__main:hover strong {
  color: var(--color-primary);
}

/* 左侧状态条 */
.history-row__indicator {
  width: 3px;
  height: 28px;
  flex-shrink: 0;
  border-radius: 2px;
  opacity: 0;
}
.history-row.status-verifying .history-row__indicator { background: var(--color-warning); opacity: 1; }
.history-row.status-issued .history-row__indicator { background: var(--color-success); opacity: 1; }
.history-row.status-failed .history-row__indicator { background: var(--color-error); opacity: 1; }

/* 状态徽标 */
.history-badge {
  width: 26px;
  height: 26px;
  flex-shrink: 0;
  display: grid;
  place-items: center;
  border-radius: var(--radius-full);
  font-size: 11px;
  color: #fff;
}
.history-badge.is-pending { background: var(--text-tertiary); }
.history-badge.is-verifying { background: var(--color-warning); color: #fff; }
.history-badge.is-issued { background: var(--color-success); }
.history-badge.is-failed { background: var(--color-error); }
.history-badge.is-expired { background: var(--border-subtle); color: var(--text-tertiary); }
.history-badge.sm {
  width: auto;
  height: auto;
  padding: 1px 8px;
  font-size: var(--text-xs);
  border-radius: var(--radius-full);
  display: inline-flex;
}

/* 信息区 */
.history-row__info {
  flex: 1;
  min-width: 0;
}
.history-row__info strong {
  display: block;
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: color 0.15s;
}
.history-meta {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: 2px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
.meta-divider { opacity: 0.35; }
.challenge-tag {
  padding: 0 6px;
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.02em;
}
.staging-tag {
  width: 16px;
  height: 16px;
  display: grid;
  place-items: center;
  border-radius: 3px;
  background: rgba(175, 82, 222, 0.12);
  color: #af52de;
  font-size: 9px;
  font-weight: 700;
}

/* 时间 */
.history-row__time {
  flex-shrink: 0;
  font-size: var(--text-xs);
  color: var(--text-quaternary, var(--text-tertiary));
  font-variant-numeric: tabular-nums;
  font-family: var(--font-mono);
}

/* 操作按钮 */
.history-row__ops {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: var(--space-2);
  opacity: 0;
  transition: opacity 0.15s;
}
.history-row:hover .history-row__ops,
.history-row__ops:focus-within {
  opacity: 1;
}

.op-btn {
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: all 0.15s;
}
.op-btn:hover {
  background: var(--bg-input);
  color: var(--text-primary);
}
.op-btn--danger:hover {
  background: var(--color-error-muted);
  color: var(--color-error);
}

/* ── 展开详情 ── */
.history-row__detail {
  border-top: 1px solid var(--border-subtle);
  padding: var(--space-4);
  background: var(--bg-input);
}

.detail-cols {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: var(--space-3) var(--space-5);
}

.detail-col {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.detail-col dt {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-tertiary);
  font-weight: var(--weight-medium);
}
.detail-col dd {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  font-family: var(--font-mono);
}
.text-warn { color: var(--color-warning); }
.days-hint {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 10px;
}

/* 错误块 */
.detail-error {
  margin-top: var(--space-3);
  padding-top: var(--space-3);
  border-top: 1px dashed var(--border-subtle);
  display: flex;
  flex-direction: column;
  gap: 3px;
  grid-column: 1 / -1;
}
.detail-error dt {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-error);
  font-weight: var(--weight-medium);
}
.detail-error dd {
  font-size: var(--text-xs);
  color: var(--color-error);
  line-height: 1.55;
  word-break: break-word;
  max-height: 80px;
  overflow-y: auto;
  padding: var(--space-2);
  border-radius: var(--radius-sm);
  background: rgba(255, 69, 58, 0.06);
}

/* ── 证书下载区 ── */
.cert-pem-block {
  margin-top: var(--space-3);
  padding-top: var(--space-3);
  border-top: 1px dashed var(--border-subtle);
}
.pem-file {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  border: 1px solid var(--border-subtle);
}
.pem-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.pem-info strong {
  font-size: var(--text-sm);
  color: var(--text-primary);
}
.pem-info small {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

/* ── 动画 ── */
.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.25s ease;
  overflow: hidden;
}
.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
  margin-top: 0;
  margin-bottom: 0;
}

.slide-down-enter-active,
.slide-down-leave-active {
  transition: all 0.25s ease;
  overflow: hidden;
}
.slide-down-enter-from,
.slide-down-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}
</style>
