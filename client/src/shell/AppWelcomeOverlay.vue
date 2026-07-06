<template>
  <Transition name="welcome">
    <div v-if="visible" class="welcome-overlay">
      <section class="welcome-shell" role="dialog" aria-modal="true" aria-labelledby="welcome-title">
        <aside class="welcome-rail">
          <div class="welcome-brand">
            <span><GIcon name="router" :size="24" /></span>
            <div>
              <strong>Gate</strong>
              <small>Beta Sprint 2</small>
            </div>
          </div>

          <nav class="welcome-steps" aria-label="First launch steps">
            <button
              v-for="(item, index) in flowSteps"
              :key="item.key"
              type="button"
              :class="{ active: step === index, done: step > index }"
              @click="step = Math.min(step, index)"
            >
              <span>{{ index + 1 }}</span>
              <div>
                <strong>{{ item.title }}</strong>
                <small>{{ item.caption }}</small>
              </div>
            </button>
          </nav>
        </aside>

        <main class="welcome-main">
          <header class="welcome-header">
            <div>
              <p>{{ currentStep.caption }}</p>
              <h1 id="welcome-title">{{ currentStep.title }}</h1>
            </div>
            <button type="button" class="welcome-skip" @click="skipWizard">跳过</button>
          </header>

          <div class="welcome-progress">
            <span :style="{ width: `${progressPercent}%` }" />
          </div>

          <section class="welcome-content">
            <div v-if="currentStep.key === 'welcome'" class="welcome-intro">
              <div class="intro-mark">
                <GIcon name="rocket" :size="30" />
              </div>
              <h2>5 分钟内完成部署并创建第一个 Tunnel</h2>
              <p>
                这个向导会检查本机环境、配置服务器、测试连接、选择常见场景模板，然后创建第一个 Tunnel。
                你不需要先阅读文档。
              </p>
              <div class="intro-grid">
                <article v-for="item in introCards" :key="item.title">
                  <GIcon :name="item.icon" :size="18" />
                  <strong>{{ item.title }}</strong>
                  <span>{{ item.description }}</span>
                </article>
              </div>
            </div>

            <div v-else-if="currentStep.key === 'environment'" class="check-panel">
              <div class="panel-heading">
                <div>
                  <strong>Deployment Checker</strong>
                  <p>{{ deploymentReport?.summary ?? "正在检查 Rust Server、配置、日志、权限和监听端口。" }}</p>
                </div>
                <GButton variant="secondary" icon="refresh" :loading="deploymentLoading" @click="runDeploymentCheck">
                  重新检查
                </GButton>
              </div>

              <div class="finding-list">
                <article v-for="finding in deploymentFindings" :key="finding.id" :class="`is-${finding.status}`">
                  <span><GIcon :name="findingIcon(finding.status)" :size="16" /></span>
                  <div>
                    <strong>{{ finding.label }}</strong>
                    <p>{{ finding.reason }}</p>
                    <small>{{ finding.solution }}</small>
                  </div>
                </article>
              </div>
            </div>

            <div v-else-if="currentStep.key === 'server'" class="server-wizard">
              <div class="connection-status" :class="connectionTone">
                <span />
                <strong>{{ connectionStatusLabel }}</strong>
                <small>{{ connectionStatusDescription }}</small>
              </div>

              <div class="server-form">
                <label>
                  <span>服务器地址</span>
                  <input v-model.trim="server.host" autocomplete="off" placeholder="gate.example.com 或 127.0.0.1" />
                </label>
                <label>
                  <span>端口</span>
                  <input v-model.number="server.port" inputmode="numeric" type="number" placeholder="7000" />
                </label>
                <label class="server-form__token">
                  <span>Token</span>
                  <input v-model.trim="server.token" autocomplete="off" type="password" placeholder="从服务端配置复制 Token" />
                </label>
              </div>

              <div class="server-actions">
                <GButton variant="primary" icon="plug-zap" :loading="connectionLoading" @click="testConnection">
                  测试连接
                </GButton>
                <GButton variant="secondary" icon="shield-check" :loading="deploymentLoading" @click="runDeploymentCheck">
                  部署检查
                </GButton>
              </div>

              <section v-if="recentServers.length" class="recent-servers">
                <header>
                  <strong>Recent Server</strong>
                  <span>快速重连，收藏预留</span>
                </header>
                <button v-for="recent in recentServers" :key="recent.serverAddr" type="button" @click="applyRecentServer(recent.serverAddr)">
                  <GIcon name="history" :size="14" />
                  <span>{{ recent.serverAddr }}</span>
                  <small>{{ recent.successCount }} 次成功</small>
                </button>
              </section>
            </div>

            <div v-else-if="currentStep.key === 'test'" class="test-panel">
              <div v-if="!connectionReport" class="test-empty">
                <GIcon name="plug-zap" :size="34" />
                <h2>先测试服务器连接</h2>
                <p>测试会区分 DNS、Token、服务器未启动、端口不可达和超时。</p>
                <GButton variant="primary" icon="plug-zap" :loading="connectionLoading" @click="testConnection">
                  测试连接
                </GButton>
              </div>

              <article v-else class="connection-result" :class="{ ok: connectionReport.ok }">
                <div class="result-heading">
                  <span><GIcon :name="connectionReport.ok ? 'check-circle' : 'alert-circle'" :size="22" /></span>
                  <div>
                    <strong>{{ connectionReport.title }}</strong>
                    <small>{{ connectionReport.code }} · {{ connectionReport.elapsedMs }}ms</small>
                  </div>
                </div>

                <dl>
                  <div>
                    <dt>错误原因</dt>
                    <dd>{{ connectionReport.reason }}</dd>
                  </div>
                  <div>
                    <dt>可能原因</dt>
                    <dd>{{ connectionReport.possibleCause }}</dd>
                  </div>
                  <div>
                    <dt>解决方案</dt>
                    <dd>{{ connectionReport.solution }}</dd>
                  </div>
                </dl>

                <div class="result-actions">
                  <GButton variant="secondary" icon="refresh" :loading="connectionLoading" @click="testConnection">
                    重新测试
                  </GButton>
                  <GButton variant="ghost" icon="logs" @click="openLogs">查看日志</GButton>
                  <GButton variant="ghost" icon="copy" @click="copyConnectionError">复制错误</GButton>
                </div>
              </article>

              <section v-if="connectionHistory.length" class="history-list">
                <header>
                  <strong>Connection History</strong>
                  <span>最近 10 次</span>
                </header>
                <article v-for="entry in connectionHistory.slice(0, 4)" :key="entry.id">
                  <span :class="entry.result">{{ entry.result === "success" ? "成功" : "失败" }}</span>
                  <strong>{{ entry.serverAddr }}</strong>
                  <small>{{ entry.failureReason || `${entry.elapsedMs}ms` }}</small>
                </article>
              </section>
            </div>

            <div v-else-if="currentStep.key === 'tunnel'" class="first-tunnel">
              <div class="preset-strip">
                <button
                  v-for="scenario in quickStartScenarios"
                  :key="scenario.id"
                  type="button"
                  :class="{ active: selectedScenarioId === scenario.id }"
                  @click="applyScenario(scenario.id)"
                >
                  <GIcon :name="scenario.icon" :size="16" />
                  <span>{{ scenario.title }}</span>
                </button>
              </div>

              <div class="template-strip">
                <button
                  v-for="template in tunnelTemplates"
                  :key="template.id"
                  type="button"
                  :class="{ active: selectedTemplateId === template.id, reserved: template.availability === 'reserved' }"
                  :disabled="template.availability === 'reserved'"
                  @click="applyTemplate(template.id)"
                >
                  <GIcon :name="template.icon" :size="15" />
                  <span>{{ template.title }}</span>
                </button>
              </div>

              <div class="tunnel-form">
                <label>
                  <span>建议名称</span>
                  <input v-model.trim="tunnelForm.name" autocomplete="off" />
                </label>
                <label>
                  <span>本地地址</span>
                  <input v-model.trim="tunnelForm.localHost" autocomplete="off" />
                </label>
                <label>
                  <span>本地端口</span>
                  <input v-model.number="tunnelForm.localPort" inputmode="numeric" type="number" />
                </label>
                <label>
                  <span>公网端口</span>
                  <input v-model.number="tunnelForm.remotePort" inputmode="numeric" type="number" />
                </label>
              </div>

              <div class="template-summary">
                <strong>{{ selectedTemplate.title }} · {{ tunnelForm.protocol.toUpperCase() }}</strong>
                <span>{{ tunnelForm.localHost }}:{{ tunnelForm.localPort || "-" }} → gate.dev:{{ tunnelForm.remotePort || "-" }}</span>
              </div>
            </div>

            <div v-else class="finish-panel">
              <div class="finish-mark">
                <GIcon name="check-circle" :size="34" />
              </div>
              <h2>你的第一个 Tunnel 已准备好</h2>
              <p>{{ createdTunnelName }} 已创建。你可以进入 Tunnel 页面启动、查看日志，或继续创建更多模板化 Tunnel。</p>
              <div class="finish-actions">
                <GButton variant="primary" icon="router" @click="finishWizard">进入 Tunnels</GButton>
                <GButton variant="secondary" icon="activity" @click="openDiagnostics">打开诊断中心</GButton>
              </div>
            </div>
          </section>

          <footer class="welcome-footer">
            <GButton v-if="step > 0 && currentStep.key !== 'finish'" variant="ghost" @click="step -= 1">上一步</GButton>
            <span class="welcome-error">{{ inlineError }}</span>
            <GButton
              v-if="currentStep.key !== 'finish'"
              variant="primary"
              trailing-icon="arrow-right"
              :loading="primaryLoading"
              @click="goNext"
            >
              {{ primaryLabel }}
            </GButton>
          </footer>
        </main>
      </section>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue"
