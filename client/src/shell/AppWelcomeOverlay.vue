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
              <small>{{ t('welcome.brand.smartGuide') }}</small>
            </div>
          </div>

          <div class="rail-illustration" aria-hidden="true">
            <div class="node local">{{ t('welcome.nodes.local') }}</div>
            <div class="node server">{{ t('welcome.nodes.server') }}</div>
            <div class="node public">{{ t('welcome.nodes.public') }}</div>
            <span class="line line-a" />
            <span class="line line-b" />
          </div>

          <div class="path-panel">
            <p>{{ t('welcome.rail.currentPath') }}</p>
            <div class="path-list">
              <span v-for="item in pathItems" :key="item">{{ item }}</span>
            </div>
          </div>

          <div class="knowledge-panel">
            <p>{{ t('welcome.rail.knowledgeCards') }}</p>
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
                {{ t('welcome.actions.restart') }}
              </button>
              <button type="button" class="text-action" @click="skipWizard">
                {{ t('welcome.actions.skip') }}
              </button>
              <button
                type="button"
                class="icon-action"
                :aria-label="t('welcome.actions.closeForLater')"
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
              <h2>{{ t('welcome.landing.title') }}</h2>
              <p>{{ t('welcome.landing.description') }}</p>

              <div class="welcome-points">
                <article>
                  <GIcon name="message" :size="18" />
                  <strong>{{ t('welcome.landing.points.simple.title') }}</strong>
                  <span>{{ t('welcome.landing.points.simple.body') }}</span>
                </article>
                <article>
                  <GIcon name="sparkles" :size="18" />
                  <strong>{{ t('welcome.landing.points.auto.title') }}</strong>
                  <span>{{ t('welcome.landing.points.auto.body') }}</span>
                </article>
                <article>
                  <GIcon name="circle-help" :size="18" />
                  <strong>{{ t('welcome.landing.points.explain.title') }}</strong>
                  <span>{{ t('welcome.landing.points.explain.body') }}</span>
                </article>
              </div>

              <label class="never-show">
                <input v-model="neverShowChoice" type="checkbox" />
                <span>{{ t('welcome.landing.neverShow') }}</span>
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
                    <strong v-if="message.titleKey || message.title">
                      {{
                        message.titleKey ? t(message.titleKey, message.params ?? {}) : message.title
                      }}
                    </strong>
                    <p>
                      {{ message.bodyKey ? t(message.bodyKey, message.params ?? {}) : message.body }}
                    </p>
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
                    <strong>{{ t('welcome.education.whyServerTitle') }}</strong>
                    <p>{{ t('welcome.education.whyServerBody') }}</p>
                  </div>
                </div>

                <div class="provider-grid">
                  <article
                    v-for="provider in localizedCloudProviders"
                    :key="provider.id"
                    :class="`tone-${provider.tone}`">
                    <strong>{{ provider.name }}</strong>
                    <span>{{ provider.note }}</span>
                  </article>
                </div>

                <div class="reserved-deploy">
                  <GIcon name="rocket" :size="18" />
                  <div>
                    <strong>{{ t('welcome.education.reservedDeployTitle') }}</strong>
                    <span>{{ t('welcome.education.reservedDeployBody') }}</span>
                  </div>
                </div>

                <div class="deploy-command-panel">
                  <header>
                    <div>
                      <p>Docker</p>
                      <strong>{{ t('welcome.education.dockerQuickStartTitle') }}</strong>
                    </div>
                    <button type="button" @click="copyDockerDeployCommand">
                      <GIcon name="copy" :size="14" />
                      {{ t('welcome.actions.copy') }}
                    </button>
                  </header>
                  <pre><code>{{ visibleDockerDeployCommand }}</code></pre>
                </div>

                <div class="education-actions">
                  <GButton
                    variant="secondary"
                    icon="servers"
                    @click="switchToEnvironmentFromEducation">
                    {{ t('welcome.actions.serverReady') }}
                  </GButton>
                </div>
              </section>

              <section
                v-else-if="screen === 'environment'"
                ref="activePanelRef"
                class="environment-panel">
                <label class="chat-input">
                  <span>{{ t('welcome.fields.serverAddress') }}</span>
                  <input
                    v-model.trim="answers.serverAddress"
                    autocomplete="off"
                    :placeholder="t('welcome.placeholders.serverAddress')" />
                </label>

                <div class="environment-grid">
                  <button
                    v-for="environment in localizedServerEnvironmentOptions"
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
                    <span>{{ t('welcome.fields.serverPort') }}</span>
                    <input v-model.number="answers.serverPort" type="number" min="1" max="65535" />
                  </label>
                  <label class="chat-input">
                    <span>Token</span>
                    <input
                      v-model.trim="answers.serverToken"
                      autocomplete="off"
                      :placeholder="t('welcome.placeholders.token')" />
                  </label>
                </div>

                <div class="deploy-command-panel">
                  <header>
                    <div>
                      <p>{{ answers.deployMode === 'docker' ? 'Docker' : 'Linux VPS' }}</p>
                      <strong>{{ t('welcome.deployment.copyToServer') }}</strong>
                    </div>
                    <button type="button" @click="copyDeployCommand">
                      <GIcon name="copy" :size="14" />
                      {{ t('welcome.actions.copy') }}
                    </button>
                  </header>
                  <pre><code>{{ visibleActiveDeployCommand }}</code></pre>
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
                  <span>{{ t('welcome.fields.domain') }}</span>
                  <input
                    v-model.trim="answers.domainName"
                    autocomplete="off"
                    placeholder="api.example.com" />
                </label>

                <div
                  v-if="answers.domainMode && answers.domainMode !== 'has-domain'"
                  class="plain-note">
                  <GIcon name="info-circle" :size="17" />
                  <span>{{ t('welcome.domain.noDomainNote') }}</span>
                </div>
              </section>

              <section
                v-else-if="screen === 'scenario'"
                ref="activePanelRef"
                class="scenario-panel">
                <div class="scenario-grid">
                  <button
                    v-for="scenario in localizedScenarioRecommendations"
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
                    <span>{{ t('welcome.fields.tunnelName') }}</span>
                    <input
                      v-model.trim="answers.customName"
                      autocomplete="off"
                      :placeholder="selectedScenario.defaultName" />
                  </label>
                  <label>
                    <span>{{ t('welcome.fields.localPort') }}</span>
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
                      <p>{{ t('welcome.review.recommendedConfig') }}</p>
                      <h2>{{ recommendation.tunnelName }}</h2>
                    </div>
                    <span>{{ recommendation.protocol.toUpperCase() }}</span>
                  </header>

                  <dl class="config-list">
                    <div>
                      <dt>{{ t('welcome.review.server') }}</dt>
                      <dd>{{ recommendation.server }}</dd>
                    </div>
                    <div>
                      <dt>{{ t('welcome.review.protocol') }}</dt>
                      <dd>{{ recommendation.protocol.toUpperCase() }}</dd>
                    </div>
                    <div>
                      <dt>{{ t('welcome.review.local') }}</dt>
                      <dd>{{ recommendation.local }}</dd>
                    </div>
                    <div>
                      <dt>{{ t('welcome.review.remotePort') }}</dt>
                      <dd>{{ recommendation.remote }}</dd>
                    </div>
                    <div>
                      <dt>{{ t('welcome.review.domain') }}</dt>
                      <dd>{{ recommendation.domain }}</dd>
                    </div>
                    <div>
                      <dt>{{ t('welcome.review.certificate') }}</dt>
                      <dd>{{ recommendation.certificate }}</dd>
                    </div>
                  </dl>

                  <div class="access-preview">
                    <span>{{ t('welcome.review.accessPreview') }}</span>
                    <code>{{ recommendation.accessPreview }}</code>
                  </div>
                </div>

                <div class="why-card">
                  <GIcon name="circle-help" :size="18" />
                  <div>
                    <strong>{{ t('welcome.review.whyTitle') }}</strong>
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
              {{ t('welcome.actions.skip') }}
            </GButton>
            <GButton v-else variant="ghost" @click="goBack">
              {{ t('common.back') }}
            </GButton>

            <span class="inline-error">{{ inlineError }}</span>

            <GButton
              v-if="screen === 'welcome'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="startWizard">
              {{ t('welcome.actions.start') }}
            </GButton>
            <GButton
              v-else-if="screen === 'server-education'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="continueWithoutServer">
              {{ t('welcome.actions.continueAfterLearning') }}
            </GButton>
            <GButton
              v-else-if="screen === 'environment'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="continueFromEnvironment">
              {{ t('welcome.actions.continue') }}
            </GButton>
            <div v-else-if="screen === 'deployment'" class="deployment-footer-actions">
              <GButton
                variant="secondary"
                icon="activity"
                :loading="deploymentTesting"
                @click="testDeploymentConnection">
                {{ t('welcome.actions.testConnection') }}
              </GButton>
              <GButton variant="primary" trailing-icon="arrow-right" @click="continueFromDeployment">
                {{ t('welcome.actions.continue') }}
              </GButton>
            </div>
            <GButton
              v-else-if="screen === 'domain'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="continueFromDomain">
              {{ t('welcome.actions.continue') }}
            </GButton>
            <GButton
              v-else-if="screen === 'scenario'"
              variant="primary"
              trailing-icon="arrow-right"
              @click="continueFromScenario">
              {{ t('welcome.actions.generateRecommendation') }}
            </GButton>
            <GButton
              v-else-if="screen === 'review'"
              variant="primary"
              :icon="answers.serverOwnership === 'has-server' ? 'plus' : 'servers'"
              :loading="creating"
              @click="handleReviewAction">
              {{
                answers.serverOwnership === 'has-server'
                  ? t('welcome.actions.confirmAndCreate')
                  : t('welcome.actions.goAddServer')
              }}
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
        <p>{{ t('welcome.tour.kicker') }}</p>
        <h2>{{ currentTour.title }}</h2>
        <span>{{ currentTour.body }}</span>
        <footer>
          <small>{{ tourIndex + 1 }} / {{ tourItems.length }}</small>
          <GButton variant="primary" size="sm" @click="nextTour">
            {{
              tourIndex === tourItems.length - 1
                ? t('welcome.actions.finish')
                : t('welcome.actions.nextSpot')
            }}
          </GButton>
        </footer>
      </article>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
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
  type TranslateFn,
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
  titleKey?: string
  body?: string
  bodyKey?: string
  params?: Record<string, unknown>
}

