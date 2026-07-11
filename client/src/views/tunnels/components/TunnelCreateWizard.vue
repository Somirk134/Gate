<template>
  <Transition name="wizard">
    <div v-if="visible" class="wizard-backdrop" @keydown.esc="close">
      <section class="wizard" role="dialog" aria-modal="true" tabindex="-1">
        <header class="wizard__header">
          <div>
            <p>{{ t('tunnel.wizard.flow.brandTag') }}</p>
            <h2>{{ stepTitle }}</h2>
          </div>
          <button type="button" class="wizard__close" @click="close">
            <GIcon name="close" :size="16" />
          </button>
        </header>

        <div class="wizard__steps">
          <span
            v-for="item in steps"
            :key="item.index"
            :class="{ active: step === item.index, done: step > item.index }">
            {{ item.index }}
          </span>
        </div>

        <main class="wizard__body">
          <section v-if="step === 1" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.flow.step1Title') }}</strong>
              <p>{{ t('tunnel.wizard.flow.step1Desc') }}</p>
            </div>
            <div v-if="!servers.length" class="empty-state">
              <GIcon name="servers" :size="24" />
              <span>{{ t('tunnel.wizard.flow.noServers') }}</span>
            </div>
            <div v-else class="server-grid">
              <button
                v-for="server in servers"
                :key="server.id"
                type="button"
                class="server-card"
                :class="{ active: form.serverId === server.id, offline: !isServerConnected(server) }"
                @click="selectServer(server)">
                <span class="server-card__icon"><GIcon name="servers" :size="20" /></span>
                <span class="server-card__main">
                  <strong>{{ server.name }}</strong>
                  <small>{{ serverAddress(server) }}</small>
                </span>
                <span class="server-card__status" :class="server.status">
                  {{ serverStatusLabel(server.status) }}
                </span>
                <span class="server-card__meta">
                  <span>{{ server.region || t('tunnel.wizard.flow.regionUnknown') }}</span>
                  <span>{{ server.ping ? `${server.ping}ms` : t('tunnel.wizard.flow.notTested') }}</span>
                  <span>{{ server.overview.os || t('tunnel.wizard.flow.osUnknown') }}</span>
                </span>
              </button>
            </div>
          </section>

          <section v-else-if="step === 2" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.flow.step2Title') }}</strong>
              <p>{{ t('tunnel.wizard.flow.step2Desc') }}</p>
            </div>
            <div class="service-tools">
              <label>
                <span>{{ t('tunnel.wizard.flow.searchService') }}</span>
                <input
                  v-model.trim="serviceQuery"
                  autocomplete="off"
                  :disabled="discoveryLoading || probingService"
                  :placeholder="t('tunnel.wizard.flow.searchPlaceholder')"
                  @keydown.enter.prevent="searchLocalService" />
              </label>
              <div class="service-tools__actions">
                <GButton
                  variant="secondary"
                  icon="search"
                  :loading="probingService"
                  :disabled="discoveryLoading"
                  @click="searchLocalService">
                  {{ t('tunnel.wizard.flow.manualSearch') }}
                </GButton>
                <GButton
                  variant="ghost"
                  icon="refresh"
                  :loading="discoveryLoading"
                  :disabled="probingService"
                  @click="startDiscoveryScan({ selectFirst: true })">
                  {{ t('tunnel.wizard.flow.rescan') }}
                </GButton>
              </div>
            </div>
            <div class="manual-service">
              <label>
                <span>{{ t('tunnel.wizard.flow.labels.host') }}</span>
                <input
                  v-model.trim="manualService.host"
                  autocomplete="off"
                  :disabled="discoveryLoading || probingService"
                  placeholder="127.0.0.1" />
              </label>
              <label>
                <span>{{ t('tunnel.wizard.flow.labels.port') }}</span>
                <input
                  v-model.number="manualService.port"
                  inputmode="numeric"
                  type="number"
                  :disabled="discoveryLoading || probingService"
                  placeholder="8088" />
              </label>
              <label>
                <span>{{ t('tunnel.wizard.flow.serviceLabel') }}</span>
                <input
                  v-model.trim="manualService.name"
                  autocomplete="off"
                  :disabled="discoveryLoading || probingService"
                  placeholder="Spring Boot" />
              </label>
              <GButton
                variant="secondary"
                icon="plus"
                :loading="probingService"
                :disabled="discoveryLoading"
                @click="addManualService">
                {{ t('tunnel.wizard.flow.manualAdd') }}
              </GButton>
            </div>
            <p v-if="serviceSearchMessage" class="wizard-alert" :class="{ error: serviceSearchError }">
              {{ serviceSearchMessage }}
            </p>
            <p v-if="discoverySummary" class="wizard-alert">{{ discoverySummary }}</p>
            <p v-if="discoveryError" class="wizard-alert error">{{ discoveryError }}</p>
            <div v-if="discoveryLoading || scanLogs.length" class="discovery-log" aria-live="polite">
              <div class="discovery-log__header">
                <span v-if="discoveryLoading" class="discovery-state__spinner" />
                <div>
                  <strong>{{ t('tunnel.wizard.flow.scanningTitle') }}</strong>
                  <small v-if="scanProgressLabel">{{ scanProgressLabel }}</small>
                </div>
              </div>
              <div ref="scanLogRef" class="discovery-log__body">
                <div
                  v-for="(entry, index) in scanLogs"
                  :key="`${entry.port}-${index}`"
                  class="discovery-log__line"
                  :class="{ found: entry.found, idle: !entry.port }">
                  <span v-if="entry.port" class="discovery-log__port">:{{ entry.port }}</span>
                  <span>{{ entry.message }}</span>
                </div>
              </div>
            </div>
            <div class="service-list">
              <button
                v-for="service in filteredLocalServices"
                :key="`${service.host}:${service.port}:${service.pid ?? 'none'}`"
                type="button"
                :class="{
                  active: selectedServiceKey === serviceKey(service),
                  unreachable: service.reachable === false,
                }"
                @click="selectService(service)">
                <GIcon :name="serviceIcon(service)" :size="18" />
                <div>
                  <strong>{{ service.label }}</strong>
                  <small>{{ serviceMeta(service) }}</small>
                </div>
                <div class="service-list__aside">
                  <span class="service-reachability" :class="reachabilityTone(service)">
                    {{ reachabilityLabel(service) }}
                  </span>
                  <code>{{ service.host }}:{{ service.port }}</code>
                </div>
              </button>
            </div>
            <div v-if="!discoveryLoading && !filteredLocalServices.length" class="empty-state">
              <GIcon name="search" :size="22" />
              <span>{{ t('tunnel.wizard.flow.noMatchingService') }}</span>
            </div>
          </section>

          <section v-else-if="step === 3" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.flow.step3Title') }}</strong>
              <p>{{ t('tunnel.wizard.flow.step3Desc') }}</p>
            </div>
            <div class="protocol-grid">
              <button
                v-for="protocol in protocols"
                :key="protocol"
                type="button"
                :class="{ active: form.protocol === protocol }"
                @click="form.protocol = protocol">
                <GIcon :name="protocol === 'tcp' ? 'router' : protocol === 'https' ? 'shield-check' : 'globe'" :size="20" />
                <strong>{{ protocol.toUpperCase() }}</strong>
                <small v-if="protocol === recommendedProtocol">{{ t('tunnel.wizard.flow.recommended') }}</small>
              </button>
            </div>
          </section>

          <section v-else-if="step === 4 && !isHttpLike" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.flow.step4Title') }}</strong>
              <p>{{ t('tunnel.wizard.flow.step4DescTcp') }}</p>
            </div>
            <label>
              <span>{{ t('tunnel.wizard.flow.manualPort') }}</span>
              <input
                v-model.number="form.remotePort"
                inputmode="numeric"
                type="number"
                placeholder="18080" />
            </label>
            <p v-if="portCheckMessage" class="wizard-alert" :class="{ error: portConflict }">
              {{ portCheckMessage }}
            </p>
          </section>

          <section v-else-if="step === 4 && isHttpLike" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.flow.step4TitleDomain') }}</strong>
              <p>{{ t('tunnel.wizard.flow.step4DescDomain') }}</p>
            </div>
            <p class="wizard-alert">{{ t('tunnel.wizard.flow.subdomainModeHint') }}</p>
            <label>
              <span>{{ t('tunnel.wizard.flow.labels.domain') }}</span>
              <div
                v-if="form.protocol === 'https'"
                ref="domainPickerRef"
                class="domain-picker"
                :class="{ 'domain-picker--open': domainPickerOpen }">
                <button
                  type="button"
                  class="domain-picker__trigger"
                  :disabled="certificatesLoading || !availableDomainOptions.length"
                  @click="toggleDomainPicker">
                  <span>{{ selectedDomainLabel }}</span>
                  <GIcon name="chevron-down" :size="14" class="domain-picker__chevron" />
                </button>
                <div v-if="domainPickerOpen" class="domain-picker__menu" role="listbox">
                  <button
                    v-for="option in availableDomainOptions"
                    :key="option.value"
                    type="button"
                    class="domain-picker__option"
                    :class="{ active: selectedCertDomain === option.value }"
                    role="option"
                    :aria-selected="selectedCertDomain === option.value"
                    @click="selectCertDomain(option.value)">
                    {{ option.label }}
                  </button>
                </div>
              </div>
              <input
                v-else
                v-model.trim="form.host"
                autocomplete="off"
                list="tunnel-domain-suggestions"
                placeholder="dev.example.com"
                @blur="applySubdomainDefaults" />
              <datalist v-if="form.protocol === 'http'" id="tunnel-domain-suggestions">
                <option
                  v-for="option in suggestedSubdomainOptions"
                  :key="`suggest-${option}`"
                  :value="option" />
              </datalist>
            </label>
            <label v-if="form.protocol === 'https'">
              <span>{{ t('tunnel.wizard.flow.subdomainLabel') }}</span>
              <div class="subdomain-row">
                <input
                  v-model.trim="subdomainPrefix"
                  autocomplete="off"
                  placeholder="dev"
                  @input="syncHttpsHostFromParts" />
                <span class="subdomain-row__dot">.</span>
                <input
                  :value="selectedCertDomain || t('tunnel.wizard.flow.subdomainBasePlaceholder')"
                  readonly
                  class="subdomain-row__base" />
              </div>
              <div class="prefix-presets">
                <button
                  v-for="prefix in SUBDOMAIN_PREFIX_PRESETS"
                  :key="prefix"
                  type="button"
                  :class="{ active: subdomainPrefix === prefix }"
                  @click="selectSubdomainPrefix(prefix)">
                  {{ prefix }}
                </button>
              </div>
              <small class="field-hint">{{ t('tunnel.wizard.flow.subdomainInputHint') }}</small>
            </label>
            <label>
              <span>{{ t('tunnel.wizard.flow.standardPortLabel') }}</span>
              <input :value="form.remotePort ?? standardPublicPort(form.protocol)" readonly />
            </label>
            <p
              v-if="form.protocol === 'https' && !certificatesLoading && !availableDomainOptions.length"
              class="wizard-alert error">
              {{ t('tunnel.wizard.flow.noCertificateDomains') }}
            </p>
            <p v-if="previewAccessUrl" class="wizard-alert access-preview">
              {{ t('tunnel.wizard.flow.accessPreview', { url: previewAccessUrl }) }}
            </p>
            <p v-if="form.host && !certificateCoversSelectedHost" class="wizard-alert error">
              {{ t('tunnel.wizard.flow.subdomainCertMissing', { host: form.host }) }}
            </p>
            <div v-if="form.protocol === 'https'" class="certificate-state">
              <GIcon :name="hasCertificate ? 'check-circle' : 'alert-circle'" :size="18" />
              <span>{{ certificateMessage }}</span>
            </div>
          </section>

          <section v-else-if="step === 5 && isHttpLike" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.flow.step5TitleDns') }}</strong>
              <p>{{ t('tunnel.wizard.flow.dnsGuideDesc') }}</p>
            </div>
            <div v-if="dnsRecordGuide" class="dns-guide">
              <div class="dns-guide__record">
                <div><span>{{ t('tunnel.wizard.flow.dnsType') }}</span><strong>{{ dnsRecordGuide.type }}</strong></div>
                <div><span>{{ t('tunnel.wizard.flow.dnsName') }}</span><strong>{{ dnsRecordGuide.name }}</strong></div>
                <div><span>{{ t('tunnel.wizard.flow.dnsValue') }}</span><strong>{{ dnsRecordGuide.value }}</strong></div>
                <div><span>{{ t('tunnel.wizard.flow.dnsHost') }}</span><strong>{{ dnsRecordGuide.host }}</strong></div>
              </div>
              <p class="field-hint">{{ t('tunnel.wizard.flow.dnsGuideNote') }}</p>
            </div>
            <p v-else class="wizard-alert error">{{ t('tunnel.wizard.flow.dnsGuideMissing') }}</p>
          </section>

          <section v-else-if="step === 5" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.flow.step5TitleTcp') }}</strong>
              <p>{{ t('tunnel.wizard.flow.step5DescTcp') }}</p>
            </div>
            <p class="wizard-alert">{{ t('tunnel.wizard.flow.tcpNoDomainCert') }}</p>
          </section>

          <section v-else class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.confirmCreate') }}</strong>
              <p>{{ t('tunnel.wizard.flow.step6Desc') }}</p>
            </div>
            <div class="confirm-list">
              <div><span>{{ t('tunnel.wizard.flow.confirmList.server') }}</span><strong>{{ form.serverName }}</strong></div>
              <div><span>{{ t('tunnel.wizard.flow.confirmList.localService') }}</span><strong>{{ form.localHost }}:{{ form.localPort }}</strong></div>
              <div><span>{{ t('tunnel.wizard.flow.confirmList.protocol') }}</span><strong>{{ form.protocol.toUpperCase() }}</strong></div>
              <div><span>{{ t('tunnel.wizard.flow.confirmList.remotePort') }}</span><strong>{{ form.remotePort ?? '-' }}</strong></div>
              <div v-if="isHttpLike"><span>{{ t('tunnel.wizard.flow.confirmList.domain') }}</span><strong>{{ previewAccessUrl || form.host || '-' }}</strong></div>
            </div>
            <div v-if="diagnosis" class="diagnosis-list">
              <article
                v-for="finding in diagnosis.findings"
                :key="finding.id"
                :class="finding.status">
                <strong>{{ finding.reason }}</strong>
                <small>{{ finding.solution }}</small>
              </article>
            </div>
          </section>
        </main>

        <footer class="wizard__footer">
          <GButton v-if="step > 1" variant="ghost" @click="step -= 1">{{ t('tunnel.wizard.previous') }}</GButton>
          <span class="wizard__error">{{ errorMessage }}</span>
          <GButton
            v-if="step < 6"
            variant="primary"
            trailing-icon="arrow-right"
            :loading="stepLoading"
            @click="next">
            {{ t('tunnel.wizard.next') }}
          </GButton>
          <GButton v-else variant="secondary" icon="activity" @click="runDiagnosis">
            {{ t('tunnel.wizard.flow.healthCheck') }}
          </GButton>
          <GButton v-if="step === 6" variant="primary" icon="plus" :loading="creating" @click="createTunnel">
            {{ t('tunnel.wizard.finish') }}
          </GButton>
        </footer>
      </section>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, nextTick, onUnmounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import { certificateService } from '@views/certificates/service'
