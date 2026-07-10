<template>
  <Transition name="cert-import">
    <div v-if="visible" class="import-backdrop" @keydown.esc="close">
      <section class="import-dialog" role="dialog" aria-modal="true" tabindex="-1">
        <header class="import-dialog__header">
          <div>
            <p>HTTPS / TLS</p>
            <h2>{{ t('certificate.importDialog.title') }}</h2>
          </div>
          <button type="button" class="import-dialog__close" @click="close">
            <GIcon name="close" :size="16" />
          </button>
        </header>

        <main class="import-dialog__body">
          <label class="import-field">
            <span>{{ t('certificate.importDialog.domainLabel') }}</span>
            <input
              v-model.trim="domain"
              autocomplete="off"
              :placeholder="t('certificate.importDialog.domainPlaceholder')" />
          </label>

          <div class="import-zone"
            :class="{ 'is-dragover': isDragover, 'has-file': !!certContent }"
            @dragover.prevent="isDragover = true"
            @dragleave.prevent="isDragover = false"
            @drop.prevent="handleDrop">
            <input
              ref="certInput"
              type="file"
              accept=".pem,.crt,.cer,.txt"
              class="import-zone__input"
              @change="handleCertSelect" />
            <div v-if="!certContent" class="import-zone__placeholder">
              <GIcon name="upload" :size="28" />
              <strong>{{ t('certificate.importDialog.dragHere') }}</strong>
              <small>{{ t('certificate.importDialog.certLabel') }} · {{ t('certificate.importDialog.supportedFormats') }}</small>
            </div>
            <div v-else class="import-zone__file">
              <GIcon name="file-text" :size="20" />
              <strong>{{ certFileName }}</strong>
              <small>{{ formatBytes(certContent.length) }}</small>
              <button type="button" @click="clearCert">
                <GIcon name="close" :size="14" />
              </button>
            </div>
          </div>

          <div class="import-zone import-zone--key"
            :class="{ 'is-dragover': isKeyDragover, 'has-file': !!keyContent }"
            @dragover.prevent="isKeyDragover = true"
            @dragleave.prevent="isKeyDragover = false"
            @drop.prevent="handleKeyDrop">
            <input
              ref="keyInput"
              type="file"
              accept=".key,.pem,.txt"
              class="import-zone__input"
              @change="handleKeySelect" />
            <div v-if="!keyContent" class="import-zone__placeholder">
              <GIcon name="key" :size="28" />
              <strong>{{ t('certificate.importDialog.dragHere') }}</strong>
              <small>{{ t('certificate.importDialog.keyLabel') }} · {{ t('certificate.importDialog.supportedFormats') }}</small>
            </div>
            <div v-else class="import-zone__file">
              <GIcon name="key" :size="20" />
              <strong>{{ keyFileName }}</strong>
              <small>{{ formatBytes(keyContent.length) }}</small>
              <button type="button" @click="clearKey">
                <GIcon name="close" :size="14" />
              </button>
            </div>
          </div>

          <!-- 验证结果 -->
          <div v-if="validation" class="validation-result" :class="{ 'is-valid': validation.valid, 'is-invalid': !validation.valid }">
            <div class="validation-result__header">
              <GIcon :name="validation.valid ? 'check-circle' : 'alert-circle'" :size="18" />
              <strong>{{ validation.valid ? t('certificate.importDialog.validationResult.valid') : t('certificate.importDialog.validationResult.invalid') }}</strong>
            </div>
            <dl v-if="validation.valid" class="validation-grid">
              <div>
                <dt>{{ t('certificate.importDialog.validationResult.commonName') }}</dt>
                <dd>{{ validation.commonName }}</dd>
              </div>
              <div>
                <dt>{{ t('certificate.importDialog.validationResult.issuer') }}</dt>
                <dd>{{ validation.issuer }}</dd>
              </div>
              <div>
                <dt>{{ t('certificate.importDialog.validationResult.algorithm') }}</dt>
                <dd>{{ validation.algorithm }}</dd>
              </div>
              <div>
                <dt>{{ t('certificate.importDialog.validationResult.daysRemaining') }}</dt>
                <dd :class="{ 'is-expired': validation.isExpired }">
                  {{ validation.daysRemaining }}
                  <small v-if="validation.isExpired">({{ t('certificate.importDialog.validationResult.expired') }})</small>
                </dd>
              </div>
              <div>
                <dt>{{ t('certificate.importDialog.validationResult.keyMatch') }}</dt>
                <dd>{{ validation.keyValid ? t('certificate.importDialog.validationResult.yes') : t('certificate.importDialog.validationResult.no') }}</dd>
              </div>
              <div>
                <dt>{{ t('certificate.importDialog.validationResult.tlsSupport') }}</dt>
                <dd>{{ validation.tlsSupported ? t('certificate.importDialog.validationResult.yes') : t('certificate.importDialog.validationResult.no') }}</dd>
              </div>
            </dl>
            <p v-if="!validation.valid" class="validation-error">{{ validationError }}</p>
          </div>
        </main>

        <footer class="import-dialog__footer">
          <GButton variant="ghost" @click="close">
            {{ t('certificate.close') }}
          </GButton>
          <div class="import-dialog__footer-right">
            <GButton
              variant="secondary"
              icon="shield-check"
              :loading="validating"
              :disabled="!canValidate"
              @click="validate">
              {{ t('certificate.importDialog.validate') }}
            </GButton>
            <GButton
              variant="primary"
              icon="upload"
              :loading="importing"
              :disabled="!canImport"
              @click="doImport">
              {{ t('certificate.importDialog.import') }}
            </GButton>
          </div>
        </footer>
      </section>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import { useFeedback } from '@composables/useFeedback'
