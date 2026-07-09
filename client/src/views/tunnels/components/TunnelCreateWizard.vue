<template>
  <Transition name="wizard">
    <div
      v-if="visible"
      class="wizard-backdrop"
      @click.self="close"
      @keydown.esc="close"
    >
      <section
        class="wizard"
        role="dialog"
        aria-modal="true"
        aria-labelledby="wizard-title"
        tabindex="-1"
      >
        <header class="wizard__header">
          <div>
            <p>Create Tunnel Wizard</p>
            <h2 id="wizard-title">
              {{ stepTitle }}
            </h2>
          </div>
          <button
            type="button"
            class="wizard__close"
            aria-label="关闭"
            @click="close"
          >
            <GIcon
              name="close"
              :size="16"
            />
          </button>
        </header>

        <div
          v-if="step < 5"
          class="wizard__steps"
          aria-label="创建步骤"
        >
          <span
            v-for="item in steps"
            :key="item.index"
            :class="{ active: step === item.index, done: step > item.index }"
          >
            {{ item.index }}
          </span>
        </div>

        <main class="wizard__body">
          <section
            v-if="step === 1"
            class="wizard-step"
          >
            <div class="wizard-copy">
              <strong>选择一个场景</strong>
              <p>Gate 会自动填入建议名称、端口、标签和模板。你仍然可以在下一步调整。</p>
            </div>
            <div class="preset-grid">
              <button
                v-for="scenario in quickStartScenarios"
                :key="scenario.id"
                type="button"
                class="preset-card"
                :class="{ active: selectedScenarioId === scenario.id }"
                @click="applyScenario(scenario.id)"
              >
                <span><GIcon
                  :name="scenario.icon"
                  :size="18"
                /></span>
                <strong>{{ scenario.title }}</strong>
                <small>{{ scenario.description }}</small>
              </button>
            </div>
          </section>

          <section
            v-else-if="step === 2"
            class="wizard-step"
          >
            <div class="wizard-copy">
              <strong>选择 Tunnel 模板</strong>
              <p>Templates generate recommended TCP or HTTP tunnel settings.</p>
            </div>
            <div class="template-list">
              <button
                v-for="template in tunnelTemplates"
                :key="template.id"
                type="button"
                class="template-row"
                :class="{
                  active: selectedTemplateId === template.id,
                  reserved: template.availability === 'reserved',
                }"
                :disabled="template.availability === 'reserved'"
                @click="applyTemplate(template.id)"
              >
                <span><GIcon
                  :name="template.icon"
                  :size="18"
                /></span>
                <div>
                  <strong>{{ template.title }}</strong>
                  <small>{{ template.description }}</small>
                </div>
                <code>{{ template.localPort }} → {{ template.remotePort }}</code>
              </button>
            </div>
          </section>

          <section
            v-else-if="step === 3"
            class="wizard-step wizard-step--form"
          >
            <label>
              <span>Tunnel 名称</span>
              <input
                v-model.trim="form.name"
                autocomplete="off"
                :placeholder="suggestedName"
              >
            </label>

            <div class="form-grid">
              <label>
                <span>本地地址</span>
                <input
                  v-model.trim="form.localHost"
                  autocomplete="off"
                  placeholder="127.0.0.1"
                >
              </label>
              <label>
                <span>协议</span>
                <select v-model="form.protocol">
                  <option value="tcp">TCP</option>
                  <option value="http">HTTP（已有能力）</option>
                </select>
              </label>
            </div>

            <div class="form-grid">
              <label>
                <span>本地端口</span>
                <input
                  v-model.number="form.localPort"
                  inputmode="numeric"
                  type="number"
                >
              </label>
              <label>
                <span>公网端口</span>
                <input
                  v-model.number="form.remotePort"
                  inputmode="numeric"
                  type="number"
                >
              </label>
            </div>

            <div class="form-grid">
              <label>
                <span>项目</span>
                <select v-model="form.projectId">
                  <option
                    v-if="!projects.length"
                    value=""
                  >未分组</option>
                  <option
                    v-for="project in projects"
                    :key="project.id"
                    :value="project.id"
                  >{{ project.name }}</option>
                </select>
              </label>
              <label>
                <span>服务器</span>
                <select v-model="form.serverName">
                  <option
                    v-if="!serverNames.length"
                    value=""
                  >未连接服务器</option>
                  <option
                    v-for="serverName in serverNames"
                    :key="serverName"
                    :value="serverName"
                  >{{ serverName }}</option>
                </select>
              </label>
            </div>

            <label class="wizard-check">
              <input
                v-model="form.autoStart"
                type="checkbox"
              >
              <span>创建后自动启动 Tunnel</span>
            </label>
          </section>

          <section
            v-else-if="step === 4"
            class="wizard-step wizard-step--confirm"
          >
            <div class="confirm-list">
              <div><span>场景</span><strong>{{ selectedScenario?.title ?? "自定义" }}</strong></div>
              <div><span>模板</span><strong>{{ selectedTemplate.title }}</strong></div>
              <div><span>协议</span><strong>{{ form.protocol.toUpperCase() }}</strong></div>
              <div><span>服务器</span><strong>{{ form.serverName || "未连接服务器" }}</strong></div>
              <div><span>本地服务</span><strong>{{ form.localHost }}:{{ form.localPort || "-" }}</strong></div>
              <div><span>公网端口</span><strong>{{ form.remotePort || "-" }}</strong></div>
              <div><span>访问地址</span><strong>{{ publicPreview }}</strong></div>
              <div><span>标签</span><strong>{{ form.tags.join(", ") || "-" }}</strong></div>
            </div>

            <div
              v-if="errorMessage"
              class="wizard-error-card"
            >
              <GIcon
                name="alert-circle"
                :size="18"
              />
              <div>
                <strong>配置还不完整</strong>
                <p>{{ errorMessage }}</p>
              </div>
            </div>
          </section>

          <section
            v-else
            class="wizard-success"
          >
            <span><GIcon
              name="check-circle"
              :size="28"
            /></span>
            <h2>创建成功</h2>
            <p>{{ createdName }} 已加入 Tunnel 列表，可以立即启动或继续调整设置。</p>
          </section>
        </main>

        <footer class="wizard__footer">
          <GButton
            v-if="step > 1 && step < 5"
            variant="ghost"
            @click="step -= 1"
          >
            上一步
          </GButton>
          <span class="wizard__error">{{ step < 4 ? errorMessage : "" }}</span>
          <GButton
            v-if="step < 4"
            variant="primary"
            trailing-icon="arrow-right"
            @click="next"
          >
            下一步
          </GButton>
          <GButton
            v-else-if="step === 4"
            variant="primary"
            icon="plus"
            @click="createTunnel"
          >
            创建
          </GButton>
          <GButton
            v-else
            variant="primary"
            @click="finish"
          >
            完成
          </GButton>
        </footer>
      </section>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"
