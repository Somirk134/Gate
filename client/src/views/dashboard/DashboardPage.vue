<!--
  DashboardPage — 开发者工作台
  ------------------------------------------------------------------
  整个软件的控制中心。所有数据来自 Mock，通过 Dashboard Store 统一管理，
  后续可无缝替换为真实接口。

  布局：Header → Welcome → Quick Actions → Statistics → Projects
        → Tunnels → Servers → Activity + Monitor → News → Footer
-->
<template>
  <div class="dashboard-page">
    <!-- Loading State -->
    <DashboardLoadingState v-if="isLoading" />

    <!-- Error State -->
    <GCard v-else-if="isError" variant="plain" padding="lg">
      <GErrorState
        title="加载失败"
        :message="error || '无法连接到服务，请检查网络后重试。'"
        retry
        @retry="retry"
      />
    </GCard>

    <!-- Ready / Empty -->
    <template v-else>
      <!-- Header -->
      <DashboardHeader
        title="开发者工作台"
        description="你的本地到公网隧道控制中心"
        :last-updated="lastUpdated"
        @refresh="refresh"
      />

      <!-- Empty State -->
      <DashboardEmptyState
        v-if="!hasProjects"
        @create="handleQuickAction('new-project')"
        @connect="handleQuickAction('connect-server')"
      />

      <!-- Full Dashboard -->
      <template v-else>
        <!-- Welcome -->
        <WelcomeCard
          v-scroll-reveal
          username="开发者"
          :quote="randomQuote"
          version="v0.4.0-beta"
          :running-count="runningTunnels.length"
          :server-count="connectedServerCount"
        />

        <!-- Quick Actions -->
        <section v-scroll-reveal class="dashboard-section">
          <div class="dashboard-section__head">
            <div class="dashboard-section__title">
              <GIcon name="zap" :size="16" class="dashboard-section__title-icon" />
              <span>快捷操作</span>
            </div>
          </div>
          <div class="dashboard-grid--actions">
            <QuickActionCard
              v-for="action in actions"
              :key="action.id"
              :icon="action.icon"
              :label="action.label"
              :shortcut="action.shortcut"
              :variant="action.variant"
              @click="handleQuickAction(action.id)"
            />
          </div>
        </section>

        <!-- Statistics -->
        <StatisticsSection
          v-scroll-reveal
          :statistics="statistics"
        />

        <!-- Project Overview -->
        <ProjectOverview
          v-scroll-reveal
          :projects="projects"
          @open="openProject"
          @toggle-pin="togglePin"
          @toggle-favorite="toggleFavorite"
          @start="startProject"
          @stop="stopProject"
          @view-all="goToProjects"
        />

        <!-- Running Tunnels -->
        <TunnelOverview
          v-scroll-reveal
          :tunnels="runningTunnels"
          @start="startTunnel"
          @stop="stopTunnel"
          @detail="openTunnel"
          @view-all="goToTunnels"
        />

        <!-- Server Status -->
        <ServerStatus
          v-scroll-reveal
          :servers="servers"
          @view-all="goToServers"
          @add-server="handleQuickAction('connect-server')"
        />

        <!-- Activity + Monitor -->
        <div v-scroll-reveal class="dashboard-grid--bottom">
          <ActivityTimeline :activities="activities" />
          <MonitorCard :resource="resource" />
        </div>

        <!-- News -->
        <NewsCard
          v-scroll-reveal
          :news="news"
          @open="openNews"
        />

        <!-- Footer -->
        <footer v-scroll-reveal class="dashboard-footer">
          <span>Gate v0.4.0-beta · 仅供开发预览</span>
          <div class="dashboard-footer__links">
            <a class="dashboard-footer__link" href="#">文档</a>
            <a class="dashboard-footer__link" href="#">GitHub</a>
            <a class="dashboard-footer__link" href="#">反馈</a>
          </div>
        </footer>
      </template>
    </template>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router"
import { useFeedback } from "@composables/useFeedback"
import GCard from "@components/base/GCard.vue"
import GIcon from "@components/icons/GIcon.vue"
import GErrorState from "@components/feedback/GErrorState.vue"

import DashboardHeader from "./components/DashboardHeader.vue"
import WelcomeCard from "./components/WelcomeCard.vue"
import QuickActionCard from "./components/QuickActionCard.vue"
import ProjectOverview from "./components/ProjectOverview.vue"
import TunnelOverview from "./components/TunnelOverview.vue"
import ServerStatus from "./components/ServerStatus.vue"
import StatisticsSection from "./components/StatisticsSection.vue"
import ActivityTimeline from "./components/ActivityTimeline.vue"
import MonitorCard from "./components/MonitorCard.vue"
import NewsCard from "./components/NewsCard.vue"
import DashboardEmptyState from "./components/DashboardEmptyState.vue"
import DashboardLoadingState from "./components/DashboardLoadingState.vue"

import { useDashboardData } from "./composables/useDashboardData"
import { vScrollReveal } from "./composables/useScrollReveal"
import type { DashboardProject, DashboardTunnel, DashboardNews } from "./types"

import "./styles/dashboard.css"

const router = useRouter()
const { toast } = useFeedback()

const {
  projects,
  servers,
  activities,
  statistics,
  resource,
  news,
  actions,
  error,
  isLoading,
  isError,
  hasProjects,
  randomQuote,
  lastUpdated,
  runningTunnels,
  connectedServerCount,
  refresh,
  retry,
  togglePin,
  toggleFavorite,
  startTunnel,
  stopTunnel,
} = useDashboardData()

// ── Quick Actions ──
function handleQuickAction(id: string) {
  switch (id) {
    case "new-project":
      toast.info("即将打开：新建项目")
      router.push("/projects?new=1")
      break
    case "connect-server":
      toast.info("即将打开：连接服务器")
      router.push("/servers")
      break
    case "new-tunnel":
      toast.info("即将打开：新建 Tunnel")
      router.push("/projects")
      break
    case "open-logs":
      router.push("/logs")
      break
    case "open-settings":
      router.push("/settings")
      break
    case "check-update":
      toast.info("正在检查更新…")
      setTimeout(() => toast.success("当前已是最新版本 v0.4.0-beta"), 1200)
      break
  }
}

// ── Project handlers ──
function openProject(project: DashboardProject) {
  router.push(`/projects/${project.id}`)
}

function startProject(project: DashboardProject) {
  toast.success(`正在启动项目「${project.name}」`)
}

function stopProject(project: DashboardProject) {
  toast.warning(`已停止项目「${project.name}」`)
}

function goToProjects() {
  router.push("/projects")
}

function goToTunnels() {
  router.push("/projects")
}

function goToServers() {
  router.push("/servers")
}

function openTunnel(tunnel: DashboardTunnel) {
  toast.info(`查看隧道：${tunnel.name}`)
}

function openNews(item: DashboardNews) {
  toast.info(item.title)
}
</script>
