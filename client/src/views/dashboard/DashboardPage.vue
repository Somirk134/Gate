<template>
  <section class="dashboard-page" :aria-label="t('dashboard.ariaLabel')">
    <header class="dashboard-header">
      <div class="dashboard-header__copy">
        <h1>{{ t('dashboard.welcomeBack') }}</h1>
        <p>{{ t('dashboard.controlCenter') }}</p>
      </div>

      <div class="dashboard-header__meta">
        <GButton variant="secondary" icon="refresh" :loading="loading" @click="refresh">
          {{ t('dashboard.refresh') }}
        </GButton>
        <div class="header-chip">
          <GIcon name="refresh" :size="14" />
          <span>{{ t('dashboard.lastUpdated', { time: formattedLastUpdated }) }}</span>
        </div>
      </div>
    </header>

    <div v-if="showSkeleton" class="metric-grid" aria-hidden="true">
      <article v-for="index in 6" :key="index" class="metric-card metric-card--loading">
        <div>
          <GSkeleton variant="text" width="72%" />
          <GSkeleton variant="text" width="46%" height="22px" />
          <GSkeleton variant="text" width="64%" />
        </div>
        <GSkeleton variant="circle" width="44px" height="44px" />
      </article>
    </div>

    <template v-else>
      <section class="metric-grid">
        <article v-for="card in metricCards" :key="card.key" class="metric-card">
          <div class="metric-card__body">
            <p>{{ card.label }}</p>
            <strong>{{ card.value }}</strong>
            <small :class="card.helperTone">{{ card.helper }}</small>
          </div>
          <span class="metric-card__icon" :class="`is-${card.tone}`">
            <GIcon :name="card.icon" :size="21" />
          </span>
        </article>
      </section>

      <GEmptyState
        v-if="isRuntimeEmpty"
        class="dashboard-onboarding"
        :title="t('dashboard.empty.firstRun')"
        :description="t('dashboard.empty.firstRunDesc')">
        <template #action>
          <div class="empty-actions">
            <GButton variant="primary" icon="plus" @click="router.push('/tunnels?create=1')">
              {{ t('dashboard.empty.createTunnel') }}
            </GButton>
            <GButton variant="secondary" icon="servers" @click="router.push('/servers')">
              {{ t('dashboard.empty.connectServer') }}
            </GButton>
          </div>
        </template>
      </GEmptyState>

      <div class="dashboard-grid">
        <section class="dashboard-panel dashboard-panel--traffic">
          <div class="panel-heading">
            <div>
              <h2>{{ t('dashboard.chart.trafficTrend') }}</h2>
              <p>{{ t('dashboard.range.last24h') }}</p>
            </div>
            <div class="range-tabs" role="tablist">
              <button
                v-for="range in rangeOptions"
                :key="range.value"
                type="button"
                :class="{ active: trafficRange === range.value }"
                @click="trafficRange = range.value">
                {{ range.label }}
              </button>
            </div>
          </div>

          <div v-if="trafficPoints.length" class="traffic-chart">
            <svg viewBox="0 0 680 260" role="img" :aria-label="t('dashboard.chart.trafficTrend')">
              <defs>
                <linearGradient id="gate-upload-area" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="#3f7cff" stop-opacity="0.22" />
                  <stop offset="100%" stop-color="#3f7cff" stop-opacity="0" />
                </linearGradient>
                <linearGradient id="gate-download-area" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="#39c27f" stop-opacity="0.2" />
                  <stop offset="100%" stop-color="#39c27f" stop-opacity="0" />
                </linearGradient>
              </defs>
              <g class="traffic-chart__grid">
                <line v-for="line in 5" :key="line" x1="44" x2="650" :y1="axisY(line)" :y2="axisY(line)" />
              </g>
              <polygon class="traffic-chart__upload-area" :points="trafficUploadArea" />
              <polygon class="traffic-chart__download-area" :points="trafficDownloadArea" />
              <polyline class="traffic-chart__upload" :points="trafficUploadPolyline" />
              <polyline class="traffic-chart__download" :points="trafficDownloadPolyline" />
              <circle
                v-if="trafficPoints.length === 1"
                :cx="singleTrafficPoint.x"
                :cy="singleTrafficPoint.y"
                r="5" />
            </svg>
            <div class="traffic-chart__legend">
              <span><i class="is-upload" />{{ t('dashboard.upload') }}</span>
              <span><i class="is-download" />{{ t('dashboard.download') }}</span>
              <span v-if="trafficPoints.length === 1">{{ t('dashboard.chart.singleSample') }}</span>
            </div>
          </div>

          <GEmptyState
            v-else
            :title="t('dashboard.empty.noTraffic')"
            :description="t('dashboard.empty.noTrafficDesc')" />
        </section>

        <section class="dashboard-panel dashboard-panel--donut">
          <div class="panel-heading">
            <div>
              <h2>{{ t('dashboard.chart.tunnelTypes') }}</h2>
              <p>{{ t('dashboard.metric.tunnelRepository') }}</p>
            </div>
          </div>

          <div v-if="tunnelTypeStats.length" class="type-chart">
            <div class="donut" :style="donutStyle">
              <span>{{ dashboard.overview.tunnelCount }}</span>
              <small>{{ t('dashboard.total') }}</small>
            </div>
            <div class="type-list">
              <article v-for="item in tunnelTypeStats" :key="item.type">
                <i :style="{ background: item.color }" />
                <span>{{ item.label }}</span>
                <strong>{{ item.count }} ({{ item.percent }})</strong>
              </article>
            </div>
          </div>

          <GEmptyState
            v-else
            :title="t('dashboard.empty.noTunnel')"
            :description="t('dashboard.empty.noTunnelDesc')" />
        </section>

        <section class="dashboard-panel dashboard-panel--compact">
          <div class="panel-heading">
            <div>
              <h2>{{ t('dashboard.connectionStatus') }}</h2>
              <p>{{ t('dashboard.metric.gatewaySession') }}</p>
            </div>
          </div>
          <div class="connection-bar" :aria-label="t('dashboard.connectionStatus')">
            <i class="is-running" :style="{ flexGrow: Math.max(1, tunnelState.running) }" />
            <i class="is-warning" :style="{ flexGrow: Math.max(1, tunnelState.warning) }" />
            <i class="is-stopped" :style="{ flexGrow: Math.max(1, tunnelState.stopped) }" />
          </div>
          <div class="connection-stats">
            <article>
              <strong>{{ tunnelState.running }}</strong>
              <span>{{ t('dashboard.status.running') }}</span>
              <small>{{ formatPercent(tunnelState.runningRate) }}</small>
            </article>
            <article>
              <strong>{{ tunnelState.warning }}</strong>
              <span>{{ t('dashboard.status.warning') }}</span>
              <small>{{ formatPercent(tunnelState.warningRate) }}</small>
            </article>
            <article>
              <strong>{{ tunnelState.stopped }}</strong>
              <span>{{ t('dashboard.status.stopped') }}</span>
              <small>{{ formatPercent(tunnelState.stoppedRate) }}</small>
            </article>
          </div>
        </section>

        <section class="dashboard-panel dashboard-panel--compact">
          <div class="panel-heading panel-heading--split">
            <div>
              <h2>{{ t('dashboard.chart.requestStats') }}</h2>
              <p>{{ t('dashboard.todayRequests') }}</p>
            </div>
            <strong class="panel-value">{{ formatNumber(httpRequestTotal) }}</strong>
          </div>
          <div v-if="requestBars.length" class="mini-bars">
            <i
              v-for="(bar, index) in requestBars"
              :key="`request-${index}`"
              :style="{ height: `${bar}%` }" />
          </div>
          <GEmptyState
            v-else
            :title="t('dashboard.empty.noHttp')"
            :description="t('dashboard.empty.noHttpDesc')" />
        </section>

        <section class="dashboard-panel dashboard-panel--compact">
          <div class="panel-heading panel-heading--split">
            <div>
              <h2>{{ t('dashboard.chart.errorStats') }}</h2>
              <p>{{ t('dashboard.todayErrors') }}</p>
            </div>
            <strong class="panel-value is-error">{{ formatNumber(httpErrorCount) }}</strong>
          </div>
          <div v-if="errorBars.length" class="mini-bars mini-bars--error">
            <i
              v-for="(bar, index) in errorBars"
              :key="`error-${index}`"
              :style="{ height: `${bar}%` }" />
          </div>
          <GEmptyState
            v-else
            :title="t('dashboard.empty.noHttp')"
            :description="t('dashboard.empty.noHttpDesc')" />
        </section>

        <section class="dashboard-panel dashboard-panel--quick">
          <div class="panel-heading">
            <div>
              <h2>{{ t('dashboard.quickActions') }}</h2>
              <p>{{ t('dashboard.controlCenter') }}</p>
            </div>
          </div>
          <div class="quick-list">
            <button
              v-for="action in quickActions"
              :key="action.key"
              type="button"
              @click="router.push(action.path)">
              <span><GIcon :name="action.icon" :size="18" /></span>
              <div>
                <strong>{{ action.label }}</strong>
                <small>{{ action.description }}</small>
              </div>
              <GIcon name="chevron-right" :size="15" />
            </button>
          </div>
        </section>

        <section class="dashboard-panel dashboard-panel--recent">
          <div class="panel-heading">
            <div>
              <h2>{{ t('dashboard.chart.recentTunnels') }}</h2>
              <p>{{ t('dashboard.metric.tunnelRepository') }}</p>
            </div>
            <button class="panel-link" type="button" @click="router.push('/tunnels')">
              {{ t('dashboard.viewAll') }}
            </button>
          </div>

          <div v-if="recentTunnels.length" class="tunnel-table">
            <div class="tunnel-table__head">
              <span>{{ t('dashboard.table.name') }}</span>
              <span>{{ t('dashboard.table.type') }}</span>
              <span>{{ t('dashboard.table.domain') }}</span>
              <span>{{ t('dashboard.table.status') }}</span>
              <span>{{ t('dashboard.table.traffic') }}</span>
              <span>{{ t('dashboard.table.latency') }}</span>
              <span>{{ t('dashboard.table.action') }}</span>
            </div>
            <article v-for="tunnel in recentTunnels" :key="tunnel.id" class="tunnel-row">
              <strong>{{ tunnel.name }}</strong>
              <span class="protocol-pill" :class="`is-${protocolTone(tunnel.protocol)}`">
                {{ protocolLabel(tunnel.protocol) }}
              </span>
              <span>{{ tunnel.host || t('dashboard.noDomain') }}</span>
              <span class="status-pill" :class="`is-${statusTone(tunnel.status)}`">
                {{ statusLabel(tunnel.status) }}
              </span>
              <span>{{ formatBytes(tunnel.trafficBytes ?? 0) }}</span>
              <span>{{ formatLatency(tunnel.averageResponseTimeMs ?? 0) }}</span>
              <button type="button" @click="router.push('/tunnels')">
                <GIcon name="more-vertical" :size="15" />
              </button>
            </article>
          </div>

          <GEmptyState
            v-else
            :title="t('dashboard.empty.noTunnel')"
            :description="t('dashboard.empty.noTunnelDesc')">
            <template #action>
              <GButton variant="primary" icon="plus" @click="router.push('/tunnels?create=1')">
                {{ t('dashboard.empty.createTunnel') }}
              </GButton>
            </template>
          </GEmptyState>
        </section>
      </div>
    </template>

    <p v-if="error" class="dashboard-error">
      {{ error }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import GEmptyState from '@components/feedback/GEmptyState.vue'
import GSkeleton from '@components/feedback/GSkeleton.vue'
import { useMonitoringDashboard } from '@/monitoring/composables/useMonitoringDashboard'
import type { DashboardMetricCardMeta, DashboardTunnel, TrafficTrendPoint } from '@/monitoring/types'

type TrafficRange = '24h' | '7d' | '30d'
type MetricKey =
  | 'totalTunnels'
  | 'onlineTunnels'
  | 'activeConnections'
  | 'traffic'
  | 'latency'
  | 'runtimeUptime'

const router = useRouter()
const { t, locale } = useI18n()
const { dashboard, healthStatus, lastUpdated, loading, error, refresh } = useMonitoringDashboard()
const trafficRange = ref<TrafficRange>('24h')

const defaultMetricCardMeta: DashboardMetricCardMeta[] = [
  { key: 'totalTunnels', icon: 'router', tone: 'primary' },
  { key: 'onlineTunnels', icon: 'check-circle', tone: 'success' },
  { key: 'activeConnections', icon: 'users', tone: 'secondary' },
  { key: 'traffic', icon: 'activity', tone: 'info' },
  { key: 'latency', icon: 'clock', tone: 'warning' },
  { key: 'runtimeUptime', icon: 'shield-check', tone: 'healthy' },
]

const rangeOptions = computed<Array<{ value: TrafficRange; label: string }>>(() => [
  { value: '24h', label: t('dashboard.range.last24h') },
  { value: '7d', label: t('dashboard.range.last7d') },
  { value: '30d', label: t('dashboard.range.last30d') },
])

const quickActions = computed(() => [
  {
    key: 'createTunnel',
    icon: 'router',
    label: t('dashboard.quick.createTunnel'),
    description: t('dashboard.quick.createTunnelDesc'),
    path: '/tunnels?create=1',
  },
  {
    key: 'addServer',
    icon: 'servers',
    label: t('dashboard.quick.addServer'),
    description: t('dashboard.quick.addServerDesc'),
    path: '/servers',
  },
  {
    key: 'certificate',
    icon: 'shield-check',
    label: t('dashboard.quick.requestCertificate'),
    description: t('dashboard.quick.requestCertificateDesc'),
    path: '/certificates',
  },
  {
    key: 'logs',
    icon: 'logs',
    label: t('dashboard.quick.viewLogs'),
    description: t('dashboard.quick.viewLogsDesc'),
    path: '/logs',
  },
])

const showSkeleton = computed(
  () =>
    loading.value &&
    dashboard.value.overview.tunnelCount === 0 &&
    dashboard.value.overview.totalTraffic === 0 &&
    dashboard.value.recentActivity.length === 0,
)

const formattedLastUpdated = computed(() => formatRelativeTime(lastUpdated.value.getTime()))

const metricCardMeta = computed(() => {
  const cards = dashboard.value.visualSummary?.metricCards
  return new Map((cards?.length ? cards : defaultMetricCardMeta).map((card) => [card.key, card]))
})

const metricCards = computed(() =>
  [
    {
      key: 'totalTunnels',
      label: t('dashboard.metric.totalTunnels'),
      value: formatNumber(dashboard.value.overview.tunnelCount),
      helper: t('dashboard.metric.runningSummary', {
        count: dashboard.value.overview.runningTunnel,
      }),
      helperTone: 'is-positive',
    },
    {
      key: 'onlineTunnels',
      label: t('dashboard.metric.onlineTunnels'),
      value: formatNumber(dashboard.value.overview.runningTunnel),
      helper: t('dashboard.metric.onlineRate', { rate: formatPercent(tunnelState.value.runningRate) }),
      helperTone: 'is-positive',
    },
    {
      key: 'activeConnections',
      label: t('dashboard.metric.activeConnections'),
      value: formatNumber(dashboard.value.statistics.connection.currentConnection),
      helper: t('dashboard.metric.gatewaySession'),
      helperTone: '',
    },
    {
      key: 'traffic',
      label: t('dashboard.metric.todayTraffic'),
      value: formatBytes(dashboard.value.statistics.traffic.todayTrafficBytes),
      helper: t('dashboard.metric.totalTraffic', {
        value: formatBytes(dashboard.value.statistics.traffic.totalTrafficBytes),
      }),
      helperTone: 'is-positive',
    },
    {
      key: 'latency',
      label: t('dashboard.metric.averageLatency'),
      value: formatLatency(dashboard.value.overview.averageRttMs),
      helper: t('dashboard.metric.tunnelMetrics'),
      helperTone: '',
    },
    {
      key: 'runtimeUptime',
      label: t('dashboard.metric.runtimeUptime'),
      value: formatDuration(dashboard.value.overview.runtimeUptimeSeconds),
      helper: t('dashboard.healthStatusText', {
        status: t(`dashboard.healthStatus.${healthStatus.value}`),
      }),
      helperTone: healthStatus.value === 'healthy' ? 'is-positive' : '',
    },
  ].map((card) => {
    const meta = metricMetaFor(card.key as MetricKey)
    return {
      ...card,
      icon: meta.icon,
      tone: card.key === 'runtimeUptime' ? healthStatus.value : meta.tone,
    }
  }),
)

const isRuntimeEmpty = computed(
  () =>
    dashboard.value.overview.tunnelCount === 0 &&
    dashboard.value.serverStatus.length === 0 &&
    dashboard.value.overview.totalTraffic === 0,
)

const trafficPoints = computed(() => {
  const windowMs =
    trafficRange.value === '24h'
      ? 24 * 60 * 60 * 1000
      : trafficRange.value === '7d'
        ? 7 * 24 * 60 * 60 * 1000
        : 30 * 24 * 60 * 60 * 1000
  const cutoff = Date.now() - windowMs
  return dashboard.value.trafficTrend.filter((point) => point.timestamp >= cutoff)
})

const maxTrafficBytes = computed(() =>
  Math.max(
    1,
    ...trafficPoints.value.map((point) => Math.max(point.uploadBytes, point.downloadBytes)),
  ),
)
const trafficUploadPolyline = computed(() => buildTrafficPolyline(trafficPoints.value, 'upload'))
const trafficDownloadPolyline = computed(() => buildTrafficPolyline(trafficPoints.value, 'download'))
const trafficUploadArea = computed(() => buildTrafficArea(trafficPoints.value, 'upload'))
const trafficDownloadArea = computed(() => buildTrafficArea(trafficPoints.value, 'download'))
const singleTrafficPoint = computed(() =>
  pointForTraffic(trafficPoints.value[0], 0, 1, maxTrafficBytes.value),
)

const tunnelState = computed(() => {
  const summary = dashboard.value.visualSummary?.tunnelState
  if (summary) {
    return {
      running: normalizeCount(summary.running),
      warning: normalizeCount(summary.warning),
      stopped: normalizeCount(summary.stopped),
      runningRate: normalizeRatio(summary.runningRate),
      warningRate: normalizeRatio(summary.warningRate),
      stoppedRate: normalizeRatio(summary.stoppedRate),
    }
  }

  const total = Math.max(1, dashboard.value.tunnels.length)
  const running = dashboard.value.tunnels.filter((tunnel) => tunnel.status === 'running').length
  const warning = dashboard.value.tunnels.filter((tunnel) => isAttentionStatus(tunnel.status)).length
  const stopped = Math.max(0, dashboard.value.tunnels.length - running - warning)
  return {
    running,
    warning,
    stopped,
    runningRate: (running / total) * 100,
    warningRate: (warning / total) * 100,
    stoppedRate: (stopped / total) * 100,
  }
})

const tunnelTypeStats = computed(() => {
  const colors = {
    tcp: '#4d73ff',
    http: '#3ac886',
    https: '#ffb547',
    udp: '#8b5cf6',
    unknown: '#94a3b8',
  }

  const summary = dashboard.value.visualSummary?.protocolDistribution
  const rows =
    summary?.length
      ? summary
      : (['tcp', 'http', 'https', 'udp'] as const).map((type) => {
          const count = dashboard.value.tunnels.filter((tunnel) => tunnel.protocol === type).length
          const total = Math.max(1, dashboard.value.tunnels.length)
          return { protocol: type, count, percent: (count / total) * 100 }
        })

  return rows
    .map((type) => ({
      type: type.protocol,
      label: protocolLabel(type.protocol),
      count: normalizeCount(type.count),
      color: colors[type.protocol] ?? colors.unknown,
      percent: formatPercent(type.percent),
    }))
    .filter((item) => item.count > 0)
})

const donutStyle = computed(() => {
  const total = tunnelTypeStats.value.reduce((sum, item) => sum + item.count, 0)
  if (!total) return {}
  let cursor = 0
  const stops = tunnelTypeStats.value.map((item) => {
    const start = cursor
    cursor += (item.count / total) * 100
    return `${item.color} ${start}% ${cursor}%`
  })
  return { background: `conic-gradient(${stops.join(', ')})` }
})

const httpRequestTotal = computed(
  () => dashboard.value.visualSummary?.requestTotal ?? dashboard.value.statistics.http?.requestsTotal ?? 0,
)

const httpErrorCount = computed(() => {
  const statusCodes = dashboard.value.statistics.http?.statusCodes ?? {}
  const statusErrors = Object.entries(statusCodes)
    .filter(([code]) => Number(code) >= 400)
    .reduce((sum, [, count]) => sum + count, 0)

  const tunnelErrors = dashboard.value.tunnels
    .filter((tunnel) => tunnel.protocol === 'http' || tunnel.protocol === 'https')
    .reduce((sum, tunnel) => {
      const requests = tunnel.requestCount ?? 0
      const success = Math.round(requests * (tunnel.successRate ?? 0))
      return sum + Math.max(0, requests - success)
    }, 0)

  return dashboard.value.visualSummary?.errorTotal ?? Math.max(statusErrors, tunnelErrors)
})

const requestBuckets = computed(() => {
  const buckets = dashboard.value.visualSummary?.requestBuckets
  return buckets?.length ? buckets : buildRequestBuckets('all')
})
const errorBuckets = computed(() => {
  const buckets = dashboard.value.visualSummary?.errorBuckets
  return buckets?.length ? buckets : buildRequestBuckets('error')
})
const requestBars = computed(() => normalizeBars(requestBuckets.value))
const errorBars = computed(() => normalizeBars(errorBuckets.value))

const recentTunnels = computed(() =>
  [...dashboard.value.tunnels]
    .sort((left, right) => Number(right.status === 'running') - Number(left.status === 'running'))
    .slice(0, 6),
)

function metricMetaFor(key: MetricKey): DashboardMetricCardMeta {
  return (
    metricCardMeta.value.get(key) ??
    defaultMetricCardMeta.find((card) => card.key === key) ??
    defaultMetricCardMeta[0]
  )
}

function normalizeCount(value: number): number {
  return Number.isFinite(value) ? Math.max(0, Math.round(value)) : 0
}

function normalizeRatio(value: number): number {
  return Number.isFinite(value) ? Math.max(0, Math.min(100, value)) : 0
}

function isAttentionStatus(status: DashboardTunnel['status']): boolean {
  return status !== 'running' && status !== 'stopped'
}

function statusTone(status: DashboardTunnel['status']): 'running' | 'warning' | 'stopped' {
  if (status === 'running') return 'running'
  if (status === 'stopped') return 'stopped'
  return 'warning'
}

function buildTrafficPolyline(points: TrafficTrendPoint[], kind: 'upload' | 'download'): string {
  if (!points.length) return ''
  return points
    .map((point, index) => {
      const chartPoint = pointForTraffic(point, index, points.length, maxTrafficBytes.value)
      return `${chartPoint.x},${kind === 'upload' ? chartPoint.uploadY : chartPoint.downloadY}`
    })
    .join(' ')
}

function buildTrafficArea(points: TrafficTrendPoint[], kind: 'upload' | 'download'): string {
  if (!points.length) return ''
  const line = buildTrafficPolyline(points, kind)
  const start = pointForTraffic(points[0], 0, points.length, maxTrafficBytes.value)
  const end = pointForTraffic(points[points.length - 1], points.length - 1, points.length, maxTrafficBytes.value)
  return `${start.x},224 ${line} ${end.x},224`
}

function pointForTraffic(
  point: TrafficTrendPoint | undefined,
  index: number,
  total: number,
  max: number,
) {
  const width = 606
  const height = 172
  const left = 44
  const top = 34
  const x = left + (total <= 1 ? width / 2 : (index / (total - 1)) * width)
  const upload = point?.uploadBytes ?? 0
  const download = point?.downloadBytes ?? 0

  return {
    x,
    y: top + height - (Math.max(upload, download) / max) * height,
    uploadY: top + height - (upload / max) * height,
    downloadY: top + height - (download / max) * height,
  }
}

function axisY(line: number) {
  return 34 + (line - 1) * 43
}

function buildRequestBuckets(kind: 'all' | 'error') {
  const bucketCount = 18
  const now = Date.now()
  const bucketMs = (24 * 60 * 60 * 1000) / bucketCount
  const buckets = Array.from({ length: bucketCount }, () => 0)
  const requests = dashboard.value.tunnels.flatMap((tunnel) => tunnel.recentRequests ?? [])

  for (const request of requests) {
    if (kind === 'error' && request.status < 400) continue
    const age = now - request.timestamp
    if (age < 0 || age > 24 * 60 * 60 * 1000) continue
    const index = Math.min(bucketCount - 1, Math.floor(age / bucketMs))
    buckets[bucketCount - 1 - index] += 1
  }

  return buckets
}

function normalizeBars(values: number[]) {
  const max = Math.max(0, ...values)
  if (max === 0) return []
  return values.map((value) => Math.max(8, Math.round((value / max) * 100)))
}

function protocolLabel(protocol: DashboardTunnel['protocol'] | 'unknown') {
  return t(`dashboard.protocol.${protocolTone(protocol)}`)
}

function protocolTone(protocol: DashboardTunnel['protocol'] | 'unknown') {
  const knownProtocol = ['tcp', 'udp', 'http', 'https']
  return knownProtocol.includes(protocol) ? protocol : 'unknown'
}

function statusLabel(status: DashboardTunnel['status']) {
  return t(`dashboard.status.${statusKey(status)}`)
}

function statusKey(status: DashboardTunnel['status']) {
  const knownStatus = [
    'running',
    'stopped',
    'warning',
    'starting',
    'stopping',
    'restarting',
    'recovering',
    'error',
  ]
  return knownStatus.includes(status) ? status : 'unknown'
}

function formatNumber(value: number): string {
  return new Intl.NumberFormat(locale.value === 'en-US' ? 'en-US' : 'zh-CN').format(value)
}

function formatPercent(value: number): string {
  return `${Math.max(0, Math.min(100, value)).toFixed(value >= 99 || value === 0 ? 0 : 1)}%`
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const index = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)))
  const value = bytes / 1024 ** index
  return `${value.toFixed(value >= 10 || index === 0 ? 0 : 1)} ${units[index]}`
}

