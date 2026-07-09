<template>
  <Transition name="smart-wizard">
    <div v-if="visible" class="smart-onboarding" @keydown.esc="closeForLater">
      <section
        class="wizard-shell"
        role="dialog"
        aria-modal="true"
        aria-labelledby="smart-wizard-title">
        <aside class="wizard-rail">
          <div class="rail-brand">
            <span><GIcon name="router" :size="24" /></span>
            <div>
              <strong>Gate</strong>
              <small>智能引导</small>
            </div>
          </div>

          <div class="rail-illustration" aria-hidden="true">
            <div class="node local">本地</div>
            <div class="node server">服务器</div>
            <div class="node public">公网</div>
            <span class="line line-a" />
            <span class="line line-b" />
          </div>

          <div class="path-panel">
            <p>当前路径</p>
            <div class="path-list">
              <span v-for="item in pathItems" :key="item">{{ item }}</span>
            </div>
          </div>

          <div class="knowledge-panel">
            <p>知识卡片</p>
            <article v-for="card in visibleKnowledgeCards" :key="card.id">
              <GIcon :name="card.icon" :size="16" />
              <div>
                <strong>{{ card.title }}</strong>
                <span>{{ card.body }}</span>
              </div>
            </article>
          </div>
        </aside>

        <main class="wizard-main">
          <header class="wizard-header">
            <div>
              <p>{{ screenCaption }}</p>
              <h1 id="smart-wizard-title">
                {{ screenTitle }}
              </h1>
            </div>
            <div class="wizard-header__actions">
              <button
                v-if="screen !== 'welcome'"
                type="button"
                class="text-action"
                @click="restartWizard">
                重新开始
              </button>
              <button type="button" class="text-action" @click="skipWizard">跳过</button>
              <button
                type="button"
                class="icon-action"
                aria-label="稍后继续"
                @click="closeForLater">
                <GIcon name="close" :size="16" />
              </button>
            </div>
          </header>

          <div class="wizard-progress" aria-hidden="true">
            <span :style="{ width: `${progressPercent}%` }" />
          </div>

          <section ref="wizardContentRef" class="wizard-content">
            <div v-if="screen === 'welcome'" ref="activePanelRef" class="welcome-screen">
              <div class="welcome-mark">
                <GIcon name="sparkles" :size="32" />
              </div>
              <h2>像聊天一样完成 Gate 配置</h2>
              <p>
                我会问几个简单问题，然后自动推荐隧道类型、协议、端口、域名和证书策略。 预计 3-5
                分钟完成。
              </p>

              <div class="welcome-points">
                <article>
                  <GIcon name="message" :size="18" />
                  <strong>不填复杂表单</strong>
                  <span>每次只回答一个问题。</span>
                </article>
                <article>
                  <GIcon name="sparkles" :size="18" />
                  <strong>自动生成配置</strong>
                  <span>根据场景推荐协议和端口。</span>
                </article>
                <article>
                  <GIcon name="circle-help" :size="18" />
                  <strong>随时解释概念</strong>
                  <span>用简单语言说明为什么。</span>
                </article>
              </div>

              <label class="never-show">
                <input v-model="neverShowChoice" type="checkbox" />
                <span>以后不再显示</span>
              </label>
            </div>

            <template v-else>
              <div class="chat-log" aria-live="polite">
                <article
                  v-for="message in conversation"
                  :key="message.id"
                  class="chat-message"
                  :class="`is-${message.role}`">
                  <span v-if="message.role === 'gate'" class="avatar">
                    <GIcon name="sparkles" :size="14" />
                  </span>
                  <div>
                    <strong v-if="message.title">{{ message.title }}</strong>
                    <p>{{ message.body }}</p>
                  </div>
                </article>
              </div>

              <section
                v-if="screen === 'server-question'"
                ref="activePanelRef"
                class="question-panel">
                <button
                  v-for="option in serverOwnershipOptions"
                  :key="option.value"
                  type="button"
                  class="choice-card"
                  @click="chooseServerOwnership(option.value)">
                  <GIcon :name="option.icon" :size="20" />
                  <strong>{{ option.label }}</strong>
                  <span>{{ option.description }}</span>
                </button>
              </section>

              <section
                v-else-if="screen === 'server-education'"
                ref="activePanelRef"
                class="education-panel">
                <div class="explain-card">
                  <GIcon name="servers" :size="22" />
                  <div>
                    <strong>为什么需要公网服务器？</strong>
                    <p>
                      你的电脑通常在家里、公司或校园网里，外面的人访问不到。
                      公网服务器像一个门口，先接住请求，再交给 Gate 转回你的本地服务。
                    </p>
                  </div>
                </div>

                <div class="provider-grid">
                  <article
                    v-for="provider in cloudProviders"
                    :key="provider.id"
                    :class="`tone-${provider.tone}`">
                    <strong>{{ provider.name }}</strong>
                    <span>{{ provider.note }}</span>
                  </article>
                </div>

                <div class="reserved-deploy">
                  <GIcon name="rocket" :size="18" />
                  <div>
                    <strong>一键部署已预留</strong>
                    <span>未来会在这里直接选择云厂商并自动部署 Gate 服务端。</span>
                  </div>
                </div>

                <div class="education-actions">
                  <GButton
                    variant="secondary"
                    icon="servers"
                    @click="switchToEnvironmentFromEducation">
                    我已经准备好服务器
                  </GButton>
                </div>
              </section>

              <section
                v-else-if="screen === 'environment'"
                ref="activePanelRef"
                class="environment-panel">
                <label class="chat-input">
                  <span>服务器地址</span>
                  <input
                    v-model.trim="answers.serverAddress"
                    autocomplete="off"
                    placeholder="例如 203.0.113.10 或 gate.example.com" />
                </label>

                <div class="environment-grid">
                  <button
                    v-for="environment in serverEnvironmentOptions"
                    :key="environment.id"
                    type="button"
                    class="environment-card"
                    :class="{
                      active: answers.serverEnvironment === environment.id,
                      reserved: environment.reserved,
                    }"
                    :disabled="environment.reserved"
                    @click="chooseEnvironment(environment.id)">
                    <GIcon :name="environment.icon" :size="18" />
                    <strong>{{ environment.title }}</strong>
                    <span>{{ environment.description }}</span>
                    <small>{{ environment.recommendedDeploy }}</small>
                  </button>
                </div>
              </section>

              <section
                v-else-if="screen === 'deployment'"
                ref="activePanelRef"
                class="deployment-panel">
                <div class="deployment-mode-grid">
                  <button
                    v-for="mode in deployModeOptions"
                    :key="mode.value"
                    type="button"
                    class="environment-card"
                    :class="{ active: answers.deployMode === mode.value }"
                    @click="answers.deployMode = mode.value">
                    <GIcon :name="mode.icon" :size="18" />
                    <strong>{{ mode.label }}</strong>
                    <span>{{ mode.description }}</span>
                  </button>
                </div>

                <div class="deployment-form">
                  <label class="chat-input">
                    <span>Gate Server 端口</span>
                    <input v-model.number="answers.serverPort" type="number" min="1" max="65535" />
                  </label>
                  <label class="chat-input">
                    <span>Token</span>
                    <input
                      v-model.trim="answers.serverToken"
                      autocomplete="off"
                      placeholder="建议使用 16 位以上随机字符串" />
                  </label>
                </div>

                <div class="deploy-command-panel">
                  <header>
                    <div>
                      <p>{{ answers.deployMode === 'docker' ? 'Docker' : 'Linux VPS' }}</p>
                      <strong>复制到服务器执行</strong>
                    </div>
                    <button type="button" @click="copyDeployCommand">
                      <GIcon name="copy" :size="14" />
                      复制
                    </button>
                  </header>
                  <pre><code>{{ activeDeployCommand }}</code></pre>
                </div>

                <div v-if="deploymentReport" class="deployment-test-result">
                  <GIcon
                    :name="deploymentReport.ok ? 'check-circle' : 'alert-triangle'"
                    :size="18" />
                  <div>
                    <strong>{{ deploymentReport.title }}</strong>
                    <p>{{ deploymentReport.reason }}</p>
                    <small>{{ deploymentReport.solution }}</small>
                  </div>
                </div>
              </section>

              <section v-else-if="screen === 'domain'" ref="activePanelRef" class="domain-panel">
                <div class="choice-row">
                  <button
                    v-for="option in domainOptions"
                    :key="option.value"
                    type="button"
                    class="choice-card"
                    :class="{ active: answers.domainMode === option.value }"
                    @click="chooseDomainMode(option.value)">
                    <GIcon :name="option.icon" :size="20" />
                    <strong>{{ option.label }}</strong>
                    <span>{{ option.description }}</span>
                  </button>
                </div>

                <label v-if="answers.domainMode === 'has-domain'" class="chat-input">
                  <span>域名</span>
                  <input
                    v-model.trim="answers.domainName"
                    autocomplete="off"
                    placeholder="api.example.com" />
                </label>

                <div
                  v-if="answers.domainMode && answers.domainMode !== 'has-domain'"
                  class="plain-note">
                  <GIcon name="info-circle" :size="17" />
                  <span>没有域名也可以正常使用，Gate 会推荐 IP + 端口的访问方式。</span>
                </div>
              </section>

              <section
                v-else-if="screen === 'scenario'"
                ref="activePanelRef"
                class="scenario-panel">
                <div class="scenario-grid">
                  <button
                    v-for="scenario in scenarioRecommendations"
                    :key="scenario.id"
                    type="button"
                    class="scenario-card"
                    :class="{ active: answers.scenarioId === scenario.id }"
                    @click="chooseScenario(scenario.id)">
                    <GIcon :name="scenario.icon" :size="18" />
                    <strong>{{ scenario.title }}</strong>
                    <span>{{ scenario.description }}</span>
                    <small
                      >{{ scenario.protocol.toUpperCase() }} · {{ scenario.localPort }} →
                      {{ scenario.remotePort }}</small
                    >
                  </button>
                </div>

                <div class="quick-adjust">
                  <label>
                    <span>隧道名称</span>
                    <input
                      v-model.trim="answers.customName"
                      autocomplete="off"
                      :placeholder="selectedScenario.defaultName" />
                  </label>
                  <label>
                    <span>本地端口</span>
                    <input
                      v-model.number="answers.customLocalPort"
                      inputmode="numeric"
                      type="number"
                      :placeholder="String(selectedScenario.localPort)" />
                  </label>
                </div>
              </section>

              <section v-else ref="activePanelRef" class="review-panel">
                <div class="recommendation-card">
                  <header>
                    <div>
                      <p>推荐配置</p>
                      <h2>{{ recommendation.tunnelName }}</h2>
                    </div>
                    <span>{{ recommendation.protocol.toUpperCase() }}</span>
                  </header>

                  <dl class="config-list">
                    <div>
                      <dt>服务器</dt>
                      <dd>{{ recommendation.server }}</dd>
                    </div>
                    <div>
                      <dt>协议</dt>
                      <dd>{{ recommendation.protocol.toUpperCase() }}</dd>
                    </div>
                    <div>
                      <dt>本地</dt>
                      <dd>{{ recommendation.local }}</dd>
                    </div>
                    <div>
                      <dt>远程端口</dt>
                      <dd>{{ recommendation.remote }}</dd>
                    </div>
                    <div>
                      <dt>域名</dt>
                      <dd>{{ recommendation.domain }}</dd>
                    </div>
                    <div>
                      <dt>证书</dt>
                      <dd>{{ recommendation.certificate }}</dd>
                    </div>
                  </dl>

                  <div class="access-preview">
                    <span>访问预览</span>
                    <code>{{ recommendation.accessPreview }}</code>
                  </div>
                </div>

                <div class="why-card">
                  <GIcon name="circle-help" :size="18" />
                  <div>
                    <strong>为什么推荐这样配置？</strong>
                    <ul>
                      <li v-for="reason in recommendation.reasonList" :key="reason">
                        {{ reason }}
                      </li>
                    </ul>
                  </div>
                </div>
              </section>
            </template>
          </section>

          <footer class="wizard-footer">
            <GButton v-if="screen === 'welcome'" variant="ghost" @click="skipWizard">
              跳过
            </GButton>
            <GButton v-else variant="ghost" @click="goBack"> 返回 </GButton>

            <span class="inline-error">{{ inlineError }}</span>

            <GButton
              v-if="screen === 'welcome'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="startWizard">
              开始配置
            </GButton>
            <GButton
              v-else-if="screen === 'server-education'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="continueWithoutServer">
              我先了解，继续
            </GButton>
            <GButton
              v-else-if="screen === 'environment'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="continueFromEnvironment">
              继续
            </GButton>
            <div v-else-if="screen === 'deployment'" class="deployment-footer-actions">
              <GButton
                variant="secondary"
                icon="activity"
                :loading="deploymentTesting"
                @click="testDeploymentConnection">
                测试连接
              </GButton>
              <GButton variant="primary" trailing-icon="arrow-right" @click="continueFromDeployment">
                继续
              </GButton>
            </div>
            <GButton
              v-else-if="screen === 'domain'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="continueFromDomain">
              继续
            </GButton>
            <GButton
              v-else-if="screen === 'scenario'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="continueFromScenario">
              生成推荐配置
            </GButton>
            <GButton
              v-else-if="screen === 'review'"
              variant="primary"
              :icon="answers.serverOwnership === 'has-server' ? 'plus' : 'servers'"
              :loading="creating"
              @click="handleReviewAction">
              {{ answers.serverOwnership === 'has-server' ? '确认并创建' : '去添加服务器' }}
            </GButton>
          </footer>
        </main>
      </section>
    </div>
  </Transition>

  <Transition name="tour">
    <div v-if="tourVisible" class="tour-overlay">
      <div class="tour-scrim" @click="finishTour" />
      <div v-if="spotlightRect" class="tour-ring" :style="spotlightStyle" />
      <article class="tour-card" :style="tourCardStyle">
        <p>快速认识 Gate</p>
        <h2>{{ currentTour.title }}</h2>
        <span>{{ currentTour.body }}</span>
        <footer>
          <small>{{ tourIndex + 1 }} / {{ tourItems.length }}</small>
          <GButton variant="primary" size="sm" @click="nextTour">
            {{ tourIndex === tourItems.length - 1 ? '完成' : '下一处' }}
          </GButton>
        </footer>
      </article>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import {
  buildSmartRecommendation,
  cloudProviders,
  createDefaultAnswers,
  findScenario,
  knowledgeCards,
  scenarioRecommendations,
  serverEnvironmentOptions,
  smartOnboardingKeys,
  type DeployMode,
  type DomainMode,
  type ServerEnvironmentId,
  type ServerOwnership,
  type SmartWizardAnswers,
} from '@/onboarding/smartWizard'
import { diagnosticsService, type ConnectionTestReport } from '@/services'
import { useTunnelStore } from '@/views/tunnels/store/tunnel'

