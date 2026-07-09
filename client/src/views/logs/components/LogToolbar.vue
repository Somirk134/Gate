<template>
  <header class="log-toolbar">
    <LogSearch
      :model-value="filter.keyword"
      @update:model-value="patch({ keyword: $event })"
    />
    <LogFilter
      :filter="filter"
      @update:filter="$emit('update:filter', $event)"
    />
    <div class="log-toolbar__actions">
      <GIconButton
        name="arrow-down"
        :active="autoScroll"
        tooltip="Auto scroll"
        @click="$emit('toggle-auto-scroll')"
      />
      <GIconButton
        :name="paused ? 'play' : 'pause'"
        :active="paused"
        :tooltip="paused ? 'Resume' : 'Pause'"
        @click="togglePaused"
      />
      <GIconButton
        name="trash"
        tooltip="Clear logs"
        @click="$emit('clear')"
      />
      <GIconButton
        name="download"
        tooltip="Export logs"
        @click="$emit('export')"
      />
      <GIconButton
        name="copy"
        tooltip="Copy all"
        @click="$emit('copy-all')"
      />
      <GIconButton
        name="refresh"
        tooltip="Refresh"
        @click="$emit('refresh')"
      />
    </div>
  </header>
</template>

<script setup lang="ts">
import GIconButton from "@components/base/GIconButton.vue"
import LogFilter from "./LogFilter.vue"
import LogSearch from "./LogSearch.vue"
import type { LogFilter as LogFilterState } from "../types"

const props = defineProps<{
  filter: LogFilterState
  paused: boolean
  autoScroll: boolean
}>()

const emit = defineEmits<{
  "update:filter": [value: LogFilterState]
  "toggle-auto-scroll": []
  pause: []
  resume: []
  clear: []
  export: []
  "copy-all": []
  refresh: []
}>()

function patch(value: Partial<LogFilterState>) {
  emit("update:filter", { ...props.filter, ...value })
}

function togglePaused() {
  if (props.paused) emit("resume")
  else emit("pause")
}
</script>
