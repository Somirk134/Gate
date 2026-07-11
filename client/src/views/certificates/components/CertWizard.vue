<template>
  <Teleport to="body">
    <Transition name="cert-wizard">
      <div v-if="visible" class="wizard-backdrop" @keydown.esc="close">
        <div class="wizard-backdrop__scrim" aria-hidden="true" @click="close" />
        <section class="wizard" role="dialog" aria-modal="true" @mousedown.stop>
        <header class="wizard__header">
          <div>
            <p>HTTPS / TLS</p>
            <h2>{{ t('certificate.wizard.title') }}</h2>
          </div>
          <button type="button" class="wizard__close" @click="close">
            <GIcon name="close" :size="16" />
          </button>
        </header>

        <div class="wizard__steps">
          <div
            v-for="(stepLabel, index) in stepLabels"
            :key="index"
            class="wizard__step-item">
            <span
              class="wizard__step-dot"
              :class="{ active: currentStep === index, done: currentStep > index }"
              :title="stepLabel">
              <GIcon v-if="currentStep > index" name="check" :size="12" />
              <template v-else>{{ index + 1 }}</template>
            </span>
            <span
              v-if="index < stepLabels.length - 1"
              class="wizard__step-line"
              :class="{ done: currentStep > index }" />
          </div>
        </div>

        <main class="wizard__body">
          <!-- Step 1: Server -->
          <section v-if="currentStep === 0" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('certificate.wizard.steps.server') }}</strong>
              <p>{{ t('certificate.wizard.serverLabel') }}</p>
            </div>
            <div v-if="!servers.length" class="wizard-empty">
              <GIcon name="servers" :size="24" />
              <span>{{ t('certificate.wizard.noServers') }}</span>
            </div>
            <div v-else class="server-grid">
              <button
                v-for="server in servers"
                :key="server.id"
                type="button"
                class="server-card"
                :class="{ active: form.serverId === server.id }"
                @click="form.serverId = server.id">
                <span class="server-card__icon"><GIcon name="servers" :size="20" /></span>
                <span class="server-card__main">
                  <strong>{{ server.name }}</strong>
                  <small>{{ server.publicIp || server.host }}</small>
                </span>
                <span class="server-card__status" :class="server.status">
                  {{ serverStatusLabel(server.status) }}
                </span>
              </button>
            </div>
          </section>

          <!-- Step 2: Domain -->
          <section v-else-if="currentStep === 1" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('certificate.wizard.steps.domain') }}</strong>
            </div>
            <label class="wizard-field">
              <span>{{ t('certificate.wizard.domainLabel') }}</span>
              <input
                ref="domainInputRef"
                v-model="form.domain"
                autocomplete="off"
                :placeholder="t('certificate.wizard.domainPlaceholder')"
                @keydown.enter.prevent="next" />
            </label>
          </section>

          <!-- Step 3: Email + Challenge Type -->
          <section v-else-if="currentStep === 2" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('certificate.wizard.steps.email') }}</strong>
              <p>{{ t('certificate.wizard.emailHint') }}</p>
            </div>
            <label class="wizard-field">
              <span>{{ t('certificate.wizard.emailLabel') }}</span>
              <input
                v-model.trim="form.email"
                type="email"
                autocomplete="email"
                list="acme-email-options"
                :placeholder="t('certificate.wizard.emailPlaceholder')"
                @keydown.enter.prevent="next" />
              <datalist id="acme-email-options">
                <option v-for="email in emailOptions" :key="email" :value="email" />
              </datalist>
            </label>
            <div v-if="emailOptions.length" class="email-reuse">
              <span class="email-reuse__label">{{ t('certificate.wizard.reuseEmail') }}</span>
              <div class="email-reuse__chips">
                <button
                  v-for="email in emailOptions"
                  :key="`reuse-${email}`"
                  type="button"
                  class="email-reuse__chip"
                  :class="{ active: form.email === email }"
                  @click="applySavedEmail(email)">
                  {{ email }}
                </button>
              </div>
            </div>
            <label class="wizard-field">
              <span>{{ t('certificate.wizard.challengeType') }}</span>
              <div class="challenge-options">
                <button
                  type="button"
                  class="challenge-option"
                  :class="{ active: form.challengeType === 'http01' }"
                  @click="form.challengeType = 'http01'">
                  <span class="challenge-option__header">
                    <GIcon name="globe" :size="18" />
                    <strong>{{ t('certificate.wizard.http01') }}</strong>
                    <GIcon v-if="form.challengeType === 'http01'" name="check-circle" :size="16" class="challenge-option__check" />
                  </span>
                  <p>{{ t('certificate.wizard.http01Desc') }}</p>
                </button>
                <button
                  type="button"
                  class="challenge-option"
                  :class="{ active: form.challengeType === 'dns01' }"
                  @click="form.challengeType = 'dns01'">
                  <span class="challenge-option__header">
                    <GIcon name="network" :size="18" />
                    <strong>{{ t('certificate.wizard.dns01') }}</strong>
                    <GIcon v-if="form.challengeType === 'dns01'" name="check-circle" :size="16" class="challenge-option__check" />
                  </span>
                  <p>{{ t('certificate.wizard.dns01Desc') }}</p>
                </button>
              </div>
            </label>
            <label class="wizard-check">
              <input v-model="form.staging" type="checkbox" />
              <span>{{ t('certificate.wizard.staging') }}</span>
            </label>
            <p class="wizard-check__hint">{{ t('certificate.wizard.stagingHint') }}</p>
          </section>

          <!-- Step 4: DNS Verification / ACME Request -->
          <section v-else-if="currentStep === 3" class="wizard-step">
            <!-- Preparing: creating ACME order -->
            <div v-if="prepareStatus === 'pending'" class="dns-preparing">
              <GIcon name="loader" :size="28" spin />
              <span>{{ t('certificate.wizard.dnsStep.preparing') }}</span>
            </div>

            <template v-else-if="prepareStatus === 'failed'">
              <div class="wizard-result is-failed">
                <GIcon name="alert-circle" :size="36" />
                <h3>{{ t('certificate.wizard.dnsStep.prepareFailed') }}</h3>
                <p class="dns-error">{{ prepareError }}</p>
              </div>
              <GButton variant="secondary" icon="refresh" block @click="prepareAcme">
                {{ t('certificate.refresh') }}
              </GButton>
            </template>

            <!-- DNS-01: show TXT record details -->
            <template v-else-if="prepareStatus === 'success' && form.challengeType === 'dns01'">
              <div class="wizard-copy">
                <strong>{{ t('certificate.wizard.dnsStep.title') }}</strong>
                <p>{{ t('certificate.wizard.dnsStep.desc') }}</p>
              </div>

              <div class="dns-record">
                <div class="dns-record__row">
                  <span class="dns-record__label">{{ t('certificate.wizard.dnsStep.recordType') }}</span>
                  <span class="dns-record__value dns-record__value--tag">TXT</span>
                </div>
                <div class="dns-record__row">
                  <span class="dns-record__label">{{ t('certificate.wizard.dnsStep.host') }}</span>
                  <div class="dns-record__value-group">
                    <code class="dns-record__value">{{ acmeResult?.txtHost }}</code>
                    <button type="button" class="dns-copy-btn" @click="copyText(acmeResult?.txtHost || '')">
                      <GIcon name="copy" :size="14" />
                    </button>
                  </div>
                </div>
                <div class="dns-record__row">
                  <span class="dns-record__label">{{ t('certificate.wizard.dnsStep.value') }}</span>
                  <div class="dns-record__value-group">
                    <code class="dns-record__value dns-record__value--long">{{ acmeResult?.txtValue }}</code>
                    <button type="button" class="dns-copy-btn" @click="copyText(acmeResult?.txtValue || '')">
                      <GIcon name="copy" :size="14" />
                    </button>
                  </div>
                </div>
              </div>

              <p class="dns-hint">
                <GIcon name="info-circle" :size="14" />
                {{ t('certificate.wizard.dnsStep.propagating') }}
              </p>

              <!-- Verify result: background verification in progress -->
              <div v-if="verifyStatus === 'pending'" class="dns-verifying">
                <GIcon name="loader" :size="24" spin />
                <div class="dns-verifying__text">
                  <span>{{ t('certificate.wizard.dnsStep.verifying') }}</span>
                  <span class="dns-verifying__hint">{{ t('certificate.wizard.dnsStep.backgroundHint') }}</span>
                </div>
              </div>
              <div v-else-if="verifyStatus === 'failed'" class="dns-verify-error">
                <GIcon name="alert-circle" :size="18" />
                <span>{{ verifyError }}</span>
              </div>
            </template>

            <!-- HTTP-01: auto verification -->
            <template v-else-if="prepareStatus === 'success' && form.challengeType === 'http01'">
              <div class="wizard-copy">
                <strong>{{ t('certificate.wizard.dnsStep.http01Title') }}</strong>
                <p>{{ t('certificate.wizard.dnsStep.http01Desc') }}</p>
              </div>
              <div v-if="verifyStatus === 'pending'" class="dns-verifying">
                <GIcon name="loader" :size="24" spin />
                <div class="dns-verifying__text">
                  <span>{{ t('certificate.wizard.dnsStep.verifying') }}</span>
                  <span class="dns-verifying__hint">{{ t('certificate.wizard.dnsStep.backgroundHint') }}</span>
                </div>
              </div>
              <div v-else-if="verifyStatus === 'failed'" class="dns-verify-error">
                <GIcon name="alert-circle" :size="18" />
                <span>{{ verifyError }}</span>
              </div>
            </template>
          </section>

          <!-- Step 5: Result -->
          <section v-else-if="currentStep === 4" class="wizard-step">
            <div v-if="verifyStatus === 'success'" class="wizard-result is-success">
              <GIcon name="check-circle" :size="40" />
              <h3>{{ t('certificate.wizard.dnsStep.verifySuccess') }}</h3>
              <p>{{ form.domain }}</p>
            </div>
            <div v-else-if="verifyStatus === 'failed'" class="wizard-result is-failed">
              <GIcon name="alert-circle" :size="40" />
              <h3>{{ t('certificate.wizard.failed') }}</h3>
              <p>{{ verifyError }}</p>
            </div>
          </section>
        </main>

        <footer class="wizard__footer">
          <GButton variant="ghost" @click="close">
            {{ t('certificate.close') }}
          </GButton>
          <div class="wizard__footer-right">
            <GButton
              v-if="currentStep > 0 && currentStep < 3"
              variant="secondary"
              icon="chevron-left"
              @click="prev">
              {{ t('certificate.wizard.prev') }}
            </GButton>
            <GButton
              v-if="currentStep < 2"
              variant="primary"
              :disabled="!canProceed"
              trailing-icon="chevron-right"
              @click="next">
              {{ t('certificate.wizard.next') }}
            </GButton>
            <GButton
              v-if="currentStep === 2"
              variant="primary"
              :disabled="!canProceed"
              icon="chevron-right"
              @click="goToDnsStep">
              {{ t('certificate.wizard.next') }}
            </GButton>
            <!-- DNS-01: I've added the record → start background verify -->
            <GButton
              v-if="currentStep === 3 && prepareStatus === 'success' && form.challengeType === 'dns01' && (verifyStatus === 'idle' || verifyStatus === 'failed')"
              variant="primary"
              icon="zap"
              @click="startBackgroundVerify">
              {{ t('certificate.wizard.dnsStep.iHaveAdded') }}
            </GButton>
            <!-- HTTP-01: Start background verification -->
            <GButton
              v-if="currentStep === 3 && prepareStatus === 'success' && form.challengeType === 'http01' && verifyStatus === 'idle'"
              variant="primary"
              icon="zap"
              @click="startBackgroundVerify">
              {{ t('certificate.wizard.dnsStep.http01Ready') }}
            </GButton>
            <GButton
              v-if="currentStep === 3 && verifyStatus === 'failed'"
              variant="secondary"
              icon="refresh"
              @click="prepareAcme">
              {{ t('certificate.refresh') }}
            </GButton>
          </div>
        </footer>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { listen } from '@tauri-apps/api/event'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import { translateIfExists } from '@/utils/i18n'