import type { CertificateSummary } from '@views/certificates/types'
import {
  discoveryService,
  type DiscoveryScanComplete,
  type DiscoveryScanProgress,
  type LocalServiceRecord,
  type TunnelDiagnosis,
} from '@/services'
import type { Disposable } from '@/utils/disposable'
import type { Server, ServerStatus } from '@views/servers'
import type { TunnelFormData, TunnelProtocol } from '../types'
import {
  SUBDOMAIN_PREFIX_PRESETS,
  applySubdomainTunnelDefaults,
  buildDnsRecordGuide,
  buildSubdomainHost,
  buildTunnelPublicUrl,
  certificateCoversHost,
  listCertificateBaseDomains,
  splitSubdomainHost,
  standardPublicPort,
  suggestSubdomainPrefix,
  suggestedSubdomainHosts,
} from '../utils/domainAccess'
import { isValidPort } from '../utils'

const props = defineProps<{
  visible: boolean
  projects: Array<{ id: string; name: string }>
  servers: Server[]
  defaultProjectId?: string
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  submit: [form: TunnelFormData]
}>()

const protocols: Array<'tcp' | 'http' | 'https'> = ['tcp', 'http', 'https']
const { t } = useI18n()

interface ScanLogEntry {
  port: number
  found: boolean
  message: string
}