interface WizardDraft {
  screen: WizardScreen
  answers: SmartWizardAnswers
  conversation: ChatMessage[]
  history: WizardScreen[]
}

const router = useRouter()
const tunnelStore = useTunnelStore()
const { t } = useI18n()

const visible = ref(false)
const screen = ref<WizardScreen>('welcome')
const screenHistory = ref<WizardScreen[]>([])
const conversation = ref<ChatMessage[]>([])
const neverShowChoice = ref(false)
const inlineErrorKey = ref('')
const inlineErrorParams = ref<Record<string, unknown>>({})
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

const serverOwnershipOptions = computed<Array<{
  value: ServerOwnership
  label: string
  description: string
  icon: string
}>>(() => [
  {
    value: 'has-server',
    label: t('welcome.serverOwnership.hasServer.label'),
    description: t('welcome.serverOwnership.hasServer.description'),
    icon: 'servers',
  },
  {
    value: 'no-server',
    label: t('welcome.serverOwnership.noServer.label'),
    description: t('welcome.serverOwnership.noServer.description'),
    icon: 'cloud',
  },
  {
    value: 'unknown-server',
    label: t('welcome.serverOwnership.unknownServer.label'),
    description: t('welcome.serverOwnership.unknownServer.description'),
    icon: 'circle-help',
  },
])

