<template>
  <section class="dashboard-page" aria-label="Gate dashboard">
    <header class="dashboard-hero">
      <div class="dashboard-hero__status">
        <span class="status-orb" :class="`is-${connectionTone}`" />
        <div>
          <p>{{ connectionLabel }}</p>
          <h1>Gate is {{ connectionStatusText }}</h1>
        </div>
      </div>
      <div class="dashboard-hero__actions">
        <GButton variant="secondary" icon="refresh" :loading="loading" @click="refresh">
          刷新
        </GButton>
        <GButton variant="primary" icon="plus" @click="router.push('/tunnels?create=1')">
          创建 Tunnel
        </GButton>
      </div>
    </header>

    <div class="dashboard-overview">
      <article class="metric-card metric-card--wide">
        <span class="metric-card__icon is-success"><GIcon name="wifi" :size="18" /></span>
        <div>
          <p>当前连接</p>
          <strong>{{ dashboard.statistics.connection.currentConnection }}</strong>
          <small>{{ Math.round(dashboard.overview.averageRttMs) }}ms RTT</small>
        </div>
      </article>

      <article class="metric-card">
        <span class="metric-card__icon"><GIcon name="router" :size="18" /></span>
        <div>
          <p>Tunnel</p>
          <strong>{{ dashboard.overview.runningTunnel }} / {{ dashboard.overview.tunnelCount }}</strong>
          <small>运行中 / 总数</small>
        </div>
      </article>

      <article class="metric-card">
        <span class="metric-card__icon is-info"><GIcon name="download" :size="18" /></span>
        <div>
          <p>今日流量</p>
          <strong>{{ formatBytes(dashboard.overview.todayTraffic) }}</strong>
          <small>累计上下行</small>
        </div>
      </article>

      <article class="metric-card">
        <span class="metric-card__icon is-warning"><GIcon name="zap" :size="18" /></span>
        <div>
          <p>实时速度</p>
          <strong>{{ formatSpeed(totalSpeed) }}</strong>
          <small>↓ {{ formatSpeed(dashboard.statistics.traffic.downloadSpeedBps) }} · ↑ {{ formatSpeed(dashboard.statistics.traffic.uploadSpeedBps) }}</small>
        </div>
      </article>

      <article class="metric-card">
        <span class="metric-card__icon is-purple"><GIcon name="clock" :size="18" /></span>
        <div>
          <p>运行时间</p>
          <strong>{{ formatDuration(dashboard.overview.runtimeUptimeSeconds) }}</strong>
          <small>Runtime uptime</small>
        </div>
      </article>
    </div>

    <div class="dashboard-grid">
      <section class="dashboard-panel dashboard-panel--servers">
        <div class="panel-heading">
          <div>
            <p>Server Status</p>
            <h2>服务器状态</h2>
          </div>
          <GIcon name="servers" :size="18" />
        </div>
        <div class="server-strip">
          <article v-for="server in serverCards" :key="server.label" class="server-pill">
            <span :class="`is-${server.tone}`" />
            <div>
              <strong>{{ server.count }}</strong>
              <small>{{ server.label }}</small>
            </div>
          </article>
        </div>
        <div class="health-row">
          <span>Health score</span>
          <strong>{{ Math.round(dashboard.overview.healthScore) }}%</strong>
          <div class="health-meter" aria-hidden="true">
            <i :style="{ width: `${Math.round(dashboard.overview.healthScore)}%` }" />
          </div>
        </div>
      </section>

      <section class="dashboard-panel dashboard-panel--speed">
        <div class="panel-heading">
          <div>
            <p>Realtime Speed</p>
            <h2>实时速度</h2>
          </div>
          <GIcon name="chart-line" :size="18" />
        </div>
        <div v-if="sparkline.length" class="speed-sparkline" aria-label="Realtime speed trend">
          <span
            v-for="(point, index) in sparkline"
            :key="`${point.timestamp}-${index}`"
            :style="{ height: `${point.height}%` }"
          />
        </div>
        <div v-else class="mini-empty">
          <GIcon name="chart-line" :size="22" />
          <span>暂无数据</span>
        </div>
        <div class="speed-row">
          <span>Download <strong>{{ formatSpeed(dashboard.statistics.traffic.downloadSpeedBps) }}</strong></span>
          <span>Upload <strong>{{ formatSpeed(dashboard.statistics.traffic.uploadSpeedBps) }}</strong></span>
        </div>
      </section>

      <section class="dashboard-panel dashboard-panel--logs">
        <div class="panel-heading">
          <div>
            <p>Recent Logs</p>
            <h2>最近日志</h2>
          </div>
          <button class="panel-link" type="button" @click="router.push('/logs')">
            <GIcon name="arrow-right" :size="14" />
          </button>
        </div>
        <div v-if="recentLogs.length" class="recent-log-list">
          <article v-for="log in recentLogs" :key="log.id" class="recent-log">
            <span :class="`is-${log.level.toLowerCase()}`">{{ log.level }}</span>
            <p>{{ log.message }}</p>
            <small>{{ formatClock(log.timestamp) }}</small>
          </article>
        </div>
        <div v-else class="mini-empty">
          <GIcon name="logs" :size="22" />
          <span>暂无日志</span>
        </div>
      </section>

      <section class="dashboard-panel dashboard-panel--version">
        <div class="panel-heading">
          <div>
            <p>Version</p>
            <h2>版本信息</h2>
          </div>
          <GIcon name="sparkles" :size="18" />
        </div>
        <dl class="version-list">
          <div>
            <dt>Desktop</dt>
            <dd>v{{ appVersion }}</dd>
          </div>
          <div>
            <dt>Runtime</dt>
            <dd>{{ connectionStatusText }}</dd>
          </div>
          <div>
            <dt>Updated</dt>
            <dd>{{ formatClock(dashboard.generatedAt) }}</dd>
          </div>
        </dl>
      </section>
    </div>

    <p v-if="error" class="dashboard-error">{{ error }}</p>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue"