const form = reactive<TunnelFormData>({
  name: '',
  protocol: 'tcp',
  localHost: '127.0.0.1',
  localPort: null,
  remotePort: null,
  host: '',
  path: '/',
  projectId: '',
  serverId: '',
  serverName: '',
  autoStart: true,
  remark: '',
  tags: [],
})

const isHttpLike = computed(() => form.protocol === 'http' || form.protocol === 'https')
const steps = computed(() => [
  { index: 1, title: t('tunnel.wizard.flow.step1Title') },
  { index: 2, title: t('tunnel.wizard.flow.step2Title') },
  { index: 3, title: t('tunnel.wizard.flow.step3Title') },
  {
    index: 4,
    title: isHttpLike.value
      ? t('tunnel.wizard.flow.step4TitleDomain')
      : t('tunnel.wizard.flow.step4Title'),
  },
  {
    index: 5,
    title: isHttpLike.value
      ? t('tunnel.wizard.flow.step5TitleDns')
      : t('tunnel.wizard.flow.step5TitleTcp'),
  },
  { index: 6, title: t('tunnel.wizard.flow.step6Title') },
])

const step = ref(1)
const stepLoading = ref(false)
const creating = ref(false)
const discoveryLoading = ref(false)
const probingService = ref(false)
const errorMessage = ref('')
const discoveryError = ref('')
const discoverySummary = ref('')
const scanLogs = ref<ScanLogEntry[]>([])
const scanSessionId = ref('')
const scanProgress = ref({ index: 0, total: 0 })
const scanLogRef = ref<HTMLElement | null>(null)
const selectFirstAfterScan = ref(false)
const localServices = ref<LocalServiceRecord[]>([])
const serviceQuery = ref('')
const serviceSearchMessage = ref('')
const serviceSearchError = ref(false)
const selectedService = ref<LocalServiceRecord | null>(null)
const portConflict = ref(false)
const portCheckMessage = ref('')
const certificateDomains = ref<string[]>([])
const certificates = ref<CertificateSummary[]>([])
const certificatesLoading = ref(false)
const selectedCertDomain = ref('')
const domainPickerOpen = ref(false)
const domainPickerRef = ref<HTMLElement | null>(null)
const subdomainPrefix = ref('dev')
const diagnosis = ref<TunnelDiagnosis | null>(null)
const manualService = reactive<{
  host: string
  port: number | null
  name: string
}>({
  host: '127.0.0.1',
  port: null,
  name: '',
})

