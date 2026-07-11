<template>
  <Transition name="wizard">
    <div v-if="visible" class="wizard-backdrop" @keydown.esc="close">
      <section class="wizard" role="dialog" aria-modal="true">
        <header class="wizard__header">
          <div>
            <p>{{ t('domains.wizard.brand') }}</p>
            <h2>{{ stepTitle }}</h2>
          </div>
          <button type="button" class="wizard__close" @click="close"><GIcon name="close" :size="16" /></button>
        </header>

        <div class="wizard__steps">
          <span v-for="index in 5" :key="index" :class="{ active: step === index, done: step > index }">{{ index }}</span>
        </div>

        <main class="wizard__body">
          <section v-if="step === 1" class="wizard-step">
            <p>{{ t('domains.wizard.step1.desc') }}</p>
            <div class="server-grid">
              <button
                v-for="server in connectedServers"
                :key="server.id"
                type="button"
                class="server-card"
                :class="{ active: form.serverId === server.id }"
                @click="form.serverId = server.id">
                <strong>{{ server.name }}</strong>
                <small>{{ server.publicIp }}</small>
              </button>
            </div>
          </section>

          <section v-else-if="step === 2" class="wizard-step">
            <p class="wizard-hint">{{ t('domains.wizard.step2.desc') }}</p>

            <div class="domain-mode-tabs" role="tablist">
              <button
                v-for="mode in entryModes"
                :key="mode.value"
                type="button"
                role="tab"
                class="domain-mode-tabs__item"
                :class="{ active: entryMode === mode.value }"
                :aria-selected="entryMode === mode.value"
                @click="entryMode = mode.value">
                {{ mode.label }}
              </button>
            </div>

            <template v-if="entryMode === 'root'">
              <label class="domain-wizard__field">
                <span class="domain-wizard__label">{{ t('domains.wizard.step2.modeRoot') }}</span>
                <GHostInput v-model="rootHost" :placeholder="t('domains.wizard.step2.rootPlaceholder')" />
              </label>
            </template>

            <template v-else-if="entryMode === 'subdomain'">
              <label class="domain-wizard__field">
                <span class="domain-wizard__label">{{ t('domains.wizard.step2.baseDomain') }}</span>
                <div class="domain-wizard__select-wrap">
                  <input
                    v-model.trim="baseDomain"
                    class="domain-wizard__input"
                    list="domain-base-suggestions"
                    :placeholder="t('domains.wizard.step2.basePlaceholder')" />
                  <datalist id="domain-base-suggestions">
                    <option v-for="option in baseDomainOptions" :key="option" :value="option" />
                  </datalist>
                </div>
              </label>
              <label class="domain-wizard__field">
                <span class="domain-wizard__label">{{ t('domains.wizard.step2.prefix') }}</span>
                <div class="subdomain-row">
                  <input
                    v-model.trim="subdomainPrefix"
                    class="domain-wizard__input"
                    :placeholder="t('domains.wizard.step2.prefixPlaceholder')" />
                  <span class="subdomain-row__dot">.</span>
                  <input
                    :value="normalizedBaseDomain || t('domains.wizard.step2.basePlaceholder')"
                    readonly
                    class="domain-wizard__input subdomain-row__base" />
                </div>
                <div class="prefix-presets">
                  <button
                    v-for="prefix in SUBDOMAIN_PREFIX_PRESETS"
                    :key="prefix"
                    type="button"
                    :class="{ active: subdomainPrefix === prefix }"
                    @click="subdomainPrefix = prefix">
                    {{ prefix }}
                  </button>
                </div>
              </label>
            </template>

            <template v-else-if="entryMode === 'wildcard'">
              <label class="domain-wizard__field">
                <span class="domain-wizard__label">{{ t('domains.wizard.step2.baseDomain') }}</span>
                <input
                  v-model.trim="baseDomain"
                  class="domain-wizard__input"
                  list="domain-base-suggestions-wildcard"
                  :placeholder="t('domains.wizard.step2.basePlaceholder')" />
                <datalist id="domain-base-suggestions-wildcard">
                  <option v-for="option in baseDomainOptions" :key="`w-${option}`" :value="option" />
                </datalist>
              </label>
              <p class="wizard-hint">{{ t('domains.wizard.step2.wildcardHint') }}</p>
              <div v-if="wildcardHost" class="domain-preview-chip">*.{{ normalizedBaseDomain }}</div>
            </template>

            <template v-else>
              <label class="domain-wizard__field">
                <span class="domain-wizard__label">{{ t('domains.wizard.step2.baseDomain') }}</span>
                <input
                  v-model.trim="baseDomain"
                  class="domain-wizard__input"
                  list="domain-base-suggestions-batch"
                  :placeholder="t('domains.wizard.step2.basePlaceholder')" />
                <datalist id="domain-base-suggestions-batch">
                  <option v-for="option in baseDomainOptions" :key="`b-${option}`" :value="option" />
                </datalist>
              </label>
              <p class="wizard-hint">{{ t('domains.wizard.step2.batchHint') }}</p>
              <div class="prefix-presets prefix-presets--batch">
                <button
                  v-for="prefix in SUBDOMAIN_PREFIX_PRESETS"
                  :key="`batch-${prefix}`"
                  type="button"
                  :class="{ active: batchPrefixes.has(prefix) }"
                  @click="toggleBatchPrefix(prefix)">
                  {{ prefix }}
                </button>
              </div>
              <label class="domain-wizard__field">
                <span class="domain-wizard__label">{{ t('domains.wizard.step2.prefix') }}</span>
                <div class="batch-prefix-input">
                  <input
                    v-model.trim="customBatchPrefix"
                    class="domain-wizard__input"
                    :placeholder="t('domains.wizard.step2.prefixPlaceholder')"
                    @keydown.enter.prevent="addCustomBatchPrefix" />
                  <GButton variant="secondary" size="sm" @click="addCustomBatchPrefix">{{ t('common.create') }}</GButton>
                </div>
              </label>
              <ul v-if="batchHosts.length" class="batch-host-list">
                <li v-for="host in batchHosts" :key="host">{{ host }}</li>
              </ul>
              <p v-if="batchHosts.length" class="wizard-hint">
                {{ t('domains.wizard.step2.batchSelected', { count: batchHosts.length }) }}
              </p>
            </template>

            <div class="protocol-row">
              <span class="domain-wizard__label">{{ t('domains.wizard.step2.protocol') }}</span>
              <div class="protocol-row__choices">
                <label><input v-model="form.protocol" type="radio" value="https" /> HTTPS</label>
                <label><input v-model="form.protocol" type="radio" value="http" /> HTTP</label>
              </div>
              <p v-if="form.protocol === 'https'" class="wizard-hint">{{ t('domains.wizard.step2.protocolHttpsHint') }}</p>
            </div>

            <div v-if="primaryHost && entryMode !== 'batch'" class="domain-preview-chip">{{ primaryHost }}</div>
            <p v-if="step2Error" class="wizard-error">{{ step2Error }}</p>
          </section>

          <section v-else-if="step === 3" class="wizard-step">
            <GButton variant="secondary" icon="refresh" :loading="dnsLoading" @click="checkDns">
              {{ t('domains.wizard.step3.check') }}
            </GButton>
            <p v-if="entryMode === 'batch'" class="wizard-hint">{{ t('domains.wizard.step3.batchNote') }}</p>
            <pre v-if="dnsResult" class="wizard-pre">{{ dnsResult }}</pre>
          </section>

          <section v-else-if="step === 4" class="wizard-step">
            <p class="wizard-hint">{{ t('domains.wizard.step4.desc') }}</p>
            <label class="domain-wizard__field">
              <span class="domain-wizard__label">{{ t('domains.wizard.step4.project') }}</span>
              <div class="domain-wizard__select-wrap">
                <select v-model="form.projectId" class="domain-wizard__select">
                  <option value="">{{ t('domains.wizard.step4.optional') }}</option>
                  <option v-for="project in projects" :key="project.id" :value="project.id">{{ project.name }}</option>
                </select>
                <GIcon name="chevron-down" :size="14" class="domain-wizard__select-chevron" />
              </div>
            </label>
            <label class="domain-wizard__field">
              <span class="domain-wizard__label">{{ t('domains.wizard.step4.tunnel') }}</span>
              <div class="domain-wizard__select-wrap">
                <select v-model="form.tunnelId" class="domain-wizard__select" :disabled="batchHosts.length > 1">
                  <option value="">{{ t('domains.wizard.step4.optional') }}</option>
                  <option v-for="tunnel in recommendedTunnels" :key="tunnel.id" :value="tunnel.id">
                    {{ tunnel.name }} · {{ tunnel.protocol }}
                  </option>
                </select>
                <GIcon name="chevron-down" :size="14" class="domain-wizard__select-chevron" />
              </div>
            </label>
            <p v-if="batchHosts.length > 1" class="wizard-hint">{{ t('domains.wizard.step4.batchTunnelHint') }}</p>
          </section>

          <section v-else class="wizard-step">
            <dl class="review-list">
              <div><dt>{{ t('domains.wizard.review.server') }}</dt><dd>{{ selectedServer?.name || '-' }}</dd></div>
              <div><dt>{{ t('domains.wizard.review.tunnel') }}</dt><dd>{{ selectedTunnel?.name || t('domains.wizard.step4.optional') }}</dd></div>
              <div>
                <dt>{{ hostsToCreate.length > 1 ? t('domains.wizard.review.hosts') : t('domains.wizard.review.host') }}</dt>
                <dd>
                  <ul v-if="hostsToCreate.length > 1" class="review-hosts">
                    <li v-for="host in hostsToCreate" :key="host">{{ host }}</li>
                  </ul>
                  <span v-else>{{ hostsToCreate[0] || '-' }}</span>
                </dd>
              </div>
              <div><dt>{{ t('domains.wizard.review.protocol') }}</dt><dd>{{ form.protocol.toUpperCase() }}</dd></div>
              <div v-if="previewUrl"><dt>URL</dt><dd>{{ previewUrl }}</dd></div>
            </dl>
          </section>
        </main>

        <footer class="wizard__footer">
          <GButton variant="ghost" :disabled="step === 1" @click="step -= 1">{{ t('common.back') }}</GButton>
          <GButton v-if="step < 5" variant="primary" :disabled="!canNext" @click="step += 1">{{ t('domains.wizard.next') }}</GButton>
          <GButton v-else variant="primary" :loading="submitting" @click="submit">{{ t('domains.wizard.confirm') }}</GButton>
        </footer>
      </section>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GHostInput from '@components/form/GHostInput.vue'