import { useRouter } from "vue-router"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"
import { useMonitoringDashboard } from "@/monitoring/composables/useMonitoringDashboard"
import { useLog } from "@views/logs/hooks"

const router = useRouter()
const { dashboard, healthStatus, loading, error, refresh } = useMonitoringDashboard()
const { logs } = useLog()
const appVersion = "0.1.0"

const connectionTone = computed(() => {
  if (healthStatus.value === "healthy") return "online"
  if (healthStatus.value === "warning") return "warning"
  if (healthStatus.value === "critical") return "error"
  return "offline"
})

const connectionLabel = computed(() => {
  if (healthStatus.value === "healthy") return "Connected"
  if (healthStatus.value === "warning") return "Needs attention"
  if (healthStatus.value === "critical") return "Connection issue"
  return "Offline"
})

const connectionStatusText = computed(() => {
  if (healthStatus.value === "healthy") return "online"
  if (healthStatus.value === "warning") return "degraded"
  if (healthStatus.value === "critical") return "blocked"
  return "offline"
})

const totalSpeed = computed(
  () => dashboard.value.statistics.traffic.downloadSpeedBps + dashboard.value.statistics.traffic.uploadSpeedBps,
)

const serverCards = computed(() =>
  dashboard.value.serverStatus.map((server) => ({
    ...server,
    tone: server.label.toLowerCase().includes("online")
      ? "online"
      : server.label.toLowerCase().includes("warning")
        ? "warning"
        : "offline",
  })),
)

const sparkline = computed(() => {
  const points = dashboard.value.realtimeSpeed.slice(-26)
  const max = Math.max(1, ...points.map((point) => point.downloadBps + point.uploadBps))
  return points.map((point) => ({
    timestamp: point.timestamp,
    height: Math.max(12, Math.round(((point.downloadBps + point.uploadBps) / max) * 100)),
  }))
})

const recentLogs = computed(() => logs.value.slice(-5).reverse())

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B"
  const units = ["B", "KB", "MB", "GB", "TB"]
  const index = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)))
  const value = bytes / 1024 ** index
  return `${value.toFixed(value >= 10 || index === 0 ? 0 : 1)} ${units[index]}`
}

function formatSpeed(bytesPerSecond: number): string {
  return `${formatBytes(bytesPerSecond)}/s`
}

function formatDuration(seconds: number): string {
  const day = Math.floor(seconds / 86400)
  const hour = Math.floor((seconds % 86400) / 3600)
  const minute = Math.floor((seconds % 3600) / 60)
  if (day) return `${day}d ${hour}h`
  if (hour) return `${hour}h ${minute}m`
  return `${Math.max(1, minute)}m`
}

function formatClock(timestamp: number): string {
  return new Intl.DateTimeFormat("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).format(timestamp)
}
</script>

<style scoped>
.dashboard-page {
  width: min(100%, var(--content-max-width));
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
}

.dashboard-hero {
  min-height: 116px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-6);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-xl);
  background: linear-gradient(180deg, var(--bg-surface-raised), var(--bg-surface));
  box-shadow: var(--shadow-sm);
}

.dashboard-hero__status {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.status-orb {
  position: relative;
  width: 18px;
  height: 18px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
  box-shadow: 0 0 0 8px var(--status-offline-bg);
}

.status-orb::after {
  content: "";
  position: absolute;
  inset: -7px;
  border-radius: inherit;
  border: 1px solid currentColor;
  opacity: 0.38;
  animation: g-ping 1.8s var(--ease-out) infinite;
}

.status-orb.is-online { color: var(--status-online); background: var(--status-online); box-shadow: 0 0 0 8px var(--status-online-bg); }
.status-orb.is-warning { color: var(--status-warning); background: var(--status-warning); box-shadow: 0 0 0 8px var(--status-warning-bg); }
.status-orb.is-error { color: var(--status-error); background: var(--status-error); box-shadow: 0 0 0 8px var(--status-error-bg); }
.status-orb.is-offline { color: var(--status-offline); }

.dashboard-hero p,
.panel-heading p,
.metric-card p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
  text-transform: uppercase;
}