const stepTitle = computed(() => steps.value.find((item) => item.index === step.value)?.title ?? t('tunnel.wizard.flow.step6Title'))
const recommendedProtocol = computed(() => selectedService.value?.recommendedProtocol ?? 'tcp')
const selectedServer = computed(
  () => props.servers.find((server) => server.id === form.serverId) ?? null,
)
const selectedServiceKey = computed(() => (selectedService.value ? serviceKey(selectedService.value) : ''))
const filteredLocalServices = computed(() => {
  const query = serviceQuery.value.trim().toLowerCase()
  if (!query) return localServices.value
  return localServices.value.filter((service) =>
    [
      service.label,
      service.technology,
      service.process,
      service.executable,
      service.host,
      service.bindAddress,
      String(service.port),
    ]
      .filter(Boolean)
      .join(' ')
      .toLowerCase()
      .includes(query),
  )
})
const normalizedHost = computed(() => form.host?.trim().toLowerCase() ?? '')
const suggestedSubdomainOptions = computed(() => suggestedSubdomainHosts(certificates.value))
const certificateCoversSelectedHost = computed(() =>
  normalizedHost.value ? certificateCoversHost(normalizedHost.value, certificates.value) : true,
)
const hasCertificate = computed(() => certificateCoversSelectedHost.value)
const previewAccessUrl = computed(() =>
  buildTunnelPublicUrl({
    protocol: form.protocol,
    host: form.host,
    path: form.path,
    remotePort: form.remotePort,
  }),
)
const serverPublicIp = computed(
  () => selectedServer.value?.publicIp || selectedServer.value?.settings.host || '',
)
const dnsRecordGuide = computed(() =>
  buildDnsRecordGuide(form.host, serverPublicIp.value),
)
const availableDomainOptions = computed(() => {
  const seen = new Set<string>()
  const options: Array<{ value: string; label: string }> = []

  for (const cert of certificates.value) {
    if (cert.status !== 'active' && cert.status !== 'expiringSoon') continue
    for (const domain of [cert.domain, ...cert.san]) {
      const normalized = domain.trim().toLowerCase()
      if (!normalized || seen.has(normalized)) continue
      seen.add(normalized)
      const suffix =
        cert.status === 'expiringSoon'
          ? t('tunnel.wizard.flow.domainExpiringSoon', { days: cert.daysRemaining })
          : cert.domain.toLowerCase() === normalized
            ? ''
            : t('tunnel.wizard.flow.domainSanSuffix')
      options.push({
        value: normalized,
        label: suffix ? `${normalized} (${suffix})` : normalized,
      })
    }
  }

  return options.sort((left, right) => left.value.localeCompare(right.value))
})
const selectedDomainLabel = computed(() => {
  if (certificatesLoading.value) return t('common.loading')
  if (!availableDomainOptions.value.length) return t('tunnel.wizard.flow.noCertificateDomains')
  const option = availableDomainOptions.value.find((item) => item.value === selectedCertDomain.value)
  return option?.label ?? selectedCertDomain.value ?? t('tunnel.wizard.flow.domainSelectPlaceholder')
})
const certificateMessage = computed(() => {
  if (!normalizedHost.value) return t('tunnel.wizard.flow.certPromptHttps')
  return hasCertificate.value ? t('tunnel.wizard.flow.certFound') : t('tunnel.wizard.flow.certNotFound')
})
const scanProgressLabel = computed(() => {
  if (!scanProgress.value.total) return ''
  return t('tunnel.wizard.flow.scanProgress', {
    current: scanProgress.value.index,
    total: scanProgress.value.total,
  })
})

let scanListeners: Disposable[] = []

function teardownScanListeners() {
  scanListeners.forEach((listener) => listener.dispose())
  scanListeners = []
}

async function setupScanListeners() {
  teardownScanListeners()
  scanListeners = await Promise.all([
    discoveryService.onScanProgress(handleScanProgress),
    discoveryService.onScanComplete(handleScanComplete),
  ])
}

function handleScanProgress(payload: DiscoveryScanProgress) {
  if (payload.scanId !== scanSessionId.value) return
  scanProgress.value = { index: payload.index, total: payload.total }
  scanLogs.value.push({
    port: payload.port,
    found: payload.found,
    message: payload.found
      ? t('tunnel.wizard.flow.scanLogFound', { port: payload.port })
      : t('tunnel.wizard.flow.scanLogNotFound', { port: payload.port }),
  })
  void nextTick(() => {
    scanLogRef.value?.scrollTo({ top: scanLogRef.value.scrollHeight, behavior: 'smooth' })
  })
}

function handleScanComplete(payload: DiscoveryScanComplete) {
  if (payload.scanId !== scanSessionId.value) return
  const services = Array.isArray(payload.items) ? payload.items : []
  localServices.value = services
  discoveryLoading.value = false
  discoverySummary.value =
    services.length > 0
      ? t('tunnel.wizard.flow.scanResultCount', { count: services.length })
      : t('tunnel.wizard.flow.scanResultEmpty')
  scanLogs.value.push({
    port: 0,
    found: services.length > 0,
    message: t('tunnel.wizard.flow.scanLogDone', { count: services.length }),
  })
  void nextTick(() => {
    scanLogRef.value?.scrollTo({ top: scanLogRef.value.scrollHeight, behavior: 'smooth' })
  })
  if (selectFirstAfterScan.value) {
    const reachable = services.find((service) => service.reachable !== false)
    if (reachable) {
      selectService(reachable)
    } else if (services[0]) {
      selectService(services[0])
    } else {
      selectedService.value = null
      form.localPort = null
    }
  }
  selectFirstAfterScan.value = false
}

