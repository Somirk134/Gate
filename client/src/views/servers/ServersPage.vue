<template>
  <section class="servers-page">
    <header class="server-page-header">
      <div>
        <p>{{ t('server.runtimeServers') }}</p>
        <h1>{{ t('server.title') }}</h1>
        <span>{{
          t('server.summary', { online: onlineServers.length, total: servers.length })
        }}</span>
      </div>
      <div class="server-page-header__actions">
        <GButton variant="secondary" icon="refresh" :loading="isLoading" @click="refresh">
          {{ t('server.refresh') }}
        </GButton>
        <GButton variant="primary" icon="plus" @click="openCreate">
          {{ t('server.addServer') }}
        </GButton>
      </div>
    </header>

    <GCard v-if="isError" variant="plain" padding="lg">
      <GErrorState :title="t('server.loadFailed')" :message="error" retry @retry="retry" />
    </GCard>

    <div v-else-if="!hasServers && !isLoading" class="server-empty">
      <div class="server-empty__icon">
        <GIcon name="servers" :size="34" />
      </div>
      <h2>{{ t('server.emptyTitle') }}</h2>
      <p>{{ t('server.emptyGuide') }}</p>
      <div class="server-empty__actions">
        <GButton variant="primary" icon="plus" @click="openCreate">
          {{ t('server.addFirstServer') }}
        </GButton>
        <GButton variant="secondary" icon="help" @click="router.push('/help')">
          {{ t('server.openHelp') }}
        </GButton>
      </div>
    </div>

    <div v-else class="server-shell">
      <aside class="server-list-panel">
        <div class="server-list-panel__toolbar">
          <label class="server-search">
            <GIcon name="search" :size="15" />
            <input v-model.trim="query" :placeholder="t('server.searchPlaceholder')" />
          </label>
        </div>

        <div class="server-list-panel__body">
          <button
            v-for="server in filteredServers"
            :key="server.id"
            type="button"
            class="server-list-row"
            :class="{ active: selectedId === server.id }"
            @click="selectedId = server.id">
            <span class="server-list-row__status" :class="`is-${statusTone(server.status)}`" />
            <span class="server-list-row__icon">
              <GIcon name="servers" :size="16" />
            </span>
            <span class="server-list-row__main">
              <strong>{{ server.name }}</strong>
              <small>{{ server.settings.host }}:{{ server.settings.port }}</small>
            </span>
            <span class="server-list-row__meta">
              <strong>{{ statusLabel(server.status) }}</strong>
              <small>{{ server.ping ? `${server.ping}ms` : server.lastConnectedAt }}</small>
            </span>
          </button>

          <div v-if="!filteredServers.length" class="server-list-empty">
            <GIcon name="search" :size="24" />
            <span>{{ t('server.noMatching') }}</span>
          </div>
        </div>
      </aside>

      <main class="server-detail-panel">
        <template v-if="selectedServer">
          <div class="server-detail-header">
            <div>
              <div class="server-detail-header__title">
                <span :class="`is-${statusTone(selectedServer.status)}`" />
                <h2>{{ selectedServer.name }}</h2>
              </div>
              <p>{{ selectedServer.settings.host }}:{{ selectedServer.settings.port }}</p>
            </div>
            <div class="server-detail-header__actions">
              <GButton
                v-if="selectedServer.status !== 'connected'"
                variant="primary"
                icon="plug"
                :loading="isConnecting(selectedServer.id)"
                @click="connectSelected">
                {{ t('server.actions.connect') }}
              </GButton>
              <GButton v-else variant="secondary" icon="power" @click="disconnectSelected">
                {{ t('server.actions.disconnect') }}
              </GButton>
              <GButton
                variant="secondary"
                icon="activity"
                :loading="testingId === selectedServer.id"
                @click="testSelected">
                {{ t('server.actions.test') }}
              </GButton>
              <GButton variant="ghost" icon="settings" @click="openEdit(selectedServer)">
                {{ t('server.actions.edit') }}
              </GButton>
              <GButton variant="danger" icon="trash" @click="removeSelected">
                {{ t('server.actions.delete') }}
              </GButton>
            </div>
          </div>

          <div class="server-summary-grid">
            <article>
              <span>{{ t('server.detail.connectionStatus') }}</span>
              <strong>{{ statusLabel(selectedServer.status) }}</strong>
            </article>
            <article>
              <span>{{ t('server.detail.authSession') }}</span>
              <strong>{{ selectedServer.version }}</strong>
            </article>
            <article>
              <span>{{ t('server.detail.lastConnected') }}</span>
              <strong>{{ selectedServer.lastConnectedAt }}</strong>
            </article>
            <article>
              <span>RTT</span>
              <strong>{{ selectedServer.ping ? `${selectedServer.ping} ms` : '-' }}</strong>
            </article>
          </div>

          <section class="server-info-section">
            <div class="server-info-section__head">
              <h3>{{ t('server.detail.connectionConfig') }}</h3>
              <button type="button" @click="copyServerInfo(selectedServer)">
                <GIcon name="copy" :size="15" />
              </button>
            </div>
            <dl class="server-info-list">
              <div>
                <dt>{{ t('server.detail.host') }}</dt>
                <dd>{{ selectedServer.settings.host }}</dd>
              </div>
              <div>
                <dt>{{ t('server.detail.port') }}</dt>
                <dd>{{ selectedServer.settings.port }}</dd>
              </div>
              <div>
                <dt>Token</dt>
                <dd>{{ maskToken(selectedServer.settings.token) }}</dd>
              </div>
              <div>
                <dt>{{ t('server.detail.type') }}</dt>
                <dd>{{ selectedServer.kind }}</dd>
              </div>
              <div>
                <dt>{{ t('server.detail.region') }}</dt>
                <dd>{{ selectedServer.region || '-' }}</dd>
              </div>
              <div>
                <dt>{{ t('server.detail.autoConnect') }}</dt>
                <dd>
                  {{
                    selectedServer.settings.autoConnect
                      ? t('server.detail.on')
                      : t('server.detail.off')
                  }}
                </dd>
              </div>
            </dl>
          </section>

          <section class="server-info-section">
            <div class="server-info-section__head">
              <h3>{{ t('server.detail.health') }}</h3>
              <span>{{ selectedServer.health.score }}/100</span>
            </div>
            <div class="server-health-list">
              <article v-for="item in selectedServer.health.items" :key="item.key">
                <GIcon :name="item.icon" :size="16" />
                <div>
                  <strong>{{ item.label }}</strong>
                  <p>{{ item.message }}</p>
                </div>
                <small>{{ item.latency ? `${item.latency}ms` : '-' }}</small>
              </article>
            </div>
          </section>

          <section v-if="selectedServer.logs.length" class="server-info-section">
            <div class="server-info-section__head">
              <h3>{{ t('server.detail.recentError') }}</h3>
            </div>
            <div class="server-error-line">
              {{ selectedServer.logs[0]?.message }}
            </div>
          </section>
        </template>

        <div v-else class="server-detail-placeholder">
          <GIcon name="servers" :size="34" />
          <span>{{ t('server.selectPrompt') }}</span>
        </div>
      </main>
    </div>

    <Transition name="server-dialog">
      <div v-if="dialogVisible" class="server-dialog-backdrop" @click.self="closeDialog">
        <form class="server-dialog" @submit.prevent="submitForm">
          <header>
            <div>
              <p>{{ editingId ? t('server.dialog.editTitle') : t('server.dialog.addTitle') }}</p>
              <h2>{{ editingId ? t('server.dialog.editTitle') : t('server.dialog.addTitle') }}</h2>
            </div>
            <button type="button" :aria-label="t('server.dialog.close')" @click="closeDialog">
              <GIcon name="close" :size="16" />
            </button>
          </header>

          <main class="server-dialog__main">
            <section class="server-form-section">
              <div class="server-helper-banner">
                <GIcon name="plug-zap" :size="20" />
                <div>
                  <strong>{{ t('server.dialog.helperTitle') }}</strong>
                  <p>{{ t('server.dialog.helperDesc') }}</p>
                </div>
              </div>

              <div class="server-preset-row">
                <button
                  v-for="preset in connectionPresets"
                  :key="preset.id"
                  type="button"
                  class="server-preset"
                  @click="applyPreset(preset.id)">
                  <GIcon :name="preset.icon" :size="16" />
                  <span>{{ preset.label }}</span>
                </button>
              </div>
              <p class="server-local-note">
                <GIcon name="info-circle" :size="14" />
                {{ t('server.dialog.localNote', { token: localServerToken }) }}
              </p>
              <p class="server-mode-hint">
                <GIcon :name="serverModeHint.icon" :size="15" />
                <span>{{ serverModeHint.text }}</span>
              </p>

              <label>
                <span>{{ t('server.dialog.name') }}</span>
                <input
                  v-model.trim="form.name"
                  autocomplete="off"
                  :placeholder="t('server.dialog.namePlaceholder')" />
                <small>{{ t('server.dialog.nameHint') }}</small>
              </label>

              <div class="server-form-grid">
                <label>
                  <span>{{ t('server.dialog.host') }}</span>
                  <input
                    v-model.trim="form.host"
                    autocomplete="off"
                    :placeholder="t('server.dialog.hostPlaceholder')"
                    required />
                  <small>{{ t('server.dialog.hostHint') }}</small>
                </label>
                <label>
                  <span>{{ t('server.dialog.port') }}</span>
                  <input v-model.number="form.port" type="number" min="1" max="65535" required />
                  <small>{{ t('server.dialog.portHint') }}</small>
                </label>
              </div>

              <label>
                <span class="server-label-row">
                  Token
                  <em>{{ t('server.dialog.tokenHint') }}</em>
                </span>
                <div class="server-token-control">
                  <input
                    v-model.trim="form.token"
                    autocomplete="off"
                    :type="tokenVisible ? 'text' : 'password'"
                    :placeholder="t('server.dialog.tokenPlaceholder')"
                    required />
                  <button
                    type="button"
                    :title="
                      tokenVisible ? t('server.dialog.hideToken') : t('server.dialog.showToken')
                    "
                    @click="tokenVisible = !tokenVisible">
                    <GIcon :name="tokenVisible ? 'eye-off' : 'eye'" :size="15" />
                  </button>
                </div>
                <small>{{ t('server.dialog.tokenNote', { token: localServerToken }) }}</small>
              </label>

              <div class="server-kind-picker">
                <div class="server-kind-picker__head">
                  <span>{{ t('server.dialog.type') }}</span>
                  <small>{{ t('server.dialog.typeHint') }}</small>
                </div>
                <div class="server-kind-grid-simple">
                  <button
                    v-for="option in kindOptions"
                    :key="option.value"
                    type="button"
                    class="server-kind-card"
                    :class="{ active: form.kind === option.value }"
                    @click="selectKind(option.value)">
                    <GIcon :name="option.icon" :size="16" />
                    <span>
                      <strong>{{ option.label }}</strong>
                      <small>{{ option.description }}</small>
                    </span>
                  </button>
                </div>
              </div>

              <div class="server-form-grid">
                <label>
                  <span>{{ t('server.dialog.region') }}</span>
                  <input
                    v-model.trim="form.region"
                    autocomplete="off"
                    :placeholder="t('server.dialog.regionPlaceholder')" />
                  <small>{{ t('server.dialog.regionHint') }}</small>
                </label>
                <label>
                  <span>{{ t('server.dialog.remark') }}</span>
                  <input
                    v-model.trim="form.remark"
                    autocomplete="off"
                    :placeholder="t('server.dialog.remarkPlaceholder')" />
                  <small>{{ t('server.dialog.remarkHint') }}</small>
                </label>
              </div>

              <label class="server-check server-check--with-hint">
                <input v-model="form.autoConnect" type="checkbox" />
                <span>
                  {{ t('server.dialog.autoConnect') }}
                  <small>{{ t('server.dialog.autoConnectHint') }}</small>
                </span>
              </label>

              <p v-if="formError" class="server-form-error">
                {{ formError }}
              </p>
            </section>

            <aside class="server-guide-panel">
              <header class="server-guide-panel__header">
                <span>{{ t('server.guide.title') }}</span>
                <strong>{{ t('server.guide.subtitle') }}</strong>
              </header>

              <details class="server-guide-panel__block" open>
                <summary>
                  <GIcon name="monitor" :size="16" /> {{ t('server.guide.localTitle') }}
                </summary>
                <p>{{ t('server.guide.localDesc') }}</p>
                <div class="server-guide-command">
                  <code>{{ localServerCommand }}</code>
                  <button type="button" class="server-guide-copy" @click="copyLocalServerCommand">
                    <GIcon name="copy" :size="14" />
                    {{ t('server.guide.copy') }}
                  </button>
                </div>
                <div class="server-guide-kv">
                  <span>{{ t('server.guide.host') }}</span
                  ><b>127.0.0.1</b> <span>{{ t('server.guide.port') }}</span
                  ><b>7000</b> <span>Token</span><b>{{ localServerToken }}</b>
                </div>
                <details class="server-guide-advanced">
                  <summary>{{ t('server.guide.customLocal') }}</summary>
                  <div class="server-guide-command">
                    <code>{{ customLocalServerCommand }}</code>
                    <button
                      type="button"
                      class="server-guide-copy"
                      @click="
                        copyGuideCommand(
                          customLocalServerCommand,
                          t('server.guide.customLocalCommand'),
                        )
                      ">
                      <GIcon name="copy" :size="14" />
                      {{ t('server.guide.copy') }}
                    </button>
                  </div>
                </details>
              </details>

              <details class="server-guide-panel__block">
                <summary>
                  <GIcon name="cloud" :size="16" /> {{ t('server.guide.cloudTitle') }}
                </summary>
                <p>{{ t('server.guide.cloudDesc') }}</p>
                <div class="server-guide-command">
                  <span>{{ t('server.guide.sourceStart') }}</span>
                  <code>{{ remoteServerCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyGuideCommand(remoteServerCommand, t('server.guide.sourceCommand'))">
                    <GIcon name="copy" :size="14" />
                    {{ t('server.guide.copy') }}
                  </button>
                </div>
                <div class="server-guide-command">
                  <span>{{ t('server.guide.binaryStart') }}</span>
                  <code>{{ remoteBinaryCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyGuideCommand(remoteBinaryCommand, t('server.guide.binaryCommand'))">
                    <GIcon name="copy" :size="14" />
                    {{ t('server.guide.copy') }}
                  </button>
                </div>
                <div class="server-guide-command">
                  <span>Docker Compose</span>
                  <code>{{ dockerComposeCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="
                      copyGuideCommand(dockerComposeCommand, t('server.guide.dockerCommand'))
                    ">
                    <GIcon name="copy" :size="14" />
                    {{ t('server.guide.copy') }}
                  </button>
                </div>
                <p>{{ t('server.guide.dockerDesc') }}</p>
              </details>

              <details class="server-guide-panel__block">
                <summary>
                  <GIcon name="shield" :size="16" /> {{ t('server.guide.firewallTitle') }}
                </summary>
                <ul>
                  <li>{{ t('server.guide.firewall7000') }}</li>
                  <li>{{ t('server.guide.firewall5800') }}</li>
                  <li>{{ t('server.guide.firewallTunnel') }}</li>
                  <li>{{ t('server.guide.firewallHttp') }}</li>
                </ul>
                <div class="server-guide-command">
                  <span>{{ t('server.guide.ufw') }}</span>
                  <code>{{ ufwCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyGuideCommand(ufwCommand, t('server.guide.ufwCommand'))">
                    <GIcon name="copy" :size="14" />
                    {{ t('server.guide.copy') }}
                  </button>
                </div>
                <div class="server-guide-command">
                  <span>{{ t('server.guide.firewalld') }}</span>
                  <code>{{ firewalldCommand }}</code>
                  <button
                    type="button"
                    class="server-guide-copy"
                    @click="copyGuideCommand(firewalldCommand, t('server.guide.firewalldCommand'))">
                    <GIcon name="copy" :size="14" />
                    {{ t('server.guide.copy') }}
                  </button>
                </div>
                <p>{{ t('server.guide.firewallNote') }}</p>
              </details>

              <details class="server-guide-panel__block">
                <summary>
                  <GIcon name="clipboard-list" :size="16" /> {{ t('server.guide.formTitle') }}
                </summary>
                <ul>
                  <li>{{ t('server.guide.formHost') }}</li>
                  <li>{{ t('server.guide.formPort') }}</li>
                  <li>{{ t('server.guide.formToken') }}</li>
                  <li>{{ t('server.guide.formType') }}</li>
                </ul>
              </details>

              <details class="server-guide-panel__block">
                <summary>
                  <GIcon name="circle-help" :size="16" /> {{ t('server.guide.typeTitle') }}
                </summary>
                <ul>
                  <li>{{ t('server.guide.typePersonal') }}</li>
                  <li>{{ t('server.guide.typeCloud') }}</li>
                  <li>{{ t('server.guide.typeNas') }}</li>
                  <li>{{ t('server.guide.typeCompany') }}</li>
                  <li>{{ t('server.guide.typeDocker') }}</li>
                </ul>
              </details>
            </aside>
          </main>

          <footer>
            <GButton variant="ghost" @click="closeDialog"> {{ t('common.cancel') }} </GButton>
            <GButton variant="primary" type="submit" :loading="saving">
              {{ t('common.save') }}
            </GButton>
          </footer>
        </form>
      </div>
    </Transition>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useFeedback } from '@composables/useFeedback'
import GButton from '@components/base/GButton.vue'
import GCard from '@components/base/GCard.vue'
import GErrorState from '@components/feedback/GErrorState.vue'
import GIcon from '@components/icons/GIcon.vue'
import { useServer } from './composables/useServer'
import { defaultServerForm } from './store/server'
import type { Server, ServerFormData, ServerKind, ServerStatus } from './types'
import './styles/server.css'

const router = useRouter()
const { t } = useI18n()
const { toast, confirmDanger } = useFeedback()
const {
  servers,
  onlineServers,
  isLoading,
  isError,
  error,
  hasServers,
  refresh,
  retry,
  create,
  update,
  remove,
  connect,
  disconnect,
  checkHealth,
} = useServer()

const query = ref('')
const selectedId = ref<string | null>(null)
const dialogVisible = ref(false)
const editingId = ref<string | null>(null)
const saving = ref(false)
const connectingId = ref<string | null>(null)
const testingId = ref<string | null>(null)
const formError = ref('')
const tokenVisible = ref(false)

const form = reactive<ServerFormData>({ ...defaultServerForm, tags: [] })
const localServerToken = 'gate-alpha-token'
const localServerCommand = 'npm run dev:server'
const customLocalServerCommand =
  'npm run dev:server:local -- -Addr "127.0.0.1:7001" -Token "replace-with-a-long-random-token"'
const remoteServerCommand =
  'GATE_SERVER_ADDR=0.0.0.0:7000 GATE_AUTH_TOKEN=replace-with-a-long-random-token cargo run -p gate-server --release'
const remoteBinaryCommand =
  'GATE_SERVER_ADDR=0.0.0.0:7000 GATE_AUTH_TOKEN=replace-with-a-long-random-token ./gate-server'
const dockerComposeCommand =
  'GATE_AUTH_TOKEN=replace-with-a-long-random-token GATE_PORT=5800 docker compose up -d'
const ufwCommand = 'sudo ufw allow 7000/tcp && sudo ufw allow 18080/tcp && sudo ufw reload'
const firewalldCommand =
  'sudo firewall-cmd --permanent --add-port=7000/tcp && sudo firewall-cmd --permanent --add-port=18080/tcp && sudo firewall-cmd --reload'

const connectionPresets = computed(() => [
  {
    id: 'local',
    label: t('server.presets.local'),
    icon: 'monitor',
    host: '127.0.0.1',
    port: 7000,
    kind: 'personal' as ServerKind,
    region: 'local',
    name: t('server.presets.localName'),
    token: localServerToken,
  },
  {
    id: 'vps',
    label: t('server.presets.vps'),
    icon: 'cloud',
    host: '',
    port: 7000,
    kind: 'cloud' as ServerKind,
    region: '',
    name: t('server.presets.vpsName'),
    token: '',
  },
  {
    id: 'docker',
    label: 'Docker',
    icon: 'box',
    host: '127.0.0.1',
    port: 5800,
    kind: 'docker' as ServerKind,
    region: 'docker',
    name: t('server.presets.dockerName'),
    token: '',
  },
])

const kindOptions = computed<
  Array<{
    value: ServerKind
    label: string
    icon: string
    description: string
  }>
>(() => [
  {
    value: 'personal',
    label: t('server.kinds.personal.label'),
    icon: 'user',
    description: t('server.kinds.personal.description'),
  },
  {
    value: 'cloud',
    label: t('server.kinds.cloud.label'),
    icon: 'cloud',
    description: t('server.kinds.cloud.description'),
  },
  {
    value: 'nas',
    label: t('server.kinds.nas.label'),
    icon: 'hard-drive',
    description: t('server.kinds.nas.description'),
  },
  {
    value: 'company',
    label: t('server.kinds.company.label'),
    icon: 'shield',
    description: t('server.kinds.company.description'),
  },
  {
    value: 'docker',
    label: t('server.kinds.docker.label'),
    icon: 'box',
    description: t('server.kinds.docker.description'),
  },
])

const filteredServers = computed(() => {
  const keyword = query.value.toLowerCase()
  if (!keyword) return servers.value
  return servers.value.filter((server) =>
    [server.name, server.settings.host, String(server.settings.port), server.region, ...server.tags]
      .join(' ')
      .toLowerCase()
      .includes(keyword),
  )
})

const selectedServer = computed(() =>
  selectedId.value ? servers.value.find((server) => server.id === selectedId.value) : undefined,
)

const serverModeHint = computed(() => {
  if (form.kind === 'docker') {
    return {
      icon: 'box',
      text: t('server.modeHint.docker'),
    }
  }
  if (form.kind === 'cloud') {
    return {
      icon: 'cloud',
      text: t('server.modeHint.cloud'),
    }
  }
  return {
    icon: 'monitor',
    text: t('server.modeHint.local'),
  }
})

watch(
  servers,
  (list) => {
    if (!selectedId.value || !list.some((server) => server.id === selectedId.value)) {
      selectedId.value = list[0]?.id ?? null
    }
  },
  { immediate: true },
)

onMounted(() => {
  void refresh()
})

function openCreate() {
  editingId.value = null
  Object.assign(form, { ...defaultServerForm, tags: [] })
  formError.value = ''
  tokenVisible.value = false
  dialogVisible.value = true
}

function openEdit(server: Server) {
  editingId.value = server.id
  Object.assign(form, {
    name: server.name,
    kind: server.kind,
    host: server.settings.host,
    port: server.settings.port,
    token: server.settings.token,
    region: server.region,
    remark: server.settings.remark,
    tags: [...server.tags],
    heartbeatInterval: server.settings.heartbeatInterval,
    reconnectInterval: server.settings.reconnectInterval,
    autoConnect: server.settings.autoConnect,
  })
  formError.value = ''
  tokenVisible.value = false
  dialogVisible.value = true
}

function closeDialog() {
  dialogVisible.value = false
}

async function submitForm() {
  formError.value = validateForm()
  if (formError.value) return

  saving.value = true
  try {
    if (editingId.value) {
      await update(editingId.value, { ...form, tags: [...form.tags] })
      toast.success(t('server.notifications.updated'))
    } else {
      const created = await create({ ...form, tags: [...form.tags] })
      selectedId.value = created.id
      toast.success(t('server.notifications.saved'))
    }
    closeDialog()
  } catch (err) {
    formError.value = err instanceof Error ? err.message : t('server.notifications.saveFailed')
  } finally {
    saving.value = false
  }
}

async function connectSelected() {
  if (!selectedServer.value) return
  connectingId.value = selectedServer.value.id
  try {
    await connect(selectedServer.value.id)
    toast.success(t('server.notifications.connected'))
  } catch (err) {
    toast.error(err instanceof Error ? err.message : t('server.notifications.connectFailed'))
  } finally {
    connectingId.value = null
  }
}

async function disconnectSelected() {
  if (!selectedServer.value) return
  try {
    await disconnect(selectedServer.value.id)
    toast.warning(t('server.notifications.disconnected'))
  } catch (err) {
    toast.error(err instanceof Error ? err.message : t('server.notifications.disconnectFailed'))
  }
}

async function testSelected() {
  if (!selectedServer.value) return
  testingId.value = selectedServer.value.id
  try {
    await checkHealth(selectedServer.value.id)
    toast.success(t('server.notifications.testPassed'))
  } catch (err) {
    toast.error(err instanceof Error ? err.message : t('server.notifications.testFailed'))
  } finally {
    testingId.value = null
  }
}

function removeSelected() {
  const server = selectedServer.value
  if (!server) return
  confirmDanger({
    title: t('server.notifications.deleteTitle'),
    content: t('server.notifications.deleteContent', { name: server.name }),
    confirmText: t('common.delete'),
    onConfirm: async () => {
      try {
        await remove(server.id)
        toast.success(t('server.notifications.deleted'))
      } catch (err) {
        toast.error(err instanceof Error ? err.message : t('server.notifications.deleteFailed'))
      }
    },
  })
}

async function copyServerInfo(server: Server) {
  const text = [
    `name=${server.name}`,
    `host=${server.settings.host}`,
    `port=${server.settings.port}`,
    `status=${server.status}`,
    `lastConnectedAt=${server.lastConnectedAt}`,
  ].join('\n')
  await navigator.clipboard.writeText(text)
  toast.success(t('server.notifications.copiedInfo'))
}

async function copyLocalServerCommand() {
  await copyGuideCommand(localServerCommand, t('server.guide.localCommand'))
}

async function copyGuideCommand(text: string, label: string) {
  await navigator.clipboard.writeText(text)
  toast.success(t('server.notifications.copiedCommand', { label }))
}

function validateForm() {
  if (!form.host.trim()) return t('server.validation.hostRequired')
  if (!form.port || form.port < 1 || form.port > 65535) return t('server.validation.portRange')
  if (!form.token.trim()) return t('server.validation.tokenRequired')
  return ''
}

function applyPreset(id: string) {
  const preset = connectionPresets.value.find((item) => item.id === id)
  if (!preset) return
  form.name = preset.name
  form.host = preset.host
  form.port = preset.port
  form.kind = preset.kind
  form.region = preset.region
  form.token = preset.token
  formError.value = ''
}

function selectKind(kind: ServerKind) {
  form.kind = kind
}

function isConnecting(id: string) {
  return connectingId.value === id
}

function statusTone(status: ServerStatus) {
  if (status === 'connected') return 'online'
  if (status === 'connecting' || status === 'reconnecting') return 'warning'
  if (status === 'error') return 'error'
  return 'offline'
}

function statusLabel(status: ServerStatus) {
  return t(`server.statusLabels.${status}`)
}

function maskToken(token: string) {
  if (!token) return '-'
  if (token.length <= 8) return '********'
  return `${token.slice(0, 4)}****${token.slice(-4)}`
}
</script>

<style scoped>
.servers-page {
  width: min(100%, var(--content-max-width));
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.server-page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  flex-shrink: 0;
}

.server-page-header p,
.server-dialog header p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.server-page-header h1 {
  margin-top: 2px;
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.server-page-header span {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-secondary);
}

.server-page-header__actions,
.server-empty__actions,
.server-detail-header__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.server-empty,
.server-detail-placeholder,
.server-list-empty {
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  color: var(--text-tertiary);
  text-align: center;
}

.server-empty {
  min-height: 460px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-xl);
  background: var(--bg-surface);
  padding: var(--space-6);
}

.server-empty__icon {
  width: 86px;
  height: 86px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-2xl);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.server-empty h2 {
  color: var(--text-primary);
  font-size: var(--text-2xl);
  letter-spacing: 0;
}

.server-empty p {
  max-width: 560px;
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.server-shell {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(300px, 390px) minmax(0, 1fr);
  gap: var(--space-4);
}

.server-list-panel,
.server-detail-panel,
.server-info-section {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.server-list-panel {
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.server-list-panel__toolbar {
  padding: var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
}

.server-search {
  height: 36px;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-tertiary);
  padding: 0 var(--space-3);
}

.server-search:focus-within {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.server-search input {
  min-width: 0;
  flex: 1;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text-primary);
}

.server-list-panel__body {
  min-height: 0;
  overflow: auto;
  padding: var(--space-2);
}

.server-list-row {
  width: 100%;
  min-height: 74px;
  display: grid;
  grid-template-columns: 10px 36px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  margin-top: var(--space-1);
  padding: var(--space-3);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
}

.server-list-row:hover,
.server-list-row.active {
  border-color: var(--border-default);
  background: var(--bg-surface-hover);
}

.server-list-row.active {
  box-shadow: inset 2px 0 0 var(--color-primary);
}

.server-list-row__status,
.server-detail-header__title > span {
  width: 9px;
  height: 9px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.is-online {
  background: var(--status-online);
  color: var(--status-online);
}
.is-warning {
  background: var(--status-warning);
  color: var(--status-warning);
}
.is-error {
  background: var(--status-error);
  color: var(--status-error);
}
.is-offline {
  background: var(--status-offline);
  color: var(--status-offline);
}

.server-list-row__icon {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--color-primary);
}

.server-list-row__main,
.server-list-row__meta {
  min-width: 0;
}

.server-list-row__main strong,
.server-list-row__main small,
.server-list-row__meta strong,
.server-list-row__meta small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-list-row__main small,
.server-list-row__meta small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-list-row__meta {
  max-width: 100px;
  color: var(--text-secondary);
  font-size: var(--text-xs);
  text-align: right;
}

.server-list-empty {
  min-height: 220px;
}

.server-detail-panel {
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: var(--space-5);
}

.server-detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.server-detail-header__title {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.server-detail-header h2 {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.server-detail-header p {
  margin-top: var(--space-1);
  color: var(--text-secondary);
  font-family: var(--font-mono);
}

.server-summary-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
  margin-top: var(--space-5);
}

.server-summary-grid article {
  min-height: 82px;
  display: grid;
  align-content: center;
  gap: var(--space-1);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.server-summary-grid span,
.server-info-list dt {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-summary-grid strong {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-info-section {
  margin-top: var(--space-4);
  padding: var(--space-4);
}

.server-info-section__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}

.server-info-section__head h3 {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
}

.server-info-section__head button {
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.server-info-section__head button:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.server-info-list {
  display: grid;
  gap: var(--space-2);
}

.server-info-list div {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.server-info-list dd {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-health-list {
  display: grid;
  gap: var(--space-2);
}

.server-health-list article {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-2);
  min-height: 46px;
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.server-health-list strong,
.server-health-list p {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-health-list p,
.server-health-list small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-error-line {
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
  color: var(--color-error);
  padding: var(--space-3);
  overflow-wrap: anywhere;
}

.server-detail-placeholder {
  min-height: 420px;
}

.server-dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  display: grid;
  place-items: center;
  padding: var(--space-6);
  background: var(--color-overlay);
  backdrop-filter: blur(12px);
}

.server-dialog {
  width: min(980px, 100%);
  max-height: min(760px, calc(100vh - 48px));
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  overflow: hidden;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.server-dialog header,
.server-dialog footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-5);
}

.server-dialog header {
  border-bottom: 1px solid var(--border-subtle);
}

.server-dialog header h2 {
  margin-top: 2px;
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
}

.server-dialog header button {
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

.server-dialog header button:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.server-dialog__main {
  min-height: 0;
  overflow: hidden;
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(330px, 380px);
  align-items: start;
  gap: var(--space-5);
  padding: var(--space-5);
}

.server-form-section {
  max-height: 100%;
  overflow: auto;
  display: grid;
  gap: var(--space-4);
  min-width: 0;
  padding-right: var(--space-1);
}

.server-dialog label {
  display: grid;
  gap: var(--space-2);
}

.server-dialog label span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.server-dialog label small,
.server-kind-picker__head small,
.server-check small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: var(--leading-normal);
}

.server-label-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.server-label-row em {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-style: normal;
  font-weight: var(--weight-regular);
}

.server-dialog input,
.server-dialog select,
.server-dialog textarea {
  width: 100%;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: 0 var(--space-3);
  outline: 0;
}

.server-dialog input,
.server-dialog select {
  height: 38px;
}

.server-dialog textarea {
  resize: vertical;
  min-height: 84px;
  padding-block: var(--space-2);
}

.server-dialog input:focus,
.server-dialog select:focus,
.server-dialog textarea:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.server-helper-banner {
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr);
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid color-mix(in srgb, var(--color-primary) 28%, transparent);
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.server-helper-banner strong {
  display: block;
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.server-helper-banner p {
  margin-top: 2px;
  color: var(--text-secondary);
  font-size: var(--text-sm);
  line-height: var(--leading-normal);
}

.server-preset-row {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-2);
}

.server-preset {
  min-height: 38px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-weight: var(--weight-medium);
  cursor: pointer;
}

.server-preset:hover {
  border-color: var(--color-primary);
  color: var(--text-primary);
  background: var(--bg-surface-hover);
}

.server-local-note {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  min-width: 0;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: var(--leading-normal);
}

.server-local-note code {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  overflow-wrap: anywhere;
}

.server-mode-hint {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr);
  gap: var(--space-2);
  align-items: flex-start;
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  line-height: var(--leading-relaxed);
}

.server-mode-hint svg {
  margin-top: 2px;
  color: var(--color-primary);
}

.server-form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.server-token-control {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 38px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.server-token-control:focus-within {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.server-token-control input {
  border: 0;
  box-shadow: none !important;
  background: transparent;
  font-family: var(--font-mono);
}

.server-token-control button {
  display: grid;
  place-items: center;
  border: 0;
  border-left: 1px solid var(--border-subtle);
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.server-token-control button:hover {
  color: var(--text-primary);
  background: var(--bg-surface-hover);
}

.server-kind-picker {
  display: grid;
  gap: var(--space-2);
}

.server-kind-picker__head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.server-kind-grid-simple {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-2);
}

.server-kind-card {
  min-height: 62px;
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: var(--space-2) var(--space-3);
  text-align: left;
  cursor: pointer;
}

.server-kind-card:hover,
.server-kind-card.active {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
  color: var(--text-primary);
}

.server-kind-card strong,
.server-kind-card small {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-kind-card strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.server-kind-card small {
  margin-top: 2px;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-check {
  display: inline-flex !important;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: flex-start;
  gap: var(--space-2) !important;
}

.server-check input {
  width: 16px;
  height: 16px;
  accent-color: var(--color-primary);
}

.server-check--with-hint > span {
  display: grid;
  gap: 2px;
}

.server-form-error {
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
  color: var(--color-error);
  padding: var(--space-3);
}

.server-guide-panel {
  max-height: 100%;
  overflow: auto;
  display: grid;
  align-content: start;
  gap: var(--space-2);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  background: color-mix(in srgb, var(--bg-input) 72%, transparent);
}

.server-guide-panel__header {
  display: grid;
  gap: 2px;
  padding: var(--space-1) var(--space-1) var(--space-2);
}

.server-guide-panel__header span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.server-guide-panel__header strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
}

.server-guide-panel__block {
  padding: 0;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  overflow: hidden;
}

.server-guide-panel__block[open] {
  padding-bottom: var(--space-3);
}

.server-guide-panel__block summary {
  display: flex;
  align-items: center;
  min-height: 42px;
  gap: var(--space-2);
  padding: 0 var(--space-3);
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  cursor: pointer;
  list-style: none;
}

.server-guide-panel__block summary::-webkit-details-marker {
  display: none;
}

.server-guide-panel__block summary::after {
  content: '';
  width: 7px;
  height: 7px;
  margin-left: auto;
  border-right: 1.5px solid currentColor;
  border-bottom: 1.5px solid currentColor;
  opacity: 0.65;
  transform: rotate(45deg) translateY(-2px);
  transition: transform var(--duration-fast) var(--ease-out);
}

.server-guide-panel__block[open] summary::after {
  transform: rotate(225deg) translateY(-1px);
}

.server-guide-panel__block p,
.server-guide-panel__block li {
  color: var(--text-secondary);
  font-size: var(--text-xs);
  line-height: var(--leading-relaxed);
}

.server-guide-panel__block p,
.server-guide-panel__block ul,
.server-guide-panel__block ol,
.server-guide-panel__block .server-guide-command,
.server-guide-panel__block .server-guide-kv,
.server-guide-panel__block .server-guide-advanced {
  margin-inline: var(--space-3);
}

.server-guide-panel__block code {
  display: block;
  overflow: auto;
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  padding: var(--space-2);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.server-guide-command {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--space-2);
  align-items: end;
}

.server-guide-command + .server-guide-command {
  margin-top: var(--space-2);
}

.server-guide-command > span {
  grid-column: 1 / -1;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.server-guide-kv {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  gap: var(--space-2) var(--space-3);
  padding: var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
}

.server-guide-kv span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.server-guide-kv b {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.server-inline-code,
.server-dialog label small code {
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  padding: 1px 4px;
}

.server-guide-copy {
  width: 70px;
  min-height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-1);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 0 var(--space-2);
  font-size: var(--text-xs);
  cursor: pointer;
}

.server-guide-copy:hover {
  border-color: var(--color-primary);
  color: var(--text-primary);
}

.server-guide-advanced {
  display: grid;
  gap: var(--space-2);
}

.server-guide-advanced summary {
  width: fit-content;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  cursor: pointer;
}

.server-guide-advanced summary:hover {
  color: var(--text-secondary);
}

.server-guide-panel__block ol,
.server-guide-panel__block ul {
  display: grid;
  gap: var(--space-1);
  padding-left: var(--space-4);
}

.server-dialog footer {
  justify-content: flex-end;
  border-top: 1px solid var(--border-subtle);
}

.server-dialog-enter-active,
.server-dialog-leave-active {
  transition: opacity var(--duration-base) var(--ease-out);
}

.server-dialog-enter-from,
.server-dialog-leave-to {
  opacity: 0;
}

@media (max-width: 1120px) {
  .server-shell {
    grid-template-columns: 1fr;
  }

  .server-list-panel {
    max-height: 360px;
  }

  .server-summary-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 760px) {
  .server-page-header,
  .server-detail-header {
    flex-direction: column;
  }

  .server-page-header__actions,
  .server-detail-header__actions {
    width: 100%;
    align-items: stretch;
    flex-direction: column;
  }

  .server-summary-grid,
  .server-form-grid,
  .server-preset-row,
  .server-kind-grid-simple,
  .server-dialog__main {
    grid-template-columns: 1fr;
  }

  .server-dialog__main {
    overflow: auto;
  }

  .server-form-section,
  .server-guide-panel {
    max-height: none;
    overflow: visible;
  }

  .server-guide-panel {
    padding: var(--space-2);
  }
}
</style>