import { findTemplate, quickStartScenarios, tunnelTemplates } from "@/onboarding/presets"
import type { TunnelFormData } from "../types"

const props = defineProps<{
  visible: boolean
  projects: Array<{ id: string; name: string }>
  serverNames: string[]
}>()

const emit = defineEmits<{
  "update:visible": [value: boolean]
  submit: [form: TunnelFormData]
}>()

const steps = [
  { index: 1, title: "Quick Start" },
  { index: 2, title: "Tunnel Template" },
  { index: 3, title: "本地配置" },
  { index: 4, title: "确认创建" },
]

const step = ref(1)
const errorMessage = ref("")
const createdName = ref("")
const selectedScenarioId = ref("local-dev")
const selectedTemplateId = ref("tcp")

const form = reactive<TunnelFormData>({
  name: "",
  protocol: "tcp",
  localHost: "127.0.0.1",
  localPort: null,
  remotePort: null,
  projectId: "",
  serverName: "",
  autoStart: false,
  remark: "",
  tags: [],
})

const selectedScenario = computed(() =>
  quickStartScenarios.find((scenario) => scenario.id === selectedScenarioId.value),
)
const selectedTemplate = computed(() => findTemplate(selectedTemplateId.value))
const stepTitle = computed(() => steps.find((item) => item.index === step.value)?.title ?? "创建成功")
const suggestedName = computed(() => form.name || selectedScenario.value?.suggestedName || selectedTemplate.value.suggestedName)
const publicPreview = computed(() => (form.remotePort ? `:${form.remotePort}` : "保存后由 Runtime 返回"))

