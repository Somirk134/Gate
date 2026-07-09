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
      <template #label> 隧道名称 </template>
      <GInput
        v-model="form.name"
        placeholder="例如：api-gateway"
        :state="errors.name ? 'error' : 'normal'"
        :maxlength="40"
        clearable
        @update:model-value="validateField('name')" />
    </GFormField>

    <!-- 协议 -->
    <GFormField>
      <template #label> 协议 </template>
      <TunnelProtocolSelect v-model="form.protocol" />
    </GFormField>

    <!-- 本地主机 -->
    <GFormField :error="errors.localHost">
      <template #label> 本地主机 </template>
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
        <template #label> 本地端口 </template>
        <TunnelPortInput v-model="form.localPort" />
      </GFormField>
      <GFormField :error="errors.remotePort" required>
        <template #label> 公网端口 </template>
        <TunnelPortInput v-model="form.remotePort" />
      </GFormField>
    </div>

    <!-- 备注 -->
    <GFormField>
      <template #label> 备注 </template>
      <GTextarea
        v-model="form.remark"
        placeholder="内部备注，仅自己可见…"
        :rows="2"
        :maxlength="200"
        resizable />
    </GFormField>

    <!-- 自动启动 -->
    <div class="tunnel-settings__row">
      <div class="tunnel-settings__row-text">
        <span class="tunnel-settings__row-label">自动启动</span>
        <span class="tunnel-settings__row-hint">应用启动时自动运行该隧道</span>
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
          压缩
          <GBadge variant="neutral" type="soft" size="sm">预留</GBadge>
        </span>
        <span class="tunnel-settings__row-hint">启用数据压缩，降低带宽占用（即将支持）</span>
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
          加密
          <GBadge variant="neutral" type="soft" size="sm">预留</GBadge>
        </span>
        <span class="tunnel-settings__row-hint">端到端加密传输（即将支持）</span>
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
      <GButton variant="ghost" icon="refresh" @click="reset"> 重置 </GButton>
      <GButton
        variant="primary"
        icon="save"
        :loading="saving"
        :disabled="!isValid || !dirty"
        @click="handleSave">
        保存设置
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
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

interface SettingsForm {
  name: string
  protocol: TunnelProtocol
  localHost: string
  localPort: number | null
  remotePort: number | null
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
}>({})

const saving = ref(false)
let snapshot = ''

function syncForm() {
  form.name = props.tunnel.name
  form.protocol = props.tunnel.protocol
  form.localHost = props.tunnel.localHost
  form.localPort = props.tunnel.localPort
  form.remotePort = props.tunnel.remotePort
  form.remark = props.tunnel.remark
  form.autoStart = props.tunnel.autoStart
  form.compression = props.tunnel.compression
  form.encryption = props.tunnel.encryption
  snapshot = JSON.stringify(form)
  errors.name = undefined
  errors.localHost = undefined
  errors.localPort = undefined
  errors.remotePort = undefined
}

watch(
  () => props.tunnel.id,
  () => syncForm(),
  { immediate: true },
)

const dirty = computed(() => JSON.stringify(form) !== snapshot)

const isValid = computed(
  () =>
    form.name.trim().length >= 2 &&
    !errors.name &&
    !errors.localHost &&
    !errors.localPort &&
    !errors.remotePort &&
    isValidPort(form.localPort) &&
    isValidPort(form.remotePort),
)

function validateField(field: keyof typeof errors) {
  if (field === 'name') {
    const v = form.name.trim()
    if (v.length === 0) errors.name = '名称不能为空'
    else if (v.length < 2) errors.name = '名称至少 2 个字符'
    else if (v.length > 40) errors.name = '名称不能超过 40 个字符'
    else errors.name = undefined
  }
  if (field === 'localHost') {
    if (!form.localHost.trim()) errors.localHost = '本地主机不能为空'
    else errors.localHost = undefined
  }
  if (field === 'localPort') {
    if (!isValidPort(form.localPort)) errors.localPort = '端口范围 1-65535'
    else errors.localPort = undefined
  }
  if (field === 'remotePort') {
    if (!isValidPort(form.remotePort)) errors.remotePort = '端口范围 1-65535'
    else errors.remotePort = undefined
  }
}

function reset() {
  syncForm()
  toast.info('已重置未保存的更改')
}

function handleSave() {
  validateField('name')
  validateField('localHost')
  validateField('localPort')
  validateField('remotePort')
  if (!isValid.value) return
  saving.value = false
  emit('save', props.tunnel.id, {
    name: form.name,
    protocol: form.protocol,
    localHost: form.localHost,
    localPort: form.localPort,
    remotePort: form.remotePort,
    remark: form.remark,
    autoStart: form.autoStart,
  })
  snapshot = JSON.stringify(form)
  toast.success(`隧道「${form.name}」设置已保存`)
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