function formatLatency(milliseconds: number): string {
  if (!Number.isFinite(milliseconds) || milliseconds <= 0) return '0 ms'
  return `${Math.round(milliseconds)} ms`
}

function formatDuration(seconds: number): string {
  const day = Math.floor(seconds / 86400)
  const hour = Math.floor((seconds % 86400) / 3600)
  const minute = Math.floor((seconds % 3600) / 60)
  if (day) return `${day}${t('dashboard.time.day')}`
  if (hour) return `${hour}${t('dashboard.time.hour')} ${minute}${t('dashboard.time.minute')}`
  if (minute) return `${minute}${t('dashboard.time.minute')}`
  return `${Math.max(0, Math.floor(seconds))}${t('dashboard.time.second')}`
}

function formatRelativeTime(timestamp: number): string {
  if (!Number.isFinite(timestamp)) return '--'
  const diffSeconds = Math.max(0, Math.floor((Date.now() - timestamp) / 1000))
  if (diffSeconds < 60) return t('dashboard.relative.justNow')
  const minutes = Math.floor(diffSeconds / 60)
  if (minutes < 60) return t('dashboard.relative.minutesAgo', { count: minutes })
  const hours = Math.floor(minutes / 60)
  return t('dashboard.relative.hoursAgo', { count: hours })
}
</script>

