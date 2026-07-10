<template>
  <section class="cert-page">
    <!-- ═══════════════ 顶部 Hero ═══════════════ -->
    <header class="cert-hero">
      <div>
        <p class="cert-hero__eyebrow">{{ t('certificate.eyebrow') }}</p>
        <h1>{{ t('certificate.title') }}</h1>
        <span class="cert-hero__subtitle">{{ t('certificate.subtitle') }}</span>
      </div>
      <div class="cert-hero__actions">
        <GButton variant="secondary" icon="refresh" :loading="loading" @click="refresh">
          {{ t('certificate.refresh') }}
        </GButton>
        <GButton variant="secondary" icon="upload" @click="importVisible = true">
          {{ t('certificate.import') }}
        </GButton>
        <GButton variant="primary" icon="plus" @click="wizardVisible = true">
          {{ t('certificate.request') }}
        </GButton>
      </div>
    </header>

    <!-- ═══════════════ 加载骨架 ═══════════════ -->
    <CertLoading v-if="loading && certificates.length === 0" />

    <!-- ═══════════════ 错误状态 ═══════════════ -->
    <GCard v-else-if="error" variant="plain" padding="lg">
      <GErrorState
        :title="t('certificate.loading')"
        :message="error"
        retry
        @retry="refresh" />
    </GCard>

    <!-- ═══════════════ 空状态 ═══════════════ -->
    <div v-else-if="certificates.length === 0" class="cert-empty">
      <div class="cert-empty__illustration">
        <GIcon name="shield-check" :size="38" />
      </div>
      <h2>{{ t('certificate.emptyTitle') }}</h2>
      <p>{{ t('certificate.emptyDescription') }}</p>
      <div class="cert-empty__features">
        <span><GIcon name="check" :size="14" /> {{ t('certificate.emptyFeatures.https') }}</span>
        <span><GIcon name="check" :size="14" /> {{ t('certificate.emptyFeatures.autoRenewal') }}</span>
        <span><GIcon name="check" :size="14" /> {{ t('certificate.emptyFeatures.tls13') }}</span>
        <span><GIcon name="check" :size="14" /> {{ t('certificate.emptyFeatures.sni') }}</span>
      </div>
      <div class="cert-empty__actions">
        <GButton variant="primary" icon="plus" @click="wizardVisible = true">
          {{ t('certificate.request') }}
        </GButton>
        <GButton variant="secondary" icon="upload" @click="importVisible = true">
          {{ t('certificate.import') }}
        </GButton>
      </div>
    </div>

    <!-- ═══════════════ 主内容 ═══════════════ -->
    <template v-else>
      <!-- 第一行：统计卡片 -->
      <section class="cert-stats">
        <article class="cert-stat-card" :class="`is-${statTone('total')}`">
          <div class="cert-stat-card__body">
            <p>{{ t('certificate.stats.total') }}</p>
            <strong class="cert-stat-card__value">{{ animatedStats.total }}</strong>
          </div>
          <span class="cert-stat-card__icon"><GIcon name="shield-check" :size="20" /></span>
        </article>

        <article class="cert-stat-card is-success">
          <div class="cert-stat-card__body">
            <p>{{ t('certificate.stats.active') }}</p>
            <strong class="cert-stat-card__value">{{ animatedStats.active }}</strong>
          </div>
          <span class="cert-stat-card__icon"><GIcon name="check-circle" :size="20" /></span>
        </article>

        <article class="cert-stat-card" :class="stats.expiringSoon > 0 ? 'is-warning' : 'is-neutral'">
          <div class="cert-stat-card__body">
            <p>{{ t('certificate.stats.expiringSoon') }}</p>
            <strong class="cert-stat-card__value">{{ animatedStats.expiringSoon }}</strong>
            <small>{{ t('certificate.stats.expiringSoonHint') }}</small>
          </div>
          <span class="cert-stat-card__icon"><GIcon name="clock" :size="20" /></span>
        </article>

        <article class="cert-stat-card" :class="stats.autoRenewalFailed > 0 ? 'is-error' : 'is-success'">
          <div class="cert-stat-card__body">
            <p>{{ t('certificate.stats.autoRenewal') }}</p>
            <strong class="cert-stat-card__value">
              {{ stats.autoRenewalFailed > 0
                ? t('certificate.stats.autoRenewalFailed', { count: stats.autoRenewalFailed })
                : t('certificate.stats.autoRenewalNormal') }}
            </strong>
          </div>
          <span class="cert-stat-card__icon"><GIcon name="refresh" :size="20" /></span>
        </article>
      </section>

      <!-- 第二行：可视化面板 -->
      <section class="cert-viz-grid">
        <!-- 健康评分 -->
        <article class="cert-panel cert-panel--health">
          <div class="cert-panel__heading">
            <h3>{{ t('certificate.health.title') }}</h3>
          </div>
          <div class="health-score">
            <svg viewBox="0 0 120 120" class="health-score__svg">
              <circle cx="60" cy="60" r="52" class="health-score__track" />
              <circle
                cx="60" cy="60" r="52"
                class="health-score__fill"
                :class="`is-${healthTone}`"
                :stroke-dasharray="healthCircumference"
                :stroke-dashoffset="healthDashOffset" />
            </svg>
            <div class="health-score__center">
              <strong>{{ animatedHealthScore }}</strong>
              <small>{{ t(`certificate.health.${healthChecksAllPassed ? 'allGood' : 'needsAttention'}`) }}</small>
            </div>
          </div>
          <div class="health-checks">
            <span v-for="(check, key) in healthCheckItems" :key="key" class="health-check" :class="{ 'is-ok': check }">
              <GIcon :name="check ? 'check' : 'alert-circle'" :size="14" />
              {{ checkLabels[key] }}
            </span>
          </div>
        </article>

        <!-- 状态分布圆环图 -->
        <article class="cert-panel cert-panel--donut">
          <div class="cert-panel__heading">
            <h3>{{ t('certificate.donut.title') }}</h3>
          </div>
          <div class="donut-chart">
            <div class="donut-chart__ring" :style="donutStyle">
              <span>{{ stats.total }}</span>
              <small>{{ t('certificate.donut.total') }}</small>
            </div>
            <div class="donut-chart__legend">
              <article v-for="item in donutData" :key="item.key">
                <i :style="{ background: item.color }" />
                <span>{{ item.label }}</span>
                <strong>{{ item.count }} ({{ item.percent }})</strong>
              </article>
            </div>
          </div>
        </article>

        <!-- 趋势图 -->
        <article class="cert-panel cert-panel--trend">
          <div class="cert-panel__heading">
            <h3>{{ t('certificate.trend.title') }}</h3>
          </div>
          <div v-if="trendData.length" class="trend-chart">
            <svg viewBox="0 0 320 120" class="trend-chart__svg">
              <g class="trend-chart__grid">
                <line v-for="i in 4" :key="`grid-${i}`" x1="0" :x2="320" :y1="i * 24" :y2="i * 24" />
              </g>
              <polyline
                v-for="series in trendSeries"
                :key="series.name"
                :points="series.points"
                :stroke="series.color"
                class="trend-chart__line" />
            </svg>
            <div class="trend-chart__legend">
              <span v-for="series in trendSeries" :key="series.name">
                <i :style="{ background: series.color }" />
                {{ series.name }}
              </span>
            </div>
          </div>
          <div v-else class="trend-empty">
            <GIcon name="chart-line" :size="24" />
            <span>{{ t('certificate.trend.noData') }}</span>
          </div>
        </article>
      </section>

      <!-- 自动续期面板 -->
      <section class="cert-panel cert-panel--renewal">
        <div class="cert-panel__heading">
          <h3>{{ t('certificate.autoRenewalPanel.title') }}</h3>
          <GButton
            variant="ghost"
            size="sm"
            icon="zap"
            :loading="renewalExecuting"
            @click="executeRenewal">
            {{ t('certificate.autoRenewalPanel.executeNow') }}
          </GButton>
        </div>
        <div class="renewal-grid">
          <article>
            <span>{{ t('certificate.autoRenewalPanel.enabled') }}</span>
            <strong :class="renewalStatus.enabled ? 'is-success' : 'is-error'">
              {{ renewalStatus.enabled ? t('certificate.autoRenewalPanel.enabled') : t('certificate.autoRenewalPanel.disabled') }}
            </strong>
          </article>
          <article>
            <span>{{ t('certificate.autoRenewalPanel.nextCheck') }}</span>
            <strong>{{ t('certificate.autoRenewalPanel.hours', { hours: renewalStatus.nextCheckHours }) }}</strong>
          </article>
          <article>
            <span>{{ t('certificate.autoRenewalPanel.schedule') }}</span>
            <strong>Every {{ Math.round(renewalStatus.checkIntervalSeconds / 3600) }}h</strong>
          </article>
          <article>
            <span>{{ t('certificate.autoRenewalPanel.lastRun') }}</span>
            <strong :class="renewalStatus.lastRenewSuccess ? 'is-success' : 'is-error'">
              {{ renewalStatus.lastRenewTime
                ? (renewalStatus.lastRenewSuccess ? t('certificate.autoRenewalPanel.success') : t('certificate.autoRenewalPanel.failed'))
                : t('certificate.autoRenewalPanel.noLastRun') }}
            </strong>
          </article>
        </div>
        <p v-if="!renewalStatus.enabled" class="renewal-warning">
          <GIcon name="alert-triangle" :size="14" />
          {{ t('certificate.autoRenewalPanel.notConfigured') }}
        </p>
      </section>

      <!-- 第三行：列表 + 详情 -->
      <div class="cert-workspace">
        <!-- 左侧列表 -->
        <aside class="cert-list" :aria-label="t('certificate.listAria')">
          <!-- 工具栏 -->
          <div class="cert-toolbar">
            <label class="cert-search">
              <GIcon name="search" :size="15" />
              <input v-model.trim="query" :placeholder="t('certificate.searchPlaceholder')" />
            </label>
            <select v-model="filter">
              <option value="all">{{ t('certificate.filters.all') }}</option>
              <option value="active">{{ t('certificate.filters.active') }}</option>
              <option value="expiringSoon">{{ t('certificate.filters.expiringSoon') }}</option>
              <option value="expired">{{ t('certificate.filters.expired') }}</option>
              <option value="failed">{{ t('certificate.filters.failed') }}</option>
            </select>
            <select v-model="sortBy">
              <option value="updatedAt">{{ t('certificate.sort.updatedAt') }}</option>
              <option value="expireTime">{{ t('certificate.sort.expireTime') }}</option>
              <option value="domain">{{ t('certificate.sort.domain') }}</option>
              <option value="status">{{ t('certificate.sort.status') }}</option>
            </select>
          </div>

          <div class="cert-list__header">
            <strong>{{ t('certificate.resultCount', { count: filteredCerts.length }) }}</strong>
            <span>{{ query ? t('certificate.matching', { query }) : t('certificate.ready') }}</span>
          </div>

          <!-- 证书卡片列表 -->
          <button
            v-for="cert in filteredCerts"
            :key="cert.domain"
            type="button"
            class="cert-card"
            :class="{ active: selectedDomain === cert.domain }"
            @click="selectCert(cert.domain)">
            <span class="cert-card__status" :class="`is-${statusTone(cert.status)}`" />
            <span class="cert-card__icon" :class="`is-${statusTone(cert.status)}`">
              <GIcon name="globe" :size="18" />
            </span>
            <div class="cert-card__main">
              <strong>{{ cert.domain }}</strong>
              <small>{{ cert.issuer }} · {{ cert.algorithm }}</small>
            </div>
            <div class="cert-card__meta">
              <span :class="`cert-badge is-${statusTone(cert.status)}`">
                {{ statusLabel(cert.status) }}
              </span>
              <small v-if="cert.daysRemaining >= 0">
                {{ t('certificate.card.daysRemaining', { days: cert.daysRemaining }) }}
              </small>
              <small v-else class="is-expired">{{ t('certificate.card.expired') }}</small>
            </div>
          </button>

          <div v-if="!filteredCerts.length" class="cert-list__empty">
            <GIcon name="search" :size="24" />
            <span>{{ t('certificate.noMatching') }}</span>
          </div>
        </aside>

        <!-- 右侧详情 -->
        <main class="cert-detail" aria-live="polite">
          <template v-if="selectedDetail">
            <!-- 详情头部 -->
            <div class="detail-header">
              <div>
                <div class="detail-title-row">
                  <span :class="`is-${statusTone(selectedDetail.summary.status)}`" />
                  <h2>{{ selectedDetail.summary.domain }}</h2>
                </div>
                <p>{{ selectedDetail.summary.issuer }} · {{ selectedDetail.summary.algorithm }}</p>
              </div>
              <div class="detail-actions">
                <GButton variant="secondary" size="sm" icon="copy" @click="copyPem(selectedDetail.summary.domain)">
                  {{ t('certificate.copyPem') }}
                </GButton>
                <GButton variant="secondary" size="sm" icon="download" @click="exportPem(selectedDetail.summary.domain)">
                  {{ t('certificate.exportPem') }}
                </GButton>
                <GButton variant="secondary" size="sm" icon="refresh" :loading="actionLoading === 'redeploy'" @click="redeployCert(selectedDetail.summary.domain)">
                  {{ t('certificate.redeploy') }}
                </GButton>
                <GButton variant="secondary" size="sm" icon="zap" :loading="actionLoading === 'renew'" @click="renewCert(selectedDetail.summary.domain)">
                  {{ t('certificate.renewNow') }}
                </GButton>
                <button type="button" class="icon-action" @click="deleteCert(selectedDetail.summary.domain)">
                  <GIcon name="trash" :size="16" />
                </button>
              </div>
            </div>

            <!-- 生命周期时间轴 -->
            <section class="cert-lifecycle">
              <div class="cert-panel__heading">
                <h3>{{ t('certificate.lifecycle.title') }}</h3>
              </div>
              <div class="lifecycle-timeline">
                <div
                  v-for="(stage, i) in lifecycleStages"
                  :key="stage.key"
                  class="lifecycle-stage"
                  :class="{ 'is-done': stage.done, 'is-current': stage.current }">
                  <span class="lifecycle-stage__dot">
                    <GIcon v-if="stage.done" name="check" :size="12" />
                    <GIcon v-else name="circle" :size="10" />
                  </span>
                  <div class="lifecycle-stage__body">
                    <strong>{{ stage.label }}</strong>
                    <small v-if="stage.time">{{ stage.time }}</small>
                  </div>
                  <span v-if="i < lifecycleStages.length - 1" class="lifecycle-stage__line" :class="{ 'is-done': stage.done }" />
                </div>
              </div>
            </section>

            <!-- 指标网格 -->
            <div class="detail-metrics">
              <article>
                <span>{{ t('certificate.fields.issuedAt') }}</span>
                <strong>{{ formatDate(selectedDetail.summary.createTime) }}</strong>
              </article>
              <article>
                <span>{{ t('certificate.fields.expiresAt') }}</span>
                <strong>{{ formatDate(selectedDetail.summary.expireTime) }}</strong>
              </article>
              <article>
                <span>{{ t('certificate.fields.daysRemaining') }}</span>
                <strong :class="{ 'is-warning': selectedDetail.summary.daysRemaining <= 30, 'is-error': selectedDetail.summary.daysRemaining < 0 }">
                  {{ selectedDetail.summary.daysRemaining }}
                </strong>
              </article>
              <article>
                <span>{{ t('certificate.fields.algorithm') }}</span>
                <strong>{{ selectedDetail.summary.algorithm }}</strong>
              </article>
              <article>
                <span>{{ t('certificate.fields.tlsVersion') }}</span>
                <strong>{{ selectedDetail.summary.tlsVersion }}</strong>
              </article>
              <article>
                <span>{{ t('certificate.fields.deployStatus') }}</span>
                <strong :class="`is-${deployTone(selectedDetail.summary.deployStatus)}`">
                  {{ deployLabel(selectedDetail.summary.deployStatus) }}
                </strong>
              </article>
            </div>

            <!-- 自动续期开关 + SAN -->
            <div class="detail-grid">
              <section class="cert-detail-card">
                <div class="cert-panel__heading">
                  <h3>{{ t('certificate.fields.autoRenewal') }}</h3>
                </div>
                <div class="auto-renewal-toggle">
                  <button
                    type="button"
                    class="toggle-switch"
                    :class="{ on: selectedDetail.summary.autoRenewalEnabled }"
                    @click="toggleAutoRenewal(selectedDetail.summary.domain, !selectedDetail.summary.autoRenewalEnabled)">
                    <span class="toggle-switch__knob" />
                  </button>
                  <span>{{ selectedDetail.summary.autoRenewalEnabled ? t('certificate.autoRenewalOn') : t('certificate.autoRenewalOff') }}</span>
                </div>
                <dl class="detail-info">
                  <div>
                    <dt>{{ t('certificate.fields.lastRenewTime') }}</dt>
                    <dd>{{ selectedDetail.summary.renewTime ? formatDate(selectedDetail.summary.renewTime) : '-' }}</dd>
                  </div>
                  <div v-if="selectedDetail.summary.lastError">
                    <dt>{{ t('certificate.fields.lastError') }}</dt>
                    <dd class="is-error">{{ selectedDetail.summary.lastError }}</dd>
                  </div>
                </dl>
              </section>

              <section class="cert-detail-card">
                <div class="cert-panel__heading">
                  <h3>{{ t('certificate.fields.san') }}</h3>
                </div>
                <div class="san-list">
                  <span v-for="name in selectedDetail.summary.san" :key="name" class="san-tag">{{ name }}</span>
                  <span v-if="!selectedDetail.summary.san.length" class="is-empty">-</span>
                </div>
              </section>
            </div>

            <!-- 域名关联 -->
            <section class="cert-detail-card">
              <div class="cert-panel__heading">
                <h3>{{ t('certificate.associations.title') }}</h3>
              </div>
              <div v-if="domainAssociations" class="associations">
                <div class="assoc-section">
                  <span class="assoc-label">{{ t('certificate.associations.domains') }}</span>
                  <div class="assoc-list">
                    <span v-for="d in domainAssociations.domains" :key="d.host" class="assoc-tag" :class="`is-${d.status}`">
                      <GIcon name="globe" :size="12" />
                      {{ d.host }}
                    </span>
                    <span v-if="!domainAssociations.domains.length" class="is-empty">
                      {{ domainAssociations.dbAvailable ? t('certificate.associations.noAssociations') : t('certificate.associations.dbUnavailable') }}
                    </span>
                  </div>
                </div>
                <div class="assoc-section">
                  <span class="assoc-label">{{ t('certificate.associations.tunnels') }}</span>
                  <div class="assoc-list">
                    <span v-for="t in domainAssociations.tunnels" :key="t.tunnelId" class="assoc-tag">
                      <GIcon name="router" :size="12" />
                      {{ t.tunnelId }}
                    </span>
                    <span v-if="!domainAssociations.tunnels.length" class="is-empty">-</span>
                  </div>
                </div>
              </div>
              <div v-else class="assoc-loading">
                <GIcon name="loader" :size="16" spin />
              </div>
            </section>

            <!-- PEM 预览 -->
            <section class="cert-detail-card">
              <div class="cert-panel__heading">
                <h3>PEM</h3>
                <GButton variant="ghost" size="sm" icon="copy" @click="copyPem(selectedDetail.summary.domain)">
                  {{ t('certificate.copyPem') }}
                </GButton>
              </div>
              <pre v-if="selectedDetail.certificatePem" class="pem-viewer">{{ selectedDetail.certificatePem }}</pre>
              <p v-else class="is-empty">{{ t('certificate.noPem') }}</p>
            </section>
          </template>

          <div v-else class="cert-detail__placeholder">
            <GIcon name="shield-check" :size="34" />
            <span>{{ t('certificate.selectPrompt') }}</span>
          </div>
        </main>
      </div>
    </template>

    <!-- 申请历史（独立面板，无证书时也显示） -->
    <CertHistory :refresh-trigger="historyRefreshTrigger" @record-updated="onHistoryRecordUpdated" />

    <!-- ═══════════════ 弹窗 ═══════════════ -->
    <CertWizard
      v-model:visible="wizardVisible"
      :servers="serverOptions"
      @submitted="handleWizardSubmitted" />

    <CertImportDialog
      v-model:visible="importVisible"
      @imported="handleImported" />
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GCard from '@components/base/GCard.vue'
import GIcon from '@components/icons/GIcon.vue'
import GErrorState from '@components/feedback/GErrorState.vue'
import { useFeedback } from '@composables/useFeedback'
import { certificateService } from './service'
import CertWizard from './components/CertWizard.vue'
import CertImportDialog from './components/CertImportDialog.vue'
import CertLoading from './components/CertLoading.vue'
import CertHistory from './components/CertHistory.vue'
import type {
  CertificateDetailResponse,
  CertificateDomainAssociations,
  CertificateStats,
  CertificateSummary,
  CertFilterType,
  CertSortType,
  AutoRenewalStatusResponse,
} from './types'
import { useServerStore } from '@views/servers'

