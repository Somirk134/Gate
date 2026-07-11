<!--
  TunnelDialog — 创建/编辑隧道对话框（Modern Dialog）
  ------------------------------------------------------------------
  统一处理创建与编辑两种模式。全部字段实时校验。
  字段：名称 / 协议 / 本地地址 / 本地端口 / 公网端口 /
  所属项目 / 所属服务器 / 自动启动 / 备注 / 标签
-->
<template>
  <Transition name="dialog-fade">
    <div v-if="visible" class="tunnel-dialog__overlay">
      <Transition name="dialog-pop" appear>
        <div v-if="visible" class="tunnel-dialog" @click.stop>
          <!-- 头部 -->
          <header class="tunnel-dialog__header">
            <div class="tunnel-dialog__title-wrap">
              <span class="tunnel-dialog__icon" :style="previewStyle">
                <GIcon :name="protocolPreset.icon" :size="20" />
              </span>
              <div>
                <h3 class="tunnel-dialog__title">
                  {{ isEdit ? t('tunnel.dialog.editTitle') : t('tunnel.dialog.createTitle') }}
                </h3>
                <p class="tunnel-dialog__subtitle">
                  {{
                    isEdit
                      ? t('tunnel.dialog.editSubtitle')
                      : t('tunnel.dialog.createSubtitle')
                  }}
                </p>
              </div>
            </div>
            <GIconButton name="close" variant="ghost" size="sm" @click="handleClose" />
          </header>

          <!-- 主体表单 -->
          <div class="tunnel-dialog__body">
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

            <div v-if="form.protocol === 'https'" class="tunnel-domain-panel">
              <p class="tunnel-domain-panel__hint">{{ t('tunnel.settings.subdomainModeHint') }}</p>
              <GFormField :error="errors.host" :required="form.protocol === 'https'">
                <template #label>{{ t('tunnel.settings.host') }}</template>
                <GInput
                  v-model="form.host"
                  placeholder="dev.example.com"
                  prefix="globe"
                  :state="errors.host ? 'error' : 'normal'"
                  @update:model-value="onHostChanged" />
                <template #hint>{{ t('tunnel.settings.subdomainInputHint') }}</template>
              </GFormField>
              <div v-if="previewAccessUrl" class="tunnel-domain-panel__preview">
                {{ t('tunnel.wizard.flow.accessPreview', { url: previewAccessUrl }) }}
              </div>
              <GFormField :error="errors.remotePort">
                <template #label>{{ t('tunnel.settings.standardPortLabel') }}</template>
                <TunnelPortInput v-model="form.remotePort" />
                <template v-if="isHttpLike" #hint>
                  {{ t('tunnel.wizard.flow.highPortHint') }}
                </template>
              </GFormField>
            </div>

            <!-- 本地地址 -->
            <GFormField :error="errors.localHost" required>
              <template #label>{{ t('tunnel.settings.localAddress') }}</template>
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
              <GFormField :error="errors.remotePort" :required="form.protocol === 'tcp' || form.protocol === 'http'">
                <template #label>{{ t('tunnel.settings.remotePort') }}</template>
                <TunnelPortInput v-model="form.remotePort" :disabled="form.protocol === 'https'" />
                <template v-if="form.protocol === 'http'" #hint>
                  {{ t('tunnel.wizard.flow.httpHighPortHint') }}
                </template>
              </GFormField>
            </div>

            <!-- 所属项目 / 服务器 -->
            <div class="tunnel-port-row">
              <GFormField>
                <template #label>{{ t('tunnel.dialog.project') }}</template>
                <div class="tunnel-dialog__select-wrap">
                  <select v-model="form.projectId" class="tunnel-dialog__select">
                    <option v-for="p in projects" :key="p.id" :value="p.id">
                      {{ p.name }}
                    </option>
                  </select>
                  <GIcon name="chevron-down" :size="14" class="tunnel-dialog__select-chevron" />
                </div>
              </GFormField>
              <GFormField>
                <template #label>{{ t('tunnel.dialog.server') }}</template>
                <div class="tunnel-dialog__select-wrap">
                  <select
                    v-model="form.serverName"
                    class="tunnel-dialog__select"
                    @change="form.serverId = ''">
                    <option v-for="s in serverNames" :key="s" :value="s">
                      {{ s }}
                    </option>
                  </select>
                  <GIcon name="chevron-down" :size="14" class="tunnel-dialog__select-chevron" />
                </div>
              </GFormField>
            </div>

            <!-- 自动启动 -->
            <div class="tunnel-dialog__row">
              <div class="tunnel-dialog__row-text">
                <span class="tunnel-dialog__row-label">{{ t('tunnel.settings.autoStart') }}</span>
                <span class="tunnel-dialog__row-hint">{{ t('tunnel.settings.autoStartHint') }}</span>
              </div>
              <button
                type="button"
                class="tunnel-toggle"
                :class="{ 'tunnel-toggle--on': form.autoStart }"
                @click="form.autoStart = !form.autoStart">
                <span class="tunnel-toggle__thumb" />
              </button>
            </div>

            <!-- 标签 -->
            <GFormField>
              <template #label>{{ t('tunnel.dialog.tags') }}</template>
              <div class="tunnel-tag-input" :class="{ 'tunnel-tag-input--focused': tagFocused }">
                <TunnelTag
                  v-for="tag in form.tags"
                  :key="tag"
                  :name="tag"
                  removable
                  @remove="removeTag" />
                <input
                  v-model="tagInput"
                  class="tunnel-tag-input__field"
                  :placeholder="t('tunnel.dialog.tagPlaceholder')"
                  @focus="tagFocused = true"
                  @blur="onTagBlur"
                  @keydown.enter.prevent="addTag"
                  @keydown.backspace="onBackspace" />
              </div>
              <div class="tunnel-tag-suggest">
                <button
                  v-for="tag in suggestedTags"
                  :key="tag.name"
                  type="button"
                  class="tunnel-tag-suggest__chip"
                  :style="{ color: tag.color }"
                  @click="addSuggestedTag(tag.name)">
                  <GIcon name="plus" :size="10" />
                  {{ tag.label }}
                </button>
              </div>
            </GFormField>

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
          </div>

          <!-- 底部 -->
          <footer class="tunnel-dialog__footer">
            <GButton variant="ghost" @click="handleClose">
              {{ t('common.cancel') }}
            </GButton>
            <GButton
              variant="primary"
              :icon="isEdit ? 'save' : 'plus'"
              :loading="submitting"
              :disabled="!isValid"
              @click="handleSubmit">
              {{ isEdit ? t('common.save') : t('tunnel.create') }}
            </GButton>
          </footer>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GIconButton from '@components/base/GIconButton.vue'