<style scoped>
.dashboard-page {
  width: min(100%, 1180px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 22px;
  color: var(--text-primary);
}

.dashboard-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.dashboard-header__copy h1 {
  color: var(--text-primary);
  font-size: 26px;
  font-weight: 700;
  line-height: 1.2;
  letter-spacing: 0;
}

.dashboard-header__copy p {
  margin-top: var(--space-2);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.dashboard-header__meta {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.header-chip {
  min-height: 38px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0 var(--space-4);
  border: 1px solid rgba(108, 124, 147, 0.14);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-surface) 92%, #ffffff 8%);
  color: var(--color-primary);
  box-shadow: 0 10px 30px rgba(34, 55, 94, 0.06);
}

.header-chip span {
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 12px;
}

.metric-card,
.dashboard-panel {
  border: 1px solid rgba(108, 124, 147, 0.12);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-surface) 94%, #ffffff 6%);
  box-shadow: 0 16px 34px rgba(40, 56, 89, 0.08);
}

.metric-card {
  min-height: 122px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: 18px;
}

.metric-card--loading > div:first-child {
  flex: 1;
}

.metric-card__body {
  min-width: 0;
}

.metric-card p,
.panel-heading p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.metric-card p::before {
  content: '';
  display: inline-block;
  width: 3px;
  height: 3px;
  margin-right: 6px;
  vertical-align: middle;
  border-radius: var(--radius-full);
  background: var(--color-primary);
}

.metric-card strong {
  display: block;
  margin-top: 10px;
  color: var(--text-primary);
  font-size: 24px;
  font-weight: 760;
  letter-spacing: 0;
}

.metric-card small {
  display: block;
  margin-top: 8px;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: 1.35;
}

.metric-card small.is-positive {
  color: var(--color-success);
}

.metric-card__icon {
  width: 48px;
  height: 48px;
  display: grid;
  place-items: center;
  border-radius: 999px;
  color: var(--color-primary);
  background: var(--color-primary-muted);
  flex-shrink: 0;
}

.metric-card__icon.is-success,
.metric-card__icon.is-healthy {
  color: var(--color-success);
  background: var(--color-success-muted);
}

.metric-card__icon.is-info {
  color: var(--color-info);
  background: var(--color-info-muted);
}

.metric-card__icon.is-warning {
  color: var(--color-warning);
  background: var(--color-warning-muted);
}

.metric-card__icon.is-critical {
  color: var(--color-error);
  background: var(--color-error-muted);
}

.metric-card__icon.is-secondary,
.metric-card__icon.is-offline {
  color: var(--color-secondary);
  background: var(--color-secondary-muted);
}

.dashboard-onboarding {
  border: 1px dashed var(--border-default);
  border-radius: 8px;
  background: var(--bg-surface);
}

.empty-actions {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
  justify-content: center;
}

.dashboard-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.7fr) minmax(320px, 1fr);
  gap: 12px;
}