const { t, locale } = useI18n()
const { toast, notify, confirm, confirmDanger } = useFeedback()
const serverStore = useServerStore()

/* ────────── 状态 ────────── */
const certificates = ref<CertificateSummary[]>([])
const selectedDomain = ref<string | null>(null)
const selectedDetail = ref<CertificateDetailResponse | null>(null)
const domainAssociations = ref<CertificateDomainAssociations | null>(null)
const stats = ref<CertificateStats>({
  total: 0, active: 0, expiringSoon: 0, expired: 0, failed: 0,
  autoRenewalOk: 0, autoRenewalFailed: 0, healthScore: 0,
  statusDistribution: { active: 0, expiringSoon: 0, expired: 0, failed: 0 },
  healthChecks: { autoRenewal: false, acme: false, dns: false, http01: false, tls13: false, sni: false },
  generatedAt: 0,
})
const renewalStatus = ref<AutoRenewalStatusResponse>({
  enabled: false, acmeStaging: false, acmeHttp01Port: 80,
  checkIntervalSeconds: 86400, renewBeforeDays: 30, scheduleDescription: '',
  lastRenewSuccess: true, nextCheckHours: 0, generatedAt: 0,
  lastRenewTime: null, lastError: null, acmeEmail: null, acmeDirectoryUrl: null,
})
const loading = ref(false)
const error = ref('')
const actionLoading = ref<string | null>(null)
const renewalExecuting = ref(false)
const wizardVisible = ref(false)
const importVisible = ref(false)
const historyRefreshTrigger = ref(0)

