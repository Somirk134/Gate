<template>
  <section class="diagnostics-page">
    <header class="diagnostics-hero">
      <div>
        <p>Diagnostics Center</p>
        <h1>连接与部署诊断</h1>
        <span>定位 DNS、Token、服务端、端口、超时、版本和本机环境问题。</span>
      </div>
      <div class="hero-actions">
        <GButton variant="secondary" icon="refresh" :loading="loading" @click="loadAll">重新诊断</GButton>
        <GButton variant="primary" icon="copy" @click="copyDiagnostics">复制诊断信息</GButton>
      </div>
    </header>

    <div class="diagnostics-layout">
      <main class="diagnostics-main">
        <section class="diagnostic-section">
          <header class="section-heading">
            <div>
              <strong>Server Connection Wizard</strong>
              <p>填写服务器地址、端口和 Token，立即获得结构化连接结果。</p>
            </div>
            <span :class="connectionBadgeClass">{{ connectionBadge }}</span>
          </header>

          <div class="connection-form">
            <label>
              <span>服务器地址</span>
              <input v-model.trim="server.host" autocomplete="off" placeholder="gate.example.com" />
            </label>
            <label>
              <span>端口</span>
              <input v-model.number="server.port" inputmode="numeric" type="number" placeholder="7000" />
            </label>
            <label class="connection-form__token">
              <span>Token</span>
              <input v-model.trim="server.token" autocomplete="off" type="password" placeholder="服务端 Token" />
            </label>
          </div>

          <div class="section-actions">
            <GButton variant="primary" icon="plug-zap" :loading="connectionLoading" @click="testConnection">
              测试连接
            </GButton>
            <GButton variant="secondary" icon="shield-check" :loading="deploymentLoading" @click="runDeployment">
              运行部署检查
            </GButton>
          </div>

          <article v-if="connectionReport" class="connection-card" :class="{ ok: connectionReport.ok }">
            <div class="connection-card__title">
              <GIcon :name="connectionReport.ok ? 'check-circle' : 'alert-circle'" :size="22" />
              <div>
                <strong>{{ connectionReport.title }}</strong>
                <span>{{ connectionReport.code }} · {{ connectionReport.elapsedMs }}ms</span>
              </div>
            </div>
            <dl>
              <div><dt>错误原因</dt><dd>{{ connectionReport.reason }}</dd></div>
              <div><dt>可能原因</dt><dd>{{ connectionReport.possibleCause }}</dd></div>
              <div><dt>解决方案</dt><dd>{{ connectionReport.solution }}</dd></div>
            </dl>
            <div class="section-actions">
              <GButton variant="ghost" icon="logs" @click="openLogs">查看日志</GButton>
              <GButton variant="ghost" icon="copy" @click="copyConnectionError">复制错误</GButton>
            </div>
          </article>
        </section>

        <section class="diagnostic-section">
          <header class="section-heading">
            <div>
              <strong>Connection Diagnostics</strong>
              <p>服务器在线、端口开放、Token、版本、时间同步和延迟。</p>
            </div>
          </header>
          <div class="diagnostic-grid">
            <article v-for="item in connectionChecks" :key="item.label" :class="`is-${item.status}`">
              <GIcon :name="findingIcon(item.status)" :size="18" />
              <strong>{{ item.label }}</strong>
              <span>{{ item.value }}</span>
            </article>
          </div>
        </section>

        <section class="diagnostic-section">
          <header class="section-heading">
            <div>
              <strong>Deployment Checker</strong>
              <p>{{ deploymentReport?.summary ?? "检查 Rust Server、配置文件、日志目录、权限、监听端口和配置合法性。" }}</p>
            </div>
          </header>
          <div class="finding-list">
            <article v-for="finding in deploymentReport?.findings ?? []" :key="finding.id" :class="`is-${finding.status}`">
              <GIcon :name="findingIcon(finding.status)" :size="17" />
              <div>
                <strong>{{ finding.label }}</strong>
                <p>{{ finding.reason }}</p>
                <small>{{ finding.solution }}</small>
              </div>
            </article>
          </div>
        </section>
      </main>

      <aside class="diagnostics-side">
        <section class="side-panel">
          <header>
            <strong>Version Checker</strong>
            <span :class="versionMismatch ? 'warn' : 'ok'">{{ versionMismatch ? "需确认" : "一致" }}</span>
          </header>
          <dl class="info-list">
            <div><dt>客户端</dt><dd>{{ systemInfo?.clientVersion ?? "-" }}</dd></div>
            <div><dt>服务端</dt><dd>{{ systemInfo?.serverVersion ?? "-" }}</dd></div>
            <div><dt>协议</dt><dd>{{ systemInfo?.protocolVersion ?? "-" }}</dd></div>
          </dl>
          <p v-if="versionMismatch" class="side-warning">客户端与服务端版本未确认一致，建议先升级到相同 Beta 版本。</p>
        </section>

        <section class="side-panel">
          <header>
            <strong>System Info</strong>
            <button type="button" @click="copySystemInfo"><GIcon name="copy" :size="14" /></button>
          </header>
          <dl class="info-list">
            <div v-for="item in systemInfoRows" :key="item.label">
              <dt>{{ item.label }}</dt>
              <dd>{{ item.value }}</dd>
            </div>
          </dl>
        </section>

        <section class="side-panel">
          <header>
            <strong>Recent Server</strong>
            <span>收藏预留</span>
          </header>
          <div class="compact-list">
            <button v-for="recent in recentServers" :key="recent.serverAddr" type="button" @click="applyRecent(recent.serverAddr)">
              <span>{{ recent.serverAddr }}</span>
              <small>{{ recent.successCount }} 次成功</small>
            </button>
            <p v-if="!recentServers.length">暂无最近连接。</p>
          </div>
        </section>

        <section class="side-panel">
          <header>
            <strong>Connection History</strong>
            <button type="button" @click="clearHistory"><GIcon name="trash" :size="14" /></button>
          </header>
          <div class="history-compact">
            <article v-for="entry in connectionHistory" :key="entry.id">
              <span :class="entry.result">{{ entry.result === "success" ? "成功" : "失败" }}</span>
              <div>
                <strong>{{ entry.serverAddr }}</strong>
                <small>{{ entry.failureReason || `${entry.elapsedMs}ms` }}</small>
              </div>
            </article>
            <p v-if="!connectionHistory.length">暂无连接记录。</p>
          </div>
        </section>
      </aside>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue"