.dashboard-panel {
  min-width: 0;
  padding: 22px;
}

.dashboard-panel--traffic {
  min-height: 258px;
}

.dashboard-panel--donut {
  min-height: 258px;
}

.dashboard-panel--quick {
  min-height: 320px;
}

.dashboard-panel--recent {
  min-height: 320px;
}

.panel-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: 18px;
}

.panel-heading--split {
  align-items: center;
}

.panel-heading h2 {
  color: var(--text-primary);
  font-size: 15px;
  font-weight: 720;
  letter-spacing: 0;
}

.panel-value {
  color: var(--text-primary);
  font-size: 24px;
  font-weight: 760;
}

.panel-value.is-error {
  color: var(--color-error);
}

.range-tabs {
  display: inline-flex;
  gap: 2px;
  padding: 2px;
  border: 1px solid rgba(108, 124, 147, 0.13);
  border-radius: 8px;
  background: var(--bg-input);
}

.range-tabs button {
  height: 28px;
  padding: 0 12px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: var(--text-xs);
}

.range-tabs button.active {
  background: var(--bg-surface);
  color: var(--text-primary);
  box-shadow: var(--shadow-xs);
}

.traffic-chart {
  display: grid;
  gap: var(--space-3);
}

.traffic-chart svg {
  width: 100%;
  height: 228px;
}

