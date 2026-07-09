<!--
  ServerDialog — 创建/编辑服务器对话框（Modern Dialog）
  ------------------------------------------------------------------
  统一处理创建与编辑两种模式。全部字段实时校验。
  字段：名称 / 类型 / 主机 / 端口 / Token / 地区 / 备注 / 标签 /
  心跳间隔 / 重连间隔 / 自动连接
-->
<template>
  <Transition name="dialog-fade">
    <div v-if="visible" class="server-dialog__overlay">
      <Transition name="dialog-pop" appear>
        <div v-if="visible" class="server-dialog" @click.stop>
          <!-- 头部 -->
          <header class="server-dialog__header">
            <div class="server-dialog__title-wrap">
              <span class="server-dialog__icon" :style="previewStyle">
                <GIcon :name="kindPreset.icon" :size="20" />
              </span>
              <div>
                <h3 class="server-dialog__title">
                  {{ isEdit ? t('server.dialog.editTitle') : t('server.dialog.addTitle') }}
                </h3>
                <p class="server-dialog__subtitle">
                  {{
                    isEdit
                      ? t('server.dialog.editSubtitle')
                      : t('server.dialog.addSubtitle')
                  }}
                </p>
              </div>
            </div>
            <GIconButton name="close" variant="ghost" size="sm" @click="handleClose" />
          </header>

          <!-- 主体表单 -->
          <div class="server-dialog__body">
            <!-- 名称 -->
            <GFormField :error="errors.name" required>
              <template #label>{{ t('server.dialog.name') }}</template>
              <GInput
                v-model="form.name"
                :placeholder="t('server.settings.namePlaceholder')"
                :state="errors.name ? 'error' : 'normal'"
                :maxlength="40"
                clearable
                @update:model-value="validateField('name')" />
            </GFormField>

            <!-- 类型 -->
            <GFormField>
              <template #label>{{ t('server.dialog.type') }}</template>
              <ServerKindSelect v-model="form.kind" />
            </GFormField>

            <!-- 主机 / 端口 -->
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

            <!-- 地区 -->
            <GFormField>
              <template #label>{{ t('server.dialog.region') }}</template>
              <GInput
                v-model="form.region"
                :placeholder="t('server.dialog.regionPlaceholder')"
                prefix="globe" />
            </GFormField>

            <!-- 心跳 / 重连间隔 -->
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
            <div class="server-dialog__row">
              <div class="server-dialog__row-text">
                <span class="server-dialog__row-label">{{ t('server.settings.autoConnect') }}</span>
                <span class="server-dialog__row-hint">{{ t('server.settings.autoConnectHint') }}</span>
              </div>
              <button
                type="button"
                class="server-toggle"
                :class="{ 'server-toggle--on': form.autoConnect }"
                @click="form.autoConnect = !form.autoConnect">
                <span class="server-toggle__thumb" />
              </button>
            </div>

            <!-- 标签 -->
            <GFormField>
              <template #label>{{ t('server.dialog.tags') }}</template>
              <div class="server-tag-input" :class="{ 'server-tag-input--focused': tagFocused }">
                <ServerTag
                  v-for="tag in form.tags"
                  :key="tag"
                  :name="tag"
                  removable
                  @remove="removeTag" />
                <input
                  v-model="tagInput"
                  class="server-tag-input__field"
                  :placeholder="t('server.dialog.tagPlaceholder')"
                  @focus="tagFocused = true"
                  @blur="onTagBlur"
                  @keydown.enter.prevent="addTag"
                  @keydown.backspace="onBackspace" />
              </div>
              <div class="server-tag-suggest">
                <button
                  v-for="tag in suggestedTags"
                  :key="tag.name"
                  type="button"
                  class="server-tag-suggest__chip"
                  :style="{ color: tag.color }"
                  @click="addSuggestedTag(tag.name)">
                  <GIcon name="plus" :size="10" />
                  {{ tag.label }}
                </button>
              </div>
            </GFormField>

            <!-- 备注 -->
            <GFormField>
              <template #label>{{ t('server.dialog.remark') }}</template>
              <GTextarea
                v-model="form.remark"
                :placeholder="t('server.settings.remarkPlaceholder')"
                :rows="2"
                :maxlength="200"
                resizable />
            </GFormField>
          </div>

          <!-- 底部 -->
          <footer class="server-dialog__footer">
            <GButton variant="ghost" @click="handleClose">
              {{ t('common.cancel') }}
            </GButton>
            <GButton
              variant="primary"
              :icon="isEdit ? 'save' : 'plus'"
              :loading="submitting"
              :disabled="!isValid"
              @click="handleSubmit">
              {{ isEdit ? t('common.save') : t('server.addServer') }}
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
import GPortInput from '@components/form/GPortInput.vue'
import GTextarea from '@components/form/GTextarea.vue'
import GFormField from '@components/form/GFormField.vue'
import ServerKindSelect from './ServerKindSelect.vue'
import ServerTag from './ServerTag.vue'
import type { Server, ServerFormData } from '../types'
import { KIND_MAP, SERVER_TAGS, isValidHost, isValidPort, isValidToken } from '../utils'
import { defaultServerForm } from '../store/server'

const props = defineProps<{
  visible: boolean
  /** 传入服务器则为编辑模式 */
  server?: Server | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  submit: [form: ServerFormData, isEdit: boolean]
}>()
const { t } = useI18n()

const isEdit = computed(() => !!props.server)
const submitting = ref(false)
const tagFocused = ref(false)
const tagInput = ref('')
const showToken = ref(false)

const form = reactive<ServerFormData>({
  ...defaultServerForm,
})

const errors = reactive<{
  name?: string
  host?: string
  port?: string
  token?: string
  heartbeatInterval?: string
  reconnectInterval?: string
}>({})

const kindPreset = computed(() => KIND_MAP[form.kind])

const previewStyle = computed(() => ({
  background: `${kindPreset.value.color}22`,
  color: kindPreset.value.color,
}))

const suggestedTags = computed(() =>
  SERVER_TAGS.filter((tag) => !form.tags.includes(tag.name))
    .slice(0, 6)
    .map((tag) => ({
      ...tag,
      label: t(`server.tags.${tag.name}`),
    })),
)

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
    !errors.token,
)

// 初始化 / 重置表单
watch(
  () => props.visible,
  (v) => {
    if (v) {
      if (props.server) {
        form.name = props.server.settings.name
        form.kind = props.server.kind
        form.host = props.server.settings.host
        form.port = props.server.settings.port
        form.token = props.server.settings.token
        form.region = props.server.region
        form.remark = props.server.settings.remark
        form.tags = [...props.server.tags]
        form.heartbeatInterval = props.server.settings.heartbeatInterval
        form.reconnectInterval = props.server.settings.reconnectInterval
        form.autoConnect = props.server.settings.autoConnect
      } else {
        resetForm()
      }
      clearErrors()
      showToken.value = false
    }
  },
  { immediate: true },
)

function resetForm() {
  Object.assign(form, defaultServerForm)
  tagInput.value = ''
}

function clearErrors() {
  errors.name = undefined
  errors.host = undefined
  errors.port = undefined
  errors.token = undefined
  errors.heartbeatInterval = undefined
  errors.reconnectInterval = undefined
}

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
  validateField('host')
  validateField('port')
  validateField('token')
  validateField('heartbeatInterval')
  validateField('reconnectInterval')
  if (!isValid.value) return
  submitting.value = false
  emit('submit', { ...form, tags: [...form.tags] }, isEdit.value)
  emit('update:visible', false)
}
</script>
