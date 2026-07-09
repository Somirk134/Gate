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
              <p>{{ trafficRangeCaption }}</p>
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

          <div v-if="projectTrafficSeries.length" class="traffic-chart">
            <svg viewBox="0 0 680 260" role="img" :aria-label="t('dashboard.chart.trafficTrend')">
              <g class="traffic-chart__grid">
                <line
                  v-for="label in trafficYAxisLabels"
                  :key="label.value"
                  :x1="trafficChartBounds.left"
                  :x2="trafficChartBounds.right"
                  :y1="label.y"
                  :y2="label.y" />
              </g>
              <g class="traffic-chart__axis">
                <text
                  v-for="label in trafficYAxisLabels"
                  :key="`y-${label.value}`"
                  class="traffic-chart__y-label"
                  :x="trafficChartBounds.left - 10"
                  :y="label.y">
                  {{ label.text }}
                </text>
                <text
                  v-for="label in trafficXAxisLabels"
                  :key="`x-${label.index}`"
                  class="traffic-chart__x-label"
                  :x="label.x"
                  :y="236">
                  {{ label.text }}
                </text>
              </g>
              <polyline
                v-for="series in projectTrafficSeries"
                :key="series.id"
                class="traffic-chart__project-line"
                :points="series.polyline"
                :stroke="series.color" />
            </svg>
            <div class="traffic-chart__legend">
              <span
                v-for="series in projectTrafficSeries"
                :key="series.id"
                :title="`${series.name} ${formatBytes(series.total)}`">
                <i :style="{ background: series.color }" />
                <b>{{ series.name }}</b>
                <small>{{ formatBytes(series.total) }}</small>
              </span>
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
      </div>
    </template>

    <p v-if="error" class="dashboard-error">
      {{ error }}
    </p>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import GEmptyState from '@components/feedback/GEmptyState.vue'
import GSkeleton from '@components/feedback/GSkeleton.vue'
import { useMonitoringDashboard } from '@/monitoring/composables/useMonitoringDashboard'
import { useProject } from '@views/projects/composables/useProject'
import type { DashboardMetricCardMeta, DashboardTunnel } from '@/monitoring/types'
import type { ProjectColor } from '@views/projects/types'

type TrafficRange = '24h' | '7d' | '30d'
type MetricKey =
  | 'totalTunnels'
  | 'onlineTunnels'
  | 'activeConnections'
  | 'traffic'
  | 'latency'
  | 'runtimeUptime'

interface ProjectTrafficSample {
  timestamp: number
  totals: Record<string, number>
}

interface TrafficChartPoint {
  x: number
  y: number
  value: number
}

const router = useRouter()
const { t, locale } = useI18n()
const {
  dashboard,
  healthStatus,
  lastUpdated,
  loading,
  error,
  refresh: refreshDashboard,
} = useMonitoringDashboard()
const { projects, refresh: refreshProjects } = useProject()
const trafficRange = ref<TrafficRange>('24h')

const TRAFFIC_HISTORY_STORAGE_KEY = 'gate.dashboard.projectTrafficHistory'
const TRAFFIC_HISTORY_SAMPLE_MS = 5 * 60 * 1000
const TRAFFIC_HISTORY_RETENTION_MS = 30 * 24 * 60 * 60 * 1000
const TRAFFIC_HISTORY_MAX_SAMPLES = 30 * 24 * 12 + 24
const TRAFFIC_CHART_Y_TICKS = 5
const trafficChartBounds = {
  left: 62,
  right: 650,
  top: 30,
  bottom: 204,
}

const projectColorMap: Record<ProjectColor, string> = {
  blue: '#3f7cff',
  green: '#39c27f',
  purple: '#a855f7',
  orange: '#f97316',
  red: '#ef4444',
  cyan: '#06b6d4',
  pink: '#ec4899',
  indigo: '#6366f1',
  teal: '#14b8a6',
  amber: '#f59e0b',
  slate: '#94a3b8',
}

const fallbackProjectColors = [
  '#3f7cff',
  '#39c27f',
  '#a855f7',
  '#f97316',
  '#06b6d4',
  '#ec4899',
  '#f59e0b',
  '#14b8a6',
]

const projectTrafficHistory = ref<ProjectTrafficSample[]>(loadProjectTrafficHistory())

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

const trafficRangeCaption = computed(() => {
  const activeRange = rangeOptions.value.find((range) => range.value === trafficRange.value)
  return `${activeRange?.label ?? t('dashboard.range.last24h')} · ${t('dashboard.chart.projectTotalTraffic')}`
})

const currentProjectTrafficTotals = computed(() => {
  const tunnelsById = new Map(dashboard.value.tunnels.map((tunnel) => [tunnel.id, tunnel]))
  return projects.value.reduce<Record<string, number>>((totals, project) => {
    totals[project.id] = project.tunnelIds.reduce((sum, tunnelId) => {
      const tunnel = tunnelsById.get(tunnelId)
      return sum + (tunnel?.trafficBytes ?? 0)
    }, 0)
    return totals
  }, {})
})