import { useRouter } from "vue-router"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"
import { diagnosticsService } from "@/services"
import type {
  ConnectionHistoryEntry,
  ConnectionTestReport,
  DeploymentCheckReport,
  DiagnosticFinding,
  RecentServer,
} from "@/services"
import { findTemplate, quickStartScenarios, tunnelTemplates } from "@/onboarding/presets"
import { useTunnelStore } from "@/views/tunnels/store/tunnel"
import type { TunnelFormData } from "@/views/tunnels/types"

const FIRST_LAUNCH_KEY = "gate.firstLaunch.completed"

const router = useRouter()
const tunnelStore = useTunnelStore()

const visible = ref(false)
const step = ref(0)
const deploymentLoading = ref(false)
const connectionLoading = ref(false)
const deploymentReport = ref<DeploymentCheckReport | null>(null)
const connectionReport = ref<ConnectionTestReport | null>(null)
const recentServers = ref<RecentServer[]>([])
const connectionHistory = ref<ConnectionHistoryEntry[]>([])
const inlineError = ref("")
const selectedScenarioId = ref("local-dev")
const selectedTemplateId = ref("tcp")
const createdTunnelName = ref("")

const server = reactive({
  host: "127.0.0.1",
  port: 7000,
  token: "",
})

const tunnelForm = reactive<TunnelFormData>({
  name: "local-dev",
  protocol: "tcp",
  localHost: "127.0.0.1",
  localPort: 3000,
  remotePort: 18080,
  projectId: "p1",
  serverName: "Local Server",
  autoStart: false,
  remark: "",
  tags: ["Dev"],
})