type WizardScreen =
  | 'welcome'
  | 'server-question'
  | 'server-education'
  | 'environment'
  | 'deployment'
  | 'domain'
  | 'scenario'
  | 'review'

interface ChatMessage {
  id: string
  role: 'gate' | 'user'
  title?: string
  body: string
}

interface WizardDraft {
  screen: WizardScreen
  answers: SmartWizardAnswers
  conversation: ChatMessage[]
  history: WizardScreen[]
}

const router = useRouter()
const tunnelStore = useTunnelStore()

const visible = ref(false)
const screen = ref<WizardScreen>('welcome')
const screenHistory = ref<WizardScreen[]>([])
const conversation = ref<ChatMessage[]>([])
const neverShowChoice = ref(false)
const inlineError = ref('')
const creating = ref(false)
const createdTunnelName = ref('')
const deploymentTesting = ref(false)
const deploymentReport = ref<ConnectionTestReport | null>(null)
const wizardContentRef = ref<HTMLElement | null>(null)
const activePanelRef = ref<HTMLElement | null>(null)
const answers = reactive<SmartWizardAnswers>(createDefaultAnswers())

const tourVisible = ref(false)
const tourIndex = ref(0)
const spotlightRect = ref<DOMRect | null>(null)

const serverOwnershipOptions: Array<{
  value: ServerOwnership
  label: string
  description: string
  icon: string
}> = [
  {
    value: 'has-server',
    label: '我已有服务器',
    description: '继续选择服务器环境和部署方式。',
    icon: 'servers',
  },
  {
    value: 'no-server',
    label: '我没有服务器',
    description: '先了解为什么需要，再看推荐平台。',
    icon: 'cloud',
  },
  {
    value: 'unknown-server',
    label: '我不知道什么是公网服务器',
    description: '用最简单的话解释，不讲术语。',
    icon: 'circle-help',
  },
]

