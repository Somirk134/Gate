<template>
  <section class="http-page">
    <header class="http-page__header">
      <div>
        <p>HTTP Tunnel</p>
        <h1>HTTP routes</h1>
        <span>{{ httpTunnels.length }} route(s) · {{ totalRequests }} request(s)</span>
      </div>
      <div class="http-page__actions">
        <GButton variant="secondary" icon="refresh" :loading="loading" @click="refresh">
          Refresh
        </GButton>
        <GButton variant="primary" icon="plus" @click="router.push('/tunnels?create=1')">
          Create
        </GButton>
      </div>
    </header>

    <div class="http-summary">
      <article>
        <span>Requests</span>
        <strong>{{ totalRequests }}</strong>
      </article>
      <article>
        <span>Success rate</span>
        <strong>{{ formatPercent(averageSuccessRate) }}</strong>
      </article>
      <article>
        <span>Average response</span>
        <strong>{{ Math.round(averageLatency) }}ms</strong>
      </article>
      <article>
        <span>Traffic</span>
        <strong>{{ formatSpeed(totalSpeed) }}</strong>
      </article>
    </div>

    <div v-if="!httpTunnels.length" class="http-empty">
      <GIcon name="globe" :size="32" />
      <strong>No HTTP tunnels</strong>
      <span>Create a tunnel with protocol HTTP to start routing by Host and Path.</span>
    </div>

    <div v-else class="http-layout">
      <section class="http-route-list" aria-label="HTTP tunnel routes">
        <article v-for="tunnel in httpTunnels" :key="tunnel.id" class="http-route">
          <div class="http-route__title">
            <span :class="`is-${statusTone(tunnel.status)}`" />
            <div>
              <strong>{{ tunnel.name }}</strong>
              <small>{{ tunnel.localHost ?? "127.0.0.1" }}:{{ tunnel.localPort ?? "-" }}</small>
            </div>
          </div>

          <dl>
            <div>
              <dt>Host</dt>
              <dd>{{ tunnel.host || "any" }}</dd>
            </div>
            <div>
              <dt>Path</dt>
              <dd>{{ tunnel.path || "/" }}</dd>
            </div>
            <div>
              <dt>Requests</dt>
              <dd>{{ tunnel.requestCount ?? 0 }}</dd>
            </div>
            <div>
              <dt>Success</dt>
              <dd>{{ formatPercent(tunnel.successRate ?? 0) }}</dd>
            </div>
            <div>
              <dt>Avg response</dt>
              <dd>{{ Math.round(tunnel.averageResponseTimeMs ?? 0) }}ms</dd>
            </div>
          </dl>
        </article>
      </section>

      <section class="http-recent" aria-label="Recent HTTP requests">
        <div class="http-recent__heading">
          <div>
            <p>Recent requests</p>
            <h2>Live log</h2>
          </div>
          <GIcon name="logs" :size="18" />
        </div>

        <div v-if="recentRequests.length" class="request-table">
          <article v-for="request in recentRequests" :key="request.key" class="request-row">
            <span>{{ request.method }}</span>
            <p>{{ request.host }}{{ request.url }}</p>
            <strong :class="{ failed: request.status >= 400 }">{{ request.status }}</strong>
            <small>{{ request.latencyMs }}ms</small>
          </article>
        </div>

        <div v-else class="http-empty http-empty--compact">
          <GIcon name="activity" :size="24" />
          <span>Waiting for HTTP traffic</span>
        </div>
      </section>
    </div>

    <p v-if="error" class="http-error">{{ error }}</p>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue"
import { useRouter } from "vue-router"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"
import { useMonitoringDashboard } from "@/monitoring/composables/useMonitoringDashboard"
import type { DashboardTunnel, HttpRequestRecord } from "@/monitoring/types"

const router = useRouter()
const { dashboard, loading, error, refresh } = useMonitoringDashboard()

const httpTunnels = computed(() =>
  dashboard.value.tunnels.filter((tunnel) => tunnel.protocol === "http"),
)

const totalRequests = computed(() =>
  httpTunnels.value.reduce((sum, tunnel) => sum + (tunnel.requestCount ?? 0), 0),
)