import GInput from '@components/form/GInput.vue'
import GTextarea from '@components/form/GTextarea.vue'
import GFormField from '@components/form/GFormField.vue'
import TunnelProtocolSelect from './TunnelProtocolSelect.vue'
import TunnelPortInput from './TunnelPortInput.vue'
import TunnelTag from './TunnelTag.vue'
import type { Tunnel, TunnelFormData } from '../types'
import { PROTOCOL_MAP, TUNNEL_TAGS, alignPublicPortOnProtocolSwitch, alignPublicPortWithProtocol, buildTunnelPublicUrl, isValidPort, standardPublicPort } from '../utils'
import { applySubdomainTunnelDefaults } from '../utils/domainAccess'

const props = defineProps<{
  visible: boolean
  /** 传入隧道则为编辑模式 */
  tunnel?: Tunnel | null
  projects: Array<{ id: string; name: string }>
  serverNames: string[]
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  submit: [form: TunnelFormData, isEdit: boolean]
}>()
const { t } = useI18n()

const isEdit = computed(() => !!props.tunnel)
const submitting = ref(false)
const tagFocused = ref(false)
const tagInput = ref('')

const form = reactive<TunnelFormData>({
  name: '',
  protocol: 'http',
  localHost: '127.0.0.1',
  localPort: null,
  remotePort: null,
  host: '',
  path: '/',
  projectId: props.projects[0]?.id ?? '',
  serverId: '',
  serverName: props.serverNames[0] ?? '',
  autoStart: false,
  remark: '',
  tags: [],
})

const errors = reactive<{
  name?: string
  localHost?: string
  localPort?: string
  remotePort?: string
  host?: string
  path?: string
}>({})

const protocolPreset = computed(() => PROTOCOL_MAP[form.protocol])
const isHttpLike = computed(() => form.protocol === 'http' || form.protocol === 'https')
const previewAccessUrl = computed(() =>
  isHttpLike.value
    ? buildTunnelPublicUrl({
        protocol: form.protocol,
        host: form.host,
        path: form.path,
        remotePort: form.remotePort,
      })
    : '',
)

const previewStyle = computed(() => ({
  background: `${protocolPreset.value.color}22`,
  color: protocolPreset.value.color,
}))

const suggestedTags = computed(() =>
  TUNNEL_TAGS.filter((tag) => !form.tags.includes(tag.name))
    .slice(0, 6)
    .map((tag) => ({
      ...tag,
      label: t(`tunnel.tags.${tag.name}`),
    })),
)

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
    (form.protocol === 'https'
      ? Boolean(form.host?.trim()) && isValidPort(form.remotePort)
      : isValidPort(form.remotePort)),
)

function clearFieldErrors() {
  errors.name = undefined
  errors.localHost = undefined
  errors.localPort = undefined
  errors.remotePort = undefined
  errors.host = undefined
  errors.path = undefined
}