import { certificateService } from '../service'
import type { ImportValidation } from '../types'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{
  'update:visible': [value: boolean]
  imported: [domain: string]
}>()

const { t } = useI18n()
const { toast, notify } = useFeedback()

const domain = ref('')
const certContent = ref('')
const certFileName = ref('')
const keyContent = ref('')
const keyFileName = ref('')
const isDragover = ref(false)
const isKeyDragover = ref(false)
const validating = ref(false)
const importing = ref(false)
const validation = ref<ImportValidation | null>(null)
const validationError = ref('')

const certInput = ref<HTMLInputElement | null>(null)
const keyInput = ref<HTMLInputElement | null>(null)

const canValidate = computed(() => !!certContent.value && !!keyContent.value)
const canImport = computed(() => !!domain.value && !!certContent.value && !!keyContent.value && validation.value?.valid)

watch(
  () => props.visible,
  (val) => {
    if (val) {
      domain.value = ''
      certContent.value = ''
      certFileName.value = ''
      keyContent.value = ''
      keyFileName.value = ''
      validation.value = null
      validationError.value = ''
    }
  },
)

function close() {
  emit('update:visible', false)
}

function handleCertSelect(e: Event) {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) readFile(file, 'cert')
}

function handleKeySelect(e: Event) {
  const target = e.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) readFile(file, 'key')
}

function handleDrop(e: DragEvent) {
  isDragover.value = false
  const file = e.dataTransfer?.files?.[0]
  if (file) readFile(file, 'cert')
}

function handleKeyDrop(e: DragEvent) {
  isKeyDragover.value = false
  const file = e.dataTransfer?.files?.[0]
  if (file) readFile(file, 'key')
}

function readFile(file: File, type: 'cert' | 'key') {
  const reader = new FileReader()
  reader.onload = () => {
    const content = String(reader.result || '')
    if (type === 'cert') {
      certContent.value = content
      certFileName.value = file.name
      if (!domain.value) {
        const baseName = file.name.replace(/\.(pem|crt|cer|txt)$/i, '')
        domain.value = baseName
      }
    } else {
      keyContent.value = content
      keyFileName.value = file.name
    }
    validation.value = null
  }
  reader.readAsText(file)
}

function clearCert() {
  certContent.value = ''
  certFileName.value = ''
  validation.value = null
  if (certInput.value) certInput.value.value = ''
}

function clearKey() {
  keyContent.value = ''
  keyFileName.value = ''
  validation.value = null
  if (keyInput.value) keyInput.value.value = ''
}

