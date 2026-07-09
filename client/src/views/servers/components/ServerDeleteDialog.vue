<!--
  ServerDeleteDialog — 删除确认对话框（危险操作）
  ------------------------------------------------------------------
  二次确认：必须输入服务器名称才能删除。
-->
<template>
  <Transition name="dialog-fade">
    <div v-if="visible" class="server-dialog__overlay">
      <Transition name="dialog-pop" appear>
        <div v-if="visible" class="server-delete" @click.stop>
          <header class="server-delete__header">
            <span class="server-delete__icon">
              <GIcon name="alert-triangle" :size="22" />
            </span>
            <h3 class="server-delete__title">{{ t('server.deleteDialog.title') }}</h3>
          </header>

          <div class="server-delete__body">
            <p class="server-delete__warning">
              {{ t('server.deleteDialog.warning') }}
              <strong class="server-delete__name">「{{ server?.name }}」</strong>
            </p>
            <ul class="server-delete__list">
              <li>{{ t('server.deleteDialog.tunnelsUnbound') }}</li>
              <li>
                {{
                  t('server.deleteDialog.activeConnectionsClosed', {
                    count: server?.monitor.connections.active ?? 0,
                  })
                }}
              </li>
              <li>{{ t('server.deleteDialog.statsCleared') }}</li>
              <li>
                {{ t('server.deleteDialog.operation') }}<b>{{ t('server.deleteDialog.irreversible') }}</b>
              </li>
            </ul>

            <div class="server-delete__confirm-box">
              <p class="server-delete__confirm-text">
                {{ t('server.deleteDialog.confirmPrefix') }}
                <code>{{ server?.name }}</code>
                {{ t('server.deleteDialog.confirmSuffix') }}
              </p>
              <GInput
                v-model="confirmText"
                :placeholder="t('server.deleteDialog.placeholder')"
                :state="confirmText && confirmText !== server?.name ? 'error' : 'normal'"
                clearable />
            </div>
          </div>

          <footer class="server-delete__footer">
            <GButton variant="ghost" @click="handleClose">
              {{ t('common.cancel') }}
            </GButton>
            <GButton
              variant="danger"
              icon="trash"
              :loading="deleting"
              :disabled="confirmText !== server?.name"
              @click="handleDelete">
              {{ t('server.deleteDialog.confirm') }}
            </GButton>
          </footer>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GInput from '@components/form/GInput.vue'
import type { Server } from '../types'

const props = defineProps<{
  visible: boolean
  server: Server | null
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  confirm: [server: Server]
}>()
const { t } = useI18n()

const confirmText = ref('')
const deleting = ref(false)

watch(
  () => props.visible,
  (v) => {
    if (v) {
      confirmText.value = ''
      deleting.value = false
    }
  },
)

function handleClose() {
  emit('update:visible', false)
}

function handleDelete() {
  if (confirmText.value !== props.server?.name) return
  deleting.value = false
  if (props.server) emit('confirm', props.server)
  emit('update:visible', false)
}
</script>
