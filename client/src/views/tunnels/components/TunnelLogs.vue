<!--
  TunnelLogs — 工作区 Logs 标签
  ------------------------------------------------------------------
  IDE Console 风格。支持：颜色 / 等级 / 搜索 / 过滤 /
  暂停滚动 / 自动滚动 / 导出（预留）。全部 Mock。
-->
<template>
  <div class="tunnel-logs">
    <!-- 工具栏 -->
    <div class="tunnel-logs__toolbar">
      <GSearchInput
        :model-value="search"
        size="sm"
        placeholder="过滤日志…"
        class="tunnel-logs__search"
        @update:model-value="search = $event"
      />

      <div class="tunnel-logs__levels">
        <button
          v-for="lv in levels"
          :key="lv.key"
          type="button"
          class="tunnel-logs__level-btn"
          :class="[
            `tunnel-logs__level-btn--${lv.key}`,
            { 'tunnel-logs__level-btn--active': activeLevels.has(lv.key) },
          ]"
          @click="toggleLevel(lv.key)"
        >
          {{ lv.label }}
        </button>
      </div>

      <div class="tunnel-logs__spacer" />

      <GIconButton
        :name="autoScroll ? 'arrow-down' : 'minus'"
        size="sm"
        variant="ghost"
        :active="autoScroll"
        :tooltip="autoScroll ? '自动滚动：开' : '自动滚动：关'"
        @click="autoScroll = !autoScroll"
      />
      <GIconButton
        :name="paused ? 'play' : 'pause'"
        size="sm"
        variant="ghost"
        :active="paused"
        :tooltip="paused ? '已暂停滚动' : '点击暂停滚动'"
        @click="paused = !paused"
      />
      <GIconButton
        name="download"
        size="sm"
        variant="ghost"
        tooltip="导出日志（预留）"
        @click="$emit('export')"
      />
      <GIconButton
        name="trash"
        size="sm"
        variant="ghost"
        tooltip="清空"
        @click="$emit('clear')"
      />
    </div>

    <!-- 日志体 -->
    <div
      ref="bodyRef"
      class="tunnel-logs__body"
      @scroll="onScroll"
    >
      <div
        v-if="filteredLogs.length === 0"
        class="tunnel-logs__empty"
      >
        <GIcon
          name="file-text"
          :size="20"
        />
        <span>无匹配日志</span>
      </div>
      <div
        v-for="log in filteredLogs"
        :key="log.id"
        class="tunnel-log-line"
      >
        <span class="tunnel-log-line__time">{{ formatLogTime(log.timestamp) }}</span>
        <span
          class="tunnel-log-line__level"
          :class="`tunnel-log-line__level--${log.level}`"
        >
          {{ log.level }}
        </span>
        <span class="tunnel-log-line__source">[{{ log.source }}]</span>
        <span class="tunnel-log-line__msg">{{ log.message }}</span>
      </div>
    </div>

    <!-- 状态栏 -->
    <div class="tunnel-logs__statusbar">
      <span>{{ filteredLogs.length }} / {{ tunnel.logs.length }} 条</span>
      <span
        v-if="paused"
        class="tunnel-logs__paused"
      >
        <GIcon
          name="pause"
          :size="10"
        />
        已暂停
      </span>
      <span
        v-else
        class="tunnel-logs__live"
      >
        <GIcon
          name="circle"
          :size="8"
        />
        实时
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import GIconButton from "@components/base/GIconButton.vue"
import GSearchInput from "@components/form/GSearchInput.vue"
import type { Tunnel, TunnelLogLevel } from "../types"
import { formatLogTime } from "../utils"

const props = defineProps<{ tunnel: Tunnel }>()

defineEmits<{
  export: []
  clear: []
}>()

const levels: Array<{ key: TunnelLogLevel; label: string }> = [
  { key: "debug", label: "DEBUG" },
  { key: "info", label: "INFO" },
  { key: "success", label: "OK" },
  { key: "warn", label: "WARN" },
  { key: "error", label: "ERROR" },
]

const search = ref("")
const activeLevels = ref<Set<TunnelLogLevel>>(new Set(levels.map((l) => l.key)))
const autoScroll = ref(true)
const paused = ref(false)
const bodyRef = ref<HTMLElement | null>(null)

function toggleLevel(key: TunnelLogLevel) {
  if (activeLevels.value.has(key)) {
    activeLevels.value.delete(key)
  } else {
    activeLevels.value.add(key)
  }
  // 触发响应式更新
  activeLevels.value = new Set(activeLevels.value)
}

const filteredLogs = computed(() => {
  const q = search.value.trim().toLowerCase()
  return props.tunnel.logs.filter((log) => {
    if (!activeLevels.value.has(log.level)) return false
    if (q && !log.message.toLowerCase().includes(q) && !log.source.toLowerCase().includes(q)) {
      return false
    }
    return true
  })
})

function scrollToBottom() {
  if (!autoScroll.value || paused.value) return
  nextTick(() => {
    const el = bodyRef.value
    if (el) el.scrollTop = el.scrollHeight
  })
}

function onScroll() {
  const el = bodyRef.value
  if (!el) return
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 40
  if (!atBottom && autoScroll.value) {
    autoScroll.value = false
  } else if (atBottom && !autoScroll.value) {
    autoScroll.value = true
  }
}

watch(() => props.tunnel.logs.length, () => {
  if (!paused.value) scrollToBottom()
})

watch(() => props.tunnel.id, () => {
  scrollToBottom()
})

scrollToBottom()
</script>

<style scoped>
.tunnel-logs__search {
  width: 180px;
}

.tunnel-logs__levels {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.tunnel-logs__level-btn {
  height: 22px;
  padding: 0 var(--space-2);
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 10px;
  font-weight: var(--weight-semibold);
  border-radius: var(--radius-xs);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.tunnel-logs__level-btn:hover {
  color: var(--text-secondary);
}

.tunnel-logs__level-btn--active {
  border-color: currentColor;
}

.tunnel-logs__level-btn--debug.tunnel-logs__level-btn--active { color: var(--text-tertiary); }
.tunnel-logs__level-btn--info.tunnel-logs__level-btn--active { color: var(--color-info); }
.tunnel-logs__level-btn--success.tunnel-logs__level-btn--active { color: var(--color-success); }
.tunnel-logs__level-btn--warn.tunnel-logs__level-btn--active { color: var(--color-warning); }
.tunnel-logs__level-btn--error.tunnel-logs__level-btn--active { color: var(--color-error); }

.tunnel-logs__spacer {
  flex: 1;
}

.tunnel-logs__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-10);
  color: var(--text-tertiary);
}

.tunnel-logs__statusbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-1) var(--space-3);
  border-top: 1px solid var(--color-border-subtle);
  background: var(--bg-toolbar);
  font-size: 10px;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  flex-shrink: 0;
}

.tunnel-logs__live {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--color-success);
}

.tunnel-logs__paused {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--color-warning);
}
</style>