const deployModeOptions = computed<Array<{
  value: DeployMode
  label: string
  description: string
  icon: string
}>>(() => [
  {
    value: 'linux-vps',
    label: 'Linux VPS',
    description: t('welcome.deployModes.linuxVps.description'),
    icon: 'terminal',
  },
  {
    value: 'docker',
    label: 'Docker',
    description: t('welcome.deployModes.docker.description'),
    icon: 'boxes',
  },
])

const domainOptions = computed<Array<{
  value: DomainMode
  label: string
  description: string
  icon: string
}>>(() => [
  {
    value: 'has-domain',
    label: t('welcome.domainOptions.hasDomain.label'),
    description: t('welcome.domainOptions.hasDomain.description'),
    icon: 'globe',
  },
  {
    value: 'no-domain',
    label: t('welcome.domainOptions.noDomain.label'),
    description: t('welcome.domainOptions.noDomain.description'),
    icon: 'network',
  },
  {
    value: 'skip-domain',
    label: t('welcome.domainOptions.skipDomain.label'),
    description: t('welcome.domainOptions.skipDomain.description'),
    icon: 'clock',
  },
])

const tourItems = computed(() => [
  {
    target: 'dashboard',
    title: t('welcome.tour.items.dashboard.title'),
    body: t('welcome.tour.items.dashboard.body'),
  },
  {
    target: 'tunnels',
    title: t('welcome.tour.items.tunnels.title'),
    body: t('welcome.tour.items.tunnels.body'),
  },
  {
    target: 'logs',
    title: 'Log',
    body: t('welcome.tour.items.logs.body'),
  },
  {
    target: 'settings',
    title: t('welcome.tour.items.settings.title'),
    body: t('welcome.tour.items.settings.body'),
  },
])