import GIcon from '@components/icons/GIcon.vue'
import { useFeedback } from '@composables/useFeedback'
import { certificateService } from '@views/certificates/service'
import type { CertificateSummary } from '@views/certificates/types'
import {
  SUBDOMAIN_PREFIX_PRESETS,
  buildSubdomainHost,
  listCertificateBaseDomains,
} from '@views/tunnels/utils/domainAccess'
import { domainService } from '../../services/domain.service'
import { buildTunnelPublicUrl } from '@views/tunnels/utils'
import type { Server } from '@views/servers/types'
import type { Tunnel } from '@views/tunnels/types'

type DomainEntryMode = 'root' | 'subdomain' | 'wildcard' | 'batch'

const HOST_PATTERN =
  /^(?:\*\.|[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)(?:\.(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?))+$/i
const PREFIX_PATTERN = /^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$/i

const props = defineProps<{
  visible: boolean
  servers: Server[]
  tunnels: Tunnel[]
  projects: Array<{ id: string; name: string }>
  existingHosts?: string[]
}>()

const emit = defineEmits<{
  'update:visible': [visible: boolean]
  submitted: []
}>()

const { t } = useI18n()
const { notify } = useFeedback()

const step = ref(1)
const submitting = ref(false)
const dnsLoading = ref(false)
const dnsResult = ref('')
const certificates = ref<CertificateSummary[]>([])
const entryMode = ref<DomainEntryMode>('subdomain')
const rootHost = ref('')
const baseDomain = ref('')
const subdomainPrefix = ref('dev')
const batchPrefixes = ref(new Set<string>())
const customBatchPrefix = ref('')

const form = reactive({
  serverId: '',
  projectId: '',
  tunnelId: '',
  protocol: 'https',
  path: '/',
})

const entryModes = computed(() => [
  { value: 'root' as const, label: t('domains.wizard.step2.modeRoot') },
  { value: 'subdomain' as const, label: t('domains.wizard.step2.modeSubdomain') },
  { value: 'wildcard' as const, label: t('domains.wizard.step2.modeWildcard') },
  { value: 'batch' as const, label: t('domains.wizard.step2.modeBatch') },
])

const connectedServers = computed(() =>
  props.servers.filter((server) => server.status === 'connected'),
)

const recommendedTunnels = computed(() =>
  props.tunnels.filter((tunnel) => !form.serverId || tunnel.serverId === form.serverId),
)

const selectedServer = computed(() => props.servers.find((server) => server.id === form.serverId))
const selectedTunnel = computed(() => props.tunnels.find((tunnel) => tunnel.id === form.tunnelId))

const stepTitle = computed(() => t(`domains.wizard.step${step.value}.title`))

const normalizedBaseDomain = computed(() =>
  baseDomain.value.trim().toLowerCase().replace(/^\*\./, ''),
)

const baseDomainOptions = computed(() => {
  const seen = new Set<string>()
  const options: string[] = []

  for (const domain of listCertificateBaseDomains(certificates.value)) {
    if (!seen.has(domain)) {
      seen.add(domain)
      options.push(domain)
    }
  }

  for (const host of props.existingHosts ?? []) {
    const apex = toApexDomain(host)
    if (apex && !seen.has(apex)) {
      seen.add(apex)
      options.push(apex)
    }
  }

  for (const tunnel of props.tunnels) {
    const apex = tunnel.host ? toApexDomain(tunnel.host) : null
    if (apex && !seen.has(apex)) {
      seen.add(apex)
      options.push(apex)
    }
  }

  return options.sort((left, right) => left.localeCompare(right))
})

const wildcardHost = computed(() =>
  normalizedBaseDomain.value ? `*.${normalizedBaseDomain.value}` : '',
)

const batchHosts = computed(() =>
  [...batchPrefixes.value]
    .map((prefix) => buildSubdomainHost(prefix, normalizedBaseDomain.value))
    .filter(Boolean)
    .sort((left, right) => left.localeCompare(right)),
)

const hostsToCreate = computed(() => {
  if (entryMode.value === 'root') {
    const host = rootHost.value.trim().toLowerCase()
    return host ? [host] : []
  }
  if (entryMode.value === 'subdomain') {
    const host = buildSubdomainHost(subdomainPrefix.value, normalizedBaseDomain.value)
    return host ? [host] : []
  }
  if (entryMode.value === 'wildcard') {
    return wildcardHost.value ? [wildcardHost.value] : []
  }
  return batchHosts.value
})

const primaryHost = computed(() => hostsToCreate.value[0] ?? '')

const step2Error = computed(() => {
  if (entryMode.value === 'root') {
    const host = rootHost.value.trim()
    if (!host) return ''
    return HOST_PATTERN.test(host) ? '' : t('domains.wizard.step2.invalid')
  }

  if (!normalizedBaseDomain.value) {
    return hostsToCreate.value.length ? '' : t('domains.wizard.step2.baseRequired')
  }

  if (!HOST_PATTERN.test(normalizedBaseDomain.value)) {
    return t('domains.wizard.step2.invalid')
  }

  if (entryMode.value === 'subdomain') {
    const prefix = subdomainPrefix.value.trim()
    if (!prefix) return t('domains.wizard.step2.prefixInvalid')
    if (!PREFIX_PATTERN.test(prefix)) return t('domains.wizard.step2.prefixInvalid')
  }

  if (entryMode.value === 'batch' && batchHosts.value.length === 0) {
    return t('domains.wizard.step2.batchEmpty')
  }

  for (const host of hostsToCreate.value) {
    if (!HOST_PATTERN.test(host)) {
      return t('domains.wizard.step2.invalid')
    }
  }

  return ''
})

const canNext = computed(() => {
  if (step.value === 1) return Boolean(form.serverId)
  if (step.value === 2) return hostsToCreate.value.length > 0 && !step2Error.value
  return true
})

const previewUrl = computed(() => {
  const host = primaryHost.value
  if (!host || host.startsWith('*.')) return ''
  return buildTunnelPublicUrl({
    protocol: form.protocol,
    host,
    path: form.path,
  })
})

watch(
  () => props.visible,
  async (visible) => {
    if (!visible) return
    step.value = 1
    dnsResult.value = ''
    entryMode.value = 'subdomain'
    rootHost.value = ''
    subdomainPrefix.value = 'dev'
    batchPrefixes.value = new Set(['dev', 'api'])
    customBatchPrefix.value = ''
    form.serverId = connectedServers.value[0]?.id ?? ''
    form.projectId = ''
    form.tunnelId = ''
    form.protocol = 'https'
    form.path = '/'

    try {
      const listRes = await certificateService.list()
      certificates.value = listRes.certificates ?? []
    } catch {
      certificates.value = []
    }

    baseDomain.value = baseDomainOptions.value[0] ?? ''
  },
)

watch(entryMode, (mode) => {
  if (mode === 'batch' && batchPrefixes.value.size === 0) {
    batchPrefixes.value = new Set(['dev', 'api'])
  }
})

function toApexDomain(host: string): string | null {
  const normalized = host.replace(/^\*\./, '').trim().toLowerCase()
  const parts = normalized.split('.').filter(Boolean)
  if (parts.length < 2) return null
  return parts.slice(-2).join('.')
}

function toggleBatchPrefix(prefix: string) {
  const next = new Set(batchPrefixes.value)
  if (next.has(prefix)) next.delete(prefix)
  else next.add(prefix)
  batchPrefixes.value = next
}

function addCustomBatchPrefix() {
  const prefix = customBatchPrefix.value.trim().toLowerCase()
  if (!prefix || !PREFIX_PATTERN.test(prefix)) return
  const next = new Set(batchPrefixes.value)
  next.add(prefix)
  batchPrefixes.value = next
  customBatchPrefix.value = ''
}

function close() {
  emit('update:visible', false)
}

async function checkDns() {
  const host = primaryHost.value
  if (!host) return
  dnsLoading.value = true
  try {
    const result = await domainService.checkDns(host)
    dnsResult.value = JSON.stringify(result, null, 2)
  } catch (error) {
    dnsResult.value = error instanceof Error ? error.message : String(error)
  } finally {
    dnsLoading.value = false
  }
}

async function submit() {
  const hosts = hostsToCreate.value
  if (!hosts.length) return

  submitting.value = true
  let success = 0
  let failed = 0

  try {
    for (const host of hosts) {
      try {
        await domainService.create({
          host,
          tunnelId: hosts.length > 1 ? undefined : form.tunnelId || undefined,
          protocol: form.protocol,
          path: form.path,
          projectId: form.projectId || undefined,
        })
        success += 1
      } catch {
        failed += 1
      }
    }

    if (success > 0) {
      emit('submitted')
      if (failed > 0) {
        notify.success(t('domains.wizard.createPartial', { success, failed }))
      } else {
        notify.success(t('domains.wizard.createSuccess', { count: success }))
      }
      close()
    }
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.wizard-backdrop {
  position: fixed;
  inset: 0;
  z-index: 90;
  background: rgba(8, 12, 20, 0.5);
  display: grid;
  place-items: center;
  padding: var(--space-4);
}

.wizard {
  width: min(760px, 100%);
  max-height: 90vh;
  overflow: auto;
  border-radius: var(--radius-xl);
  border: 1px solid var(--border-subtle);
  background: var(--bg-surface);
  display: flex;
  flex-direction: column;
}

.wizard__header,
.wizard__footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
}

.wizard__footer {
  border-bottom: 0;
  border-top: 1px solid var(--border-subtle);
}

.wizard__steps {
  display: flex;
  gap: 8px;
  padding: 0 var(--space-4) var(--space-3);
}

.wizard__steps span {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-full);
  display: grid;
  place-items: center;
  background: var(--bg-input);
  color: var(--text-secondary);
}

.wizard__steps span.active,
.wizard__steps span.done {
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.wizard__body {
  padding: 0 var(--space-4) var(--space-4);
}

.wizard-step {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.wizard-hint {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--text-sm);
  line-height: 1.5;
}

.domain-mode-tabs {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-2);
}

.domain-mode-tabs__item {
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-out), color var(--duration-fast) var(--ease-out);
}

