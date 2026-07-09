<!--
  ServerDialog — 创建/编辑服务器对话框（Modern Dialog）
  ------------------------------------------------------------------
  统一处理创建与编辑两种模式。全部字段实时校验。
  字段：名称 / 类型 / 主机 / 端口 / Token / 地区 / 备注 / 标签 /
  心跳间隔 / 重连间隔 / 自动连接
-->
<template>
  <Transition name="dialog-fade">
    <div
      v-if="visible"
      class="server-dialog__overlay"
      @click.self="handleClose"
    >
      <Transition
        name="dialog-pop"
        appear
      >
        <div
          v-if="visible"
          class="server-dialog"
          @click.stop
        >
          <!-- 头部 -->
          <header class="server-dialog__header">
            <div class="server-dialog__title-wrap">
              <span
                class="server-dialog__icon"
                :style="previewStyle"
              >
                <GIcon
                  :name="kindPreset.icon"
                  :size="20"
                />
              </span>
              <div>
                <h3 class="server-dialog__title">
                  {{ isEdit ? "编辑服务器" : "Add Server" }}
                </h3>
                <p class="server-dialog__subtitle">
                  {{ isEdit ? "修改服务器配置，自动保存" : "填写服务器信息，连接后即可管理 Tunnel 资源" }}
                </p>
              </div>
            </div>
            <GIconButton
              name="close"
              variant="ghost"
              size="sm"
              @click="handleClose"
            />
          </header>

          <!-- 主体表单 -->
          <div class="server-dialog__body">
            <!-- 名称 -->
            <GFormField
              :error="errors.name"
              required
            >
              <template #label>
                名称
              </template>
              <GInput
                v-model="form.name"
                placeholder="例如：Tokyo Edge"
                :state="errors.name ? 'error' : 'normal'"
                :maxlength="40"
                clearable
                @update:model-value="validateField('name')"
              />
            </GFormField>

            <!-- 类型 -->
            <GFormField>
              <template #label>
                类型
              </template>
              <ServerKindSelect v-model="form.kind" />
            </GFormField>

            <!-- 主机 / 端口 -->
            <div class="server-port-row">
              <GFormField
                :error="errors.host"
                required
              >
                <template #label>
                  主机
                </template>
                <GInput
                  v-model="form.host"
                  placeholder="IP 或域名"
                  prefix="plug"
                  :state="errors.host ? 'error' : 'normal'"
                  @update:model-value="validateField('host')"
                />
              </GFormField>
              <GFormField
                :error="errors.port"
                required
              >
                <template #label>
                  端口
                </template>
                <GPortInput
                  :model-value="form.port"
                  @update:model-value="(v) => { form.port = v; validateField('port') }"
                />
              </GFormField>
            </div>

            <!-- Token -->
            <GFormField
              :error="errors.token"
              required
            >
              <template #label>
                Token
              </template>
              <GInput
                v-model="form.token"
                placeholder="服务器访问令牌"
                prefix="key"
                :state="errors.token ? 'error' : 'normal'"
                :type="showToken ? 'text' : 'password'"
                clearable
                @update:model-value="validateField('token')"
              >
                <template #suffix>
                  <GIconButton
                    :name="showToken ? 'eye-off' : 'eye'"
                    size="sm"
                    variant="ghost"
                    :tooltip="showToken ? '隐藏' : '显示'"
                    @click="showToken = !showToken"
                  />
                </template>
              </GInput>
            </GFormField>

            <!-- 地区 -->
            <GFormField>
              <template #label>
                地区
              </template>
              <GInput
                v-model="form.region"
                placeholder="例如：Tokyo, JP"
                prefix="globe"
              />
            </GFormField>

            <!-- 心跳 / 重连间隔 -->
            <div class="server-port-row">
              <GFormField :error="errors.heartbeatInterval">
                <template #label>
                  心跳间隔 (秒)
                </template>
                <GInput
                  v-model.number="form.heartbeatInterval"
                  type="number"
                  placeholder="30"
                  :state="errors.heartbeatInterval ? 'error' : 'normal'"
                  @update:model-value="validateField('heartbeatInterval')"
                />
              </GFormField>
              <GFormField :error="errors.reconnectInterval">
                <template #label>
                  重连间隔 (秒)
                </template>
                <GInput
                  v-model.number="form.reconnectInterval"
                  type="number"
                  placeholder="5"
                  :state="errors.reconnectInterval ? 'error' : 'normal'"
                  @update:model-value="validateField('reconnectInterval')"
                />
              </GFormField>
            </div>

            <!-- 自动连接 -->
            <div class="server-dialog__row">
              <div class="server-dialog__row-text">
                <span class="server-dialog__row-label">自动连接</span>
                <span class="server-dialog__row-hint">应用启动时自动连接该服务器</span>
              </div>
              <button
                type="button"
                class="server-toggle"
                :class="{ 'server-toggle--on': form.autoConnect }"
                @click="form.autoConnect = !form.autoConnect"
              >
                <span class="server-toggle__thumb" />
              </button>
            </div>

            <!-- 标签 -->
            <GFormField>
              <template #label>
                标签
              </template>
              <div
                class="server-tag-input"
                :class="{ 'server-tag-input--focused': tagFocused }"
              >
                <ServerTag
                  v-for="tag in form.tags"
                  :key="tag"
                  :name="tag"
                  removable
                  @remove="removeTag"
                />
                <input
                  v-model="tagInput"
                  class="server-tag-input__field"
                  placeholder="输入标签后回车"
                  @focus="tagFocused = true"
                  @blur="onTagBlur"
                  @keydown.enter.prevent="addTag"
                  @keydown.backspace="onBackspace"
                >
              </div>
              <div class="server-tag-suggest">
                <button
                  v-for="tag in suggestedTags"
                  :key="tag.name"
                  type="button"
                  class="server-tag-suggest__chip"
                  :style="{ color: tag.color }"
                  @click="addSuggestedTag(tag.name)"
                >
                  <GIcon
                    name="plus"
                    :size="10"
                  />
                  {{ tag.name }}
                </button>
              </div>
            </GFormField>

            <!-- 备注 -->
            <GFormField>
              <template #label>
                备注
              </template>
              <GTextarea
                v-model="form.remark"
                placeholder="内部备注，仅自己可见…"
                :rows="2"
                :maxlength="200"
                resizable
              />
            </GFormField>
          </div>

          <!-- 底部 -->
          <footer class="server-dialog__footer">
            <GButton
              variant="ghost"
              @click="handleClose"
            >
              取消
            </GButton>
            <GButton
              variant="primary"
              :icon="isEdit ? 'save' : 'plus'"
              :loading="submitting"
              :disabled="!isValid"
              @click="handleSubmit"
            >
              {{ isEdit ? "保存" : "添加服务器" }}
            </GButton>
          </footer>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"
