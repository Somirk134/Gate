<!--
  ServerSettings — 工作区 Settings 标签
  ------------------------------------------------------------------
  支持：Server Name / Host / Port / Token / Remark /
  Heartbeat Interval / Reconnect Interval / Auto Connect
  全部实时校验。保存后回写 store。
-->
<template>
  <div class="server-settings">
    <!-- 名称 -->
    <GFormField :error="errors.name" required>
      <template #label>{{ t('server.settings.name') }}</template>
      <GInput
        v-model="form.name"
        :placeholder="t('server.settings.namePlaceholder')"
        :state="errors.name ? 'error' : 'normal'"
        :maxlength="40"
        clearable
        @update:model-value="validateField('name')" />
    </GFormField>

    <!-- Host / Port -->
    <div class="server-port-row">
      <GFormField :error="errors.host" required>
        <template #label>{{ t('server.settings.host') }}</template>
        <GInput
          v-model="form.host"
          :placeholder="t('server.settings.hostPlaceholder')"
          prefix="plug"
          :state="errors.host ? 'error' : 'normal'"
          @update:model-value="validateField('host')" />
      </GFormField>
      <GFormField :error="errors.port" required>
        <template #label>{{ t('server.settings.port') }}</template>
        <GPortInput
          :model-value="form.port"
          @update:model-value="
            (v) => {
              form.port = v
              validateField('port')
            }
          " />
      </GFormField>
    </div>

    <!-- Token -->
    <GFormField :error="errors.token" required>
      <template #label> Token </template>
      <GInput
        v-model="form.token"
        :placeholder="t('server.settings.tokenPlaceholder')"
        prefix="key"
        :state="errors.token ? 'error' : 'normal'"
        :type="showToken ? 'text' : 'password'"
        clearable
        @update:model-value="validateField('token')">
        <template #suffix>
          <GIconButton
            :name="showToken ? 'eye-off' : 'eye'"
            size="sm"
            variant="ghost"
            :tooltip="showToken ? t('form.hideSecret') : t('form.showSecret')"
            @click="showToken = !showToken" />
        </template>
      </GInput>
    </GFormField>

    <!-- 备注 -->
    <GFormField>
      <template #label>{{ t('server.settings.remark') }}</template>
      <GTextarea
        v-model="form.remark"
        :placeholder="t('server.settings.remarkPlaceholder')"
        :rows="2"
        :maxlength="200"
        resizable />
    </GFormField>

    <!-- 心跳间隔 / 重连间隔 -->
    <div class="server-port-row">
      <GFormField :error="errors.heartbeatInterval">
        <template #label>{{ t('server.settings.heartbeatInterval') }}</template>
        <GInput
          v-model.number="form.heartbeatInterval"
          type="number"
          placeholder="30"
          :state="errors.heartbeatInterval ? 'error' : 'normal'"
          @update:model-value="validateField('heartbeatInterval')" />
      </GFormField>
      <GFormField :error="errors.reconnectInterval">
        <template #label>{{ t('server.settings.reconnectInterval') }}</template>
        <GInput
          v-model.number="form.reconnectInterval"
          type="number"
          placeholder="5"
          :state="errors.reconnectInterval ? 'error' : 'normal'"
          @update:model-value="validateField('reconnectInterval')" />
      </GFormField>
    </div>

    <!-- 自动连接 -->
    <div class="server-settings__row">
      <div class="server-settings__row-text">
        <span class="server-settings__row-label">{{ t('server.settings.autoConnect') }}</span>
        <span class="server-settings__row-hint">{{ t('server.settings.autoConnectHint') }}</span>
      </div>
      <button
        type="button"
        class="server-toggle"
        :class="{ 'server-toggle--on': form.autoConnect }"
        @click="form.autoConnect = !form.autoConnect">
        <span class="server-toggle__thumb" />
      </button>
    </div>

    <!-- 保存按钮 -->
    <div class="server-settings__actions">
      <GButton variant="ghost" icon="refresh" @click="reset">
        {{ t('common.reset') }}
      </GButton>
      <GButton
        variant="primary"
        icon="save"
        :loading="saving"
        :disabled="!isValid || !dirty"
        @click="handleSave">
        {{ t('server.settings.save') }}
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GIconButton from '@components/base/GIconButton.vue'
import GInput from '@components/form/GInput.vue'
import GPortInput from '@components/form/GPortInput.vue'
import GTextarea from '@components/form/GTextarea.vue'
import GFormField from '@components/form/GFormField.vue'
import type { Server, ServerFormData } from '../types'
import { isValidHost, isValidPort, isValidToken } from '../utils'
import { useFeedback } from '@composables/useFeedback'