const localizedKnowledgeCards = computed(() =>
  knowledgeCards.map((card) => ({
    ...card,
    title: t(`welcome.knowledge.${card.localeKey}.title`),
    body: t(`welcome.knowledge.${card.localeKey}.body`),
  })),
)
const localizedCloudProviders = computed(() =>
  cloudProviders.map((provider) => ({
    ...provider,
    name: t(`welcome.cloudProviders.${provider.localeKey}.name`),
    note: t(`welcome.cloudProviders.${provider.localeKey}.note`),
  })),
)
const localizedServerEnvironmentOptions = computed(() =>
  serverEnvironmentOptions.map((environment) => ({
    ...environment,
    title: t(`welcome.environments.${environment.localeKey}.title`),
    description: t(`welcome.environments.${environment.localeKey}.description`),
    recommendedDeploy: t(`welcome.environments.${environment.localeKey}.recommendedDeploy`),
  })),
)
const localizedScenarioRecommendations = computed(() =>
  scenarioRecommendations.map((scenario) => ({
    ...scenario,
    title: t(`welcome.scenarios.${scenario.localeKey}.title`),
    description: t(`welcome.scenarios.${scenario.localeKey}.description`),
  })),
)
const selectedScenario = computed(() => findScenario(answers.scenarioId))
const selectedScenarioTitle = computed(() =>
  t(`welcome.scenarios.${selectedScenario.value.localeKey}.title`),
)
const recommendation = computed(() => buildSmartRecommendation(answers, t as TranslateFn))
const currentTour = computed(() => tourItems.value[tourIndex.value])
const inlineError = computed(() =>
  inlineErrorKey.value ? t(inlineErrorKey.value, inlineErrorParams.value) : '',
)
const serverAddressForTest = computed(() => ({
  host: answers.serverAddress.trim(),
  port: answers.serverPort,
  token: answers.serverToken.trim(),
}))
const linuxDeployCommand = computed(
  () =>
    `GATE_AUTH_TOKEN="${answers.serverToken.trim() || 'your-token'}" GATE_SERVER_ADDR="0.0.0.0:${answers.serverPort || 7000}" ./gate-server`,
)
const dockerDeployCommand = computed(
  () =>
    [
      'docker run -d --name gate-server --restart unless-stopped \\',
      `  -e GATE_AUTH_TOKEN="${answers.serverToken.trim() || 'your-token'}" \\`,
      `  -e GATE_SERVER_ADDR="0.0.0.0:${answers.serverPort || 7000}" \\`,
      `  -p ${answers.serverPort || 7000}:${answers.serverPort || 7000} \\`,
      '  ghcr.io/somirk134/gate-server:v0.9.0',
    ].join('\n'),
)
const activeDeployCommand = computed(() =>
  answers.deployMode === 'docker' ? dockerDeployCommand.value : linuxDeployCommand.value,
)
// 页面仅展示脱敏命令，实际口令只在用户主动复制时写入剪贴板。
const maskDeployToken = (command: string) =>
  answers.serverToken.trim()
    ? command.replace(/GATE_AUTH_TOKEN="[^"]*"/g, 'GATE_AUTH_TOKEN="[hidden]"')
    : command
const visibleDockerDeployCommand = computed(() => maskDeployToken(dockerDeployCommand.value))
const visibleActiveDeployCommand = computed(() => maskDeployToken(activeDeployCommand.value))
const visibleKnowledgeCards = computed(() => {
  if (screen.value === 'server-question' || screen.value === 'server-education') {
    return localizedKnowledgeCards.value.filter((card) =>
      ['public-server', 'tunnel', 'domain'].includes(card.id),
    )
  }
  if (screen.value === 'deployment') {
    return localizedKnowledgeCards.value.filter((card) =>
      ['public-server', 'tunnel', 'https'].includes(card.id),
    )
  }
  if (screen.value === 'domain' || screen.value === 'review') {
    return localizedKnowledgeCards.value.filter((card) =>
      ['domain', 'https', 'certificate'].includes(card.id),
    )
  }
  return localizedKnowledgeCards.value.slice(0, 3)
})