const query = ref('')
const filter = ref<CertFilterType>('all')
const sortBy = ref<CertSortType>('updatedAt')
const direction = ref<'asc' | 'desc'>('desc')

/* ────────── 动画数值 ────────── */
const animatedStats = ref({ total: 0, active: 0, expiringSoon: 0 })
const animatedHealthScore = ref(0)

const serverOptions = computed(() =>
  serverStore.servers.map((s) => ({
    id: s.id,
    name: s.name,
    status: s.status,
    publicIp: s.publicIp,
    host: s.overview?.hostname ?? s.publicIp,
  })),
)

/* ────────── 计算属性 ────────── */
const filteredCerts = computed(() => {
  const keyword = query.value.toLowerCase()
  const filtered = certificates.value.filter((cert) => {
    const matchesFilter =
      filter.value === 'all' ||
      (filter.value === 'active' && cert.status === 'active') ||
      (filter.value === 'expiringSoon' && cert.status === 'expiringSoon') ||
      (filter.value === 'expired' && cert.status === 'expired') ||
      (filter.value === 'failed' && cert.status === 'failed')
    const matchesQuery =
      !keyword ||
      [cert.domain, cert.issuer, cert.algorithm, ...cert.san]
        .join(' ')
        .toLowerCase()
        .includes(keyword)
    return matchesFilter && matchesQuery
  })

  return [...filtered].sort((a, b) => {
    const modifier = direction.value === 'asc' ? 1 : -1
    if (sortBy.value === 'domain') return a.domain.localeCompare(b.domain) * modifier
    if (sortBy.value === 'expireTime') return (new Date(a.expireTime).getTime() - new Date(b.expireTime).getTime()) * modifier
    if (sortBy.value === 'status') return (statusOrder(a.status) - statusOrder(b.status)) * modifier
    return (new Date(a.createTime).getTime() - new Date(b.createTime).getTime()) * modifier
  })
})

