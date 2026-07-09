<!--
  ServerDeleteDialog — 删除确认对话框（危险操作）
  ------------------------------------------------------------------
  二次确认：必须输入服务器名称才能删除。
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
          class="server-delete"
          @click.stop
        >
          <header class="server-delete__header">
            <span class="server-delete__icon">
              <GIcon
                name="alert-triangle"
                :size="22"
              />
            </span>
            <h3 class="server-delete__title">
              删除服务器
            </h3>
          </header>

          <div class="server-delete__body">
            <p class="server-delete__warning">
              你即将删除服务器
              <strong class="server-delete__name">「{{ server?.name }}」</strong>
            </p>
            <ul class="server-delete__list">
              <li>该服务器下所有 Tunnel 将失去绑定资源</li>
              <li>当前 {{ server?.monitor.connections.active ?? 0 }} 个活动连接将被强制关闭</li>
              <li>所有流量统计与运行日志将被清除</li>
              <li>此操作<b>不可撤销</b></li>
            </ul>

            <div class="server-delete__confirm-box">
              <p class="server-delete__confirm-text">
                请输入服务器名称 <code>{{ server?.name }}</code> 以确认：
              </p>
              <GInput
                v-model="confirmText"
                placeholder="输入服务器名称"
                :state="confirmText && confirmText !== server?.name ? 'error' : 'normal'"
                clearable
              />
            </div>
          </div>

          <footer class="server-delete__footer">
            <GButton
              variant="ghost"
              @click="handleClose"
            >
              取消
            </GButton>
            <GButton
              variant="danger"
              icon="trash"
              :loading="deleting"
              :disabled="confirmText !== server?.name"
              @click="handleDelete"
            >
              确认删除
            </GButton>
          </footer>
        </div>
      </Transition>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, watch } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"
import GInput from "@components/form/GInput.vue"
import type { Server } from "../types"

const props = defineProps<{
  visible: boolean
  server: Server | null
}>()

const emit = defineEmits<{
  "update:visible": [value: boolean]
  confirm: [server: Server]
}>()

const confirmText = ref("")
const deleting = ref(false)

watch(
  () => props.visible,
  (v) => {
    if (v) {
      confirmText.value = ""
      deleting.value = false
    }
  },
)

function handleClose() {
  emit("update:visible", false)
}

function handleDelete() {
  if (confirmText.value !== props.server?.name) return
  deleting.value = false
  if (props.server) emit("confirm", props.server)
  emit("update:visible", false)
}
</script>
