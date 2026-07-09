<template>
  <div
    v-if="visible"
    class="log-context-menu"
    :style="{ left: `${x}px`, top: `${y}px` }"
    @click.stop
  >
    <button
      type="button"
      @click="$emit('copy')"
    >
      <GIcon
        name="copy"
        :size="14"
      />Copy
    </button>
    <button
      type="button"
      @click="$emit('copy-message')"
    >
      <GIcon
        name="clipboard"
        :size="14"
      />Copy Message
    </button>
    <button
      type="button"
      @click="$emit('copy-json')"
    >
      <GIcon
        name="file-code"
        :size="14"
      />Copy JSON
    </button>
    <button
      type="button"
      @click="$emit('details')"
    >
      <GIcon
        name="info-circle"
        :size="14"
      />View Details
    </button>
    <hr>
    <button
      type="button"
      @click="$emit('delete')"
    >
      <GIcon
        name="trash"
        :size="14"
      />Delete
    </button>
    <button
      type="button"
      @click="$emit('clear')"
    >
      <GIcon
        name="close"
        :size="14"
      />Clear
    </button>
    <button
      type="button"
      @click="$emit('export')"
    >
      <GIcon
        name="download"
        :size="14"
      />Export
    </button>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"

defineProps<{
  visible: boolean
  x: number
  y: number
}>()

defineEmits<{
  copy: []
  "copy-message": []
  "copy-json": []
  details: []
  delete: []
  clear: []
  export: []
}>()
</script>