import { useRouter } from "vue-router"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"
import { useFeedback } from "@composables/useFeedback"
import { diagnosticsService } from "@/services"
import type {
  ConnectionHistoryEntry,
  ConnectionTestReport,
  DeploymentCheckReport,
  DiagnosticFinding,
  DiagnosticStatus,
  RecentServer,
  SystemInfoReport,
} from "@/services"

const router = useRouter()
const { toast } = useFeedback()

const loading = ref(false)
const connectionLoading = ref(false)
const deploymentLoading = ref(false)
const connectionReport = ref<ConnectionTestReport | null>(null)
const deploymentReport = ref<DeploymentCheckReport | null>(null)
const systemInfo = ref<SystemInfoReport | null>(null)
const recentServers = ref<RecentServer[]>([])
const connectionHistory = ref<ConnectionHistoryEntry[]>([])

const server = reactive({
  host: "127.0.0.1",
  port: 7000,
  token: "",
})

const versionMismatch = computed(() => {
  if (!systemInfo.value) return false
  return systemInfo.value.serverVersion !== "未连接" && systemInfo.value.clientVersion !== systemInfo.value.serverVersion
})

const connectionBadge = computed(() => {
  if (connectionLoading.value) return "测试中"
  if (connectionReport.value?.ok) return "已通过"
  if (connectionReport.value) return "失败"
  return "未测试"
})

const connectionBadgeClass = computed(() => {
  if (connectionLoading.value) return "badge warning"
  if (connectionReport.value?.ok) return "badge ok"
  if (connectionReport.value) return "badge error"
  return "badge"
})

