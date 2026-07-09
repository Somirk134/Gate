<!--
  TunnelSettings — 工作区 Settings 标签
  ------------------------------------------------------------------
  支持：隧道名称 / 协议 / 本地主机 / 本地端口 /
  公网端口 / 备注 / 自动启动 / 压缩（预留）/ 加密（预留）
  全部实时校验。保存后回写 store。
-->
<template>
  <div class="tunnel-settings">
    <!-- 名称 -->
    <GFormField :error="errors.name" required>
      <template #label>{{ t('tunnel.settings.name') }}</template>
      <GInput
        v-model="form.name"
        :placeholder="t('tunnel.settings.namePlaceholder')"
        :state="errors.name ? 'error' : 'normal'"
        :maxlength="40"
        clearable
        @update:model-value="validateField('name')" />
    </GFormField>

    <!-- 协议 -->
    <GFormField>
      <template #label>{{ t('tunnel.settings.protocol') }}</template>
      <TunnelProtocolSelect v-model="form.protocol" />
    </GFormField>

    <div v-if="isHttpLike" class="tunnel-port-row">
      <GFormField :error="errors.host" :required="form.protocol === 'https'">
        <template #label>{{ t('tunnel.settings.host') }}</template>
        <GInput
          v-model="form.host"
          placeholder="api.example.com"
          prefix="globe"
          :state="errors.host ? 'error' : 'normal'"
          @update:model-value="validateField('host')" />
      </GFormField>
      <GFormField :error="errors.path">
        <template #label>{{ t('tunnel.settings.path') }}</template>
        <GInput
          v-model="form.path"
          placeholder="/"
          prefix="route"
          :state="errors.path ? 'error' : 'normal'"
          @update:model-value="validateField('path')" />
      </GFormField>
    </div>

    <!-- 本地主机 -->
    <GFormField :error="errors.localHost">
      <template #label>{{ t('tunnel.settings.localHost') }}</template>
      <GInput
        v-model="form.localHost"
        placeholder="127.0.0.1"
        prefix="plug"
        :state="errors.localHost ? 'error' : 'normal'"
        @update:model-value="validateField('localHost')" />
    </GFormField>

    <!-- 本地端口 / 公网端口 -->
    <div class="tunnel-port-row">
      <GFormField :error="errors.localPort" required>
        <template #label>{{ t('tunnel.settings.localPort') }}</template>
        <TunnelPortInput v-model="form.localPort" />
      </GFormField>
      <GFormField :error="errors.remotePort" required>
        <template #label>{{ t('tunnel.settings.remotePort') }}</template>
        <TunnelPortInput v-model="form.remotePort" />
      </GFormField>
    </div>

    <!-- 备注 -->
    <GFormField>
      <template #label>{{ t('tunnel.settings.remark') }}</template>
      <GTextarea
        v-model="form.remark"
        :placeholder="t('tunnel.settings.remarkPlaceholder')"
        :rows="2"
        :maxlength="200"
        resizable />
    </GFormField>

    <!-- 自动启动 -->
    <div class="tunnel-settings__row">
      <div class="tunnel-settings__row-text">
        <span class="tunnel-settings__row-label">{{ t('tunnel.settings.autoStart') }}</span>
        <span class="tunnel-settings__row-hint">{{ t('tunnel.settings.autoStartHint') }}</span>
      </div>
      <button
        type="button"
        class="tunnel-toggle"
        :class="{ 'tunnel-toggle--on': form.autoStart }"
        @click="form.autoStart = !form.autoStart">
        <span class="tunnel-toggle__thumb" />
      </button>
    </div>

    <!-- 压缩（预留） -->
    <div class="tunnel-settings__row tunnel-settings__row--reserved">
      <div class="tunnel-settings__row-text">
        <span class="tunnel-settings__row-label">
          {{ t('tunnel.settings.compression') }}
          <GBadge variant="neutral" type="soft" size="sm">
            {{ t('tunnel.settings.reserved') }}
          </GBadge>
        </span>
        <span class="tunnel-settings__row-hint">{{ t('tunnel.settings.compressionHint') }}</span>
      </div>
      <button
        type="button"
        class="tunnel-toggle"
        :class="{ 'tunnel-toggle--on': form.compression }"
        disabled>
        <span class="tunnel-toggle__thumb" />
      </button>
    </div>

    <!-- 加密（预留） -->
    <div class="tunnel-settings__row tunnel-settings__row--reserved">
      <div class="tunnel-settings__row-text">
        <span class="tunnel-settings__row-label">
          {{ t('tunnel.settings.encryption') }}
          <GBadge variant="neutral" type="soft" size="sm">
            {{ t('tunnel.settings.reserved') }}
          </GBadge>
        </span>
        <span class="tunnel-settings__row-hint">{{ t('tunnel.settings.encryptionHint') }}</span>
      </div>
      <button
        type="button"
        class="tunnel-toggle"
        :class="{ 'tunnel-toggle--on': form.encryption }"
        disabled>
        <span class="tunnel-toggle__thumb" />
      </button>
    </div>

    <!-- 保存按钮 -->
    <div class="tunnel-settings__actions">
      <GButton variant="ghost" icon="refresh" @click="reset">
        {{ t('common.reset') }}
      </GButton>
      <GButton
        variant="primary"
        icon="save"
        :loading="saving"
        :disabled="!isValid || !dirty"
        @click="handleSave">
        {{ t('tunnel.settings.save') }}
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GBadge from '@components/base/GBadge.vue'
import GInput from '@components/form/GInput.vue'
import GTextarea from '@components/form/GTextarea.vue'
import GFormField from '@components/form/GFormField.vue'
import TunnelProtocolSelect from './TunnelProtocolSelect.vue'
import TunnelPortInput from './TunnelPortInput.vue'
import type { Tunnel, TunnelFormData, TunnelProtocol } from '../types'
import { isValidPort } from '../utils'
import { useFeedback } from '@composables/useFeedback'