const trafficTimeBuckets = computed(() => buildTrafficTimeBuckets(trafficRange.value))
const maxProjectTrafficBytes = computed(() =>
  Math.max(
    1,
    ...projects.value.flatMap((project) =>
      buildProjectTrafficBucketValues(project.id).map((point) => point.value),
    ),
  ),
)

const projectTrafficSeries = computed(() =>
  projects.value.map((project, index) => {
    const points = buildProjectTrafficPoints(project.id)
    return {
      id: project.id,
      name: project.name,
      color: projectColorMap[project.color] ?? fallbackProjectColors[index % fallbackProjectColors.length],
      total: currentProjectTrafficTotals.value[project.id] ?? 0,
      points,
      polyline: pointsToPolyline(points),
    }
  }),
)

const trafficYAxisLabels = computed(() =>
  Array.from({ length: TRAFFIC_CHART_Y_TICKS }, (_, index) => {
    const ratio = (TRAFFIC_CHART_Y_TICKS - 1 - index) / (TRAFFIC_CHART_Y_TICKS - 1)
    const value = maxProjectTrafficBytes.value * ratio
    return {
      value,
      text: formatBytes(value),
      y: trafficChartBounds.top + index * trafficChartStepY(),
    }
  }),
)

const trafficXAxisLabels = computed(() =>
  trafficTimeBuckets.value
    .map((bucket, index) => ({
      index,
      text: formatTrafficTimeLabel(bucket.start),
      x: trafficPointX(index, trafficTimeBuckets.value.length),
    }))
    .filter((label) => shouldShowTrafficTimeLabel(label.index, trafficTimeBuckets.value.length)),
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


watch(
  () => [
    dashboard.value.generatedAt,
    projects.value.map((project) => `${project.id}:${project.tunnelIds.join(',')}`).join('|'),
  ],
  () => recordProjectTrafficSample(),
  { immediate: true },
)

async function refresh() {
  await Promise.all([refreshDashboard(), refreshProjects()])
}

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

function protocolLabel(protocol: DashboardTunnel['protocol'] | 'unknown') {
  return t(`dashboard.protocol.${protocolTone(protocol)}`)
}

function protocolTone(protocol: DashboardTunnel['protocol'] | 'unknown') {
  const knownProtocol = ['tcp', 'udp', 'http', 'https']
  return knownProtocol.includes(protocol) ? protocol : 'unknown'
}

// 后端暂未提供项目级历史曲线，首页用 5 分钟粒度缓存项目总流量，支撑 24h / 7d / 30d 的折线切换。
function recordProjectTrafficSample() {
  const totals = currentProjectTrafficTotals.value
  const hasTraffic = Object.values(totals).some((value) => value > 0)
  if (!hasTraffic && projectTrafficHistory.value.length === 0) return

  const timestamp = normalizeTrafficSampleTime(dashboard.value.generatedAt || Date.now())
  const nextHistory = projectTrafficHistory.value
    .filter((sample) => sample.timestamp !== timestamp)
    .concat({ timestamp, totals })

  projectTrafficHistory.value = pruneProjectTrafficHistory(nextHistory)
  saveProjectTrafficHistory(projectTrafficHistory.value)
}

function loadProjectTrafficHistory(): ProjectTrafficSample[] {
  if (typeof window === 'undefined') return []

  try {
    const raw = window.localStorage.getItem(TRAFFIC_HISTORY_STORAGE_KEY)
    const rows = raw ? JSON.parse(raw) : []
    if (!Array.isArray(rows)) return []

    return pruneProjectTrafficHistory(
      rows.map((row) => ({
        timestamp: Number(row?.timestamp) || 0,
        totals: normalizeTrafficTotals(row?.totals),
      })),
    )
  } catch {
    return []
  }
}

function saveProjectTrafficHistory(history: ProjectTrafficSample[]) {
  if (typeof window === 'undefined') return

  try {
    window.localStorage.setItem(TRAFFIC_HISTORY_STORAGE_KEY, JSON.stringify(history))
  } catch {
    // 本地存储空间不足时不阻塞首页渲染，下一次运行时重新从实时数据开始采样。
  }
}

function normalizeTrafficTotals(totals: unknown): Record<string, number> {
  if (!totals || typeof totals !== 'object') return {}
  return Object.entries(totals as Record<string, unknown>).reduce<Record<string, number>>(
    (normalized, [projectId, value]) => {
      normalized[projectId] = Number.isFinite(Number(value)) ? Math.max(0, Number(value)) : 0
      return normalized
    },
    {},
  )
}

function pruneProjectTrafficHistory(history: ProjectTrafficSample[]) {
  const cutoff = Date.now() - TRAFFIC_HISTORY_RETENTION_MS
  return history
    .filter((sample) => sample.timestamp >= cutoff)
    .sort((left, right) => left.timestamp - right.timestamp)
    .slice(-TRAFFIC_HISTORY_MAX_SAMPLES)
}

function normalizeTrafficSampleTime(timestamp: number) {
  return Math.floor(timestamp / TRAFFIC_HISTORY_SAMPLE_MS) * TRAFFIC_HISTORY_SAMPLE_MS
}

function buildTrafficTimeBuckets(range: TrafficRange) {
  const hourMs = 60 * 60 * 1000
  const dayMs = 24 * hourMs
  const todayStart = startOfDay(Date.now())

  if (range === '24h') {
    return Array.from({ length: 24 }, (_, index) => {
      const start = todayStart + index * hourMs
      return { start, end: start + hourMs - 1 }
    })
  }

  const days = range === '7d' ? 7 : 30
  const start = todayStart - (days - 1) * dayMs
  return Array.from({ length: days }, (_, index) => {
    const bucketStart = start + index * dayMs
    return { start: bucketStart, end: bucketStart + dayMs - 1 }
  })
}

function buildProjectTrafficBucketValues(projectId: string) {
  const now = Date.now()
  return trafficTimeBuckets.value
    .map((bucket, index) => {
      if (bucket.start > now) return undefined
      const value =
        latestProjectTrafficValue(projectId, bucket.end) ??
        (bucket.start <= now && now <= bucket.end ? currentProjectTrafficTotals.value[projectId] ?? 0 : 0)
      return { index, value }
    })
    .filter((point): point is { index: number; value: number } => Boolean(point))
}

function buildProjectTrafficPoints(projectId: string): TrafficChartPoint[] {
  return buildProjectTrafficBucketValues(projectId).map(({ index, value }) => ({
    x: trafficPointX(index, trafficTimeBuckets.value.length),
    y: trafficPointY(value),
    value,
  }))
}

function latestProjectTrafficValue(projectId: string, timestamp: number): number | undefined {
  for (let index = projectTrafficHistory.value.length - 1; index >= 0; index -= 1) {
    const sample = projectTrafficHistory.value[index]
    if (sample.timestamp <= timestamp) {
      return sample.totals[projectId] ?? 0
    }
  }
  return undefined
}

function pointsToPolyline(points: TrafficChartPoint[]) {
  return points.map((point) => `${roundChartValue(point.x)},${roundChartValue(point.y)}`).join(' ')
}

function trafficPointX(index: number, total: number) {
  const width = trafficChartBounds.right - trafficChartBounds.left
  return trafficChartBounds.left + (total <= 1 ? width / 2 : (index / (total - 1)) * width)
}

function trafficPointY(value: number) {
  const height = trafficChartBounds.bottom - trafficChartBounds.top
  return trafficChartBounds.bottom - (value / maxProjectTrafficBytes.value) * height
}

function trafficChartStepY() {
  return (trafficChartBounds.bottom - trafficChartBounds.top) / (TRAFFIC_CHART_Y_TICKS - 1)
}

function formatTrafficTimeLabel(timestamp: number) {
  const date = new Date(timestamp)
  if (trafficRange.value === '24h') return `${String(date.getHours()).padStart(2, '0')}:00`
  return `${String(date.getMonth() + 1).padStart(2, '0')}/${String(date.getDate()).padStart(2, '0')}`
}

function shouldShowTrafficTimeLabel(index: number, total: number) {
  if (trafficRange.value === '24h') return index % 4 === 0 || index === total - 1
  if (trafficRange.value === '7d') return true
  return index === 0 || index === total - 1 || index % 5 === 0
}

function startOfDay(timestamp: number) {
  const date = new Date(timestamp)
  date.setHours(0, 0, 0, 0)
  return date.getTime()
}

function roundChartValue(value: number) {
  return Math.round(value * 10) / 10
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

.traffic-chart__axis text {
  fill: var(--text-tertiary);
  font-size: 11px;
  dominant-baseline: middle;
}

.traffic-chart__y-label {
  text-anchor: end;
}

.traffic-chart__x-label {
  text-anchor: middle;
}

.traffic-chart__project-line {
  fill: none;
  stroke-width: 2.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.traffic-chart__legend {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 16px;
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.traffic-chart__legend span {
  min-width: 0;
  max-width: 210px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
}

.traffic-chart__legend i {
  width: 18px;
  height: 2px;
  border-radius: var(--radius-full);
  flex-shrink: 0;
}

.traffic-chart__legend b {
  min-width: 0;
  overflow: hidden;
  color: var(--text-secondary);
  font-weight: var(--weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.traffic-chart__legend small {
  color: var(--text-tertiary);
  white-space: nowrap;
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
