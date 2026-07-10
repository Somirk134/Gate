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
                  :placeholder="t('tunnel.wizard.flow.searchPlaceholder')"
                  @keydown.enter.prevent="searchLocalService" />
              </label>
              <div class="service-tools__actions">
                <GButton variant="secondary" icon="search" @click="searchLocalService">
                  {{ t('tunnel.wizard.flow.manualSearch') }}
                </GButton>
                <GButton variant="ghost" icon="refresh" @click="loadDiscovery">
                  {{ t('tunnel.wizard.flow.rescan') }}
                </GButton>
              </div>
            </div>
            <div class="manual-service">
              <label>
                <span>{{ t('tunnel.wizard.flow.labels.host') }}</span>
                <input v-model.trim="manualService.host" autocomplete="off" placeholder="127.0.0.1" />
              </label>
              <label>
                <span>{{ t('tunnel.wizard.flow.labels.port') }}</span>
                <input v-model.number="manualService.port" inputmode="numeric" type="number" placeholder="8088" />
              </label>
              <label>
                <span>{{ t('tunnel.wizard.flow.serviceLabel') }}</span>
                <input v-model.trim="manualService.name" autocomplete="off" placeholder="Spring Boot" />
              </label>
              <GButton variant="secondary" icon="plus" @click="addManualService">
                {{ t('tunnel.wizard.flow.manualAdd') }}
              </GButton>
            </div>
            <p v-if="serviceSearchMessage" class="wizard-alert">{{ serviceSearchMessage }}</p>
            <p v-if="discoveryError" class="wizard-alert">{{ discoveryError }}</p>
            <div class="service-list">
              <button
                v-for="service in filteredLocalServices"
                :key="`${service.host}:${service.port}:${service.pid ?? 'none'}`"
                type="button"
                :class="{ active: selectedServiceKey === serviceKey(service) }"
                @click="selectService(service)">
                <GIcon :name="serviceIcon(service)" :size="18" />
                <div>
                  <strong>{{ service.label }}</strong>
                  <small>{{ serviceMeta(service) }}</small>
                </div>
                <code>{{ service.host }}:{{ service.port }}</code>
              </button>
            </div>
            <div v-if="!filteredLocalServices.length" class="empty-state">
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

          <section v-else-if="step === 4" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.flow.step4Title') }}</strong>
              <p>{{ t('tunnel.wizard.flow.step4Desc') }}</p>
            </div>
            <label class="wizard-check">
              <input v-model="autoAllocate" type="checkbox" />
              <span>{{ t('tunnel.wizard.flow.autoAllocate') }}</span>
            </label>
            <label v-if="!autoAllocate">
              <span>{{ t('tunnel.wizard.flow.recommendedPort') }}</span>
              <select v-model.number="form.remotePort">
                <option
                  v-for="port in remotePorts.availablePorts"
                  :key="port.port"
                  :value="port.port">
                  {{ port.port }}{{ port.recommended ? ` · ${t('tunnel.wizard.flow.recommended')}` : '' }}
                </option>
              </select>
            </label>
            <label v-if="!autoAllocate">
              <span>{{ t('tunnel.wizard.flow.manualPort') }}</span>
              <input v-model.number="manualRemotePort" inputmode="numeric" type="number" />
            </label>
            <p v-if="portCheckMessage" class="wizard-alert" :class="{ error: portConflict }">
              {{ portCheckMessage }}
            </p>
          </section>

          <section v-else-if="step === 5" class="wizard-step">
            <div class="wizard-copy">
              <strong>{{ t('tunnel.wizard.flow.step5Title') }}</strong>
              <p>{{ t('tunnel.wizard.flow.step5Desc') }}</p>
            </div>
            <label v-if="isHttpLike">
              <span>{{ t('tunnel.wizard.flow.labels.domain') }}</span>
              <input v-model.trim="form.host" autocomplete="off" placeholder="api.example.com" />
            </label>
            <label v-if="isHttpLike">
              <span>{{ t('tunnel.wizard.flow.labels.path') }}</span>
              <input v-model.trim="form.path" autocomplete="off" placeholder="/" />
            </label>
            <div v-if="form.protocol === 'https'" class="certificate-state">
              <GIcon :name="hasCertificate ? 'check-circle' : 'alert-circle'" :size="18" />
              <span>{{ certificateMessage }}</span>
            </div>
            <p v-if="form.protocol === 'tcp'" class="wizard-alert">{{ t('tunnel.wizard.flow.tcpNoDomainCert') }}</p>
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
              <div><span>{{ t('tunnel.wizard.flow.confirmList.remotePort') }}</span><strong>{{ autoAllocate ? t('tunnel.wizard.flow.autoAllocate') : form.remotePort }}</strong></div>
              <div v-if="isHttpLike"><span>{{ t('tunnel.wizard.flow.confirmList.domain') }}</span><strong>{{ form.host || '-' }}</strong></div>
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
          <GButton v-if="step < 6" variant="primary" trailing-icon="arrow-right" @click="next">
            {{ t('tunnel.wizard.next') }}
          </GButton>
          <GButton v-else variant="secondary" icon="activity" @click="runDiagnosis">
            {{ t('tunnel.wizard.flow.healthCheck') }}
          </GButton>
          <GButton v-if="step === 6" variant="primary" icon="plus" @click="createTunnel">
            {{ t('tunnel.wizard.finish') }}
          </GButton>
        </footer>
      </section>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import { certificateService } from '@views/certificates/service'