import GIconButton from "@components/base/GIconButton.vue"
import GInput from "@components/form/GInput.vue"
import GPortInput from "@components/form/GPortInput.vue"
import GTextarea from "@components/form/GTextarea.vue"
import GFormField from "@components/form/GFormField.vue"
import ServerKindSelect from "./ServerKindSelect.vue"
import ServerTag from "./ServerTag.vue"
import type { Server, ServerFormData } from "../types"
import {
  KIND_MAP,
  SERVER_TAGS,
  isValidHost,
  isValidPort,
  isValidToken,
} from "../utils"
import { defaultServerForm } from "../store/server"

const props = defineProps<{
  visible: boolean
  /** 传入服务器则为编辑模式 */
  server?: Server | null
}>()

const emit = defineEmits<{
  "update:visible": [value: boolean]
  submit: [form: ServerFormData, isEdit: boolean]
}>()

const isEdit = computed(() => !!props.server)
const submitting = ref(false)
const tagFocused = ref(false)
const tagInput = ref("")
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
  SERVER_TAGS.filter((t) => !form.tags.includes(t.name)).slice(0, 6),
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
  tagInput.value = ""
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
  if (field === "name") {
    const v = form.name.trim()
    if (v.length === 0) errors.name = "名称不能为空"
    else if (v.length < 2) errors.name = "名称至少 2 个字符"
    else if (v.length > 40) errors.name = "名称不能超过 40 个字符"
    else errors.name = undefined
  }
  if (field === "host") {
    if (!form.host.trim()) errors.host = "主机不能为空"
    else if (!isValidHost(form.host)) errors.host = "无效的 IP 或域名"
    else errors.host = undefined
  }
  if (field === "port") {
    if (!isValidPort(form.port)) errors.port = "端口范围 1-65535"
    else errors.port = undefined
  }
  if (field === "token") {
    if (!form.token.trim()) errors.token = "Token 不能为空"
    else if (!isValidToken(form.token)) errors.token = "Token 至少 8 个字符"
    else errors.token = undefined
  }
  if (field === "heartbeatInterval") {
    if (form.heartbeatInterval < 1) errors.heartbeatInterval = "至少 1 秒"
    else if (form.heartbeatInterval > 300) errors.heartbeatInterval = "不能超过 300 秒"
    else errors.heartbeatInterval = undefined
  }
  if (field === "reconnectInterval") {
    if (form.reconnectInterval < 1) errors.reconnectInterval = "至少 1 秒"
    else if (form.reconnectInterval > 60) errors.reconnectInterval = "不能超过 60 秒"
    else errors.reconnectInterval = undefined
  }
}

function addTag() {
  const v = tagInput.value.trim()
  if (v && !form.tags.includes(v)) {
    form.tags.push(v)
  }
  tagInput.value = ""
}

function addSuggestedTag(name: string) {
  if (!form.tags.includes(name)) form.tags.push(name)
}

function removeTag(name: string) {
  const idx = form.tags.indexOf(name)
  if (idx !== -1) form.tags.splice(idx, 1)
}

function onBackspace() {
  if (tagInput.value === "" && form.tags.length > 0) {
    form.tags.pop()
  }
}

function onTagBlur() {
  tagFocused.value = false
  if (tagInput.value.trim()) addTag()
}

function handleClose() {
  emit("update:visible", false)
}

function handleSubmit() {
  validateField("name")
  validateField("host")
  validateField("port")
  validateField("token")
  validateField("heartbeatInterval")
  validateField("reconnectInterval")
  if (!isValid.value) return
  submitting.value = false
  emit("submit", { ...form, tags: [...form.tags] }, isEdit.value)
  emit("update:visible", false)
}
</script>