const connectionChecks = computed<Array<{ label: string; value: string; status: DiagnosticStatus }>>(() => [
  {
    label: "服务器在线",
    value: connectionReport.value?.ok ? "在线" : connectionReport.value ? connectionReport.value.title : "未测试",
    status: connectionReport.value?.ok ? "ok" : connectionReport.value ? "error" : "warning",
  },
  {
    label: "端口开放",
    value: connectionReport.value?.code === "PORT_UNREACHABLE" ? "不可达" : connectionReport.value ? "已检查" : "未测试",
    status: connectionReport.value?.code === "PORT_UNREACHABLE" ? "error" : connectionReport.value ? "ok" : "warning",
  },
  {
    label: "Token 合法",
    value: connectionReport.value?.code === "TOKEN_ERROR" || connectionReport.value?.code === "TOKEN_EMPTY" ? "未通过" : connectionReport.value?.ok ? "通过" : "待检查",
    status: connectionReport.value?.code === "TOKEN_ERROR" || connectionReport.value?.code === "TOKEN_EMPTY" ? "error" : connectionReport.value?.ok ? "ok" : "warning",
  },
  {
    label: "客户端版本",
    value: systemInfo.value?.clientVersion ?? "-",
    status: "ok",
  },
  {
    label: "服务端版本",
    value: systemInfo.value?.serverVersion ?? "-",
    status: versionMismatch.value ? "warning" : "ok",
  },
  {
    label: "时间同步",
    value: "本机时间正常，服务端时间需连接后确认",
    status: connectionReport.value?.ok ? "ok" : "warning",
  },
  {
    label: "网络延迟",
    value: connectionReport.value ? `${connectionReport.value.elapsedMs}ms` : "未测试",
    status: connectionReport.value?.ok ? "ok" : "warning",
  },
])

const systemInfoRows = computed(() => {
  const info = systemInfo.value
  if (!info) return []
  return [
    { label: "Rust", value: info.rustVersion },
    { label: "系统", value: info.os },
    { label: "CPU", value: info.cpu },
    { label: "内存", value: info.memory },
    { label: "架构", value: info.arch },
    { label: "配置目录", value: info.configDir },
    { label: "日志目录", value: info.logDir },
  ]
})

onMounted(() => {
  void loadAll()
})

async function loadAll() {
  loading.value = true
  try {
    await Promise.all([runDeployment(), loadSystemInfo()])
    refreshMemory()
  } finally {
    loading.value = false
  }
}

async function testConnection() {
  connectionLoading.value = true
  try {
    connectionReport.value = await diagnosticsService.testConnection(server)
    refreshMemory()
    toast[connectionReport.value.ok ? "success" : "error"](connectionReport.value.title)
  } finally {
    connectionLoading.value = false
  }
}

async function runDeployment() {
  deploymentLoading.value = true
  try {
    deploymentReport.value = await diagnosticsService.runDeployment(diagnosticsService.formatServerAddr(server))
  } finally {
    deploymentLoading.value = false
  }
}

async function loadSystemInfo() {
  systemInfo.value = await diagnosticsService.collectSystemInfo()
}

function refreshMemory() {
  recentServers.value = diagnosticsService.getRecentServers()
  connectionHistory.value = diagnosticsService.getConnectionHistory()
}

function applyRecent(value: string) {
  const index = value.lastIndexOf(":")
  server.host = index > -1 ? value.slice(0, index) : value
  server.port = index > -1 ? Number(value.slice(index + 1)) || 7000 : 7000
}

function findingIcon(status: DiagnosticFinding["status"]) {
  if (status === "ok") return "check-circle"
  if (status === "warning") return "alert-triangle"
  return "alert-circle"
}

function openLogs() {
  void router.push("/logs")
}

function copyConnectionError() {
  if (!connectionReport.value) return
  void diagnosticsService.copyText(JSON.stringify(connectionReport.value, null, 2))
  toast.success("错误信息已复制")
}

function copySystemInfo() {
  if (!systemInfo.value) return
  void diagnosticsService.copyText(JSON.stringify(systemInfo.value, null, 2))
  toast.success("系统信息已复制")
}

function copyDiagnostics() {
  const payload = {
    connection: connectionReport.value,
    deployment: deploymentReport.value,
    system: systemInfo.value,
    history: connectionHistory.value,
  }
  void diagnosticsService.copyText(JSON.stringify(payload, null, 2))
  toast.success("诊断信息已复制")
}

function clearHistory() {
  diagnosticsService.clearHistory()
  refreshMemory()
  toast.success("连接历史已清空")
}
</script>

<style scoped>
.diagnostics-page {
  width: min(100%, var(--content-max-width));
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.diagnostics-hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  flex-shrink: 0;
}

.diagnostics-hero p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.diagnostics-hero h1 {
  margin-top: 2px;
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.diagnostics-hero span {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-secondary);
}

.hero-actions,
.section-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.diagnostics-layout {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 340px;
  gap: var(--space-4);
}