function applyStandardPublicPort() {
  if (!isHttpLike.value || !form.host?.trim()) return
  form.remotePort = standardPublicPort(form.protocol)
}

function syncHttpsHostFromParts() {
  if (!selectedCertDomain.value) {
    form.host = ''
    return
  }
  const defaults = applySubdomainTunnelDefaults(form.protocol, buildSubdomainHost(subdomainPrefix.value, selectedCertDomain.value))
  form.host = defaults.host
  form.path = defaults.path
  form.remotePort = defaults.remotePort
}

function applyCertDomainSelection() {
  if (!selectedCertDomain.value) {
    form.host = ''
    return
  }
  subdomainPrefix.value = suggestSubdomainPrefix(selectedCertDomain.value)
  syncHttpsHostFromParts()
}

function toggleDomainPicker() {
  if (certificatesLoading.value || !availableDomainOptions.value.length) return
  domainPickerOpen.value = !domainPickerOpen.value
}

function selectCertDomain(value: string) {
  selectedCertDomain.value = value
  domainPickerOpen.value = false
  applyCertDomainSelection()
}

function applySubdomainDefaults() {
  const host = form.host.trim().toLowerCase()
  if (!host) return
  const defaults = applySubdomainTunnelDefaults(form.protocol, host)
  form.host = defaults.host
  form.path = defaults.path
  form.remotePort = defaults.remotePort
}

function selectSubdomainPrefix(prefix: string) {
  subdomainPrefix.value = prefix
  syncHttpsHostFromParts()
}

onClickOutside(domainPickerRef, () => {
  domainPickerOpen.value = false
})

watch(step, () => {
  domainPickerOpen.value = false
})

watch(
  () => props.visible,
  async (visible) => {
    if (visible) {
      await setupScanListeners()
      await reset()
      return
    }
    teardownScanListeners()
    scanSessionId.value = ''
    discoveryLoading.value = false
  },
)

watch(step, (value, oldValue) => {
  if (value === 2 && oldValue === 1 && props.visible) {
    void startDiscoveryScan({ selectFirst: true })
  }
  if (value === 4 && isHttpLike.value && props.visible) {
    void prepareDomainStep()
  }
})

onUnmounted(() => {
  teardownScanListeners()
})

watch(
  () => form.remotePort,
  async (port) => {
    if (!port) {
      portConflict.value = false
      portCheckMessage.value = ''
      return
    }
    await checkPort(port)
  },
)

watch(
  () => form.serverId,
  async () => {
    if (!props.visible || !form.serverId || !form.remotePort) return
    await checkPort(form.remotePort)
  },
)

async function reset() {
  step.value = 1
  stepLoading.value = false
  creating.value = false
  discoveryLoading.value = false
  probingService.value = false
  errorMessage.value = ''
  discoveryError.value = ''
  discoverySummary.value = ''
  scanLogs.value = []
  scanSessionId.value = ''
  scanProgress.value = { index: 0, total: 0 }
  selectFirstAfterScan.value = false
  serviceQuery.value = ''
  serviceSearchMessage.value = ''
  serviceSearchError.value = false
  selectedService.value = null
  localServices.value = []
  portCheckMessage.value = ''
  diagnosis.value = null
  form.name = ''
  form.protocol = 'tcp'
  form.localHost = '127.0.0.1'
  form.localPort = null
  form.remotePort = null
  form.host = ''
  form.path = '/'
  selectedCertDomain.value = ''
  subdomainPrefix.value = 'dev'
  form.projectId = props.defaultProjectId || props.projects[0]?.id || ''
  const initialServer = props.servers.find((server) => server.status === 'connected') ?? props.servers[0]
  form.serverId = initialServer?.id ?? ''
  form.serverName = initialServer?.name ?? ''
  form.autoStart = true
  form.remark = ''
  form.tags = []
  manualService.host = '127.0.0.1'
  manualService.port = null
  manualService.name = ''
  await loadCertificates()
}

async function startDiscoveryScan(options: { selectFirst?: boolean } = {}) {
  if (discoveryLoading.value) return
  selectFirstAfterScan.value = Boolean(options.selectFirst)
  discoveryLoading.value = true
  discoveryError.value = ''
  discoverySummary.value = ''
  scanLogs.value = []
  scanProgress.value = { index: 0, total: 0 }
  localServices.value = []

  const scanId = `scan-${Date.now()}`
  scanSessionId.value = scanId

  try {
    const result = await discoveryService.startCommonPortScan(scanId)
    scanProgress.value.total = result.total
    scanLogs.value.push({
      port: 0,
      found: false,
      message: t('tunnel.wizard.flow.scanLogStart', { total: result.total }),
    })
  } catch (error) {
    discoveryLoading.value = false
    discoveryError.value = error instanceof Error ? error.message : t('tunnel.wizard.flow.discoveryFailed')
    scanSessionId.value = ''
  }
}

async function probeService(host: string, port: number): Promise<LocalServiceRecord> {
  probingService.value = true
  serviceSearchMessage.value = ''
  serviceSearchError.value = false
  try {
    return await discoveryService.probeLocalService(host, port)
  } finally {
    probingService.value = false
  }
}

async function ensureSelectedServiceReachable(): Promise<boolean> {
  if (!form.localPort) return false
  const service = await probeService(form.localHost || '127.0.0.1', form.localPort)
  upsertAndSelectService(service)
  if (service.reachable === false) {
    serviceSearchMessage.value = t('tunnel.wizard.flow.localServiceUnreachable', {
      host: service.host,
      port: service.port,
    })
    serviceSearchError.value = true
    return false
  }
  serviceSearchMessage.value = t('tunnel.wizard.flow.localServiceReachable', {
    host: service.host,
    port: service.port,
  })
  serviceSearchError.value = false
  return true
}