const flowSteps = [
  { key: "welcome", title: "Welcome", caption: "准备开始" },
  { key: "environment", title: "检查环境", caption: "部署前检查" },
  { key: "server", title: "配置服务器", caption: "Server Connection Wizard" },
  { key: "test", title: "测试连接", caption: "连接诊断" },
  { key: "tunnel", title: "创建第一个 Tunnel", caption: "Quick Start" },
  { key: "finish", title: "完成", caption: "可以开始使用" },
] as const

const introCards = [
  { icon: "shield-check", title: "自动检查", description: "检查配置、日志、权限和端口。" },
  { icon: "plug-zap", title: "明确错误", description: "失败时给出原因、可能原因和解决方案。" },
  { icon: "sparkles", title: "模板生成", description: "常见场景一键生成推荐配置。" },
]

const currentStep = computed(() => flowSteps[step.value])
const progressPercent = computed(() => ((step.value + 1) / flowSteps.length) * 100)
const deploymentFindings = computed<DiagnosticFinding[]>(() => deploymentReport.value?.findings ?? [])
const selectedTemplate = computed(() => findTemplate(selectedTemplateId.value))
const serverAddr = computed(() => diagnosticsService.formatServerAddr(server))
const primaryLoading = computed(() => deploymentLoading.value || connectionLoading.value)
const primaryLabel = computed(() => {
  if (currentStep.value.key === "environment") return "继续配置服务器"
  if (currentStep.value.key === "server") return "测试连接"
  if (currentStep.value.key === "test") return connectionReport.value?.ok ? "继续创建 Tunnel" : "重新测试"
  if (currentStep.value.key === "tunnel") return "创建 Tunnel"
  return "下一步"
})
const connectionTone = computed(() => {
  if (connectionLoading.value) return "testing"
  if (connectionReport.value?.ok) return "ok"
  if (connectionReport.value && !connectionReport.value.ok) return "error"
  return server.host && server.port && server.token ? "ready" : "idle"
})
const connectionStatusLabel = computed(() => {
  if (connectionLoading.value) return "正在测试连接"
  if (connectionReport.value?.ok) return "连接可用"
  if (connectionReport.value && !connectionReport.value.ok) return connectionReport.value.title
  if (server.host && server.port && server.token) return "可以测试连接"
  return "等待服务器配置"
})
const connectionStatusDescription = computed(() => {
  if (connectionReport.value) return connectionReport.value.solution
  return "填写服务器地址、端口和 Token 后点击测试连接。"
})

