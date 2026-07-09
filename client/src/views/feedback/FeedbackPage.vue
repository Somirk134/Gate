<template>
  <section class="feedback-page">
    <header class="feedback-hero">
      <div>
        <p>Feedback</p>
        <h1>反馈与调试信息</h1>
        <span>把版本、系统、连接、部署检查和路径信息一次性整理好，方便提交 Bug。</span>
      </div>
      <GButton
        variant="primary"
        icon="copy"
        :loading="loading"
        @click="copyDebugInfo"
      >
        复制调试信息
      </GButton>
    </header>

    <div class="feedback-grid">
      <main class="feedback-main">
        <section class="feedback-section">
          <header>
            <strong>生成诊断信息</strong>
            <p>包含 System Info、Deployment Checker、Connection History 和最近服务器。</p>
          </header>
          <div class="action-grid">
            <button
              type="button"
              @click="copyDebugInfo"
            >
              <GIcon
                name="clipboard"
                :size="20"
              />
              <strong>复制调试信息</strong>
              <span>适合粘贴到 Issue 或聊天里。</span>
            </button>
            <button
              type="button"
              @click="openIssue"
            >
              <GIcon
                name="github"
                :size="20"
              />
              <strong>打开 GitHub Issue</strong>
              <span>带着诊断信息提交问题。</span>
            </button>
            <button
              type="button"
              @click="copyLogDir"
            >
              <GIcon
                name="logs"
                :size="20"
              />
              <strong>查看日志目录</strong>
              <span>{{ systemInfo?.logDir ?? "正在读取..." }}</span>
            </button>
            <button
              type="button"
              @click="copyConfigDir"
            >
              <GIcon
                name="settings"
                :size="20"
              />
              <strong>查看配置目录</strong>
              <span>{{ systemInfo?.configDir ?? "正在读取..." }}</span>
            </button>
          </div>
        </section>

        <section class="feedback-section">
          <header>
            <strong>错误反馈格式</strong>
            <p>所有错误都应包含错误原因、可能原因、解决方案、查看日志和复制错误。</p>
          </header>
          <div class="error-example">
            <div>
              <span>错误原因</span>
              <strong>{{ latestFailure?.failureReason || "暂无失败记录" }}</strong>
            </div>
            <div>
              <span>可能原因</span>
              <strong>DNS、Token、服务器进程、端口、防火墙或超时。</strong>
            </div>
            <div>
              <span>解决方案</span>
              <strong>运行诊断中心，按结构化建议逐项修复。</strong>
            </div>
            <div class="error-actions">
              <GButton
                variant="secondary"
                icon="logs"
                @click="openLogs"
              >
                查看日志
              </GButton>
              <GButton
                variant="secondary"
                icon="copy"
                @click="copyDebugInfo"
              >
                复制错误
              </GButton>
            </div>
          </div>
        </section>
      </main>

      <aside class="feedback-side">
        <section class="feedback-card">
          <header>
            <strong>System Info</strong>
            <button
              type="button"
              @click="copySystemInfo"
            >
              <GIcon
                name="copy"
                :size="14"
              />
            </button>
          </header>
          <dl>
            <div
              v-for="item in systemRows"
              :key="item.label"
            >
              <dt>{{ item.label }}</dt>
              <dd>{{ item.value }}</dd>
            </div>
          </dl>
        </section>

        <section class="feedback-card">
          <header>
            <strong>Deployment Checker</strong>
            <button
              type="button"
              @click="refresh"
            >
              <GIcon
                name="refresh"
                :size="14"
              />
            </button>
          </header>
          <div class="mini-findings">
            <article
              v-for="finding in deploymentReport?.findings ?? []"
              :key="finding.id"
              :class="`is-${finding.status}`"
            >
              <span />
              <div>
                <strong>{{ finding.label }}</strong>
                <small>{{ finding.reason }}</small>
              </div>
            </article>
          </div>
        </section>
      </aside>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue"
import { useRouter } from "vue-router"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"
import { useFeedback } from "@composables/useFeedback"
import { diagnosticsService } from "@/services"
import type {
  ConnectionHistoryEntry,
  DeploymentCheckReport,
  SystemInfoReport,
} from "@/services"

const router = useRouter()
const { toast } = useFeedback()

const loading = ref(false)
const systemInfo = ref<SystemInfoReport | null>(null)
const deploymentReport = ref<DeploymentCheckReport | null>(null)
const history = ref<ConnectionHistoryEntry[]>([])

const latestFailure = computed(() => history.value.find((entry) => entry.result === "failed"))
const systemRows = computed(() => {
  const info = systemInfo.value
  if (!info) return []
  return [
    { label: "客户端版本", value: info.clientVersion },
    { label: "服务端版本", value: info.serverVersion },
    { label: "协议版本", value: info.protocolVersion },
    { label: "Rust", value: info.rustVersion },
    { label: "系统", value: info.os },
    { label: "CPU", value: info.cpu },
    { label: "内存", value: info.memory },
    { label: "架构", value: info.arch },
  ]
})