const healthTone = computed(() => {
  const score = stats.value.healthScore
  if (score >= 90) return 'success'
  if (score >= 60) return 'warning'
  return 'error'
})

const healthChecksAllPassed = computed(() => Object.values(stats.value.healthChecks).every(Boolean))

const healthCircumference = 2 * Math.PI * 52
const healthDashOffset = computed(() => {
  const ratio = Math.max(0, Math.min(1, stats.value.healthScore / 100))
  return healthCircumference * (1 - ratio)
})

const healthCheckItems = computed(() => stats.value.healthChecks)
const checkLabels = computed<Record<string, string>>(() => ({
  autoRenewal: t('certificate.health.autoRenewal'),
  acme: t('certificate.health.autoRenewal'),
  dns: t('certificate.health.dns'),
  http01: t('certificate.health.http01'),
  tls13: t('certificate.health.tls13'),
  sni: t('certificate.health.sni'),
}))

const donutData = computed(() => {
  const dist = stats.value.statusDistribution
  const total = Math.max(1, dist.active + dist.expiringSoon + dist.expired + dist.failed)
  return [
    { key: 'active', label: t('certificate.donut.active'), count: dist.active, color: '#2fd17c', percent: `${Math.round((dist.active / total) * 100)}%` },
    { key: 'expiringSoon', label: t('certificate.donut.expiringSoon'), count: dist.expiringSoon, color: '#f5b84b', percent: `${Math.round((dist.expiringSoon / total) * 100)}%` },
    { key: 'expired', label: t('certificate.donut.expired'), count: dist.expired, color: '#ff5c5c', percent: `${Math.round((dist.expired / total) * 100)}%` },
    { key: 'failed', label: t('certificate.donut.failed'), count: dist.failed, color: '#9b8cff', percent: `${Math.round((dist.failed / total) * 100)}%` },
  ].filter((item) => item.count > 0)
})