function populateFormFromTunnel(tunnel: Tunnel) {
  form.name = tunnel.name
  form.protocol = tunnel.protocol
  form.localHost = tunnel.localHost
  form.localPort = tunnel.localPort
  form.remotePort = alignPublicPortWithProtocol(tunnel.protocol, tunnel.remotePort)
  form.host = tunnel.host ?? ''
  form.path = tunnel.path ?? '/'
  form.projectId = tunnel.projectId
  form.serverId = tunnel.serverId
  form.serverName = tunnel.serverName
  form.autoStart = tunnel.autoStart
  form.remark = tunnel.remark
  form.tags = [...tunnel.tags]
}

// 初始化 / 重置表单
watch(
  () => ({ visible: props.visible, tunnel: props.tunnel }),
  ({ visible, tunnel }) => {
    if (!visible) return
    if (tunnel) {
      populateFormFromTunnel(tunnel)
    } else {
      resetForm()
    }
    clearFieldErrors()
  },
  { immediate: true, deep: true },
)

function resetForm() {
  form.name = ''
  form.protocol = 'http'
  form.localHost = '127.0.0.1'
  form.localPort = null
  form.remotePort = null
  form.host = ''
  form.path = '/'
  form.projectId = props.projects[0]?.id ?? ''
  form.serverId = ''
  form.serverName = props.serverNames[0] ?? ''
  form.autoStart = false
  form.remark = ''
  form.tags = []
  tagInput.value = ''
}

function onHostChanged() {
  validateField('host')
  const host = form.host?.trim()
  if (!host || !isHttpLike.value) return
  const defaults = applySubdomainTunnelDefaults(form.protocol, host)
  form.host = defaults.host
  form.path = defaults.path
  if (!isValidPort(form.remotePort)) {
    form.remotePort = defaults.remotePort
  }
  validateField('remotePort')
}

watch(
  () => form.protocol,
  (protocol, previous) => {
    if (protocol === 'tcp') {
      form.path = ''
      return
    }
    if (protocol === 'http') {
      form.host = ''
    }
    form.path = '/'
    if (
      previous &&
      previous !== protocol &&
      (protocol === 'http' || protocol === 'https') &&
      (previous === 'http' || previous === 'https')
    ) {
      form.remotePort = alignPublicPortOnProtocolSwitch(protocol, previous, form.remotePort)
    }
    if (form.host?.trim()) onHostChanged()
  },
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
    if (!form.localHost.trim()) errors.localHost = t('tunnel.settings.validation.localAddressRequired')
    else errors.localHost = undefined
  }
  if (field === 'localPort') {
    if (!isValidPort(form.localPort)) errors.localPort = t('tunnel.settings.validation.portRange')
    else errors.localPort = undefined
  }
  if (field === 'remotePort') {
    if (form.protocol === 'https' && form.host?.trim() && !isValidPort(form.remotePort)) {
      form.remotePort = standardPublicPort(form.protocol)
      errors.remotePort = undefined
      return
    }
    if (!isValidPort(form.remotePort)) errors.remotePort = t('tunnel.settings.validation.portRange')
    else errors.remotePort = undefined
  }
  if (field === 'host') {
    const value = form.host?.trim() ?? ''
    if (form.protocol === 'https' && !value) errors.host = t('tunnel.settings.validation.httpsHostRequired')
    else if (value && /[/:?#\s]/.test(value)) errors.host = t('tunnel.settings.validation.hostInvalid')
    else errors.host = undefined
  }
  if (field === 'path') {
    const value = form.path?.trim() ?? ''
    if (value && !value.startsWith('/')) errors.path = t('tunnel.settings.validation.pathPrefix')
    else errors.path = undefined
  }
}

function addTag() {
  const v = tagInput.value.trim()
  if (v && !form.tags.includes(v)) {
    form.tags.push(v)
  }
  tagInput.value = ''
}

function addSuggestedTag(name: string) {
  if (!form.tags.includes(name)) form.tags.push(name)
}

function removeTag(name: string) {
  const idx = form.tags.indexOf(name)
  if (idx !== -1) form.tags.splice(idx, 1)
}

function onBackspace() {
  if (tagInput.value === '' && form.tags.length > 0) {
    form.tags.pop()
  }
}

function onTagBlur() {
  tagFocused.value = false
  if (tagInput.value.trim()) addTag()
}

function handleClose() {
  emit('update:visible', false)
}

function handleSubmit() {
  validateField('name')
  validateField('localHost')
  validateField('localPort')
  validateField('remotePort')
  validateField('host')
  validateField('path')
  if (!isValid.value) return
  submitting.value = false
  emit('submit', { ...form, tags: [...form.tags] }, isEdit.value)
  emit('update:visible', false)
}
</script>

<style scoped>
.tunnel-domain-panel {
  display: grid;
  gap: var(--space-3);
}

.tunnel-domain-panel__hint,
.tunnel-domain-panel__preview {
  margin: 0;
  color: var(--text-secondary);
  font-size: var(--text-sm);
  line-height: 1.5;
}

.tunnel-domain-panel__preview {
  padding: var(--space-3);
  border-radius: 8px;
  background: var(--bg-input);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}
</style>