.traffic-chart__grid line {
  stroke: color-mix(in srgb, var(--border-subtle) 80%, transparent);
  stroke-width: 1;
}

.traffic-chart polyline {
  fill: none;
  stroke-width: 3;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.traffic-chart__upload {
  stroke: #3f7cff;
}

.traffic-chart__download {
  stroke: #39c27f;
}

.traffic-chart__upload-area {
  fill: url(#gate-upload-area);
}

.traffic-chart__download-area {
  fill: url(#gate-download-area);
}

.traffic-chart circle {
  fill: #3f7cff;
}

.traffic-chart__legend {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-4);
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.traffic-chart__legend span {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
}

.traffic-chart__legend i {
  width: 18px;
  height: 2px;
  border-radius: var(--radius-full);
}

.traffic-chart__legend .is-upload {
  background: #3f7cff;
}

.traffic-chart__legend .is-download {
  background: #39c27f;
}

.type-chart {
  display: grid;
  grid-template-columns: 190px minmax(0, 1fr);
  gap: var(--space-5);
  align-items: center;
}

.donut {
  width: 164px;
  aspect-ratio: 1;
  display: grid;
  place-items: center;
  border-radius: var(--radius-full);
  position: relative;
  justify-self: center;
}

.donut::after {
  content: '';
  position: absolute;
  inset: 34px;
  border-radius: inherit;
  background: var(--bg-surface);
  box-shadow: inset 0 0 0 1px rgba(108, 124, 147, 0.1);
}

.donut span,
.donut small {
  position: relative;
  z-index: 1;
}

.donut span {
  align-self: end;
  color: var(--text-primary);
  font-size: 28px;
  font-weight: 760;
}

.donut small {
  align-self: start;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.type-list {
  display: grid;
  gap: 18px;
}

.type-list article {
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.type-list i {
  width: 10px;
  height: 10px;
  border-radius: var(--radius-full);
}

.type-list strong {
  color: var(--text-primary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.connection-bar {
  display: flex;
  height: 16px;
  overflow: hidden;
  border-radius: 999px;
  background: var(--bg-input);
}

.connection-bar i {
  min-width: 18px;
}

.connection-bar .is-running {
  background: var(--color-success);
}

.connection-bar .is-warning {
  background: var(--color-warning);
}

.connection-bar .is-stopped {
  background: var(--color-error);
}

.connection-stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
  margin-top: 18px;
}

.connection-stats article {
  display: grid;
  justify-items: center;
  gap: 4px;
}

.connection-stats strong {
  color: var(--color-success);
  font-size: 18px;
  font-weight: 760;
}

.connection-stats article:nth-child(2) strong {
  color: var(--color-warning);
}

.connection-stats article:nth-child(3) strong {
  color: var(--color-error);
}

.connection-stats span,
.connection-stats small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.mini-bars {
  height: 72px;
  display: flex;
  align-items: end;
  gap: 8px;
  padding-top: var(--space-2);
  border-bottom: 1px dashed var(--border-subtle);
}

.mini-bars i {
  flex: 1;
  min-width: 4px;
  max-width: 12px;
  border-radius: 4px 4px 0 0;
  background: color-mix(in srgb, var(--color-primary) 76%, #ffffff 24%);
}

.mini-bars--error i {
  background: color-mix(in srgb, var(--color-error) 76%, #ffffff 24%);
}

.quick-list {
  display: grid;
  gap: 10px;
}

.quick-list button {
  width: 100%;
  min-height: 56px;
  display: grid;
  grid-template-columns: 42px minmax(0, 1fr) 18px;
  align-items: center;
  gap: var(--space-3);
  padding: 0 12px;
  border: 1px solid rgba(108, 124, 147, 0.09);
  border-radius: 8px;
  background: var(--bg-input);
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
}

.quick-list button:hover {
  border-color: color-mix(in srgb, var(--color-primary) 32%, var(--border-subtle));
  background: var(--bg-surface-hover);
}

.quick-list button > span {
  width: 38px;
  height: 38px;
  display: grid;
  place-items: center;
  border-radius: 8px;
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.quick-list strong,
.quick-list small {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.quick-list strong {
  font-size: var(--text-sm);
}

.quick-list small {
  margin-top: 2px;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.panel-link {
  height: 28px;
  display: inline-flex;
  align-items: center;
  border: 0;
  background: transparent;
  color: var(--color-primary);
  cursor: pointer;
  font-size: var(--text-xs);
}

.tunnel-table {
  display: grid;
  gap: 0;
  overflow-x: auto;
}

.tunnel-table__head,
.tunnel-row {
  display: grid;
  grid-template-columns: minmax(138px, 1.15fr) 72px minmax(150px, 1fr) 82px 82px 74px 42px;
  gap: var(--space-3);
  align-items: center;
  min-width: 780px;
}

.tunnel-table__head {
  padding: 0 12px 10px;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  border-bottom: 1px solid var(--border-subtle);
}

.tunnel-row {
  min-height: 46px;
  padding: 0 12px;
  color: var(--text-secondary);
  font-size: var(--text-xs);
  border-bottom: 1px solid var(--border-subtle);
}

.tunnel-row strong,
.tunnel-row span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tunnel-row strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.protocol-pill,
.status-pill {
  width: fit-content;
  padding: 3px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: var(--weight-semibold);
}

.protocol-pill.is-tcp {
  color: #3366ff;
  background: rgba(77, 115, 255, 0.12);
}

.protocol-pill.is-http {
  color: #16a56f;
  background: rgba(58, 200, 134, 0.14);
}

.protocol-pill.is-https {
  color: #e37a16;
  background: rgba(255, 181, 71, 0.18);
}

.protocol-pill.is-udp {
  color: #7c3aed;
  background: rgba(139, 92, 246, 0.14);
}

.protocol-pill.is-unknown {
  color: #64748b;
  background: rgba(148, 163, 184, 0.14);
}

.status-pill.is-running {
  color: var(--color-success);
  background: var(--color-success-muted);
}

.status-pill.is-warning {
  color: var(--color-warning);
  background: var(--color-warning-muted);
}

.status-pill.is-stopped {
  color: var(--color-error);
  background: var(--color-error-muted);
}

.tunnel-row button {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 8px;
  background: var(--bg-input);
  color: var(--text-tertiary);
  cursor: pointer;
}

.dashboard-error {
  color: var(--color-error);
  font-size: var(--text-sm);
}

@media (max-width: 1280px) {
  .metric-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }
}

@media (max-width: 980px) {
  .dashboard-header {
    flex-direction: column;
  }

  .dashboard-header__meta {
    justify-content: flex-start;
  }

  .dashboard-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 680px) {
  .metric-grid,
  .connection-stats {
    grid-template-columns: 1fr;
  }

  .panel-heading {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