.diagnostics-main,
.diagnostics-side {
  min-height: 0;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.diagnostic-section,
.side-panel {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  padding: var(--space-4);
}

.section-heading,
.side-panel header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.section-heading strong,
.side-panel strong {
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
}

.section-heading p {
  margin-top: 2px;
  color: var(--text-secondary);
}

.badge,
.side-panel header span {
  min-height: 24px;
  display: inline-flex;
  align-items: center;
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 0 var(--space-2);
  font-size: var(--text-xs);
}

.badge.ok,
.side-panel header span.ok {
  background: var(--color-success-muted);
  color: var(--color-success);
}

.badge.warning,
.side-panel header span.warn {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}

.badge.error {
  background: var(--color-error-muted);
  color: var(--color-error);
}

.connection-form {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 140px;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.connection-form__token {
  grid-column: 1 / -1;
}

.connection-form label {
  display: grid;
  gap: var(--space-2);
}

.connection-form label span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.connection-form input {
  height: 38px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: 0 var(--space-3);
  outline: 0;
}

.connection-form input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.connection-card {
  display: grid;
  gap: var(--space-3);
  margin-top: var(--space-4);
  padding: var(--space-4);
  border: 1px solid rgba(255, 92, 92, 0.3);
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
}

.connection-card.ok {
  border-color: rgba(47, 209, 124, 0.32);
  background: var(--color-success-muted);
}

.connection-card__title {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.connection-card__title svg {
  color: var(--color-error);
}

.connection-card.ok .connection-card__title svg {
  color: var(--color-success);
}

.connection-card__title span {
  display: block;
  margin-top: 2px;
  color: var(--text-secondary);
}

.connection-card dl {
  display: grid;
  gap: var(--space-2);
}

.connection-card dl div {
  display: grid;
  grid-template-columns: 88px minmax(0, 1fr);
  gap: var(--space-3);
}

.connection-card dt,
.info-list dt {
  color: var(--text-tertiary);
}

.connection-card dd,
.info-list dd {
  color: var(--text-primary);
  overflow-wrap: anywhere;
}

.diagnostic-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.diagnostic-grid article {
  min-height: 104px;
  display: grid;
  align-content: center;
  gap: var(--space-2);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.diagnostic-grid article.is-ok svg { color: var(--color-success); }
.diagnostic-grid article.is-warning svg { color: var(--color-warning); }
.diagnostic-grid article.is-error svg { color: var(--color-error); }

.diagnostic-grid span {
  color: var(--text-secondary);
  line-height: var(--leading-normal);
}

.finding-list {
  display: grid;
  gap: var(--space-2);
}

.finding-list article {
  min-height: 74px;
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr);
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.finding-list article.is-ok svg { color: var(--color-success); }
.finding-list article.is-warning svg { color: var(--color-warning); }
.finding-list article.is-error svg { color: var(--color-error); }

.finding-list p {
  color: var(--text-secondary);
}

.finding-list small {
  display: block;
  margin-top: 2px;
  color: var(--text-tertiary);
}

.side-panel {
  display: grid;
  gap: var(--space-3);
}

.side-panel header {
  margin-bottom: 0;
}

.side-panel header button {
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

.side-panel header button:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.info-list {
  display: grid;
  gap: var(--space-2);
}

.info-list div {
  display: grid;
  grid-template-columns: 76px minmax(0, 1fr);
  gap: var(--space-2);
}

.side-warning {
  color: var(--color-warning);
  line-height: var(--leading-relaxed);
}

.compact-list,
.history-compact {
  display: grid;
  gap: var(--space-2);
}

.compact-list button,
.history-compact article {
  min-height: 42px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: var(--space-2);
}

.compact-list button {
  display: grid;
  gap: 2px;
  text-align: left;
  cursor: pointer;
}

.compact-list button:hover {
  border-color: var(--color-primary);
}

.compact-list small,
.history-compact small,
.compact-list p,
.history-compact p {
  color: var(--text-tertiary);
}

.history-compact article {
  display: grid;
  grid-template-columns: 48px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-2);
}

.history-compact article > span {
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  font-size: var(--text-xs);
}

.history-compact article > span.success {
  background: var(--color-success-muted);
  color: var(--color-success);
}

.history-compact article > span.failed {
  background: var(--color-error-muted);
  color: var(--color-error);
}

@media (max-width: 1120px) {
  .diagnostics-layout {
    grid-template-columns: 1fr;
  }

  .diagnostics-side {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 760px) {
  .diagnostics-hero,
  .section-heading {
    flex-direction: column;
    align-items: flex-start;
  }

  .connection-form,
  .diagnostic-grid,
  .diagnostics-side {
    grid-template-columns: 1fr;
  }
}
</style>