import { useFeedback } from '@/composables/useFeedback'
import { certificateService } from '../service'
import { loadRecentAcmeEmails, mergeAcmeEmailOptions, rememberAcmeEmail } from '../utils/acmeEmail'
import type { AcmePrepareResponse, CertificateWizardForm } from '../types'

const props = defineProps<{
  visible: boolean
  servers: Array<{ id: string; name: string; status: string; publicIp?: string; host?: string }>
  initialForm?: Partial<CertificateWizardForm> | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  submitted: [form: CertificateWizardForm]
  verified: []  // 新增：验证完成事件（无论成功失败，用于刷新列表）
}>()

const { t, te } = useI18n()
const { toast, notify } = useFeedback()

const currentStep = ref(0)
const domainInputRef = ref<HTMLInputElement | null>(null)

const form = ref<CertificateWizardForm>({
  serverId: '',
  domain: '',
  email: '',
  challengeType: 'http01',
  staging: false,
})

const prepareStatus = ref<'idle' | 'pending' | 'success' | 'failed'>('idle')
const prepareError = ref('')
const verifyStatus = ref<'idle' | 'pending' | 'success' | 'failed'>('idle')
const verifyError = ref('')
const acmeResult = ref<AcmePrepareResponse | null>(null)
const savedAcmeEmail = ref('')
const recentAcmeEmails = ref<string[]>([])