const screenTitle = computed(() => {
  const titleKeys: Record<WizardScreen, string> = {
    welcome: 'welcome.screens.welcome.title',
    'server-question': 'welcome.screens.serverQuestion.title',
    'server-education':
      answers.serverOwnership === 'unknown-server'
        ? 'welcome.screens.serverEducationUnknown.title'
        : 'welcome.screens.serverEducation.title',
    environment: 'welcome.screens.environment.title',
    deployment: 'welcome.screens.deployment.title',
    domain: 'welcome.screens.domain.title',
    scenario: 'welcome.screens.scenario.title',
    review: 'welcome.screens.review.title',
  }
  return t(titleKeys[screen.value])
})

const screenCaption = computed(() => {
  const captionKeys: Record<WizardScreen, string> = {
    welcome: 'welcome.screens.welcome.caption',
    'server-question': 'welcome.screens.serverQuestion.caption',
    'server-education': 'welcome.screens.serverEducation.caption',
    environment: 'welcome.screens.environment.caption',
    deployment: 'welcome.screens.deployment.caption',
    domain: 'welcome.screens.domain.caption',
    scenario: 'welcome.screens.scenario.caption',
    review: 'welcome.screens.review.caption',
  }
  return t(captionKeys[screen.value])
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
  const list = [t('welcome.path.welcome')]
  if (answers.serverOwnership) {
    const option = serverOwnershipOptions.value.find(
      (item) => item.value === answers.serverOwnership,
    )
    list.push(option?.label ?? t('welcome.path.serverFallback'))
  }
  if (answers.serverEnvironment) {
    const environment = localizedServerEnvironmentOptions.value.find(
      (item) => item.id === answers.serverEnvironment,
    )
    list.push(environment?.title ?? t('welcome.path.environmentFallback'))
  }
  if (
    answers.serverOwnership === 'has-server' &&
    ['deployment', 'domain', 'scenario', 'review'].includes(screen.value)
  ) {
    const option = deployModeOptions.value.find((item) => item.value === answers.deployMode)
    list.push(option?.label ?? t('welcome.path.deployFallback'))
  }
  if (answers.domainMode) {
    const option = domainOptions.value.find((item) => item.value === answers.domainMode)
    list.push(option?.label ?? t('welcome.path.domainFallback'))
  }
  if (answers.scenarioId && (screen.value === 'scenario' || screen.value === 'review')) {
    list.push(selectedScenarioTitle.value)
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
  clearInlineError()
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
  clearInlineError()
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
      titleKey: 'welcome.chat.openingTitle',
      bodyKey: 'welcome.chat.openingBody',
    },
  ]
}

function chooseServerOwnership(value: ServerOwnership) {
  clearInlineError()
  answers.serverOwnership = value
  const option = serverOwnershipOptions.value.find((item) => item.value === value)
  pushUser(option?.label ?? value)
  if (value === 'has-server') {
    pushGate('welcome.chat.hasServerNext')
    navigateTo('environment')
    return
  }
  pushGate(
    value === 'unknown-server'
      ? 'welcome.chat.unknownServerExplain'
      : 'welcome.chat.noServerExplain',
  )
  navigateTo('server-education')
}

function continueWithoutServer() {
  pushUser(t('welcome.actions.continueAfterLearning'))
  pushGate('welcome.chat.domainNext')
  navigateTo('domain')
}

function switchToEnvironmentFromEducation() {
  answers.serverOwnership = 'has-server'
  pushUser(t('welcome.actions.serverReady'))
  pushGate('welcome.chat.environmentNext')
  navigateTo('environment')
}

function chooseEnvironment(id: ServerEnvironmentId) {
  const environment = localizedServerEnvironmentOptions.value.find((item) => item.id === id)
  if (!environment || environment.reserved) return
  answers.serverEnvironment = id
  clearInlineError()
  deploymentReport.value = null
}

function continueFromEnvironment() {
  clearInlineError()
  if (!answers.serverAddress.trim()) {
    setInlineError('welcome.validation.serverAddressRequired')
    return
  }
  if (!answers.serverEnvironment) {
    setInlineError('welcome.validation.environmentRequired')
    return
  }
  const environment = localizedServerEnvironmentOptions.value.find(
    (item) => item.id === answers.serverEnvironment,
  )
  pushUser(
    t('welcome.chat.environmentUser', {
      address: answers.serverAddress,
      environment: environment?.title ?? t('welcome.path.serverFallback'),
    }),
  )
  pushGate('welcome.chat.deploymentNext')
  navigateTo('deployment')
}