onMounted(() => {
  void refresh()
})

async function refresh() {
  loading.value = true
  try {
    const [system, deployment] = await Promise.all([
      diagnosticsService.collectSystemInfo(),
      diagnosticsService.runDeployment(),
    ])
    systemInfo.value = system
    deploymentReport.value = deployment
    history.value = diagnosticsService.getConnectionHistory()
  } finally {
    loading.value = false
  }
}

async function copyDebugInfo() {
  if (!systemInfo.value || !deploymentReport.value) await refresh()
  const payload = {
    generatedAt: new Date().toISOString(),
    systemInfo: systemInfo.value,
    deployment: deploymentReport.value,
    recentServers: diagnosticsService.getRecentServers(),
    connectionHistory: diagnosticsService.getConnectionHistory(),
  }
  await diagnosticsService.copyText(JSON.stringify(payload, null, 2))
  toast.success("调试信息已复制")
}

async function copySystemInfo() {
  if (!systemInfo.value) await refresh()
  await diagnosticsService.copyText(JSON.stringify(systemInfo.value, null, 2))
  toast.success("系统信息已复制")
}

function copyLogDir() {
  if (!systemInfo.value) return
  void diagnosticsService.copyText(systemInfo.value.logDir)
  toast.success("日志目录路径已复制")
}

function copyConfigDir() {
  if (!systemInfo.value) return
  void diagnosticsService.copyText(systemInfo.value.configDir)
  toast.success("配置目录路径已复制")
}

function openIssue() {
  window.open("https://github.com/gate/gate/issues/new?template=bug_report.yml", "_blank", "noopener,noreferrer")
}

function openLogs() {
  void router.push("/logs")
}
</script>

<style scoped>
.feedback-page {
  width: min(100%, var(--content-max-width));
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.feedback-hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  flex-shrink: 0;
}

.feedback-hero p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.feedback-hero h1 {
  margin-top: 2px;
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.feedback-hero span {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-secondary);
}

.feedback-grid {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 340px;
  gap: var(--space-4);
}

.feedback-main,
.feedback-side {
  min-height: 0;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.feedback-section,
.feedback-card {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  padding: var(--space-4);
}

.feedback-section header,
.feedback-card header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.feedback-section strong,
.feedback-card strong {
  color: var(--text-primary);
  font-size: var(--text-lg);
}

.feedback-section header p {
  margin-top: 2px;
  color: var(--text-secondary);
}

.feedback-card header button {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.feedback-card header button:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.action-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.action-grid button {
  min-height: 128px;
  display: grid;
  align-content: center;
  gap: var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: var(--space-4);
  text-align: left;
  cursor: pointer;
}

.action-grid button:hover {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
}

.action-grid svg {
  color: var(--color-primary);
}

.action-grid span {
  color: var(--text-secondary);
  line-height: var(--leading-normal);
  overflow-wrap: anywhere;
}

.error-example {
  display: grid;
  gap: var(--space-2);
}

.error-example > div:not(.error-actions) {
  display: grid;
  grid-template-columns: 92px minmax(0, 1fr);
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.error-example span,
.feedback-card dt {
  color: var(--text-tertiary);
}

.error-example strong,
.feedback-card dd {
  color: var(--text-primary);
  overflow-wrap: anywhere;
}

.error-actions {
  display: flex;
  gap: var(--space-2);
  margin-top: var(--space-2);
}

.feedback-card dl {
  display: grid;
  gap: var(--space-2);
}

.feedback-card dl div {
  display: grid;
  grid-template-columns: 86px minmax(0, 1fr);
  gap: var(--space-2);
}

.mini-findings {
  display: grid;
  gap: var(--space-2);
}

.mini-findings article {
  display: grid;
  grid-template-columns: 9px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-2);
  min-height: 42px;
}

.mini-findings article > span {
  width: 9px;
  height: 9px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.mini-findings article.is-ok > span { background: var(--color-success); }
.mini-findings article.is-warning > span { background: var(--color-warning); }
.mini-findings article.is-error > span { background: var(--color-error); }

.mini-findings small {
  display: block;
  margin-top: 2px;
  color: var(--text-tertiary);
  overflow-wrap: anywhere;
}

@media (max-width: 1020px) {
  .feedback-grid {
    grid-template-columns: 1fr;
  }

  .feedback-side {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 720px) {
  .feedback-hero {
    flex-direction: column;
  }

  .action-grid,
  .feedback-side {
    grid-template-columns: 1fr;
  }
}
</style>