async function validate() {
  validating.value = true
  validationError.value = ''
  try {
    validation.value = await certificateService.validateImport(
      certContent.value,
      keyContent.value,
    )
  } catch (err) {
    validationError.value = err instanceof Error ? err.message : String(err)
  } finally {
    validating.value = false
  }
}

async function doImport() {
  importing.value = true
  try {
    await certificateService.importCertificate({
      domain: domain.value,
      certificatePem: certContent.value,
      privateKeyPem: keyContent.value,
    })
    toast.success(t('certificate.notifications.imported', { domain: domain.value }))
    emit('imported', domain.value)
    close()
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err)
    notify.error(t('certificate.notifications.importFailed'), msg, 10000)
  } finally {
    importing.value = false
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  return `${(bytes / 1024).toFixed(1)} KB`
}
</script>

<style scoped>
.import-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-modal);
  display: grid;
  place-items: center;
  background: var(--color-overlay);
  backdrop-filter: blur(12px);
}

.import-dialog {
  width: min(680px, calc(100vw - 48px));
  max-height: min(720px, calc(100vh - 48px));
  display: grid;
  grid-template-rows: auto minmax(0, 1fr) auto;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  background: var(--bg-surface-raised);
  box-shadow: var(--shadow-floating);
}

.import-dialog__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: var(--space-5) var(--space-5) var(--space-3);
}

.import-dialog__header p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  text-transform: uppercase;
  font-weight: var(--weight-semibold);
}

.import-dialog__header h2 {
  margin-top: 2px;
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
}

.import-dialog__close {
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
}

.import-dialog__close:hover {
  color: var(--text-primary);
  border-color: var(--border-default);
}

.import-dialog__body {
  overflow-y: auto;
  padding: 0 var(--space-5);
  display: grid;
  gap: var(--space-4);
}

.import-field {
  display: grid;
  gap: var(--space-2);
}

.import-field > span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.import-field input {
  height: 38px;
  padding: 0 var(--space-3);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.import-field input:focus {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
  outline: 0;
}

.import-zone {
  position: relative;
  min-height: 100px;
  display: grid;
  place-content: center;
  border: 1px dashed var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-out), background var(--duration-fast) var(--ease-out);
}

.import-zone:hover {
  border-color: var(--color-primary);
}

.import-zone.is-dragover {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
}

.import-zone.has-file {
  border-style: solid;
  border-color: var(--border-subtle);
}

.import-zone__input {
  position: absolute;
  inset: 0;
  opacity: 0;
  cursor: pointer;
}

.import-zone__placeholder {
  display: grid;
  place-items: center;
  gap: var(--space-1);
  color: var(--text-tertiary);
  text-align: center;
  padding: var(--space-4);
}

.import-zone__placeholder strong {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.import-zone__placeholder small {
  font-size: var(--text-xs);
}

.import-zone__file {
  display: grid;
  grid-template-columns: 24px minmax(0, 1fr) auto 28px;
  align-items: center;
  gap: var(--space-3);
  width: 100%;
  padding: var(--space-3) var(--space-4);
  color: var(--text-primary);
}

.import-zone__file strong {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--text-sm);
}

.import-zone__file small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.import-zone__file button {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.import-zone__file button:hover {
  background: var(--bg-surface-hover);
  color: var(--color-error);
}

.validation-result {
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.validation-result.is-valid {
  border-color: var(--color-success);
}

.validation-result.is-invalid {
  border-color: var(--color-error);
}

.validation-result__header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.validation-result.is-valid .validation-result__header { color: var(--color-success); }
.validation-result.is-invalid .validation-result__header { color: var(--color-error); }

.validation-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.validation-grid div {
  display: grid;
  gap: 2px;
}

.validation-grid dt {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.validation-grid dd {
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.validation-grid dd.is-expired {
  color: var(--color-error);
}

.validation-error {
  color: var(--color-error);
  font-size: var(--text-sm);
}

.import-dialog__footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) var(--space-5) var(--space-5);
}

.import-dialog__footer-right {
  display: flex;
  gap: var(--space-2);
}

.cert-import-enter-active,
.cert-import-leave-active {
  transition: opacity var(--duration-base) var(--ease-out);
}

.cert-import-enter-from,
.cert-import-leave-to {
  opacity: 0;
}
</style>