const donutStyle = computed(() => {
  const items = donutData.value
  if (!items.length) return {}
  const total = items.reduce((sum, item) => sum + item.count, 0)
  let cursor = 0
  const stops = items.map((item) => {
    const start = cursor
    cursor += (item.count / total) * 100
    return `${item.color} ${start}% ${cursor}%`
  })
  return { background: `conic-gradient(${stops.join(', ')})` }
})

const trendData = computed(() => {
  // 从证书记录中提取近 30 天的签发/续期/失败趋势
  // 如果没有足够的历史数据，返回空数组
  const now = Date.now()
  const buckets = Array.from({ length: 30 }, (_, i) => ({
    day: i,
    issued: 0,
    renewed: 0,
    failed: 0,
  }))

  for (const cert of certificates.value) {
    const created = new Date(cert.createTime).getTime()
    const ageDays = Math.floor((now - created) / 86400000)
    if (ageDays >= 0 && ageDays < 30) {
      buckets[29 - ageDays].issued++
    }
    if (cert.renewTime) {
      const renewed = new Date(cert.renewTime).getTime()
      const renewAge = Math.floor((now - renewed) / 86400000)
      if (renewAge >= 0 && renewAge < 30) {
        buckets[29 - renewAge].renewed++
      }
    }
    if (cert.status === 'failed' || cert.lastError) {
      buckets[29].failed++
    }
  }

  return buckets
})

const trendSeries = computed(() => {
  if (!trendData.value.length) return []
  const maxVal = Math.max(1, ...trendData.value.flatMap((b) => [b.issued, b.renewed, b.failed]))
  const toPoints = (key: 'issued' | 'renewed' | 'failed') =>
    trendData.value
      .map((b, i) => `${(i / (trendData.value.length - 1)) * 320},${120 - (b[key] / maxVal) * 100}`)
      .join(' ')
  return [
    { name: t('certificate.trend.issued'), color: '#5b8def', points: toPoints('issued') },
    { name: t('certificate.trend.renewed'), color: '#2fd17c', points: toPoints('renewed') },
    { name: t('certificate.trend.failed'), color: '#ff5c5c', points: toPoints('failed') },
  ]
})

const lifecycleStages = computed(() => {
  if (!selectedDetail.value) return []
  const s = selectedDetail.value.summary
  return [
    { key: 'requested', label: t('certificate.lifecycle.requested'), done: true, current: false, time: formatDate(s.createTime) },
    { key: 'issued', label: t('certificate.lifecycle.issued'), done: s.status !== 'pending', current: s.status === 'pending', time: formatDate(s.createTime) },
    { key: 'deployed', label: t('certificate.lifecycle.deployed'), done: s.deployStatus === 'deployed', current: s.deployStatus === 'pending', time: s.deployStatus === 'deployed' ? formatDate(s.createTime) : '' },
    { key: 'autoRenewal', label: t('certificate.lifecycle.autoRenewal'), done: !!s.renewTime, current: s.autoRenewalEnabled && !s.renewTime, time: s.renewTime ? formatDate(s.renewTime) : '' },
    { key: 'latest', label: t('certificate.lifecycle.latest'), done: s.status === 'active', current: s.status !== 'active' && s.status !== 'failed', time: '' },
  ]
})

/* ────────── 生命周期 ────────── */
onMounted(() => {
  void refresh()
  if (serverStore.status === 'idle') {
    void serverStore.load()
  }
})

watch(selectedDomain, (domain) => {
  if (domain) {
    void loadDetail(domain)
    void loadAssociations(domain)
  } else {
    selectedDetail.value = null
    domainAssociations.value = null
  }
})

watch(
  () => stats.value,
  (newStats) => {
    animateNumber('total', newStats.total)
    animateNumber('active', newStats.active)
    animateNumber('expiringSoon', newStats.expiringSoon)
  },
  { deep: true },
)

/* 每次打开证书向导时，强制刷新服务器列表以获取最新状态
   修复：应用重启后首次打开向导可能显示"离线"(后端连接尚未重建完成)，
   而服务器实际已运行的状态不一致问题 */
watch(wizardVisible, (visible) => {
  if (visible) {
    void serverStore.load()
  }
})

watch(
  () => stats.value.healthScore,
  (score) => {
    animateHealth(score)
  },
)

/* ────────── 数据加载 ────────── */
async function refresh() {
  loading.value = true
  error.value = ''
  try {
    const [listRes, statsRes, renewalRes] = await Promise.all([
      certificateService.list(),
      certificateService.stats().catch(() => null),
      certificateService.autoRenewalStatus().catch(() => null),
    ])
    certificates.value = listRes.certificates
    if (statsRes) stats.value = statsRes
    if (renewalRes) renewalStatus.value = renewalRes

    // 自动选中第一个
    if (!selectedDomain.value && certificates.value.length) {
      selectedDomain.value = certificates.value[0].domain
    }
  } catch (source) {
    error.value = source instanceof Error ? source.message : String(source)
  } finally {
    loading.value = false
  }
}

async function loadDetail(domain: string) {
  try {
    selectedDetail.value = await certificateService.detail(domain)
  } catch (source) {
    const msg = source instanceof Error ? source.message : String(source)
    notify.error(t('certificate.loading'), msg, 10000)
  }
}

async function loadAssociations(domain: string) {
  domainAssociations.value = null
  try {
    domainAssociations.value = await certificateService.domainAssociations(domain)
  } catch {
    // 域名数据库不可用时静默处理
  }
}

function selectCert(domain: string) {
  selectedDomain.value = domain
}

/* ────────── 操作 ────────── */
async function exportPem(domain: string) {
  try {
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
  } catch (source) {
    const msg = source instanceof Error ? source.message : String(source)
    notify.error(t('certificate.exportPem'), msg, 10000)
  }
}

async function copyPem(domain: string) {
  try {
    const pem = await certificateService.exportPem(domain)
    await navigator.clipboard.writeText(pem)
    toast.success(t('certificate.pemCopied'))
  } catch (source) {
    const msg = source instanceof Error ? source.message : String(source)
    notify.error(t('certificate.copyPem'), msg, 10000)
  }
}

function deleteCert(domain: string) {
  confirmDanger({
    title: t('certificate.deleteConfirmTitle'),
    content: t('certificate.deleteConfirmContent', { domain }),
    confirmText: t('certificate.delete'),
    onConfirm: async () => {
      try {
        await certificateService.delete(domain)
        toast.success(t('certificate.notifications.deleted', { domain }))
        await refresh()
        selectedDomain.value = certificates.value[0]?.domain ?? null
      } catch (source) {
        const msg = source instanceof Error ? source.message : String(source)
        notify.error(t('certificate.notifications.deleteFailed'), msg, 10000)
      }
    },
  })
}