function continueFromDeployment() {
  clearInlineError()
  if (!Number.isInteger(answers.serverPort) || answers.serverPort < 1 || answers.serverPort > 65535) {
    setInlineError('welcome.validation.serverPortRange')
    return
  }
  if (!answers.serverToken.trim()) {
    setInlineError('welcome.validation.serverTokenRequired')
    return
  }
  const mode = deployModeOptions.value.find((item) => item.value === answers.deployMode)
  pushUser(
    t('welcome.chat.deploymentUser', {
      mode: mode?.label ?? t('welcome.path.deployFallback'),
      port: answers.serverPort,
    }),
  )
  pushGate('welcome.chat.deploymentRecorded')
  navigateTo('domain')
}

async function testDeploymentConnection() {
  clearInlineError()
  if (!answers.serverAddress.trim()) {
    setInlineError('welcome.validation.serverAddressBeforeTest')
    return
  }
  if (!Number.isInteger(answers.serverPort) || answers.serverPort < 1 || answers.serverPort > 65535) {
    setInlineError('welcome.validation.serverPortRange')
    return
  }
  if (!answers.serverToken.trim()) {
    setInlineError('welcome.validation.serverTokenBeforeTest')
    return
  }

  deploymentTesting.value = true
  try {
    deploymentReport.value = await diagnosticsService.testConnection(serverAddressForTest.value)
  } catch (error) {
    setInlineError(
      error instanceof Error
        ? 'welcome.validation.connectionTestFailedWithMessage'
        : 'welcome.validation.connectionTestFailed',
      error instanceof Error ? { message: error.message } : {},
    )
  } finally {
    deploymentTesting.value = false
  }
}

async function copyDeployCommand() {
  await navigator.clipboard?.writeText(activeDeployCommand.value)
  pushGate('welcome.chat.deployCommandCopied')
}

async function copyDockerDeployCommand() {
  await navigator.clipboard?.writeText(dockerDeployCommand.value)
  pushGate('welcome.chat.deployCommandCopied')
}

function chooseDomainMode(value: DomainMode) {
  answers.domainMode = value
  if (value !== 'has-domain') answers.domainName = ''
  clearInlineError()
}

function continueFromDomain() {
  clearInlineError()
  if (!answers.domainMode) {
    setInlineError('welcome.validation.domainModeRequired')
    return
  }
  if (answers.domainMode === 'has-domain' && !answers.domainName.trim()) {
    setInlineError('welcome.validation.domainRequired')
    return
  }
  const option = domainOptions.value.find((item) => item.value === answers.domainMode)
  pushUser(
    answers.domainMode === 'has-domain'
      ? answers.domainName
      : (option?.label ?? t('welcome.domainOptions.skipDomain.label')),
  )
  pushGate('welcome.chat.scenarioNext')
  navigateTo('scenario')
}

function chooseScenario(id: string) {
  const scenario = findScenario(id)
  answers.scenarioId = scenario.id
  answers.customName = scenario.defaultName
  answers.customLocalPort = scenario.localPort
  clearInlineError()
}

function continueFromScenario() {
  clearInlineError()
  const port = answers.customLocalPort ?? selectedScenario.value.localPort
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    setInlineError('welcome.validation.localPortRange')
    return
  }
  pushUser(t('welcome.chat.scenarioUser', { scenario: selectedScenarioTitle.value, port }))
  pushGate('welcome.chat.reviewReady')
  navigateTo('review')
}

async function createFirstTunnel() {
  clearInlineError()
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
    setInlineError(
      error instanceof Error
        ? 'welcome.validation.createTunnelFailedWithMessage'
        : 'welcome.validation.createTunnelFailed',
      error instanceof Error ? { message: error.message } : {},
    )
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
  clearInlineError()
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

function clearInlineError() {
  inlineErrorKey.value = ''
  inlineErrorParams.value = {}
}

function setInlineError(key: string, params: Record<string, unknown> = {}) {
  inlineErrorKey.value = key
  inlineErrorParams.value = params
}

function pushGate(bodyKey: string, params?: Record<string, unknown>, titleKey?: string) {
  conversation.value.push({ id: makeId(), role: 'gate', titleKey, bodyKey, params })
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
  if (tourIndex.value >= tourItems.value.length - 1) {
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