onMounted(() => {
  visible.value = localStorage.getItem(FIRST_LAUNCH_KEY) !== "true"
  refreshConnectionMemory()
  applyScenario("local-dev")
  if (visible.value) void runDeploymentCheck()
})

async function runDeploymentCheck() {
  deploymentLoading.value = true
  inlineError.value = ""
  try {
    deploymentReport.value = await diagnosticsService.runDeployment(server.host ? serverAddr.value : undefined)
  } finally {
    deploymentLoading.value = false
  }
}

async function testConnection() {
  if (!validateServer()) return
  connectionLoading.value = true
  inlineError.value = ""
  try {
    connectionReport.value = await diagnosticsService.testConnection(server)
    refreshConnectionMemory()
  } finally {
    connectionLoading.value = false
  }
}

async function goNext() {
  inlineError.value = ""
  if (currentStep.value.key === "environment" && !deploymentReport.value) {
    await runDeploymentCheck()
  }
  if (currentStep.value.key === "server") {
    await testConnection()
    if (!connectionReport.value?.ok) {
      step.value = 3
      return
    }
  }
  if (currentStep.value.key === "test") {
    if (!connectionReport.value?.ok) {
      await testConnection()
      if (!connectionReport.value?.ok) return
    }
  }
  if (currentStep.value.key === "tunnel") {
    if (!(await createFirstTunnel())) return
  }
  step.value = Math.min(step.value + 1, flowSteps.length - 1)
}

function validateServer() {
  if (!server.host.trim()) inlineError.value = "请填写服务器地址。"
  else if (!Number.isInteger(server.port) || server.port < 1 || server.port > 65535) inlineError.value = "端口必须在 1-65535 之间。"
  else if (!server.token.trim()) inlineError.value = "请填写 Token。"
  return !inlineError.value
}

async function createFirstTunnel() {
  if (!Number.isInteger(tunnelForm.localPort) || !Number.isInteger(tunnelForm.remotePort)) {
    inlineError.value = "请确认本地端口和公网端口。"
    return false
  }
  const created = await tunnelStore.createTunnel({
    ...tunnelForm,
    name: tunnelForm.name.trim() || selectedTemplate.value.suggestedName,
    serverName: serverAddr.value,
    tags: [...tunnelForm.tags],
  })
  createdTunnelName.value = created.name
  return true
}

function applyScenario(id: string) {
  const scenario = quickStartScenarios.find((item) => item.id === id)
  if (!scenario) return
  selectedScenarioId.value = scenario.id
  selectedTemplateId.value = scenario.templateId
  const template = findTemplate(scenario.templateId)
  tunnelForm.name = scenario.suggestedName
  tunnelForm.protocol = template.protocol
  tunnelForm.localPort = scenario.localPort
  tunnelForm.remotePort = scenario.remotePort
  tunnelForm.remark = scenario.description
  tunnelForm.tags = [...new Set([...template.tags, ...scenario.tags])]
}

function applyTemplate(id: string) {
  const template = findTemplate(id)
  if (template.availability === "reserved") return
  selectedTemplateId.value = template.id
  tunnelForm.name = template.suggestedName
  tunnelForm.protocol = template.protocol
  tunnelForm.localPort = template.localPort
  tunnelForm.remotePort = template.remotePort
  tunnelForm.remark = template.description
  tunnelForm.tags = [...template.tags]
}

function applyRecentServer(value: string) {
  const [host, port] = splitServerAddr(value)
  server.host = host
  server.port = port
  connectionReport.value = null
}

function splitServerAddr(value: string): [string, number] {
  const [host, rawPort] = value.split(":")
  const port = Number(rawPort)
  return [host || "127.0.0.1", Number.isFinite(port) ? port : 7000]
}