const emailOptions = computed(() =>
  mergeAcmeEmailOptions(savedAcmeEmail.value, recentAcmeEmails.value),
)

// Tauri 事件监听器引用（用于清理）
let unlistenFn: (() => void) | null = null

// ── 监听后台 ACME 验证结果事件 ──
async function setupAcmeEventListener() {
  // 先清理旧的监听器
  if (unlistenFn) {
    unlistenFn()
    unlistenFn = null
  }

  unlistenFn = await listen<{ success: boolean; data?: any; error?: string }>(
    'acme-verify:result',
    (event) => {
      const payload = event.payload
      if (payload.success) {
        // 验证成功
        verifyStatus.value = 'success'
        verifyError.value = ''
        toast.success(`${form.value.domain} — ${t('certificate.wizard.dnsStep.verifySuccess')}`)
        emit('submitted', form.value)
        emit('verified')
        currentStep.value = 4
      } else {
        // 验证失败
        verifyStatus.value = 'failed'
        const errMsg = payload.error || t('certificate.wizard.failed')
        verifyError.value = errMsg
        notify.error(t('certificate.wizard.failed'), errMsg, 15000)
        emit('verified')  // 即使失败也通知父组件刷新列表
      }
    },
  )
}

// 组件卸载时清理事件监听
onUnmounted(() => {
  if (unlistenFn) {
    unlistenFn()
    unlistenFn = null
  }
})