function renewCert(domain: string) {
  confirm({
    title: t('certificate.renewConfirmTitle'),
    content: t('certificate.renewConfirmContent', { domain }),
    confirmText: t('certificate.renewNow'),
    onConfirm: async () => {
      actionLoading.value = 'renew'
      try {
        await certificateService.renewNow(domain)
        toast.success(t('certificate.notifications.renewTriggered', { domain }))
        await refresh()
      } catch (source) {
        const msg = source instanceof Error ? source.message : String(source)
        notify.error(t('certificate.notifications.renewFailed'), msg, 10000)
      } finally {
        actionLoading.value = null
      }
    },
  })
}

function redeployCert(domain: string) {
  confirm({
    title: t('certificate.redeployConfirmTitle'),
    content: t('certificate.redeployConfirmContent', { domain }),
    confirmText: t('certificate.redeploy'),
    onConfirm: async () => {
      actionLoading.value = 'redeploy'
      try {
        await certificateService.redeploy(domain)
        toast.success(t('certificate.notifications.redeployTriggered', { domain }))
        await refresh()
      } catch (source) {
        const msg = source instanceof Error ? source.message : String(source)
        notify.error(t('certificate.notifications.redeployFailed'), msg, 10000)
      } finally {
        actionLoading.value = null
      }
    },
  })
}

async function toggleAutoRenewal(domain: string, enabled: boolean) {
  try {
    await certificateService.toggleAutoRenewal(domain, enabled)
    toast.success(t('certificate.notifications.autoRenewalToggled', {
      action: enabled ? t('certificate.autoRenewalOn') : t('certificate.autoRenewalOff'),
    }))
    if (selectedDomain.value === domain) {
      await loadDetail(domain)
    }
  } catch (source) {
    const msg = source instanceof Error ? source.message : String(source)
    notify.error(t('certificate.notifications.autoRenewalToggleFailed'), msg, 10000)
  }
}

async function executeRenewal() {
  renewalExecuting.value = true
  try {
    // 对所有即将到期的证书触发续期
    const dueCerts = certificates.value.filter(
      (c) => c.autoRenewalEnabled && c.daysRemaining <= 30,
    )
    for (const cert of dueCerts) {
      try {
        await certificateService.renewNow(cert.domain)
      } catch {
        // 单个失败不中断整体
      }
    }
    toast.success(t('certificate.notifications.renewTriggered', { domain: `${dueCerts.length} certs` }))
    await refresh()
  } catch (source) {
    const msg = source instanceof Error ? source.message : String(source)
    notify.error(t('certificate.notifications.executeRenewalFailed'), msg, 10000)
  } finally {
    renewalExecuting.value = false
  }
}

function handleWizardSubmitted() {
  void refresh()
  historyRefreshTrigger.value++
  setTimeout(() => { wizardVisible.value = false }, 1500)
}

function handleImported(domain: string) {
  void refresh()
  selectedDomain.value = domain
  historyRefreshTrigger.value++
}

function onHistoryRecordUpdated() {
  // 申请记录状态变化（如验证通过/失败），刷新证书列表
  void refresh()
}

/* ────────── 工具函数 ────────── */
function statusTone(status: string): string {
  if (status === 'active') return 'online'
  if (status === 'expiringSoon' || status === 'pending') return 'warning'
  if (status === 'expired' || status === 'failed' || status === 'revoked') return 'error'
  return 'offline'
}

function statusLabel(status: string) {
  return t(`certificate.status.${status}`)
}

function deployTone(status: string) {
  if (status === 'deployed') return 'online'
  if (status === 'pending') return 'warning'
  return 'error'
}

function deployLabel(status: string) {
  return t(`certificate.deploy.${status}`)
}

function statusOrder(status: string) {
  const order: Record<string, number> = {
    active: 0, expiringSoon: 1, pending: 2, failed: 3, expired: 4, revoked: 5, deleted: 6, unknown: 7,
  }
  return order[status] ?? 8
}

function statTone(_key: string) {
  return 'primary'
}

function formatDate(value: string) {
  if (!value) return '-'
  return new Intl.DateTimeFormat(locale.value === 'en-US' ? 'en-US' : 'zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  }).format(new Date(value))
}

function safeFileName(value: string) {
  return value.replace(/[^a-z0-9.-]+/gi, '_').replace(/^_+|_+$/g, '') || 'certificate'
}

function animateNumber(key: 'total' | 'active' | 'expiringSoon', target: number) {
  const current = animatedStats.value[key]
  if (current === target) return
  const diff = target - current
  const steps = 20
  let step = 0
  const interval = setInterval(() => {
    step++
    animatedStats.value = {
      ...animatedStats.value,
      [key]: Math.round(current + (diff * step) / steps),
    }
    if (step >= steps) {
      animatedStats.value = { ...animatedStats.value, [key]: target }
      clearInterval(interval)
    }
  }, 20)
}

function animateHealth(target: number) {
  const current = animatedHealthScore.value
  if (current === target) return
  const diff = target - current
  const steps = 30
  let step = 0
  const interval = setInterval(() => {
    step++
    animatedHealthScore.value = Math.round(current + (diff * step) / steps)
    if (step >= steps) {
      animatedHealthScore.value = target
      clearInterval(interval)
    }
  }, 20)
}
</script>

<style scoped>
/* ═══════════════ 页面布局 ═══════════════ */
.cert-page {
  width: min(100%, var(--content-max-width));
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

/* ═══════════════ Hero ═══════════════ */
.cert-hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  flex-shrink: 0;
}

.cert-hero__eyebrow {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.cert-hero h1 {
  margin-top: 2px;
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
}

.cert-hero__subtitle {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-secondary);
}

.cert-hero__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

/* ═══════════════ 统计卡片 ═══════════════ */
.cert-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
  flex-shrink: 0;
}

.cert-stat-card {
  min-height: 100px;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 44px;
  align-items: start;
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  transition: box-shadow var(--duration-fast) var(--ease-out), border-color var(--duration-fast) var(--ease-out);
}

.cert-stat-card:hover {
  box-shadow: var(--shadow-sm);
  border-color: var(--border-default);
}

.cert-stat-card__body p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.cert-stat-card__value {
  display: block;
  margin-top: var(--space-2);
  font-size: var(--text-2xl);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.cert-stat-card__body small {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.cert-stat-card__icon {
  width: 44px;
  height: 44px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-full);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.cert-stat-card.is-success .cert-stat-card__icon { background: var(--color-success-muted); color: var(--color-success); }
.cert-stat-card.is-warning .cert-stat-card__icon { background: var(--color-warning-muted); color: var(--color-warning); }
.cert-stat-card.is-error .cert-stat-card__icon { background: var(--color-error-muted); color: var(--color-error); }
.cert-stat-card.is-neutral .cert-stat-card__icon { background: var(--bg-input); color: var(--text-secondary); }

.cert-stat-card.is-success .cert-stat-card__value { color: var(--color-success); }
.cert-stat-card.is-warning .cert-stat-card__value { color: var(--color-warning); }
.cert-stat-card.is-error .cert-stat-card__value { color: var(--color-error); }

/* ═══════════════ 可视化面板 ═══════════════ */
.cert-viz-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1.2fr);
  gap: var(--space-3);
  flex-shrink: 0;
}