watch(
  () => props.visible,
  (visible) => {
    if (visible) reset()
  },
)

function reset() {
  step.value = 1
  errorMessage.value = ""
  createdName.value = ""
  form.name = ""
  form.protocol = "tcp"
  form.localHost = "127.0.0.1"
  form.localPort = null
  form.remotePort = null
  form.projectId = props.projects[0]?.id ?? ""
  form.serverName = props.serverNames[0] ?? ""
  form.autoStart = false
  form.remark = ""
  form.tags = []
  applyScenario("local-dev")
}

function applyScenario(id: string) {
  const scenario = quickStartScenarios.find((item) => item.id === id)
  if (!scenario) return
  selectedScenarioId.value = scenario.id
  selectedTemplateId.value = scenario.templateId
  const template = findTemplate(scenario.templateId)
  form.protocol = template.protocol
  form.name = scenario.suggestedName
  form.localPort = scenario.localPort
  form.remotePort = scenario.remotePort
  form.tags = [...new Set([...template.tags, ...scenario.tags])]
  form.remark = scenario.description
  errorMessage.value = ""
}

function applyTemplate(id: string) {
  const template = findTemplate(id)
  if (template.availability === "reserved") return
  selectedTemplateId.value = template.id
  form.protocol = template.protocol
  form.name = template.suggestedName
  form.localPort = template.localPort
  form.remotePort = template.remotePort
  form.tags = [...template.tags]
  form.remark = template.description
  errorMessage.value = ""
}

function isValidPort(port: number | null) {
  return port != null && Number.isInteger(port) && port >= 1 && port <= 65535
}

function validateCurrentStep() {
  errorMessage.value = ""
  if (step.value === 3 || step.value === 4) {
    if (!props.serverNames.length) errorMessage.value = "请先在服务器页面添加并连接服务器。"
    else if (!form.serverName) errorMessage.value = "请选择一台已连接服务器。"
    else if (!form.localHost.trim()) errorMessage.value = "请填写本地地址。"
    else if (!isValidPort(form.localPort)) errorMessage.value = "本地端口必须在 1-65535 之间。"
    else if (!isValidPort(form.remotePort)) errorMessage.value = "公网端口必须在 1-65535 之间。"
  }
  return !errorMessage.value
}

function next() {
  if (!validateCurrentStep()) return
  step.value += 1
}

function createTunnel() {
  if (!validateCurrentStep()) return
  const name = form.name.trim() || suggestedName.value
  createdName.value = name
  emit("submit", { ...form, name, tags: [...form.tags] })
  step.value = 5
}

function close() {
  emit("update:visible", false)
}

function finish() {
  close()
}
</script>

<style scoped>
.wizard-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  display: grid;
  place-items: center;
  padding: var(--space-6);
  background: var(--color-overlay);
  backdrop-filter: blur(12px);
}

.wizard {
  width: min(760px, 100%);
  max-height: min(760px, calc(100vh - 48px));
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr) auto;
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-2xl);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.wizard__header,
.wizard__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-5);
}

.wizard__header {
  border-bottom: 1px solid var(--border-subtle);
}