const stepLabels = computed(() => [
  t('certificate.wizard.steps.server'),
  t('certificate.wizard.steps.domain'),
  t('certificate.wizard.steps.email'),
  t('certificate.wizard.steps.verify'),
  t('certificate.wizard.steps.request'),
])

const canProceed = computed(() => {
  if (currentStep.value === 0) return !!form.value.serverId
  if (currentStep.value === 1) return !!form.value.domain
  if (currentStep.value === 2) return !!form.value.email
  return true
})

watch(
  () => props.visible,
  async (val) => {
    if (val) {
      currentStep.value = 0
      const preset = props.initialForm ?? {}
      await loadAcmeEmailDefaults()
      form.value = {
        serverId: preset.serverId ?? '',
        domain: preset.domain ?? '',
        email: preset.email?.trim() || savedAcmeEmail.value || recentAcmeEmails.value[0] || '',
        challengeType: preset.challengeType ?? 'http01',
        staging: preset.staging ?? false,
      }
      prepareStatus.value = 'idle'
      prepareError.value = ''
      verifyStatus.value = 'idle'
      verifyError.value = ''
      acmeResult.value = null
      // 设置事件监听器（用于接收后台验证结果）
      await setupAcmeEventListener()
    } else {
      // 关闭时清理监听器
      if (unlistenFn) {
        unlistenFn()
        unlistenFn = null
      }
    }
  },
)