function refreshConnectionMemory() {
  recentServers.value = diagnosticsService.getRecentServers()
  connectionHistory.value = diagnosticsService.getConnectionHistory()
}

function findingIcon(status: DiagnosticFinding["status"]) {
  if (status === "ok") return "check-circle"
  if (status === "warning") return "alert-triangle"
  return "alert-circle"
}

function copyConnectionError() {
  if (!connectionReport.value) return
  void diagnosticsService.copyText(JSON.stringify(connectionReport.value, null, 2))
}

function openLogs() {
  void router.push("/logs")
  visible.value = false
}

function openDiagnostics() {
  markComplete()
  visible.value = false
  void router.push("/diagnostics")
}

function finishWizard() {
  markComplete()
  visible.value = false
  void router.push("/tunnels")
}

function skipWizard() {
  markComplete()
  visible.value = false
}

function markComplete() {
  localStorage.setItem(FIRST_LAUNCH_KEY, "true")
}
</script>

<style scoped>
.welcome-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  display: grid;
  place-items: center;
  padding: var(--space-5);
  background: var(--bg-app);
}

.welcome-shell {
  width: min(1120px, 100%);
  height: min(760px, calc(100vh - 40px));
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr);
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-2xl);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.welcome-rail {
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
  padding: var(--space-5);
  border-right: 1px solid var(--border-subtle);
  background: var(--bg-sidebar);
}

.welcome-brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.welcome-brand > span {
  width: 44px;
  height: 44px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.welcome-brand strong,
.welcome-steps strong {
  display: block;
  color: var(--text-primary);
}

.welcome-brand small,
.welcome-steps small {
  color: var(--text-tertiary);
}

.welcome-steps {
  display: grid;
  gap: var(--space-2);
}

.welcome-steps button {
  min-height: 58px;
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-3);
  border: 0;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-secondary);
  padding: var(--space-2);
  text-align: left;
  cursor: default;
}

.welcome-steps button.active,
.welcome-steps button.done {
  background: var(--bg-surface-hover);
}

.welcome-steps button > span {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: var(--text-tertiary);
  font-weight: var(--weight-semibold);
}

.welcome-steps button.active > span,
.welcome-steps button.done > span {
  background: var(--color-primary);
  color: var(--color-primary-fg);
}

.welcome-main {
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr) auto;
}

.welcome-header,
.welcome-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-5);
}

.welcome-header p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.welcome-header h1 {
  margin-top: 2px;
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.welcome-skip {
  height: 32px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 0 var(--space-3);
  cursor: pointer;
}

.welcome-progress {
  height: 3px;
  background: var(--bg-input);
}

.welcome-progress span {
  display: block;
  height: 100%;
  background: var(--color-primary);
  transition: width var(--duration-base) var(--ease-out);
}

.welcome-content {
  min-height: 0;
  overflow: auto;
  padding: var(--space-5);
}

.welcome-intro,
.test-empty,
.finish-panel {
  min-height: 100%;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-4);
  text-align: center;
}

.intro-mark,
.finish-mark {
  width: 72px;
  height: 72px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-xl);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.welcome-intro h2,
.test-empty h2,
.finish-panel h2 {
  max-width: 620px;
  font-size: var(--text-3xl);
  line-height: var(--leading-tight);
  letter-spacing: 0;
}