.wizard__header p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.wizard__header h2 {
  margin-top: 2px;
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.wizard__close {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.wizard__close:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.wizard__steps {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-2);
  padding: var(--space-4) var(--space-5) 0;
}

.wizard__steps span {
  height: 4px;
  overflow: hidden;
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: transparent;
}

.wizard__steps span.active,
.wizard__steps span.done {
  background: var(--color-primary);
}

.wizard__body {
  min-height: 0;
  overflow: auto;
  padding: var(--space-5);
}

.wizard-step {
  display: grid;
  gap: var(--space-4);
}

.wizard-copy {
  display: grid;
  gap: var(--space-1);
}

.wizard-copy strong {
  color: var(--text-primary);
  font-size: var(--text-lg);
}

.wizard-copy p {
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.preset-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.preset-card,
.template-row {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
}

.preset-card {
  min-height: 108px;
  display: grid;
  grid-template-columns: 36px minmax(0, 1fr);
  gap: 3px var(--space-3);
  align-items: center;
  padding: var(--space-4);
}

.preset-card span,
.template-row > span {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--color-primary);
}

.preset-card strong,
.template-row strong {
  font-size: var(--text-md);
}

.preset-card small {
  grid-column: 2;
  color: var(--text-secondary);
  line-height: var(--leading-normal);
}

.preset-card:hover,
.preset-card.active,
.template-row:hover,
.template-row.active {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
}

.template-list {
  display: grid;
  gap: var(--space-2);
}

.template-row {
  min-height: 72px;
  display: grid;
  grid-template-columns: 36px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
}

.template-row small {
  display: block;
  margin-top: 2px;
  color: var(--text-secondary);
}

.template-row code {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.template-row.reserved {
  opacity: 0.58;
  cursor: not-allowed;
}

.wizard-step--form label {
  display: grid;
  gap: var(--space-2);
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.wizard-step label span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.wizard-step input,
.wizard-step select {
  width: 100%;
  height: 38px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: 0 var(--space-3);
  outline: 0;
}

.wizard-step input:focus,
.wizard-step select:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.wizard-check {
  display: inline-flex !important;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: var(--space-2) !important;
  color: var(--text-secondary);
}

.wizard-check input {
  width: 16px;
  height: 16px;
  accent-color: var(--color-primary);
}

.confirm-list {
  display: grid;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.confirm-list div {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  min-height: 42px;
  padding: 0 var(--space-3);
}

.confirm-list div + div {
  border-top: 1px solid var(--border-subtle);
}

.confirm-list span {
  color: var(--text-tertiary);
}

.confirm-list strong {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  overflow-wrap: anywhere;
  text-align: right;
}

.wizard-error-card {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr);
  gap: var(--space-2);
  padding: var(--space-3);
  border: 1px solid rgba(255, 92, 92, 0.3);
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
  color: var(--color-error);
}

.wizard-error-card p {
  margin-top: 2px;
  color: var(--text-secondary);
}

.wizard-success {
  min-height: 300px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  text-align: center;
}

.wizard-success span {
  width: 58px;
  height: 58px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-xl);
  background: var(--color-success-muted);
  color: var(--color-success);
}

.wizard-success h2 {
  font-size: var(--text-2xl);
  letter-spacing: 0;
}

.wizard-success p {
  max-width: 360px;
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.wizard__footer {
  min-height: 66px;
  border-top: 1px solid var(--border-subtle);
}

.wizard__error {
  flex: 1;
  color: var(--color-error);
  font-size: var(--text-sm);
}

.wizard-enter-active,
.wizard-leave-active {
  transition: opacity var(--duration-base) var(--ease-out);
}

.wizard-enter-active .wizard,
.wizard-leave-active .wizard {
  transition: transform var(--duration-base) var(--ease-out), opacity var(--duration-base) var(--ease-out);
}

.wizard-enter-from,
.wizard-leave-to {
  opacity: 0;
}

.wizard-enter-from .wizard,
.wizard-leave-to .wizard {
  opacity: 0;
  transform: scale(0.97) translateY(8px);
}

@media (max-width: 720px) {
  .preset-grid,
  .form-grid {
    grid-template-columns: 1fr;
  }

  .template-row {
    grid-template-columns: 36px minmax(0, 1fr);
  }

  .template-row code {
    grid-column: 2;
  }
}
</style>