.cert-panel {
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.cert-panel__heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}

.cert-panel__heading h3 {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
}

/* ── 健康评分 ── */
.health-score {
  position: relative;
  display: grid;
  place-items: center;
  margin: var(--space-2) 0;
}

.health-score__svg {
  width: 120px;
  height: 120px;
}

.health-score__track {
  fill: none;
  stroke: var(--bg-input);
  stroke-width: 8;
}

.health-score__fill {
  fill: none;
  stroke-width: 8;
  stroke-linecap: round;
  transform: rotate(-90deg);
  transform-origin: center;
  transition: stroke-dashoffset var(--duration-slow) var(--ease-out);
}

.health-score__fill.is-success { stroke: var(--color-success); }
.health-score__fill.is-warning { stroke: var(--color-warning); }
.health-score__fill.is-error { stroke: var(--color-error); }

.health-score__center {
  position: absolute;
  display: grid;
  place-items: center;
  gap: 2px;
}

.health-score__center strong {
  font-size: var(--text-2xl);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.health-score__center small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.health-checks {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-2);
}

.health-check {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.health-check.is-ok {
  color: var(--color-success);
}

/* ── 圆环图 ── */
.donut-chart {
  display: grid;
  grid-template-columns: 120px minmax(0, 1fr);
  gap: var(--space-4);
  align-items: center;
}

.donut-chart__ring {
  width: 120px;
  aspect-ratio: 1;
  display: grid;
  place-items: center;
  border-radius: var(--radius-full);
  position: relative;
  justify-self: center;
}

.donut-chart__ring::after {
  content: '';
  position: absolute;
  inset: 24px;
  border-radius: inherit;
  background: var(--bg-surface);
  box-shadow: inset 0 0 0 1px rgba(108, 124, 147, 0.1);
}

.donut-chart__ring span,
.donut-chart__ring small {
  position: relative;
  z-index: 1;
}

.donut-chart__ring span {
  font-size: var(--text-xl);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.donut-chart__ring small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.donut-chart__legend {
  display: grid;
  gap: var(--space-2);
}

.donut-chart__legend article {
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-xs);
}

.donut-chart__legend i {
  width: 10px;
  height: 10px;
  border-radius: var(--radius-full);
}

.donut-chart__legend strong {
  color: var(--text-primary);
  font-weight: var(--weight-semibold);
  font-variant-numeric: tabular-nums;
}

/* ── 趋势图 ── */
.trend-chart__svg {
  width: 100%;
  height: 100px;
}

.trend-chart__grid line {
  stroke: var(--border-subtle);
  stroke-width: 1;
  stroke-dasharray: 3 5;
}

.trend-chart__line {
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.trend-chart__legend {
  display: flex;
  gap: var(--space-3);
  margin-top: var(--space-2);
  font-size: var(--text-xs);
}

.trend-chart__legend span {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  color: var(--text-secondary);
}

.trend-chart__legend i {
  width: 16px;
  height: 2px;
  border-radius: var(--radius-full);
}

.trend-empty {
  min-height: 100px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-2);
  color: var(--text-tertiary);
}

/* ── 自动续期面板 ── */
.cert-panel--renewal {
  flex-shrink: 0;
}

.renewal-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
}

.renewal-grid article {
  min-height: 60px;
  display: grid;
  align-content: center;
  gap: 2px;
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.renewal-grid span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.renewal-grid strong {
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  font-variant-numeric: tabular-nums;
}

.renewal-grid strong.is-success { color: var(--color-success); }
.renewal-grid strong.is-error { color: var(--color-error); }

.renewal-warning {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-warning-muted);
  border-radius: var(--radius-md);
  background: var(--color-warning-muted);
  color: var(--color-warning);
  font-size: var(--text-xs);
}

/* ═══════════════ 工作区 (列表 + 详情) ═══════════════ */
.cert-workspace {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(300px, 380px) minmax(0, 1fr);
  gap: var(--space-4);
  flex: 1;
}

/* ── 列表 ── */
.cert-list {
  min-height: 0;
  overflow: auto;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  padding: var(--space-2);
}

.cert-toolbar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 110px 120px;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.cert-search {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  height: 34px;
  padding: 0 var(--space-3);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-tertiary);
}