const deployModeOptions: Array<{
  value: DeployMode
  label: string
  description: string
  icon: string
}> = [
  {
    value: 'linux-vps',
    label: 'Linux VPS',
    description: '适合 Ubuntu / Debian / CentOS，使用 systemd 或二进制长期运行。',
    icon: 'terminal',
  },
  {
    value: 'docker',
    label: 'Docker',
    description: '适合容器化部署，配置文件和数据挂载在宿主机。',
    icon: 'boxes',
  },
]

const domainOptions: Array<{
  value: DomainMode
  label: string
  description: string
  icon: string
}> = [
  {
    value: 'has-domain',
    label: '有',
    description: '推荐 HTTPS 和自动证书。',
    icon: 'globe',
  },
  {
    value: 'no-domain',
    label: '没有',
    description: '仍然可以使用 IP + 端口。',
    icon: 'network',
  },
  {
    value: 'skip-domain',
    label: '暂时不用',
    description: '先完成隧道，之后再绑定域名。',
    icon: 'clock',
  },
]

const tourItems = [
  {
    target: 'dashboard',
    title: '首页',
    body: '这里看整体状态、运行情况和最近活动。',
  },
  {
    target: 'tunnels',
    title: '隧道',
    body: '这里管理刚创建的隧道，启动、停止和查看访问地址。',
  },
  {
    target: 'logs',
    title: 'Log',
    body: '连接失败或回调异常时，先到这里看发生了什么。',
  },
  {
    target: 'settings',
    title: '设置',
    body: '需要重新打开新手引导、调整主题或清理本地缓存时来这里。',
  },
]