async function loadCertificates() {
  certificatesLoading.value = true
  try {
    const payload = await certificateService.list()
    certificates.value = payload.certificates
    certificateDomains.value = payload.certificates
      .filter((item) => item.status === 'active' || item.status === 'expiringSoon')
      .flatMap((item) => [item.domain, ...item.san])
      .map((domain) => domain.toLowerCase())
  } catch {
    certificates.value = []
    certificateDomains.value = []
  } finally {
    certificatesLoading.value = false
  }
}

async function prepareDomainStep() {
  await loadCertificates()
  if (form.protocol !== 'https') {
    applySubdomainDefaults()
    return
  }
  const options = availableDomainOptions.value
  if (!options.length) {
    form.host = ''
    selectedCertDomain.value = ''
    return
  }
  const current = normalizedHost.value
  const baseDomains = listCertificateBaseDomains(certificates.value)
  const matchedOption =
    options.find((option) => option.value === current) ??
    options.find((option) => current.endsWith(`.${option.value}`))
  selectedCertDomain.value = matchedOption?.value ?? options[0].value
  const split = splitSubdomainHost(current, baseDomains)
  if (split && split.baseDomain === selectedCertDomain.value) {
    subdomainPrefix.value = split.prefix || suggestSubdomainPrefix(selectedCertDomain.value)
  } else {
    subdomainPrefix.value = suggestSubdomainPrefix(selectedCertDomain.value)
  }
  syncHttpsHostFromParts()
}

function selectService(service: LocalServiceRecord) {
  selectedService.value = service
  form.localHost = service.host || '127.0.0.1'
  form.localPort = service.port
  form.protocol = service.recommendedProtocol
  form.name = service.technology || service.process || `Service ${service.port}`
  form.path = form.protocol === 'tcp' ? '' : '/'
  form.tags = [service.technology, service.process].filter(Boolean)
  manualService.host = form.localHost
  manualService.port = form.localPort
}

async function checkPort(port: number) {
  const result = await discoveryService.checkRemotePort(port, form.serverId || undefined)
  portConflict.value = !result.available
  portCheckMessage.value = result.available
    ? t('tunnel.wizard.flow.portAvailable', { port })
    : t('tunnel.wizard.flow.portOccupied', { port })
}

async function runDiagnosis() {
  if (!form.localPort) return
  diagnosis.value = await discoveryService.diagnoseTunnel({
    localHost: form.localHost,
    localPort: form.localPort,
    remotePort: form.remotePort ?? 0,
    serverId: form.serverId || undefined,
  })
}

async function next() {
  if (stepLoading.value) return
  if (!(await validateStep())) return

  stepLoading.value = true
  try {
    step.value += 1
    if (step.value === 6) await runDiagnosis()
  } finally {
    stepLoading.value = false
  }
}

async function validateStep() {
  errorMessage.value = ''
  const server = selectedServer.value
  if (step.value === 1) {
    if (!server) {
      errorMessage.value = t('tunnel.wizard.flow.selectCreatedServer')
      return false
    }
    if (server.status !== 'connected') {
      errorMessage.value = t('tunnel.wizard.flow.serverNotConnected')
      return false
    }
  }
  if (step.value === 2) {
    if (!selectedService.value) {
      errorMessage.value = t('tunnel.wizard.flow.selectLocalService')
      return false
    }
    if (!(await ensureSelectedServiceReachable())) {
      errorMessage.value = t('tunnel.wizard.flow.localServiceUnreachable', {
        host: form.localHost,
        port: form.localPort ?? 0,
      })
      return false
    }
  }
  if (step.value === 4 && !isHttpLike.value) {
    if (!isValidPort(form.remotePort)) {
      errorMessage.value = t('tunnel.wizard.flow.remotePortRequired')
    } else if (portConflict.value) {
      errorMessage.value = t('tunnel.wizard.flow.remotePortConflict')
    }
  }
  if (step.value === 4 && isHttpLike.value) {
    if (!normalizedHost.value) {
      errorMessage.value = t('tunnel.wizard.flow.domainRequiredHttps')
    } else if (form.protocol === 'https') {
      if (!availableDomainOptions.value.length) {
        errorMessage.value = t('tunnel.wizard.flow.noCertificateDomains')
      } else if (!certificateCoversSelectedHost.value) {
        errorMessage.value = t('tunnel.wizard.flow.subdomainCertMissing', { host: form.host })
      }
    }
    if (!errorMessage.value) {
      applyStandardPublicPort()
      if (form.remotePort) await checkPort(form.remotePort)
      if (portConflict.value) {
        errorMessage.value = t('tunnel.wizard.flow.remotePortConflict')
      }
    }
  }
  if (step.value === 5 && isHttpLike.value && !dnsRecordGuide.value) {
    errorMessage.value = t('tunnel.wizard.flow.dnsGuideMissing')
  }
  return !errorMessage.value
}

async function createTunnel() {
  if (creating.value) return
  if (!(await validateStep())) return

  creating.value = true
  try {
    if (!(await ensureSelectedServiceReachable())) {
      errorMessage.value = t('tunnel.wizard.flow.createBlockedUnreachable', {
        host: form.localHost,
        port: form.localPort ?? 0,
      })
      return
    }

    const name = form.name.trim() || `Service ${form.localPort}`
    emit('submit', {
      ...form,
      name,
      protocol: form.protocol as TunnelProtocol,
      remotePort: form.remotePort,
      tags: [...new Set(form.tags.filter(Boolean))],
    })
    close()
  } finally {
    creating.value = false
  }
}

function selectServer(server: Server) {
  form.serverId = server.id
  form.serverName = server.name
}

async function searchLocalService() {
  const query = serviceQuery.value.trim()
  serviceSearchMessage.value = ''
  serviceSearchError.value = false
  const port = Number.parseInt(query, 10)
  if (!Number.isInteger(port) || port <= 0 || port > 65535) {
    serviceSearchMessage.value = query
      ? t('tunnel.wizard.flow.searchKeywordHint')
      : t('tunnel.wizard.flow.searchEmpty')
    serviceSearchError.value = Boolean(query)
    return
  }
  manualService.port = port
  const service = await probeService(manualService.host || '127.0.0.1', port)
  upsertAndSelectService(service)
  serviceSearchMessage.value = service.reachable === false
    ? t('tunnel.wizard.flow.probeAddedUnreachable', { host: service.host, port: service.port })
    : t('tunnel.wizard.flow.probeFoundAndSelected', { host: service.host, port: service.port })
  serviceSearchError.value = service.reachable === false
}