watch(currentStep, async (step) => {
  if (!props.visible || step !== 1) return
  await nextTick()
  domainInputRef.value?.focus()
})

function close() {
  emit('update:visible', false)
}

function next() {
  if (currentStep.value === 1) {
    form.value.domain = form.value.domain.trim()
  }
  if (currentStep.value === 2) {
    form.value.email = form.value.email.trim()
    void persistAcmeEmail(form.value.email)
  }
  if (!canProceed.value) return
  currentStep.value = Math.min(currentStep.value + 1, stepLabels.value.length - 1)
}

async function loadAcmeEmailDefaults() {
  recentAcmeEmails.value = loadRecentAcmeEmails()
  savedAcmeEmail.value = ''
  try {
    const response = await certificateService.acmeConfigGet()
    savedAcmeEmail.value = response.config.email.trim()
  } catch {
    savedAcmeEmail.value = ''
  }
}

function applySavedEmail(email: string) {
  form.value.email = email.trim()
}

async function persistAcmeEmail(email: string) {
  const trimmed = email.trim()
  if (!trimmed) return

  rememberAcmeEmail(trimmed)
  recentAcmeEmails.value = loadRecentAcmeEmails()
  savedAcmeEmail.value = trimmed

  try {
    const response = await certificateService.acmeConfigGet()
    await certificateService.acmeConfigSave({
      ...response.config,
      email: trimmed,
    })
  } catch {
    // 本地最近邮箱已保存，配置同步失败不阻断申请流程
  }
}

function prev() {
  currentStep.value = Math.max(currentStep.value - 1, 0)
}

async function goToDnsStep() {
  if (!canProceed.value) return
  form.value.email = form.value.email.trim()
  await persistAcmeEmail(form.value.email)
  currentStep.value = 3
  await prepareAcme()
}

async function prepareAcme() {
  prepareStatus.value = 'pending'
  prepareError.value = ''
  verifyStatus.value = 'idle'
  verifyError.value = ''
  acmeResult.value = null

  try {
    acmeResult.value = await certificateService.acmePrepare({
      domain: form.value.domain,
      email: form.value.email,
      challengeType: form.value.challengeType,
      staging: form.value.staging,
    })
    await persistAcmeEmail(form.value.email)
    prepareStatus.value = 'success'

    // HTTP-01 也使用后台模式（用户可关闭窗口）
    if (form.value.challengeType === 'http01') {
      await startBackgroundVerify()
    }
  } catch (err: unknown) {
    prepareStatus.value = 'failed'
    const msg = err instanceof Error ? err.message : String(err)
    try {
      const parsed = JSON.parse(msg)
      prepareError.value = parsed.details || parsed.message || msg
    } catch {
      prepareError.value = msg
    }
  }
}

