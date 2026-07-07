<template>
  <footer class="log-statusbar">
    <span>{{ filtered }} / {{ total }} logs</span>
    <span>{{ paused ? "Paused" : "Runtime" }}</span>
    <span>Auto scroll: {{ autoScroll ? "On" : "Off" }}</span>
    <span v-if="dropped">Dropped: {{ dropped }}</span>
    <span v-if="selected">Selected: {{ selected }}</span>
  </footer>
</template>

<script setup lang="ts">
defineProps<{
  total: number
  filtered: number
  paused: boolean
  autoScroll: boolean
  dropped: number
  selected?: string
}>()
</script>