const selectedScenario = computed(() => findScenario(answers.scenarioId))
const recommendation = computed(() => buildSmartRecommendation(answers))
const currentTour = computed(() => tourItems[tourIndex.value])
const serverAddressForTest = computed(() => ({
  host: answers.serverAddress.trim(),
  port: answers.serverPort,
  token: answers.serverToken.trim(),
}))
const linuxDeployCommand = computed(
  () =>
    `GATE_TOKEN="${answers.serverToken.trim() || 'your-token'}" GATE_BIND="0.0.0.0:${answers.serverPort || 7000}" ./gate-server`,
)
const dockerDeployCommand = computed(
  () =>
    [
      'docker run -d --name gate-server --restart unless-stopped \\',
      `  -e GATE_TOKEN="${answers.serverToken.trim() || 'your-token'}" \\`,
      `  -p ${answers.serverPort || 7000}:${answers.serverPort || 7000} \\`,
      '  ghcr.io/gate/gate-server:beta',
    ].join('\n'),
)
const activeDeployCommand = computed(() =>
  answers.deployMode === 'docker' ? dockerDeployCommand.value : linuxDeployCommand.value,
)
const visibleKnowledgeCards = computed(() => {
  if (screen.value === 'server-question' || screen.value === 'server-education') {
    return knowledgeCards.filter((card) => ['public-server', 'tunnel', 'domain'].includes(card.id))
  }
  if (screen.value === 'deployment') {
    return knowledgeCards.filter((card) => ['public-server', 'tunnel', 'https'].includes(card.id))
  }
  if (screen.value === 'domain' || screen.value === 'review') {
    return knowledgeCards.filter((card) => ['domain', 'https', 'certificate'].includes(card.id))
  }
  return knowledgeCards.slice(0, 3)
})

const screenTitle = computed(() => {
  const titles: Record<WizardScreen, string> = {
    welcome: '欢迎使用 Gate',
    'server-question': '你已经拥有公网服务器了吗？',
    'server-education':
      answers.serverOwnership === 'unknown-server' ? '先理解公网服务器' : '没有服务器也没关系',
    environment: '你的服务器是什么环境？',
    deployment: '部署 Gate Server 并测试连接',
    domain: '你拥有域名吗？',
    scenario: '你想用 Gate 做什么？',
    review: '推荐配置已生成',
  }
  return titles[screen.value]
})

const screenCaption = computed(() => {
  const captions: Record<WizardScreen, string> = {
    welcome: '约 3-5 分钟',
    'server-question': '第一个关键判断',
    'server-education': '基础概念',
    environment: '部署方式推荐',
    deployment: '服务器部署向导',
    domain: '访问地址',
    scenario: '使用场景',
    review: '确认即可创建',
  }
  return captions[screen.value]
})

const progressPercent = computed(() => {
  const progress: Record<WizardScreen, number> = {
    welcome: 8,
    'server-question': 18,
    'server-education': 36,
    environment: 36,
    deployment: 48,
    domain: 62,
    scenario: 76,
    review: 94,
  }
  return progress[screen.value]
})

const pathItems = computed(() => {
  const list = ['欢迎']
  if (answers.serverOwnership) {
    const option = serverOwnershipOptions.find((item) => item.value === answers.serverOwnership)
    list.push(option?.label ?? '服务器')
  }
  if (answers.serverEnvironment) {
    const environment = serverEnvironmentOptions.find(
      (item) => item.id === answers.serverEnvironment,
    )
    list.push(environment?.title ?? '环境')
  }
  if (
    answers.serverOwnership === 'has-server' &&
    ['deployment', 'domain', 'scenario', 'review'].includes(screen.value)
  ) {
    const option = deployModeOptions.find((item) => item.value === answers.deployMode)
    list.push(option?.label ?? '部署')
  }
  if (answers.domainMode) {
    const option = domainOptions.find((item) => item.value === answers.domainMode)
    list.push(option?.label ?? '域名')
  }
  if (answers.scenarioId && (screen.value === 'scenario' || screen.value === 'review')) {
    list.push(selectedScenario.value.title)
  }
  return list
})

const spotlightStyle = computed(() => {
  if (!spotlightRect.value) return {}
  return {
    left: `${spotlightRect.value.left - 6}px`,
    top: `${spotlightRect.value.top - 6}px`,
    width: `${spotlightRect.value.width + 12}px`,
    height: `${spotlightRect.value.height + 12}px`,
  }
})

const tourCardStyle = computed(() => {
  if (!spotlightRect.value) {
    return { left: '260px', top: '96px' }
  }
  const top = Math.min(window.innerHeight - 180, Math.max(24, spotlightRect.value.top - 18))
  return {
    left: `${spotlightRect.value.right + 22}px`,
    top: `${top}px`,
  }
})

watch(
  [screen, () => ({ ...answers }), conversation, screenHistory],
  () => {
    if (!visible.value || screen.value === 'welcome') return
    saveDraft()
  },
  { deep: true },
)

watch(screen, () => {
  void scrollActivePanelIntoView()
})

onMounted(() => {
  window.addEventListener('gate:onboarding:open', handleOpenEvent as EventListener)
  window.addEventListener('resize', updateSpotlight)
  openOnFirstLaunch()
})