const props = defineProps<{ server: Server }>()

const emit = defineEmits<{ save: [id: string, patch: Partial<ServerFormData>] }>()

const { toast } = useFeedback()
const { t } = useI18n()

interface SettingsForm {
  name: string
  host: string
  port: number | null
  token: string
  remark: string
  heartbeatInterval: number
  reconnectInterval: number
  autoConnect: boolean
}

const form = reactive<SettingsForm>({
  name: '',
  host: '',
  port: null,
  token: '',
  remark: '',
  heartbeatInterval: 30,
  reconnectInterval: 5,
  autoConnect: false,
})

const errors = reactive<{
  name?: string
  host?: string
  port?: string
  token?: string
  heartbeatInterval?: string
  reconnectInterval?: string
}>({})

const saving = ref(false)
const showToken = ref(false)
let snapshot = ''

function syncForm() {
  form.name = props.server.settings.name
  form.host = props.server.settings.host
  form.port = props.server.settings.port
  form.token = props.server.settings.token
  form.remark = props.server.settings.remark
  form.heartbeatInterval = props.server.settings.heartbeatInterval
  form.reconnectInterval = props.server.settings.reconnectInterval
  form.autoConnect = props.server.settings.autoConnect
  snapshot = JSON.stringify(form)
  errors.name = undefined
  errors.host = undefined
  errors.port = undefined
  errors.token = undefined
  errors.heartbeatInterval = undefined
  errors.reconnectInterval = undefined
}

watch(
  () => props.server.id,
  () => syncForm(),
  { immediate: true },
)

const dirty = computed(() => JSON.stringify(form) !== snapshot)

const isValid = computed(
  () =>
    form.name.trim().length >= 2 &&
    isValidHost(form.host) &&
    isValidPort(form.port) &&
    isValidToken(form.token) &&
    form.heartbeatInterval >= 1 &&
    form.reconnectInterval >= 1 &&
    !errors.name &&
    !errors.host &&
    !errors.port &&
    !errors.token &&
    !errors.heartbeatInterval &&
    !errors.reconnectInterval,
)

function validateField(field: keyof typeof errors) {
  if (field === 'name') {
    const v = form.name.trim()
    if (v.length === 0) errors.name = t('server.settings.validation.nameRequired')
    else if (v.length < 2) errors.name = t('server.settings.validation.nameMin')
    else if (v.length > 40) errors.name = t('server.settings.validation.nameMax')
    else errors.name = undefined
  }
  if (field === 'host') {
    if (!form.host.trim()) errors.host = t('server.settings.validation.hostRequired')
    else if (!isValidHost(form.host)) errors.host = t('server.settings.validation.hostInvalid')
    else errors.host = undefined
  }
  if (field === 'port') {
    if (!isValidPort(form.port)) errors.port = t('server.settings.validation.portRange')
    else errors.port = undefined
  }
  if (field === 'token') {
    if (!form.token.trim()) errors.token = t('server.settings.validation.tokenRequired')
    else if (!isValidToken(form.token)) errors.token = t('server.settings.validation.tokenMin')
    else errors.token = undefined
  }
  if (field === 'heartbeatInterval') {
    if (form.heartbeatInterval < 1) errors.heartbeatInterval = t('server.settings.validation.minOneSecond')
    else if (form.heartbeatInterval > 300) errors.heartbeatInterval = t('server.settings.validation.max300Seconds')
    else errors.heartbeatInterval = undefined
  }
  if (field === 'reconnectInterval') {
    if (form.reconnectInterval < 1) errors.reconnectInterval = t('server.settings.validation.minOneSecond')
    else if (form.reconnectInterval > 60) errors.reconnectInterval = t('server.settings.validation.max60Seconds')
    else errors.reconnectInterval = undefined
  }
}

function reset() {
  syncForm()
  toast.info(t('server.settings.resetToast'))
}

function handleSave() {
  validateField('name')
  validateField('host')
  validateField('port')
  validateField('token')
  validateField('heartbeatInterval')
  validateField('reconnectInterval')
  if (!isValid.value) return
  saving.value = false
  emit('save', props.server.id, {
    name: form.name,
    host: form.host,
    port: form.port,
    token: form.token,
    remark: form.remark,
    heartbeatInterval: form.heartbeatInterval,
    reconnectInterval: form.reconnectInterval,
    autoConnect: form.autoConnect,
  })
  snapshot = JSON.stringify(form)
  toast.success(t('server.settings.savedToast', { name: form.name }))
}
</script>