.welcome-intro p,
.test-empty p,
.finish-panel p {
  max-width: 640px;
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.intro-grid {
  width: min(660px, 100%);
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.intro-grid article {
  min-height: 120px;
  display: grid;
  align-content: center;
  justify-items: center;
  gap: var(--space-2);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.intro-grid article svg {
  color: var(--color-primary);
}

.intro-grid article span {
  color: var(--text-secondary);
  line-height: var(--leading-normal);
}

.check-panel,
.server-wizard,
.test-panel,
.first-tunnel {
  display: grid;
  gap: var(--space-4);
}

.panel-heading,
.recent-servers header,
.history-list header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.panel-heading strong,
.recent-servers strong,
.history-list strong {
  color: var(--text-primary);
  font-size: var(--text-lg);
}

.panel-heading p,
.recent-servers header span,
.history-list header span {
  margin-top: 2px;
  color: var(--text-secondary);
}

.finding-list {
  display: grid;
  gap: var(--space-2);
}

.finding-list article {
  min-height: 76px;
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.finding-list article > span {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-full);
  background: var(--bg-input);
}

.finding-list article.is-ok > span { color: var(--color-success); }
.finding-list article.is-warning > span { color: var(--color-warning); }
.finding-list article.is-error > span { color: var(--color-error); }

.finding-list p {
  color: var(--text-secondary);
}

.finding-list small {
  display: block;
  margin-top: 2px;
  color: var(--text-tertiary);
}

.connection-status {
  min-height: 64px;
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr);
  gap: var(--space-2) var(--space-3);
  align-items: center;
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.connection-status > span {
  grid-row: 1 / span 2;
  width: 10px;
  height: 10px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.connection-status.ready > span,
.connection-status.testing > span { background: var(--color-warning); }
.connection-status.ok > span { background: var(--color-success); }
.connection-status.error > span { background: var(--color-error); }

.connection-status small {
  color: var(--text-secondary);
}

.server-form,
.tunnel-form {
  display: grid;
  grid-template-columns: 1fr 140px;
  gap: var(--space-3);
}

.server-form__token {
  grid-column: 1 / -1;
}

.server-form label,
.tunnel-form label {
  display: grid;
  gap: var(--space-2);
}

.server-form span,
.tunnel-form span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.server-form input,
.tunnel-form input {
  height: 38px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: 0 var(--space-3);
  outline: 0;
}

.server-form input:focus,
.tunnel-form input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.server-actions,
.result-actions,
.finish-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.recent-servers,
.history-list {
  display: grid;
  gap: var(--space-2);
  padding-top: var(--space-2);
  border-top: 1px solid var(--border-subtle);
}

.recent-servers button,
.history-list article {
  min-height: 38px;
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  color: var(--text-primary);
  padding: 0 var(--space-3);
  text-align: left;
}

.recent-servers button {
  cursor: pointer;
}

.recent-servers button:hover {
  border-color: var(--color-primary);
}

.recent-servers small,
.history-list small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.connection-result {
  display: grid;
  gap: var(--space-4);
  padding: var(--space-4);
  border: 1px solid rgba(255, 92, 92, 0.3);
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
}

.connection-result.ok {
  border-color: rgba(47, 209, 124, 0.32);
  background: var(--color-success-muted);
}

.result-heading {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.result-heading > span {
  width: 42px;
  height: 42px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--color-error);
}

.connection-result.ok .result-heading > span {
  color: var(--color-success);
}

.result-heading strong {
  color: var(--text-primary);
  font-size: var(--text-lg);
}

.result-heading small {
  display: block;
  margin-top: 2px;
  color: var(--text-secondary);
}

.connection-result dl {
  display: grid;
  gap: var(--space-2);
}

.connection-result dl div {
  display: grid;
  grid-template-columns: 92px minmax(0, 1fr);
  gap: var(--space-3);
}

.connection-result dt {
  color: var(--text-tertiary);
}

.connection-result dd {
  color: var(--text-primary);
  overflow-wrap: anywhere;
}

.history-list article {
  grid-template-columns: 56px minmax(0, 1fr) auto;
}

.history-list article > span {
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  font-size: var(--text-xs);
}

.history-list article > span.success {
  background: var(--color-success-muted);
  color: var(--color-success);
}

.history-list article > span.failed {
  background: var(--color-error-muted);
  color: var(--color-error);
}

.preset-strip,
.template-strip {
  display: flex;
  gap: var(--space-2);
  overflow-x: auto;
  padding-bottom: var(--space-1);
}

.preset-strip button,
.template-strip button {
  flex: 0 0 auto;
  height: 34px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 0 var(--space-3);
  cursor: pointer;
}

.preset-strip button:hover,
.preset-strip button.active,
.template-strip button:hover,
.template-strip button.active {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
  color: var(--text-primary);
}

.template-strip button.reserved {
  opacity: 0.52;
  cursor: not-allowed;
}

.tunnel-form {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.template-summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.template-summary span {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  overflow-wrap: anywhere;
  text-align: right;
}

.welcome-footer {
  min-height: 72px;
  border-top: 1px solid var(--border-subtle);
}

.welcome-error {
  flex: 1;
  color: var(--color-error);
}

.welcome-enter-active,
.welcome-leave-active {
  transition: opacity var(--duration-base) var(--ease-out);
}

.welcome-enter-from,
.welcome-leave-to {
  opacity: 0;
}

@media (max-width: 860px) {
  .welcome-shell {
    grid-template-columns: 1fr;
  }

  .welcome-rail {
    display: none;
  }

  .intro-grid,
  .server-form,
  .tunnel-form {
    grid-template-columns: 1fr;
  }

  .template-summary {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
