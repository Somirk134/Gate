<template>
  <button
    type="button"
    class="log-line"
    :class="[`log-line--${log.level.toLowerCase()}`, { 'log-line--active': active }]"
    @click="$emit('select', log)"
    @contextmenu.prevent="$emit('contextmenu', $event, log)"
  >
    <span class="log-line__time">{{ formatLogTime(log.timestamp) }}</span>
    <span class="log-line__level">
      <GIcon :name="level.icon" :size="13" />
      {{ log.level }}
    </span>
    <span class="log-line__source">{{ log.source }}</span>
    <span class="log-line__module">{{ log.module }}</span>
    <span class="log-line__message" v-html="highlightText(log.message, keyword)" />
    <span class="log-line__trace">{{ log.traceId ?? "-" }}</span>
    <span class="log-line__request">{{ log.requestId ?? "-" }}</span>
  </button>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import type { LogItem } from "../types"
import { getLevelOption } from "../mock"
import { formatLogTime, highlightText } from "../utils"

const props = defineProps<{
  log: LogItem
  keyword: string
  active: boolean
}>()

defineEmits<{
  select: [log: LogItem]
  contextmenu: [event: MouseEvent, log: LogItem]
}>()

const level = computed(() => getLevelOption(props.log.level))
</script>