onBeforeUnmount(() => {
  window.removeEventListener('gate:onboarding:open', handleOpenEvent as EventListener)
  window.removeEventListener('resize', updateSpotlight)
})

function openOnFirstLaunch() {
  const completed = localStorage.getItem(smartOnboardingKeys.completed) === 'true'
  const neverShow = localStorage.getItem(smartOnboardingKeys.neverShow) === 'true'
  if (!completed && !neverShow) {
    openWizard(false)
  }
}

function handleOpenEvent(event: CustomEvent<{ restart?: boolean }>) {
  openWizard(Boolean(event.detail?.restart))
}

function openWizard(restart: boolean) {
  inlineError.value = ''
  visible.value = true
  if (restart) {
    resetWizard()
    return
  }
  const draft = readDraft()
  if (draft) {
    Object.assign(answers, draft.answers)
    screen.value = draft.screen
    conversation.value = draft.conversation.length
      ? draft.conversation
      : createOpeningConversation()
    screenHistory.value = draft.history
  } else {
    resetWizard()
  }
}

function resetWizard() {
  Object.assign(answers, createDefaultAnswers())
  screen.value = 'welcome'
  screenHistory.value = []
  conversation.value = []
  inlineError.value = ''
  createdTunnelName.value = ''
  deploymentReport.value = null
  localStorage.removeItem(smartOnboardingKeys.draft)
}

function restartWizard() {
  resetWizard()
  startWizard()
}

function startWizard() {
  conversation.value = createOpeningConversation()
  navigateTo('server-question')
}

function createOpeningConversation(): ChatMessage[] {
  return [
    {
      id: makeId(),
      role: 'gate',
      title: '我们先从最关键的问题开始',
      body: 'Gate 需要一个公网入口。你不用懂网络，我会根据你的回答继续问。',
    },
  ]
}

function chooseServerOwnership(value: ServerOwnership) {
  inlineError.value = ''
  answers.serverOwnership = value
  const option = serverOwnershipOptions.find((item) => item.value === value)
  pushUser(option?.label ?? value)
  if (value === 'has-server') {
    pushGate('太好了。接下来我只需要知道服务器环境，这样能推荐最省心的部署方式。')
    navigateTo('environment')
    return
  }
  pushGate(
    value === 'unknown-server'
      ? '没关系，我们先把公网服务器讲清楚。你可以先了解，再继续生成配置。'
      : '没有服务器也可以先走完配置思路。拿到服务器后，可以从设置里重新打开引导补齐。',
  )
  navigateTo('server-education')
}

function continueWithoutServer() {
  pushUser('我先了解，继续')
  pushGate('好的。下一步看域名。即使没有域名，也可以先用 IP 加端口访问。')
  navigateTo('domain')
}

function switchToEnvironmentFromEducation() {
  answers.serverOwnership = 'has-server'
  pushUser('我已经准备好服务器')
  pushGate('很好。选择服务器环境后，我会推荐部署方式。')
  navigateTo('environment')
}

function chooseEnvironment(id: ServerEnvironmentId) {
  const environment = serverEnvironmentOptions.find((item) => item.id === id)
  if (!environment || environment.reserved) return
  answers.serverEnvironment = id
  inlineError.value = ''
  deploymentReport.value = null
}

function continueFromEnvironment() {
  inlineError.value = ''
  if (!answers.serverAddress.trim()) {
    inlineError.value = '请填写服务器 IP 或域名。'
    return
  }
  if (!answers.serverEnvironment) {
    inlineError.value = '请选择服务器环境。'
    return
  }
  const environment = serverEnvironmentOptions.find((item) => item.id === answers.serverEnvironment)
  pushUser(`${answers.serverAddress}，${environment?.title ?? '服务器'}`)
  pushGate(`${environment?.recommendedDeploy ?? '已记录服务器环境'} 下一步配置 Token，并测试连接。`)
  navigateTo('deployment')
}

function continueFromDeployment() {
  inlineError.value = ''
  if (!Number.isInteger(answers.serverPort) || answers.serverPort < 1 || answers.serverPort > 65535) {
    inlineError.value = 'Gate Server 端口必须在 1-65535 之间。'
    return
  }
  if (!answers.serverToken.trim()) {
    inlineError.value = '请填写服务端 Token。'
    return
  }
  const mode = deployModeOptions.find((item) => item.value === answers.deployMode)
  pushUser(`${mode?.label ?? '部署'}，端口 ${answers.serverPort}`)
  pushGate('部署信息已记录。现在选择是否使用域名。')
  navigateTo('domain')
}

async function testDeploymentConnection() {
  inlineError.value = ''
  if (!answers.serverAddress.trim()) {
    inlineError.value = '请先填写服务器 IP 或域名。'
    return
  }
  if (!Number.isInteger(answers.serverPort) || answers.serverPort < 1 || answers.serverPort > 65535) {
    inlineError.value = 'Gate Server 端口必须在 1-65535 之间。'
    return
  }
  if (!answers.serverToken.trim()) {
    inlineError.value = '请填写服务端 Token 后再测试。'
    return
  }

  deploymentTesting.value = true
  try {
    deploymentReport.value = await diagnosticsService.testConnection(serverAddressForTest.value)
  } catch (error) {
    inlineError.value = error instanceof Error ? error.message : '连接测试失败，请检查服务器。'
  } finally {
    deploymentTesting.value = false
  }
}

async function copyDeployCommand() {
  await navigator.clipboard?.writeText(activeDeployCommand.value)
  pushGate('部署命令已复制。执行后回到这里点击“测试连接”。')
}

function chooseDomainMode(value: DomainMode) {
  answers.domainMode = value
  if (value !== 'has-domain') answers.domainName = ''
  inlineError.value = ''
}