/** 启动后台验证（非阻塞，用户可关闭窗口） */
async function startBackgroundVerify() {
  verifyStatus.value = 'pending'
  verifyError.value = ''
  try {
    const result = await certificateService.startAcmeVerify()
    if (result.started) {
      // 后台任务已启动，UI 进入"验证中"状态
      // 结果将通过 Tauri 事件异步返回
      toast.info(`${t('certificate.wizard.dnsStep.backgroundVerifying')} ${form.value.domain}`)
    } else {
      // 启动失败
      verifyStatus.value = 'failed'
      verifyError.value = result.message || 'Failed to start verification'
    }
  } catch (err: unknown) {
    verifyStatus.value = 'failed'
    const msg = err instanceof Error ? err.message : String(err)
    try {
      const parsed = JSON.parse(msg)
      verifyError.value = parsed.details || parsed.hint || parsed.message || msg
    } catch {
      verifyError.value = msg
    }
    notify.error(t('certificate.wizard.failed'), verifyError.value, 10000)
  }
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast.success(t('certificate.wizard.dnsStep.copied'))
  } catch {
    // ignore
  }
}

function serverStatusLabel(status: string) {
  const key = `server.statusLabels.${status}`
  return translateIfExists(t, te, key, status)
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
}

.wizard-backdrop__scrim {
  position: absolute;
  inset: 0;
  background: var(--color-overlay);
  backdrop-filter: blur(12px);
}

.wizard {
  position: relative;
  z-index: 1;
  width: min(720px, calc(100vw - 48px));
  max-height: min(680px, calc(100vh - 48px));
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr) auto;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.wizard__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: var(--space-5) var(--space-5) var(--space-3);
}

.wizard__header p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  text-transform: uppercase;
  font-weight: var(--weight-semibold);
}

.wizard__header h2 {
  margin-top: 2px;
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
}

.wizard__close {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
}

.wizard__close:hover {
  color: var(--text-primary);
  border-color: var(--border-default);
}

/* ── 步骤指示器 ── */
.wizard__steps {
  display: flex;
  align-items: center;
  padding: 0 var(--space-5) var(--space-3);
}

.wizard__step-item {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
}

.wizard__step-item:last-child {
  flex: 0 0 auto;
}

.wizard__step-dot {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  flex-shrink: 0;
}

.wizard__step-dot.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-muted);
}

.wizard__step-dot.done {
  border-color: var(--color-success);
  background: var(--color-success-muted);
  color: var(--color-success);
}

.wizard__step-line {
  flex: 1;
  min-width: 12px;
  height: 2px;
  margin: 0 var(--space-2);
  border-radius: var(--radius-full);
  background: var(--border-subtle);
  transition: background var(--duration-fast) var(--ease-out);
}

.wizard__step-line.done {
  background: var(--color-success);
}

/* ── Body ── */
.wizard__body {
  overflow-y: auto;
  padding: 0 var(--space-5);
}

.wizard-step {
  display: grid;
  gap: var(--space-4);
  padding: var(--space-2) 0 var(--space-4);
}

.wizard-copy strong {
  display: block;
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
}

.wizard-copy p {
  margin-top: var(--space-1);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.wizard-empty {
  min-height: 120px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-2);
  border: 1px dashed var(--border-default);
  border-radius: var(--radius-md);
  color: var(--text-tertiary);
}

.server-grid {
  display: grid;
  gap: var(--space-2);
}

.server-card {
  width: 100%;
  display: grid;
  grid-template-columns: 36px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
}

.server-card:hover {
  border-color: var(--border-default);
  background: var(--bg-surface-hover);
}

.server-card.active {
  border-color: var(--color-primary);
  box-shadow: inset 2px 0 0 var(--color-primary);
}

.server-card__icon {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.server-card__main strong,
.server-card__main small {
  display: block;
}

.server-card__main small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-family: var(--font-mono);
}