.cert-search:focus-within {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.cert-search input {
  min-width: 0;
  flex: 1;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.cert-toolbar select {
  height: 34px;
  padding: 0 var(--space-2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  font-size: var(--text-xs);
}

.cert-list__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-2);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.cert-list__header strong {
  color: var(--text-secondary);
  font-weight: var(--weight-semibold);
}

.cert-card {
  width: 100%;
  display: grid;
  grid-template-columns: 9px 36px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  margin-bottom: var(--space-1);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.cert-card:hover {
  border-color: var(--border-default);
  background: var(--bg-surface-hover);
  box-shadow: var(--shadow-xs);
}

.cert-card.active {
  border-color: var(--border-default);
  background: var(--bg-surface-hover);
  box-shadow: inset 2px 0 0 var(--color-primary);
}

.cert-card__status {
  width: 9px;
  height: 9px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.cert-card__status.is-online { background: var(--status-online); }
.cert-card__status.is-warning { background: var(--status-warning); }
.cert-card__status.is-error { background: var(--status-error); }

.cert-card__icon {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.cert-card__icon.is-online { background: var(--color-success-muted); color: var(--color-success); }
.cert-card__icon.is-warning { background: var(--color-warning-muted); color: var(--color-warning); }
.cert-card__icon.is-error { background: var(--color-error-muted); color: var(--color-error); }

.cert-card__main {
  min-width: 0;
}

.cert-card__main strong,
.cert-card__main small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cert-card__main small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.cert-card__meta {
  display: grid;
  justify-items: end;
  gap: 2px;
}

.cert-badge {
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.cert-badge.is-online { color: var(--color-success); background: var(--color-success-muted); }
.cert-badge.is-warning { color: var(--color-warning); background: var(--color-warning-muted); }
.cert-badge.is-error { color: var(--color-error); background: var(--color-error-muted); }
.cert-badge.is-offline { color: var(--text-tertiary); background: var(--bg-input); }

.cert-card__meta small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-family: var(--font-mono);
}

.cert-card__meta small.is-expired {
  color: var(--color-error);
}

.cert-list__empty {
  min-height: 200px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-2);
  color: var(--text-tertiary);
}

/* ── 详情面板 ── */
.cert-detail {
  min-width: 0;
  min-height: 0;
  overflow: auto;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  padding: var(--space-5);
}

.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.detail-title-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.detail-title-row > span {
  width: 9px;
  height: 9px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.detail-title-row > span.is-online { background: var(--status-online); }
.detail-title-row > span.is-warning { background: var(--status-warning); }
.detail-title-row > span.is-error { background: var(--status-error); }

.detail-title-row h2 {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
}

.detail-header p {
  margin-top: var(--space-1);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.detail-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
  flex-wrap: wrap;
}

.icon-action {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
}

.icon-action:hover {
  color: var(--color-error);
  border-color: var(--color-error);
}

/* ── 生命周期时间轴 ── */
.cert-lifecycle {
  margin-top: var(--space-4);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.lifecycle-timeline {
  display: flex;
  align-items: flex-start;
  gap: 0;
}

.lifecycle-stage {
  position: relative;
  flex: 1;
  display: grid;
  gap: var(--space-1);
}

.lifecycle-stage__dot {
  width: 24px;
  height: 24px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-full);
  background: var(--bg-surface);
  color: var(--text-tertiary);
  border: 2px solid var(--border-subtle);
}

.lifecycle-stage.is-done .lifecycle-stage__dot {
  background: var(--color-success);
  border-color: var(--color-success);
  color: var(--color-success);
}

.lifecycle-stage.is-current .lifecycle-stage__dot {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-muted);
  animation: cert-pulse 2s ease-in-out infinite;
}

.lifecycle-stage__body strong {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.lifecycle-stage__body small {
  display: block;
  color: var(--text-tertiary);
  font-size: 10px;
}

.lifecycle-stage__line {
  position: absolute;
  top: 12px;
  left: 50%;
  width: 100%;
  height: 2px;
  background: var(--border-subtle);
}

.lifecycle-stage__line.is-done {
  background: var(--color-success);
}

@keyframes cert-pulse {
  0%, 100% { box-shadow: 0 0 0 0 var(--color-primary-muted); }
  50% { box-shadow: 0 0 0 6px transparent; }
}

/* ── 指标网格 ── */
.detail-metrics {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
  margin-top: var(--space-4);
}

.detail-metrics article {
  min-height: 70px;
  display: grid;
  align-content: center;
  gap: 2px;
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.detail-metrics span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.detail-metrics strong {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  font-variant-numeric: tabular-nums;
}

.detail-metrics strong.is-warning { color: var(--color-warning); }
.detail-metrics strong.is-error { color: var(--color-error); }
.detail-metrics strong.is-online { color: var(--color-success); }

/* ── 详情卡片网格 ── */
.detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-4);
  margin-top: var(--space-4);
}

.cert-detail-card {
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.cert-detail-card + .cert-detail-card {
  margin-top: var(--space-4);
}

/* ── 自动续期开关 ── */
.auto-renewal-toggle {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}

.toggle-switch {
  width: 44px;
  height: 24px;
  border: 0;
  border-radius: var(--radius-full);
  background: var(--bg-input);
  position: relative;
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease-out);
}

.toggle-switch.on {
  background: var(--color-success);
}

.toggle-switch__knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  border-radius: var(--radius-full);
  background: var(--text-primary);
  transition: transform var(--duration-fast) var(--ease-out);
}

.toggle-switch.on .toggle-switch__knob {
  transform: translateX(20px);
  background: #fff;
}

.detail-info {
  display: grid;
  gap: var(--space-2);
}

.detail-info div {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.detail-info dt {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.detail-info dd {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.detail-info dd.is-error {
  color: var(--color-error);
}

/* ── SAN 列表 ── */
.san-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.san-tag {
  min-height: 24px;
  display: inline-flex;
  align-items: center;
  padding: 0 var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

/* ── 域名关联 ── */
.associations {
  display: grid;
  gap: var(--space-3);
}

.assoc-section {
  display: grid;
  gap: var(--space-2);
}

.assoc-label {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.assoc-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.assoc-tag {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  padding: 3px 8px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  font-family: var(--font-mono);
}

.assoc-tag.is-connected { border-color: var(--color-success); color: var(--color-success); }
.assoc-tag.is-disconnected { border-color: var(--text-tertiary); }

.assoc-loading {
  display: grid;
  place-items: center;
  min-height: 40px;
  color: var(--text-tertiary);
}

/* ── PEM 查看器 ── */
.pem-viewer {
  max-height: 200px;
  overflow: auto;
  padding: var(--space-3);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
}

.is-empty {
  color: var(--text-tertiary);
}

/* ── 占位符 ── */
.cert-detail__placeholder {
  min-height: 400px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  color: var(--text-tertiary);
}

/* ═══════════════ 空状态 ═══════════════ */
.cert-empty {
  min-height: 460px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  text-align: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-xl);
  background: var(--bg-surface);
}

.cert-empty__illustration {
  width: 86px;
  height: 86px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-2xl);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.cert-empty h2 {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
}

.cert-empty p {
  color: var(--text-secondary);
}

.cert-empty__features {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  justify-content: center;
  margin-top: var(--space-2);
}

.cert-empty__features span {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  color: var(--color-success);
  font-size: var(--text-sm);
}

.cert-empty__actions {
  display: flex;
  gap: var(--space-2);
  margin-top: var(--space-3);
}

/* ═══════════════ 响应式 ═══════════════ */
@media (max-width: 1120px) {
  .cert-stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .cert-viz-grid {
    grid-template-columns: 1fr;
  }

  .cert-workspace {
    grid-template-columns: 1fr;
  }

  .cert-list {
    max-height: 360px;
  }

  .detail-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .detail-grid {
    grid-template-columns: 1fr;
  }

  .renewal-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 760px) {
  .cert-hero {
    flex-direction: column;
  }

  .cert-hero__actions {
    width: 100%;
    flex-wrap: wrap;
  }

  .cert-stats,
  .detail-metrics,
  .renewal-grid {
    grid-template-columns: 1fr;
  }

  .cert-toolbar {
    grid-template-columns: 1fr;
  }

  .detail-header {
    flex-direction: column;
  }

  .detail-actions {
    width: 100%;
    flex-wrap: wrap;
  }

  .lifecycle-timeline {
    flex-direction: column;
    gap: var(--space-2);
  }

  .lifecycle-stage__line {
    display: none;
  }

  .health-checks {
    grid-template-columns: 1fr;
  }

  .donut-chart {
    grid-template-columns: 1fr;
    justify-items: center;
  }
}
</style>
