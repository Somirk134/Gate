<template>
  <div class="monitoring-dashboard">
    <header class="monitoring-dashboard__header">
      <div>
        <h1>监控中心</h1>
        <p>实时观察隧道、运行时、连接、健康状态和流量。</p>
      </div>
      <button type="button" class="monitoring-dashboard__refresh" @click="refresh">
        <GIcon name="refresh" :size="14" />
        <span>{{ loading ? '刷新中' : '刷新' }}</span>
      </button>
    </header>

    <OverviewPanel :data="dashboard.overview" />

    <div class="monitoring-dashboard__grid monitoring-dashboard__grid--charts">
      <RealtimeChart :points="dashboard.realtimeSpeed" />
      <TrafficChart :points="dashboard.trafficTrend" />
    </div>

    <div class="monitoring-dashboard__grid monitoring-dashboard__grid--main">
      <TunnelStatistics :status="dashboard.tunnelStatus" :tunnels="dashboard.tunnels" />
      <ConnectionStatistics
        :connection="dashboard.statistics.connection"
        :trend="dashboard.connectionTrend" />
    </div>

    <div class="monitoring-dashboard__grid monitoring-dashboard__grid--side">
      <DashboardWidget title="系统健康" icon="shield-check">
        <div class="monitoring-dashboard__health">
          <HealthIndicator
            v-for="signal in dashboard.systemHealth.signals"
            :key="signal.target"
            :label="signal.target"
            :status="signal.status"
            :score="signal.score"
            :message="signal.message" />
        </div>
      </DashboardWidget>
      <RuntimePanel :runtime="dashboard.statistics.runtime" />
      <SystemStatistics :system="dashboard.statistics.system" />
    </div>

    <DashboardWidget title="最近活动" icon="history">
      <div class="monitoring-dashboard__activity">
        <article v-for="activity in dashboard.recentActivity" :key="activity.id">
          <span>{{ activity.category }}</span>
          <strong>{{ activity.title }}</strong>
          <small>{{ formatTime(activity.timestamp) }}</small>
        </article>
      </div>
    </DashboardWidget>

    <p v-if="error" class="monitoring-dashboard__error">
      {{ error }}
    </p>
  </div>
</template>

<script setup lang="ts">
import GIcon from '@components/icons/GIcon.vue'
import { useMonitoringDashboard } from '@/monitoring/composables/useMonitoringDashboard'
import ConnectionStatistics from './ConnectionStatistics.vue'
import DashboardWidget from './DashboardWidget.vue'
import HealthIndicator from './HealthIndicator.vue'
import OverviewPanel from './OverviewPanel.vue'
import RealtimeChart from './RealtimeChart.vue'
import RuntimePanel from './RuntimePanel.vue'
import SystemStatistics from './SystemStatistics.vue'
import TrafficChart from './TrafficChart.vue'
import TunnelStatistics from './TunnelStatistics.vue'

const { dashboard, loading, error, refresh } = useMonitoringDashboard()

function formatTime(timestamp: number) {
  return new Intl.DateTimeFormat('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(timestamp)
}
</script>

<style scoped>
.monitoring-dashboard {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: var(--content-max-width);
  margin: 0 auto;
  gap: var(--space-5);
}

.monitoring-dashboard__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.monitoring-dashboard__header h1 {
  margin: 0;
  color: var(--text-primary);
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
  line-height: var(--leading-tight);
}

.monitoring-dashboard__header p {
  margin-top: var(--space-1);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.monitoring-dashboard__refresh {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  min-height: 32px;
  padding: 0 var(--space-3);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  background: var(--bg-card);
  font: inherit;
  cursor: pointer;
}

.monitoring-dashboard__refresh:hover {
  color: var(--text-primary);
  border-color: var(--color-border-strong);
}

.monitoring-dashboard__grid {
  display: grid;
  gap: var(--space-4);
}

.monitoring-dashboard__grid--charts,
.monitoring-dashboard__grid--main {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.monitoring-dashboard__grid--side {
  grid-template-columns: 1.15fr 0.9fr 0.9fr;
}

.monitoring-dashboard__health,
.monitoring-dashboard__activity {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.monitoring-dashboard__activity article {
  display: grid;
  grid-template-columns: 96px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  min-height: 42px;
  padding: var(--space-3);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.monitoring-dashboard__activity span,
.monitoring-dashboard__activity small {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.monitoring-dashboard__activity strong {
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.monitoring-dashboard__error {
  color: var(--color-error);
  font-size: var(--text-sm);
}

@media (max-width: 1180px) {
  .monitoring-dashboard__grid--side {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 900px) {
  .monitoring-dashboard__grid--charts,
  .monitoring-dashboard__grid--main {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .monitoring-dashboard__header {
    flex-direction: column;
  }

  .monitoring-dashboard__refresh {
    width: 100%;
    justify-content: center;
  }

  .monitoring-dashboard__activity article {
    grid-template-columns: 1fr;
  }
}
</style>