import { discoveryService, type LocalServiceRecord, type PortDiscovery, type TunnelDiagnosis } from '@/services'
import type { Server, ServerStatus } from '@views/servers'
import type { TunnelFormData, TunnelProtocol } from '../types'

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
const steps = computed(() => [
  { index: 1, title: t('tunnel.wizard.flow.step1Title') },
  { index: 2, title: t('tunnel.wizard.flow.step2Title') },
  { index: 3, title: t('tunnel.wizard.flow.step3Title') },
  { index: 4, title: t('tunnel.wizard.flow.step4Title') },
  { index: 5, title: t('tunnel.wizard.flow.step5Title') },
  { index: 6, title: t('tunnel.wizard.flow.step6Title') },
])

const step = ref(1)
const errorMessage = ref('')
const discoveryError = ref('')
const localServices = ref<LocalServiceRecord[]>([])
const serviceQuery = ref('')
const serviceSearchMessage = ref('')
const remotePorts = ref<PortDiscovery>({
  occupiedPorts: [],
  availablePorts: [],
  systemReservedPorts: [],
  gateReservedPorts: [],
  updatedAt: 0,
})
const selectedService = ref<LocalServiceRecord | null>(null)
const autoAllocate = ref(true)
const manualRemotePort = ref<number | null>(null)
const portConflict = ref(false)
const portCheckMessage = ref('')
const certificateDomains = ref<string[]>([])
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

const stepTitle = computed(() => steps.value.find((item) => item.index === step.value)?.title ?? t('tunnel.wizard.flow.step6Title'))
const isHttpLike = computed(() => form.protocol === 'http' || form.protocol === 'https')
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
const hasCertificate = computed(
  () => Boolean(normalizedHost.value) && certificateDomains.value.includes(normalizedHost.value),
)
const certificateMessage = computed(() => {
  if (!normalizedHost.value) return t('tunnel.wizard.flow.certPromptHttps')
  return hasCertificate.value ? t('tunnel.wizard.flow.certFound') : t('tunnel.wizard.flow.certNotFound')
})

watch(
  () => props.visible,
  (visible) => {
    if (visible) void reset()
  },
)

watch(manualRemotePort, async (port) => {
  if (autoAllocate.value || !port) return
  form.remotePort = port
  await checkPort(port)
})

watch(
  () => form.remotePort,
  (port) => {
    if (autoAllocate.value || !port || port === manualRemotePort.value) return
    manualRemotePort.value = port
  },
)

watch(autoAllocate, (enabled) => {
  if (enabled) {
    form.remotePort = null
    portConflict.value = false
    portCheckMessage.value = t('tunnel.wizard.flow.autoAllocateHint')
  } else {
    const recommended = remotePorts.value.availablePorts[0]?.port ?? null
    form.remotePort = recommended
    manualRemotePort.value = recommended
    if (recommended) void checkPort(recommended)
  }
})

watch(
  () => form.serverId,
  async () => {
    if (!props.visible || !form.serverId) return
    await loadRemotePorts()
    if (!autoAllocate.value && form.remotePort) {
      await checkPort(form.remotePort)
    }
  },
)

async function reset() {
  step.value = 1
  errorMessage.value = ''
  discoveryError.value = ''
  serviceQuery.value = ''
  serviceSearchMessage.value = ''
  selectedService.value = null
  autoAllocate.value = true
  manualRemotePort.value = null
  portCheckMessage.value = ''
  diagnosis.value = null
  form.name = ''
  form.protocol = 'tcp'
  form.localHost = '127.0.0.1'
  form.localPort = null
  form.remotePort = null
  form.host = ''
  form.path = '/'
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
  await Promise.all([loadDiscovery(), loadCertificates()])
}

