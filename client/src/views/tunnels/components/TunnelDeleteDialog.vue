<!--
  TunnelDeleteDialog — 删除确认对话框（危险操作）
  ------------------------------------------------------------------
  二次确认：必须输入隧道名称才能删除。
-->
<template>
  <Transition name="dialog-fade">
    <div v-if="visible" class="tunnel-dialog__overlay" @click.self="handleClose">
      <Transition name="dialog-pop" appear>
        <div v-if="visible" class="tunnel-delete" @click.stop>
          <header class="tunnel-delete__header">
            <span class="tunnel-delete__icon">
              <GIcon name="alert-triangle" :size="22" />
            </span>
            <h3 class="tunnel-delete__title">删除隧道</h3>
          </header>

          <div class="tunnel-delete__body">
            <p class="tunnel-delete__warning">
              你即将删除隧道
              <strong class="tunnel-delete__name">「{{ tunnel?.name }}」</strong>
            </p>
            <ul class="tunnel-delete__list">
              <li>该隧道的公网访问将立即中断</li>
              <li>当前 {{ tunnel?.statistics.connections ?? 0 }} 个活动连接将被强制关闭</li>
              <li>所有流量统计与运行日志将被清除</li>
              <li>此操作<b>不可撤销</b></li>
            </ul>

            <div class="tunnel-delete__confirm-box">
              <p class="tunnel-delete__confirm-text">
                请输入隧道名称 <code>{{ tunnel?.name }}</code> 以确认：
              </p>
              <GInput
                v-model="confirmText"
                placeholder="输入隧道名称"
                :state="confirmText && confirmText !== tunnel?.name ? 'error' : 'normal'"
                clearable
              />
            </div>
          </div>

          <footer class="tunnel-delete__footer">
            <GButton variant="ghost" @click="handleClose">取消</GButton>
            <GButton
              variant="danger"
              icon="trash"
              :loading="deleting"
              :disabled="confirmText !== tunnel?.name"
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
import type { Tunnel } from "../types"

const props = defineProps<{
  visible: boolean
  tunnel: Tunnel | null
}>()

const emit = defineEmits<{
  "update:visible": [value: boolean]
  confirm: [tunnel: Tunnel]
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
  if (confirmText.value !== props.tunnel?.name) return
  deleting.value = false
  if (props.tunnel) emit("confirm", props.tunnel)
  emit("update:visible", false)
}
</script>

<style scoped>
.tunnel-delete {
  width: 460px;
  max-width: calc(100vw - 48px);
  background: var(--bg-surface-raised);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-dialog);
  box-shadow: var(--shadow-floating);
  overflow: hidden;
}

.tunnel-delete__header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-5) var(--space-5) var(--space-4);
  border-bottom: 1px solid var(--color-border-subtle);
}

.tunnel-delete__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  background: var(--color-error-muted);
  color: var(--color-error);
  flex-shrink: 0;
}

.tunnel-delete__title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
}

.tunnel-delete__body {
  padding: var(--space-4) var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.tunnel-delete__warning {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.tunnel-delete__name {
  color: var(--text-primary);
}

.tunnel-delete__list {
  list-style: none;
  padding: var(--space-3);
  background: var(--color-error-muted);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.tunnel-delete__list li {
  position: relative;
  padding-left: var(--space-4);
  font-size: var(--text-xs);
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.tunnel-delete__list li::before {
  content: "•";
  position: absolute;
  left: var(--space-1);
  color: var(--color-error);
}

.tunnel-delete__list li b {
  color: var(--color-error);
}

.tunnel-delete__confirm-box {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.tunnel-delete__confirm-text {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.tunnel-delete__confirm-text code {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  background: var(--bg-surface-hover);
  padding: 1px var(--space-1);
  border-radius: var(--radius-xs);
  color: var(--text-primary);
}

.tunnel-delete__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-5) var(--space-5);
  border-top: 1px solid var(--color-border-subtle);
}
</style>
