<!--
  ServerLogs — 工作区 Logs 标签
  ------------------------------------------------------------------
  IDE Console 风格。支持：颜色 / 等级 / 搜索 / 过滤 /
  暂停滚动 / 自动滚动 / 导出（预留）。全部 Mock。
-->
<template>
  <div class="server-logs">
    <!-- 工具栏 -->
    <div class="server-logs__toolbar">
      <GSearchInput
        :model-value="search"
        size="sm"
        placeholder="过滤日志…"
        class="server-logs__search"
        @update:model-value="search = $event"
      />

      <div class="server-logs__levels">
        <button
          v-for="lv in levels"
          :key="lv.key"
          type="button"
          class="server-logs__level-btn"
          :class="[
            `server-logs__level-btn--${lv.key}`,
            { 'server-logs__level-btn--active': activeLevels.has(lv.key) },
          ]"
          @click="toggleLevel(lv.key)"
        >
          {{ lv.label }}
        </button>
      </div>

      <div class="server-logs__spacer" />

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
      <GIconButton name="download" size="sm" variant="ghost" tooltip="导出日志（预留）" @click="$emit('export')" />
      <GIconButton name="trash" size="sm" variant="ghost" tooltip="清空" @click="$emit('clear')" />
    </div>

    <!-- 日志体 -->
    <div ref="bodyRef" class="server-logs__body" @scroll="onScroll">
      <div v-if="filteredLogs.length === 0" class="server-logs__empty">
        <GIcon name="file-text" :size="20" />
        <span>无匹配日志</span>
      </div>
      <div
        v-for="log in filteredLogs"
        :key="log.id"
        class="server-log-line"
      >
        <span class="server-log-line__time">{{ formatLogTime(log.timestamp) }}</span>
        <span class="server-log-line__level" :class="`server-log-line__level--${log.level}`">
          {{ log.level }}
        </span>
        <span class="server-log-line__source">[{{ log.source }}]</span>
        <span class="server-log-line__msg">{{ log.message }}</span>
      </div>
    </div>

    <!-- 状态栏 -->
    <div class="server-logs__statusbar">
      <span>{{ filteredLogs.length }} / {{ server.logs.length }} 条</span>
      <span v-if="paused" class="server-logs__paused">
        <GIcon name="pause" :size="10" />
        已暂停
      </span>
      <span v-else class="server-logs__live">
        <GIcon name="circle" :size="8" />
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
import type { Server, ServerLogLevel } from "../types"
import { formatLogTime } from "../utils"

const props = defineProps<{ server: Server }>()

defineEmits<{
  export: []
  clear: []
}>()

const levels: Array<{ key: ServerLogLevel; label: string }> = [
  { key: "debug", label: "DEBUG" },
  { key: "info", label: "INFO" },
  { key: "success", label: "OK" },
  { key: "warn", label: "WARN" },
  { key: "error", label: "ERROR" },
]

const search = ref("")
const activeLevels = ref<Set<ServerLogLevel>>(new Set(levels.map((l) => l.key)))
const autoScroll = ref(true)
const paused = ref(false)
const bodyRef = ref<HTMLElement | null>(null)

function toggleLevel(key: ServerLogLevel) {
  if (activeLevels.value.has(key)) {
    activeLevels.value.delete(key)
  } else {
    activeLevels.value.add(key)
  }
  activeLevels.value = new Set(activeLevels.value)
}

const filteredLogs = computed(() => {
  const q = search.value.trim().toLowerCase()
  return props.server.logs.filter((log) => {
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

watch(() => props.server.logs.length, () => {
  if (!paused.value) scrollToBottom()
})

watch(() => props.server.id, () => {
  scrollToBottom()
})

scrollToBottom()
</script>