.domain-mode-tabs__item.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-muted);
}

.domain-wizard__field {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.domain-wizard__label {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.domain-wizard__input {
  width: 100%;
  height: var(--control-height-md);
  padding: 0 var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-input);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--font-size-input);
}

.domain-wizard__input:focus {
  border-color: var(--color-border-focus);
  outline: none;
  box-shadow: var(--shadow-focus);
}

.domain-wizard__select-wrap {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
}

.domain-wizard__select {
  appearance: none;
  width: 100%;
  height: var(--control-height-md);
  padding: 0 var(--space-6) 0 var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-input);
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--font-size-input);
  cursor: pointer;
}

.domain-wizard__select:focus {
  border-color: var(--color-border-focus);
  outline: none;
  box-shadow: var(--shadow-focus);
}

.domain-wizard__select-chevron {
  position: absolute;
  right: var(--space-3);
  color: var(--text-tertiary);
  pointer-events: none;
}

.subdomain-row {
  display: grid;
  grid-template-columns: minmax(120px, 1fr) auto minmax(160px, 1.2fr);
  gap: var(--space-2);
  align-items: center;
}

.subdomain-row__dot {
  color: var(--text-tertiary);
  text-align: center;
}

.subdomain-row__base {
  color: var(--text-secondary);
}