async function addManualService() {
  if (!manualService.port || manualService.port <= 0 || manualService.port > 65535) {
    serviceSearchMessage.value = t('tunnel.wizard.flow.manualPortRange')
    serviceSearchError.value = true
    return
  }
  const service = await probeService(manualService.host || '127.0.0.1', manualService.port)
  if (manualService.name.trim()) {
    service.label = `${manualService.name.trim()} :${service.port}`
    service.technology = manualService.name.trim()
  }
  upsertAndSelectService({ ...service, manual: true })
  serviceSearchMessage.value = service.reachable === false
    ? t('tunnel.wizard.flow.manualAddedUnreachable')
    : t('tunnel.wizard.flow.manualAddedSelected')
  serviceSearchError.value = service.reachable === false
}

function upsertAndSelectService(service: LocalServiceRecord) {
  const key = serviceKey(service)
  localServices.value = [
    service,
    ...localServices.value.filter((item) => serviceKey(item) !== key),
  ]
  selectService(service)
}

function isValidPort(port: number | null | undefined): port is number {
  return Number.isInteger(port) && port > 0 && port <= 65535
}

function serviceKey(service: LocalServiceRecord) {
  return `${service.host}:${service.port}:${service.pid ?? 'manual'}`
}

function serviceMeta(service: LocalServiceRecord) {
  const chunks = [
    service.process || service.executable || 'TCP LISTEN',
    service.bindAddress && service.bindAddress !== service.host ? t('tunnel.wizard.flow.boundTo', { address: service.bindAddress }) : '',
    service.manual ? t('tunnel.wizard.flow.manualLabel') : '',
  ].filter(Boolean)
  return chunks.join(' · ')
}

function reachabilityTone(service: LocalServiceRecord) {
  if (service.reachable === false) return 'unreachable'
  if (service.reachable === true) return 'reachable'
  return 'unknown'
}

function reachabilityLabel(service: LocalServiceRecord) {
  if (service.reachable === false) return t('tunnel.wizard.flow.reachabilityUnreachable')
  if (service.reachable === true) return t('tunnel.wizard.flow.reachabilityReachable')
  return t('tunnel.wizard.flow.reachabilityUnknown')
}

function isServerConnected(server: Server) {
  return server.status === 'connected'
}

function serverAddress(server: Server) {
  return `${server.settings.host}:${server.settings.port}`
}

function serverStatusLabel(status: ServerStatus) {
  const labels: Record<ServerStatus, string> = {
    connected: 'Connected',
    disconnected: 'Disconnected',
    connecting: 'Connecting',
    reconnecting: 'Reconnecting',
    offline: 'Offline',
    maintenance: 'Maintenance',
    error: 'Error',
  }
  return labels[status] ?? status
}

function serviceIcon(service: LocalServiceRecord) {
  const name = `${service.technology} ${service.process}`.toLowerCase()
  if (name.includes('redis') || name.includes('mysql') || name.includes('postgres') || name.includes('mongo')) return 'database'
  if (name.includes('docker')) return 'box'
  if (service.recommendedProtocol === 'http' || service.recommendedProtocol === 'https') return 'globe'
  return 'terminal'
}

function close() {
  emit('update:visible', false)
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
  width: min(820px, 100%);
  max-height: min(780px, calc(100vh - 48px));
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr) auto;
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: 8px;
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
  letter-spacing: 0;
}