const averageSuccessRate = computed(() => {
  if (!httpTunnels.value.length) return 0
  const total = httpTunnels.value.reduce((sum, tunnel) => sum + (tunnel.successRate ?? 0), 0)
  return total / httpTunnels.value.length
})

const averageLatency = computed(() => {
  if (!httpTunnels.value.length) return 0
  const total = httpTunnels.value.reduce((sum, tunnel) => sum + (tunnel.averageResponseTimeMs ?? 0), 0)
  return total / httpTunnels.value.length
})

const totalSpeed = computed(() =>
  httpTunnels.value.reduce(
    (sum, tunnel) => sum + tunnel.uploadSpeedBps + tunnel.downloadSpeedBps,
    0,
  ),
)

const recentRequests = computed(() =>
  httpTunnels.value
    .flatMap((tunnel) =>
      (tunnel.recentRequests ?? []).map((request) => mapRequest(tunnel, request)),
    )
    .sort((a, b) => b.timestamp - a.timestamp)
    .slice(0, 16),
)

function mapRequest(tunnel: DashboardTunnel, request: HttpRequestRecord) {
  return {
    ...request,
    key: `${tunnel.id}-${request.timestamp}-${request.method}-${request.url}`,
  }
}

function statusTone(status: DashboardTunnel["status"]) {
  if (status === "running") return "online"
  if (status === "warning") return "warning"
  return "offline"
}

function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`
}

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
</script>

<style scoped>
.http-page {
  width: min(100%, var(--content-max-width));
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.http-page__header,
.http-page__actions,
.http-route__title,
.http-recent__heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.http-page__header p,
.http-recent__heading p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.http-page__header h1 {
  margin-top: 2px;
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.http-page__header span {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-secondary);
}

.http-summary {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
}

.http-summary article,
.http-route,
.http-recent,
.http-empty {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.http-summary article {
  min-height: 86px;
  display: grid;
  align-content: center;
  gap: var(--space-1);
  padding: var(--space-4);
}

.http-summary span,
.http-route dt {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.http-summary strong {
  color: var(--text-primary);
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
}

.http-layout {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(340px, 0.9fr) minmax(0, 1.1fr);
  gap: var(--space-4);
}

.http-route-list {
  min-height: 0;
  overflow: auto;
  display: grid;
  align-content: start;
  gap: var(--space-3);
}

.http-route {
  padding: var(--space-4);
}

.http-route__title {
  justify-content: flex-start;
}

.http-route__title > span {
  width: 9px;
  height: 9px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.http-route__title strong,
.http-route__title small {
  display: block;
}

.http-route__title small {
  margin-top: 2px;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.http-route dl {
  display: grid;
  gap: var(--space-2);
  margin-top: var(--space-4);
}

.http-route dl div {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
}

.http-route dd {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.http-recent {
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: var(--space-4);
}

.http-recent__heading {
  align-items: flex-start;
  margin-bottom: var(--space-3);
}

.http-recent__heading h2 {
  margin-top: 2px;
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
}

.request-table {
  display: grid;
  gap: var(--space-2);
}

.request-row {
  min-height: 40px;
  display: grid;
  grid-template-columns: 64px minmax(0, 1fr) 58px 64px;
  align-items: center;
  gap: var(--space-3);
  padding: 0 var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.request-row span,
.request-row small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.request-row p {
  min-width: 0;
  overflow: hidden;
  color: var(--text-secondary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.request-row strong {
  color: var(--status-online);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.request-row strong.failed {
  color: var(--color-error);
}

.http-empty {
  min-height: 360px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-2);
  color: var(--text-tertiary);
  text-align: center;
}

.http-empty strong {
  color: var(--text-primary);
  font-size: var(--text-lg);
}

.http-empty--compact {
  min-height: 260px;
  border: 0;
  background: var(--bg-input);
}

.http-error {
  color: var(--color-error);
}

.is-online { background: var(--status-online) !important; }
.is-warning { background: var(--status-warning) !important; }
.is-offline { background: var(--status-offline) !important; }

@media (max-width: 980px) {
  .http-summary,
  .http-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 680px) {
  .http-page__header,
  .http-page__actions {
    align-items: flex-start;
    flex-direction: column;
  }

  .request-row {
    grid-template-columns: 54px minmax(0, 1fr) 48px;
  }

  .request-row small {
    display: none;
  }
}
</style>