.prefix-presets {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.prefix-presets button {
  padding: 4px 10px;
  border-radius: var(--radius-full);
  border: 1px solid var(--border-subtle);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  cursor: pointer;
}

.prefix-presets button.active {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-muted);
}

.batch-prefix-input {
  display: flex;
  gap: var(--space-2);
}

.batch-host-list {
  margin: 0;
  padding: var(--space-3);
  list-style: none;
  border-radius: var(--radius-md);
  background: var(--bg-input);
  display: grid;
  gap: 6px;
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.domain-preview-chip {
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.protocol-row {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding-top: var(--space-2);
  border-top: 1px solid var(--border-subtle);
}

.protocol-row__choices {
  display: flex;
  gap: var(--space-4);
}

.server-grid {
  display: grid;
  gap: var(--space-2);
}

.server-card {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  padding: var(--space-3);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-subtle);
  background: var(--bg-input);
  cursor: pointer;
}

.server-card.active {
  border-color: var(--color-primary);
}

.wizard-error {
  color: var(--color-danger);
}

.wizard-pre,
.review-list {
  background: var(--bg-input);
  border-radius: var(--radius-md);
  padding: var(--space-3);
}

.review-list div {
  display: grid;
  grid-template-columns: 120px 1fr;
  gap: var(--space-2);
  padding: 4px 0;
}

.review-hosts {
  margin: 0;
  padding-left: 1rem;
}
</style>