const props = defineProps<{ tunnel: Tunnel }>()

const emit = defineEmits<{ save: [id: string, patch: Partial<TunnelFormData>] }>()

const { toast } = useFeedback()
const { t } = useI18n()

interface SettingsForm {
  name: string
  protocol: TunnelProtocol
  localHost: string
  localPort: number | null
  remotePort: number | null
  host: string
  path: string
  remark: string
  autoStart: boolean
  compression: boolean
  encryption: boolean
}

const form = reactive<SettingsForm>({
  name: '',
  protocol: 'http',
  localHost: '127.0.0.1',
  localPort: null,
  remotePort: null,
  host: '',
  path: '/',
  remark: '',
  autoStart: false,
  compression: false,
  encryption: false,
})

const errors = reactive<{
  name?: string
  localHost?: string
  localPort?: string
  remotePort?: string
  host?: string
  path?: string
}>({})

const saving = ref(false)
let snapshot = ''

function syncForm() {
  form.name = props.tunnel.name
  form.protocol = props.tunnel.protocol
  form.localHost = props.tunnel.localHost
  form.localPort = props.tunnel.localPort
  form.remotePort = props.tunnel.remotePort
  form.host = props.tunnel.host ?? ''
  form.path = props.tunnel.path ?? '/'
  form.remark = props.tunnel.remark
  form.autoStart = props.tunnel.autoStart
  form.compression = props.tunnel.compression
  form.encryption = props.tunnel.encryption
  snapshot = JSON.stringify(form)
  errors.name = undefined
  errors.localHost = undefined
  errors.localPort = undefined
  errors.remotePort = undefined
  errors.host = undefined
  errors.path = undefined
}

watch(
  () => props.tunnel.id,
  () => syncForm(),
  { immediate: true },
)

const dirty = computed(() => JSON.stringify(form) !== snapshot)
const isHttpLike = computed(() => form.protocol === 'http' || form.protocol === 'https')

const isValid = computed(
  () =>
    form.name.trim().length >= 2 &&
    !errors.name &&
    !errors.localHost &&
    !errors.localPort &&
    !errors.remotePort &&
    !errors.host &&
    !errors.path &&
    isValidPort(form.localPort) &&
    isValidPort(form.remotePort) &&
    (form.protocol !== 'https' || Boolean(form.host.trim())),
)

function validateField(field: keyof typeof errors) {
  if (field === 'name') {
    const v = form.name.trim()
    if (v.length === 0) errors.name = t('tunnel.settings.validation.nameRequired')
    else if (v.length < 2) errors.name = t('tunnel.settings.validation.nameMin')
    else if (v.length > 40) errors.name = t('tunnel.settings.validation.nameMax')
    else errors.name = undefined
  }
  if (field === 'localHost') {
    if (!form.localHost.trim()) errors.localHost = t('tunnel.settings.validation.localHostRequired')
    else errors.localHost = undefined
  }
  if (field === 'localPort') {
    if (!isValidPort(form.localPort)) errors.localPort = t('tunnel.settings.validation.portRange')
    else errors.localPort = undefined
  }
  if (field === 'remotePort') {
    if (!isValidPort(form.remotePort)) errors.remotePort = t('tunnel.settings.validation.portRange')
    else errors.remotePort = undefined
  }
  if (field === 'host') {
    const value = form.host.trim()
    if (form.protocol === 'https' && !value) errors.host = t('tunnel.settings.validation.httpsHostRequired')
    else if (value && /[/:?#\s]/.test(value)) errors.host = t('tunnel.settings.validation.hostInvalid')
    else errors.host = undefined
  }
  if (field === 'path') {
    const value = form.path.trim()
    if (value && !value.startsWith('/')) errors.path = t('tunnel.settings.validation.pathPrefix')
    else errors.path = undefined
  }
}

function reset() {
  syncForm()
  toast.info(t('tunnel.settings.resetToast'))
}

function handleSave() {
  validateField('name')
  validateField('localHost')
  validateField('localPort')
  validateField('remotePort')
  validateField('host')
  validateField('path')
  if (!isValid.value) return
  saving.value = false
  emit('save', props.tunnel.id, {
    name: form.name,
    protocol: form.protocol,
    localHost: form.localHost,
    localPort: form.localPort,
    remotePort: form.remotePort,
    host: form.host,
    path: form.path,
    remark: form.remark,
    autoStart: form.autoStart,
  })
  snapshot = JSON.stringify(form)
  toast.success(t('tunnel.settings.savedToast', { name: form.name }))
}
</script>

<style scoped>
.tunnel-settings__actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
  padding-top: var(--space-2);
}

.tunnel-settings__row--reserved {
  opacity: 0.7;
}

.tunnel-settings__row--reserved .tunnel-settings__row-label {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
}

.tunnel-settings__row--reserved .tunnel-toggle {
  cursor: not-allowed;
}
</style>