function continueFromDomain() {
  inlineError.value = ''
  if (!answers.domainMode) {
    inlineError.value = '请选择域名状态。'
    return
  }
  if (answers.domainMode === 'has-domain' && !answers.domainName.trim()) {
    inlineError.value = '请填写域名，例如 api.example.com。'
    return
  }
  const option = domainOptions.find((item) => item.value === answers.domainMode)
  pushUser(
    answers.domainMode === 'has-domain' ? answers.domainName : (option?.label ?? '暂不使用域名'),
  )
  pushGate('收到。现在告诉我你的使用场景，我会自动选择隧道类型和默认端口。')
  navigateTo('scenario')
}

function chooseScenario(id: string) {
  const scenario = findScenario(id)
  answers.scenarioId = scenario.id
  answers.customName = scenario.defaultName
  answers.customLocalPort = scenario.localPort
  inlineError.value = ''
}

function continueFromScenario() {
  inlineError.value = ''
  const port = answers.customLocalPort ?? selectedScenario.value.localPort
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    inlineError.value = '本地端口必须在 1-65535 之间。'
    return
  }
  pushUser(`${selectedScenario.value.title}，本地端口 ${port}`)
  pushGate('我已经生成推荐配置。你只需要确认；下面也会解释为什么这样选。')
  navigateTo('review')
}

async function createFirstTunnel() {
  inlineError.value = ''
  creating.value = true
  try {
    const created = await tunnelStore.createTunnel(recommendation.value.form)
    createdTunnelName.value = created.name
    markComplete()
    visible.value = false
    await router.push('/')
    await nextTick()
    startTour()
  } catch (error) {
    inlineError.value = error instanceof Error ? error.message : '创建隧道失败，请稍后重试。'
  } finally {
    creating.value = false
  }
}

async function handleReviewAction() {
  if (answers.serverOwnership !== 'has-server') {
    markComplete()
    visible.value = false
    await router.push('/servers?create=1')
    return
  }

  await createFirstTunnel()
}

function navigateTo(nextScreen: WizardScreen) {
  if (screen.value !== nextScreen) {
    screenHistory.value.push(screen.value)
  }
  screen.value = nextScreen
}

function goBack() {
  inlineError.value = ''
  const previous = screenHistory.value.pop()
  if (previous) {
    screen.value = previous
    return
  }
  screen.value = 'welcome'
}

async function scrollActivePanelIntoView() {
  await nextTick()
  const container = wizardContentRef.value
  const target = activePanelRef.value
  if (!container || !target) return
  const containerRect = container.getBoundingClientRect()
  const targetRect = target.getBoundingClientRect()
  const top = container.scrollTop + targetRect.top - containerRect.top
  container.scrollTo({ top: Math.max(0, top), behavior: 'smooth' })
}

function closeForLater() {
  if (screen.value !== 'welcome') saveDraft()
  visible.value = false
}

function skipWizard() {
  markComplete()
  visible.value = false
  void router.push('/')
}

function markComplete() {
  localStorage.setItem(smartOnboardingKeys.completed, 'true')
  localStorage.setItem(smartOnboardingKeys.oldCompleted, 'true')
  if (neverShowChoice.value) {
    localStorage.setItem(smartOnboardingKeys.neverShow, 'true')
  }
  localStorage.removeItem(smartOnboardingKeys.draft)
}

function saveDraft() {
  const draft: WizardDraft = {
    screen: screen.value,
    answers: { ...answers },
    conversation: conversation.value,
    history: screenHistory.value,
  }
  localStorage.setItem(smartOnboardingKeys.draft, JSON.stringify(draft))
}

function readDraft(): WizardDraft | null {
  try {
    const raw = localStorage.getItem(smartOnboardingKeys.draft)
    return raw ? (JSON.parse(raw) as WizardDraft) : null
  } catch {
    return null
  }
}

function pushGate(body: string, title?: string) {
  conversation.value.push({ id: makeId(), role: 'gate', title, body })
}

function pushUser(body: string) {
  conversation.value.push({ id: makeId(), role: 'user', body })
}

function makeId() {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

function startTour() {
  tourIndex.value = 0
  tourVisible.value = true
  void nextTick(updateSpotlight)
}

function nextTour() {
  if (tourIndex.value >= tourItems.length - 1) {
    finishTour()
    return
  }
  tourIndex.value += 1
  void nextTick(updateSpotlight)
}

function finishTour() {
  tourVisible.value = false
  spotlightRect.value = null
}

function updateSpotlight() {
  if (!tourVisible.value) return
  const target = document.querySelector<HTMLElement>(
    `[data-onboarding-target="${currentTour.value.target}"]`,
  )
  spotlightRect.value = target?.getBoundingClientRect() ?? null
}
</script>

<style scoped>
.smart-onboarding {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  display: grid;
  place-items: center;
  padding: var(--space-5);
  background:
    linear-gradient(135deg, rgba(91, 141, 239, 0.18), transparent 34%),
    linear-gradient(315deg, rgba(47, 209, 124, 0.12), transparent 38%), var(--bg-app);
}

.wizard-shell {
  width: min(1160px, 100%);
  height: min(780px, calc(100vh - 40px));
  display: grid;
  grid-template-columns: 320px minmax(0, 1fr);
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-2xl);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.wizard-rail {
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  padding: var(--space-5);
  border-right: 1px solid var(--border-subtle);
  background: linear-gradient(180deg, rgba(95, 179, 255, 0.08), transparent 42%), var(--bg-sidebar);
}

.rail-brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.rail-brand > span,
.welcome-mark {
  display: grid;
  place-items: center;
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.rail-brand > span {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
}

.rail-brand strong,
.path-panel p,
.knowledge-panel p {
  color: var(--text-primary);
  font-weight: var(--weight-semibold);
}

.rail-brand small,
.path-panel span,
.knowledge-panel article span {
  color: var(--text-tertiary);
}

.rail-illustration {
  position: relative;
  min-height: 172px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background:
    linear-gradient(135deg, rgba(91, 141, 239, 0.14), transparent 48%),
    linear-gradient(180deg, var(--bg-surface), var(--bg-input));
  overflow: hidden;
}

.node {
  position: absolute;
  z-index: 1;
  min-width: 70px;
  height: 34px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-surface-raised);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.node.local {
  left: 18px;
  bottom: 22px;
}

.node.server {
  left: 50%;
  top: 28px;
  transform: translateX(-50%);
}

.node.public {
  right: 18px;
  bottom: 22px;
}

.line {
  position: absolute;
  height: 2px;
  background: linear-gradient(
    90deg,
    transparent,
    var(--color-primary),
    var(--color-success),
    transparent
  );
  transform-origin: left center;
  animation: tunnelPulse 2.4s var(--ease-out) infinite;
}

.line-a {
  left: 76px;
  top: 108px;
  width: 96px;
  transform: rotate(-31deg);
}

.line-b {
  left: 154px;
  top: 68px;
  width: 112px;
  transform: rotate(32deg);
  animation-delay: 0.6s;
}

.path-panel,
.knowledge-panel {
  display: grid;
  gap: var(--space-3);
}

.path-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.path-list span {
  min-height: 26px;
  display: inline-flex;
  align-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-full);
  background: var(--bg-input);
  padding: 0 var(--space-3);
  font-size: var(--text-xs);
}

.knowledge-panel {
  min-height: 0;
  overflow: auto;
}

.knowledge-panel article {
  display: grid;
  grid-template-columns: 26px minmax(0, 1fr);
  gap: var(--space-2);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: rgba(255, 255, 255, 0.02);
}

.knowledge-panel article svg {
  color: var(--color-info);
}

.knowledge-panel article strong,
.knowledge-panel article span {
  display: block;
}

.knowledge-panel article span {
  margin-top: 2px;
  line-height: var(--leading-normal);
}

.wizard-main {
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr) auto;
}