.dashboard-hero h1 {
  margin-top: var(--space-1);
  font-size: clamp(24px, 3vw, 34px);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
  line-height: var(--leading-tight);
}

.dashboard-hero__actions {
  display: flex;
  gap: var(--space-2);
  flex-shrink: 0;
}

.dashboard-overview {
  display: grid;
  grid-template-columns: 1.15fr repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
}

.metric-card,
.dashboard-panel {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.metric-card {
  min-height: 104px;
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4);
}

.metric-card__icon {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
  flex-shrink: 0;
}

.metric-card__icon.is-success { background: var(--color-success-muted); color: var(--color-success); }
.metric-card__icon.is-info { background: var(--color-info-muted); color: var(--color-info); }
.metric-card__icon.is-warning { background: var(--color-warning-muted); color: var(--color-warning); }
.metric-card__icon.is-purple { background: var(--color-secondary-muted); color: var(--color-secondary); }

.metric-card strong {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-primary);
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.metric-card small {
  display: block;
  margin-top: 2px;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: var(--leading-normal);
}

.dashboard-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-auto-rows: minmax(220px, auto);
  gap: var(--space-4);
}

.dashboard-panel {
  min-width: 0;
  padding: var(--space-4);
}

.panel-heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.panel-heading h2 {
  margin-top: 2px;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.panel-link {
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
}

.panel-link:hover {
  color: var(--text-primary);
  border-color: var(--border-strong);
}

.server-strip {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.server-pill {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  min-height: 72px;
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.server-pill > span {
  width: 9px;
  height: 9px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.server-pill > span.is-online { background: var(--status-online); }
.server-pill > span.is-warning { background: var(--status-warning); }
.server-pill > span.is-offline { background: var(--status-offline); }

.server-pill strong {
  display: block;
  color: var(--text-primary);
  font-size: var(--text-xl);
}

.server-pill small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.health-row {
  display: grid;
  grid-template-columns: auto auto;
  gap: var(--space-2) var(--space-3);
  align-items: center;
  margin-top: var(--space-4);
  color: var(--text-secondary);
}

.health-row strong {
  justify-self: end;
  color: var(--text-primary);
}

.health-meter {
  grid-column: 1 / -1;
  height: 8px;
  overflow: hidden;
  border-radius: var(--radius-full);
  background: var(--bg-input);
}

.health-meter i {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--color-success), var(--color-info));
}

.speed-sparkline {
  height: 118px;
  display: flex;
  align-items: end;
  gap: 4px;
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.speed-sparkline span {
  flex: 1;
  min-width: 3px;
  border-radius: var(--radius-full) var(--radius-full) 0 0;
  background: linear-gradient(180deg, var(--color-info), var(--color-primary));
}

.speed-row {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
  margin-top: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.speed-row strong {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.recent-log-list {
  display: grid;
  gap: var(--space-2);
}

.recent-log {
  min-height: 38px;
  display: grid;
  grid-template-columns: 58px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.recent-log span {
  color: var(--text-tertiary);
  font: var(--weight-semibold) var(--text-xs) var(--font-mono);
}

.recent-log span.is-info { color: var(--color-info); }
.recent-log span.is-warn { color: var(--color-warning); }
.recent-log span.is-error,
.recent-log span.is-fatal { color: var(--color-error); }
.recent-log span.is-debug,
.recent-log span.is-trace { color: var(--text-tertiary); }

.recent-log p {
  min-width: 0;
  overflow: hidden;
  color: var(--text-secondary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-log small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.version-list {
  display: grid;
  gap: var(--space-3);
}

.version-list div {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.version-list dt {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.version-list dd {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.mini-empty {
  min-height: 156px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-2);
  color: var(--text-tertiary);
}

.dashboard-error {
  color: var(--color-error);
  font-size: var(--text-sm);
}

@media (max-width: 1180px) {
  .dashboard-overview {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .metric-card--wide {
    grid-column: 1 / -1;
  }
}

@media (max-width: 900px) {
  .dashboard-hero {
    align-items: flex-start;
    flex-direction: column;
  }

  .dashboard-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .dashboard-overview,
  .server-strip {
    grid-template-columns: 1fr;
  }

  .dashboard-hero__actions,
  .speed-row {
    width: 100%;
    flex-direction: column;
  }

  .recent-log {
    grid-template-columns: 58px minmax(0, 1fr);
  }

  .recent-log small {
    display: none;
  }
}
</style>