async function loadDiscovery() {
  try {
    const [services, ports] = await Promise.all([discoveryService.localServices(), loadRemotePorts()])
    localServices.value = services
    remotePorts.value = ports
    if (services[0]) selectService(services[0])
  } catch (error) {
    discoveryError.value = error instanceof Error ? error.message : t('tunnel.wizard.flow.discoveryFailed')
  }
}

async function loadRemotePorts() {
  const ports = await discoveryService.remotePorts(form.serverId || undefined)
  remotePorts.value = ports
  return ports
}

async function loadCertificates() {
  try {
    const payload = await certificateService.list()
    certificateDomains.value = payload.certificates
      .filter((item) => item.status === 'active')
      .flatMap((item) => [item.domain, ...item.san])
      .map((domain) => domain.toLowerCase())
  } catch {
    certificateDomains.value = []
  }
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
    remotePort: autoAllocate.value ? 0 : form.remotePort ?? 0,
    serverId: form.serverId || undefined,
  })
}

async function next() {
  if (!(await validateStep())) return
  step.value += 1
  if (step.value === 6) void runDiagnosis()
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
  if (step.value === 2 && !selectedService.value) errorMessage.value = t('tunnel.wizard.flow.selectLocalService')
  if (step.value === 4 && !autoAllocate.value && portConflict.value) {
    errorMessage.value = t('tunnel.wizard.flow.remotePortConflict')
  }
  if (step.value === 5 && form.protocol === 'https' && !hasCertificate.value) {
    errorMessage.value = certificateMessage.value
  }
  return !errorMessage.value
}

async function createTunnel() {
  if (!(await validateStep())) return
  const name = form.name.trim() || `Service ${form.localPort}`
  emit('submit', {
    ...form,
    name,
    protocol: form.protocol as TunnelProtocol,
    remotePort: autoAllocate.value ? 0 : form.remotePort,
    tags: [...new Set(form.tags.filter(Boolean))],
  })
  close()
}

function selectServer(server: Server) {
  form.serverId = server.id
  form.serverName = server.name
}

async function searchLocalService() {
  const query = serviceQuery.value.trim()
  serviceSearchMessage.value = ''
  const port = Number.parseInt(query, 10)
  if (!Number.isInteger(port) || port <= 0 || port > 65535) {
    serviceSearchMessage.value = query
      ? t('tunnel.wizard.flow.searchKeywordHint')
      : t('tunnel.wizard.flow.searchEmpty')
    return
  }
  manualService.port = port
  const service = await discoveryService.probeLocalService(manualService.host || '127.0.0.1', port)
  upsertAndSelectService(service)
  serviceSearchMessage.value = service.reachable === false
    ? t('tunnel.wizard.flow.probeAddedUnreachable', { host: service.host, port: service.port })
    : t('tunnel.wizard.flow.probeFoundAndSelected', { host: service.host, port: service.port })
}

async function addManualService() {
  if (!manualService.port || manualService.port <= 0 || manualService.port > 65535) {
    serviceSearchMessage.value = t('tunnel.wizard.flow.manualPortRange')
    return
  }
  const service = await discoveryService.probeLocalService(
    manualService.host || '127.0.0.1',
    manualService.port,
  )
  if (manualService.name.trim()) {
    service.label = `${manualService.name.trim()} :${service.port}`
    service.technology = manualService.name.trim()
  }
  upsertAndSelectService({ ...service, manual: true })
  serviceSearchMessage.value = service.reachable === false
    ? t('tunnel.wizard.flow.manualAddedUnreachable')
    : t('tunnel.wizard.flow.manualAddedSelected')
}

function upsertAndSelectService(service: LocalServiceRecord) {
  const key = serviceKey(service)
  localServices.value = [
    service,
    ...localServices.value.filter((item) => serviceKey(item) !== key),
  ]
  selectService(service)
}

function serviceKey(service: LocalServiceRecord) {
  return `${service.host}:${service.port}:${service.pid ?? 'manual'}`
}

function serviceMeta(service: LocalServiceRecord) {
  const chunks = [
    service.process || service.executable || 'TCP LISTEN',
    service.bindAddress && service.bindAddress !== service.host ? t('tunnel.wizard.flow.boundTo', { address: service.bindAddress }) : '',
    service.manual ? t('tunnel.wizard.flow.manualLabel') : '',
    service.reachable === false ? t('tunnel.wizard.flow.notProbed') : '',
  ].filter(Boolean)
  return chunks.join(' · ')
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

.wizard-check {
  display: inline-flex;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: var(--space-2);
}

.wizard-check input {
  width: 16px;
  height: 16px;
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