.wizard-header,
.wizard-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-5);
}

.wizard-header p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.wizard-header h1 {
  margin-top: 2px;
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.wizard-header__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.text-action,
.icon-action {
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
}

.text-action {
  height: 32px;
  padding: 0 var(--space-3);
}

.icon-action {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
}

.text-action:hover,
.icon-action:hover {
  color: var(--text-primary);
  border-color: var(--border-strong);
}

.wizard-progress {
  height: 3px;
  background: var(--bg-input);
}

.wizard-progress span {
  display: block;
  height: 100%;
  background: linear-gradient(90deg, var(--color-primary), var(--color-success));
  transition: width var(--duration-base) var(--ease-out);
}

.wizard-content {
  min-height: 0;
  overflow: auto;
  padding: var(--space-5);
}

.welcome-screen {
  min-height: 100%;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-4);
  text-align: center;
}

.welcome-mark {
  width: 76px;
  height: 76px;
  border-radius: var(--radius-md);
}

.welcome-screen h2 {
  max-width: 640px;
  color: var(--text-primary);
  font-size: var(--text-3xl);
  line-height: var(--leading-tight);
  letter-spacing: 0;
}

.welcome-screen p {
  max-width: 680px;
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.welcome-points {
  width: min(720px, 100%);
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.welcome-points article {
  min-height: 118px;
  display: grid;
  align-content: center;
  justify-items: center;
  gap: var(--space-2);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.welcome-points svg {
  color: var(--color-primary);
}

.welcome-points strong {
  color: var(--text-primary);
}

.welcome-points span,
.never-show {
  color: var(--text-secondary);
}

.never-show {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  cursor: pointer;
}

.never-show input {
  width: 15px;
  height: 15px;
  accent-color: var(--color-primary);
}

.chat-log {
  display: grid;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.chat-message {
  display: grid;
  gap: var(--space-2);
  max-width: min(680px, 92%);
}

.chat-message.is-gate {
  grid-template-columns: 28px minmax(0, 1fr);
}

.chat-message.is-user {
  justify-self: end;
}

.chat-message > div {
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.chat-message.is-user > div {
  border-color: rgba(91, 141, 239, 0.32);
  background: var(--color-primary-muted);
}

.chat-message strong {
  display: block;
  margin-bottom: 2px;
  color: var(--text-primary);
}

.chat-message p {
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.avatar {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-full);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.question-panel,
.choice-row,
.scenario-grid,
.environment-grid,
.provider-grid,
.quick-adjust,
.education-actions {
  display: grid;
  gap: var(--space-3);
}

.question-panel,
.choice-row {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.choice-card,
.scenario-card,
.environment-card,
.provider-grid article {
  min-height: 118px;
  display: grid;
  align-content: start;
  gap: var(--space-2);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  color: var(--text-primary);
  text-align: left;
}

button.choice-card,
button.scenario-card,
button.environment-card {
  cursor: pointer;
}

.choice-card:hover,
.choice-card.active,
.scenario-card:hover,
.scenario-card.active,
.environment-card:hover,
.environment-card.active {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
}

.choice-card svg,
.scenario-card svg,
.environment-card svg {
  color: var(--color-primary);
}

.choice-card span,
.scenario-card span,
.environment-card span,
.provider-grid article span {
  color: var(--text-secondary);
  line-height: var(--leading-normal);
}

.scenario-card small,
.environment-card small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.environment-card.reserved {
  opacity: 0.52;
  cursor: not-allowed;
}

.education-panel,
.environment-panel,
.deployment-panel,
.domain-panel,
.scenario-panel,
.review-panel {
  display: grid;
  gap: var(--space-4);
}

.deployment-mode-grid,
.deployment-form {
  display: grid;
  gap: var(--space-3);
}

.deployment-mode-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.deployment-form {
  grid-template-columns: 180px minmax(0, 1fr);
}

.deploy-command-panel {
  overflow: hidden;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.deploy-command-panel header {
  min-height: 54px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
}

.deploy-command-panel header p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.deploy-command-panel header strong {
  color: var(--text-primary);
}

.deploy-command-panel header button {
  min-height: 30px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 0 var(--space-3);
  cursor: pointer;
}

.deploy-command-panel header button:hover {
  border-color: var(--color-primary);
  color: var(--text-primary);
}

.deploy-command-panel pre {
  margin: 0;
  max-height: 180px;
  overflow: auto;
  padding: var(--space-4);
  background: var(--bg-input);
}

.deploy-command-panel code {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.deployment-test-result {
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr);
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.deployment-test-result svg {
  color: var(--color-info);
}

.deployment-test-result strong {
  display: block;
  color: var(--text-primary);
}

.deployment-test-result p,
.deployment-test-result small {
  display: block;
  margin-top: 2px;
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.deployment-footer-actions {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
}

.explain-card,
.reserved-deploy,
.plain-note,
.why-card {
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr);
  gap: var(--space-3);
  align-items: start;
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.explain-card svg,
.reserved-deploy svg,
.plain-note svg,
.why-card svg {
  color: var(--color-info);
}

.explain-card p,
.reserved-deploy span,
.plain-note span,
.why-card li {
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.provider-grid {
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.provider-grid article {
  min-height: 104px;
}

.provider-grid article.tone-mainland {
  border-color: rgba(91, 141, 239, 0.28);
}

.provider-grid article.tone-free {
  border-color: rgba(47, 209, 124, 0.28);
}

.provider-grid article.tone-experience {
  border-color: rgba(245, 184, 75, 0.28);
}

.chat-input,
.quick-adjust label {
  display: grid;
  gap: var(--space-2);
}

.chat-input span,
.quick-adjust span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.chat-input input,
.quick-adjust input {
  width: 100%;
  height: 38px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: 0 var(--space-3);
  outline: 0;
}

.chat-input input:focus,
.quick-adjust input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.environment-grid {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.scenario-grid {
  grid-template-columns: repeat(4, minmax(0, 1fr));
}

.quick-adjust {
  grid-template-columns: minmax(0, 1fr) 180px;
}

.recommendation-card {
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.recommendation-card header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
  background: linear-gradient(90deg, rgba(91, 141, 239, 0.14), transparent), var(--bg-surface);
}

.recommendation-card header p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.recommendation-card h2 {
  margin-top: 2px;
  color: var(--text-primary);
  font-size: var(--text-2xl);
  letter-spacing: 0;
}

.recommendation-card header > span {
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  border-radius: var(--radius-full);
  background: var(--color-success-muted);
  color: var(--color-success);
  padding: 0 var(--space-3);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.config-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.config-list div {
  display: grid;
  gap: 2px;
  min-height: 68px;
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
}

.config-list div:nth-child(odd) {
  border-right: 1px solid var(--border-subtle);
}

.config-list dt,
.access-preview span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.config-list dd,
.access-preview code {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  overflow-wrap: anywhere;
}

.access-preview {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-4);
}

.access-preview code {
  text-align: right;
}

.why-card ul {
  display: grid;
  gap: var(--space-2);
  margin-top: var(--space-2);
  padding-left: var(--space-4);
}

.wizard-footer {
  min-height: 72px;
  border-top: 1px solid var(--border-subtle);
}

.inline-error {
  flex: 1;
  min-width: 0;
  color: var(--color-error);
  font-size: var(--text-sm);
  text-align: right;
}

.tour-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--z-popover);
  pointer-events: none;
}

.tour-scrim {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.48);
  pointer-events: auto;
}

.tour-ring {
  position: absolute;
  z-index: 1;
  border: 2px solid var(--color-primary);
  border-radius: var(--radius-md);
  box-shadow:
    0 0 0 4px var(--color-primary-muted),
    0 0 38px rgba(91, 141, 239, 0.46);
  pointer-events: none;
  transition: all var(--duration-base) var(--ease-out);
}

.tour-card {
  position: absolute;
  z-index: 2;
  width: min(330px, calc(100vw - 40px));
  display: grid;
  gap: var(--space-2);
  padding: var(--space-4);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
  pointer-events: auto;
}

.tour-card p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.tour-card h2 {
  color: var(--text-primary);
  font-size: var(--text-xl);
  letter-spacing: 0;
}

.tour-card span {
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.tour-card footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-top: var(--space-2);
}

.tour-card small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
}

.smart-wizard-enter-active,
.smart-wizard-leave-active,
.tour-enter-active,
.tour-leave-active {
  transition: opacity var(--duration-base) var(--ease-out);
}

.smart-wizard-enter-from,
.smart-wizard-leave-to,
.tour-enter-from,
.tour-leave-to {
  opacity: 0;
}

.smart-wizard-enter-active .wizard-shell,
.smart-wizard-leave-active .wizard-shell {
  transition:
    transform var(--duration-base) var(--ease-out),
    opacity var(--duration-base) var(--ease-out);
}

.smart-wizard-enter-from .wizard-shell,
.smart-wizard-leave-to .wizard-shell {
  opacity: 0;
  transform: translateY(10px) scale(0.985);
}

@keyframes tunnelPulse {
  0%,
  100% {
    opacity: 0.42;
  }
  50% {
    opacity: 1;
  }
}

@media (max-width: 980px) {
  .wizard-shell {
    grid-template-columns: 1fr;
  }

  .wizard-rail {
    display: none;
  }

  .provider-grid,
  .scenario-grid,
  .environment-grid,
  .deployment-mode-grid,
  .welcome-points,
  .question-panel,
  .choice-row {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 680px) {
  .smart-onboarding {
    padding: var(--space-2);
  }

  .wizard-shell {
    height: calc(100vh - 16px);
  }

  .wizard-header,
  .wizard-footer {
    align-items: flex-start;
    flex-direction: column;
  }

  .provider-grid,
  .scenario-grid,
  .environment-grid,
  .deployment-mode-grid,
  .deployment-form,
  .welcome-points,
  .question-panel,
  .choice-row,
  .config-list,
  .quick-adjust {
    grid-template-columns: 1fr;
  }

  .config-list div:nth-child(odd) {
    border-right: 0;
  }

  .inline-error {
    width: 100%;
    text-align: left;
  }
}
</style>