.wizard__close {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.wizard__steps {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
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

.wizard-copy p,
.wizard-alert {
  color: var(--text-secondary);
}

.wizard-alert.error,
.wizard__error {
  color: var(--color-error);
}

.server-grid,
.service-list,
.protocol-grid,
.diagnosis-list {
  display: grid;
  gap: var(--space-2);
}

.server-grid {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.protocol-grid {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.server-card,
.service-list button,
.protocol-grid button {
  min-height: 58px;
  display: grid;
  align-items: center;
  gap: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-surface);
  color: var(--text-primary);
  cursor: pointer;
  text-align: left;
}

.server-card {
  grid-template-columns: 38px minmax(0, 1fr) auto;
  min-height: 112px;
  padding: var(--space-3);
}

.server-card.offline {
  opacity: 0.72;
}

.server-card__icon {
  width: 38px;
  height: 38px;
  display: grid;
  place-items: center;
  border-radius: 8px;
  background: var(--bg-input);
  color: var(--color-primary);
}

.server-card__main,
.server-card__meta {
  min-width: 0;
  display: grid;
  gap: 3px;
}

.server-card__main strong,
.service-list strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-card__status {
  align-self: start;
  padding: 3px 8px;
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.server-card__status.connected {
  background: var(--color-success-muted);
  color: var(--color-success);
}

.server-card__status.error,
.server-card__status.offline {
  background: var(--color-error-muted);
  color: var(--color-error);
}

.server-card__meta {
  grid-column: 2 / -1;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.service-list button {
  grid-template-columns: 24px minmax(0, 1fr) auto;
  padding: var(--space-3);
}

.protocol-grid button {
  justify-items: center;
  min-height: 118px;
  padding: var(--space-4);
  text-align: center;
}

.server-card.active,
.service-list button.active,
.protocol-grid button.active {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
}

.service-tools,
.manual-service {
  display: grid;
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-surface);
}

.service-tools {
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
}

.service-tools__actions {
  display: flex;
  gap: var(--space-2);
}

.manual-service {
  grid-template-columns: 1.1fr 0.7fr 1fr auto;
  align-items: end;
}

.empty-state {
  min-height: 108px;
  display: grid;
  place-items: center;
  gap: var(--space-2);
  padding: var(--space-5);
  border: 1px dashed var(--border-default);
  border-radius: 8px;
  color: var(--text-secondary);
  text-align: center;
}

.service-list {
  max-height: 390px;
  overflow: auto;
}

.service-list button {
  grid-template-columns: 24px minmax(0, 1fr) auto;
}

.service-list button.unreachable {
  border-color: color-mix(in srgb, var(--color-error) 28%, var(--border-subtle));
}

.service-list__aside {
  display: grid;
  justify-items: end;
  gap: 4px;
}

.service-reachability {
  padding: 2px 8px;
  border-radius: var(--radius-full);
  font-size: 10px;
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.service-reachability.reachable {
  background: var(--color-success-muted);
  color: var(--color-success);
}

.service-reachability.unreachable {
  background: var(--color-error-muted);
  color: var(--color-error);
}

.service-reachability.unknown {
  background: var(--bg-input);
  color: var(--text-tertiary);
}

.discovery-log {
  display: grid;
  gap: var(--space-2);
  padding: var(--space-3);
  border: 1px dashed var(--border-default);
  border-radius: 8px;
  background: var(--bg-surface);
}

.discovery-log__header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.discovery-log__header strong {
  display: block;
  color: var(--text-primary);
}

.discovery-log__header small {
  display: block;
  margin-top: 2px;
  color: var(--text-secondary);
}

.discovery-log__body {
  max-height: 180px;
  overflow-y: auto;
  display: grid;
  gap: 4px;
  padding: var(--space-2);
  border-radius: 6px;
  background: var(--bg-input);
  font-family: var(--font-mono);
  font-size: 11px;
  line-height: 1.5;
}

.discovery-log__line {
  display: flex;
  align-items: baseline;
  gap: var(--space-2);
  color: var(--text-tertiary);
}

.discovery-log__line.found {
  color: var(--color-success);
}

.discovery-log__line.idle {
  color: var(--text-secondary);
}

.discovery-log__port {
  min-width: 52px;
  color: var(--text-secondary);
}

.discovery-state {
  min-height: 148px;
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px dashed var(--border-default);
  border-radius: 8px;
  background: var(--bg-surface);
}

.discovery-state strong {
  display: block;
  color: var(--text-primary);
}

.discovery-state small {
  display: block;
  margin-top: 2px;
  color: var(--text-secondary);
}

.discovery-state__spinner {
  width: 22px;
  height: 22px;
  flex-shrink: 0;
  border: 2px solid var(--border-default);
  border-top-color: var(--color-primary);
  border-radius: var(--radius-full);
  animation: wizard-spin 0.8s linear infinite;
}

@keyframes wizard-spin {
  to {
    transform: rotate(360deg);
  }
}

.service-list small,
.server-card small,
.protocol-grid small,
.diagnosis-list small {
  color: var(--text-tertiary);
}

.service-list code,
.confirm-list strong {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

label {
  display: grid;
  gap: var(--space-2);
}

label span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.field-hint {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: 1.5;
}

.subdomain-row {
  display: grid;
  grid-template-columns: minmax(96px, 140px) auto 1fr;
  gap: var(--space-2);
  align-items: center;
}

.subdomain-row__dot {
  color: var(--text-secondary);
  font-weight: var(--weight-semibold);
}

.subdomain-row__base {
  background: var(--bg-surface);
  color: var(--text-secondary);
}

.access-preview {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.prefix-presets {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.prefix-presets button {
  min-height: 28px;
  padding: 0 10px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.prefix-presets button.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: rgb(91 141 239 / 10%);
}

.dns-guide {
  display: grid;
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-input);
}

.dns-guide__record {
  display: grid;
  gap: var(--space-2);
}

.dns-guide__record div {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  min-height: 34px;
  padding: 0 var(--space-3);
  border-radius: 6px;
  background: var(--bg-surface);
}

.dns-guide__record span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.dns-guide__record strong {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

input,
select {
  width: 100%;
  height: 38px;
  border: 1px solid var(--border-default);
  border-radius: 8px;
  background: var(--bg-input);
  color: var(--text-primary);
  padding: 0 var(--space-3);
  outline: 0;
}

.domain-picker {
  position: relative;
}

.domain-picker__trigger {
  width: 100%;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  padding: 0 var(--space-3);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  background: var(--bg-input);
  color: var(--text-primary);
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-out);
}

.domain-picker__trigger:hover:not(:disabled) {
  border-color: var(--color-border-strong);
}

.domain-picker__trigger:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.domain-picker--open .domain-picker__trigger {
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}

.domain-picker__chevron {
  flex-shrink: 0;
  color: var(--text-tertiary);
  transition: transform var(--duration-fast) var(--ease-out);
}

.domain-picker--open .domain-picker__chevron {
  transform: rotate(180deg);
}

.domain-picker__menu {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  z-index: 20;
  max-height: 220px;
  overflow: auto;
  border: 1px solid var(--border-default);
  border-radius: 8px;
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.domain-picker__option {
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border: 0;
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
}

.domain-picker__option:hover,
.domain-picker__option.active {
  background: var(--bg-surface);
}

.certificate-state {
  min-height: 42px;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0 var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-input);
}

.confirm-list {
  display: grid;
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-input);
}

.confirm-list div {
  min-height: 40px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: 0 var(--space-3);
}

.confirm-list div + div {
  border-top: 1px solid var(--border-subtle);
}

.confirm-list span {
  color: var(--text-tertiary);
}

.diagnosis-list article {
  display: grid;
  gap: 2px;
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-input);
}

.diagnosis-list article.error {
  border-color: color-mix(in srgb, var(--color-error) 36%, var(--border-subtle));
}

.wizard__footer {
  min-height: 66px;
  border-top: 1px solid var(--border-subtle);
}

.wizard__error {
  flex: 1;
  font-size: var(--text-sm);
}

.wizard-enter-active,
.wizard-leave-active {
  transition: opacity 180ms var(--ease-out);
}

.wizard-enter-active .wizard,
.wizard-leave-active .wizard {
  transition:
    transform 180ms var(--ease-out),
    opacity 180ms var(--ease-out);
}

.wizard-enter-from,
.wizard-leave-to {
  opacity: 0;
}

.wizard-enter-from .wizard,
.wizard-leave-to .wizard {
  opacity: 0;
  transform: translateY(8px);
}

@media (max-width: 720px) {
  .server-grid,
  .service-tools,
  .manual-service,
  .protocol-grid {
    grid-template-columns: 1fr;
  }

  .service-list button {
    grid-template-columns: 24px minmax(0, 1fr);
  }

  .service-list code {
    grid-column: 2;
  }
}
</style>