.server-card__status {
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.server-card__status.connected {
  color: var(--color-success);
  background: var(--color-success-muted);
}

.server-card__status.disconnected {
  color: var(--text-tertiary);
  background: var(--bg-surface-hover);
}

/* ── Form fields ── */
.wizard-field {
  display: grid;
  gap: var(--space-2);
}

.wizard-field > span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.wizard-field input {
  height: 38px;
  padding: 0 var(--space-3);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.wizard-field input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
  outline: 0;
}

.email-reuse {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.email-reuse__label {
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.email-reuse__chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.email-reuse__chip {
  max-width: 100%;
  padding: 6px 12px;
  border-radius: var(--radius-full);
  border: 1px solid var(--border-subtle);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  transition: border-color var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out);
}

.email-reuse__chip:hover,
.email-reuse__chip.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-muted);
}

/* ── Challenge options ── */
.challenge-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-3);
}

.challenge-option {
  display: grid;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
  text-align: left;
  transition: border-color var(--duration-fast) var(--ease-out), background var(--duration-fast) var(--ease-out);
}

.challenge-option:hover {
  border-color: var(--color-primary);
  background: var(--bg-surface-hover);
}

.challenge-option.active {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
}

.challenge-option__header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--text-primary);
}

.challenge-option__header strong {
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
}

.challenge-option.active .challenge-option__header {
  color: var(--color-primary);
}

.challenge-option__check {
  margin-left: auto;
  color: var(--color-primary);
}

.challenge-option p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: 1.5;
}

.wizard-check {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  cursor: pointer;
}

.wizard-check input {
  width: 16px;
  height: 16px;
  accent-color: var(--color-primary);
}

.wizard-check__hint {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  margin-left: 24px;
}

/* ── DNS record details ── */
.dns-preparing {
  min-height: 160px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  color: var(--color-primary);
}

.dns-record {
  display: grid;
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.dns-record__row {
  display: grid;
  grid-template-columns: 100px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-3);
}

.dns-record__label {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.dns-record__value-group {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
}

.dns-record__value {
  flex: 1;
  min-width: 0;
  padding: var(--space-2) var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  overflow-wrap: anywhere;
}

.dns-record__value--tag {
  width: fit-content;
  padding: 2px 10px;
  border-radius: var(--radius-full);
  background: var(--color-primary-muted);
  color: var(--color-primary);
  font-weight: var(--weight-semibold);
  border: 0;
}

.dns-record__value--long {
  word-break: break-all;
}

.dns-copy-btn {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  color: var(--text-tertiary);
  cursor: pointer;
}

.dns-copy-btn:hover {
  color: var(--color-primary);
  border-color: var(--color-primary);
}

.dns-hint {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.dns-verifying {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  color: var(--color-primary);
  font-size: var(--text-sm);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  border: 1px solid var(--color-primary-alpha-200, rgba(59,130,246,.15));
}

.dns-verifying__text {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.dns-verifying__hint {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  opacity: .8;
}

.dns-verify-error {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  color: var(--color-error);
  font-size: var(--text-sm);
  padding: var(--space-3);
  border: 1px solid var(--color-error-muted);
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
}

.dns-error {
  color: var(--color-error);
  font-size: var(--text-sm);
  text-align: center;
  word-break: break-word;
  white-space: pre-line;
  line-height: 1.6;
}

/* ── Result ── */
.wizard-result {
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  min-height: 200px;
  text-align: center;
}

.wizard-result.is-success { color: var(--color-success); }
.wizard-result.is-failed { color: var(--color-error); }

.wizard-result h3 {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
}

.wizard-result p {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

/* ── Footer ── */
.wizard__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-5) var(--space-5);
}

.wizard__footer-right {
  display: flex;
  gap: var(--space-2);
}

.cert-wizard-enter-active,
.cert-wizard-leave-active {
  transition: opacity var(--duration-base) var(--ease-out);
}

.cert-wizard-enter-from,
.cert-wizard-leave-to {
  opacity: 0;
}

@media (max-width: 580px) {
  .challenge-options {
    grid-template-columns: 1fr;
  }

  .dns-record__row {
    grid-template-columns: 1fr;
  }
}
</style>
