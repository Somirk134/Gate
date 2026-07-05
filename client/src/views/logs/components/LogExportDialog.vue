<template>
  <div v-if="visible" class="log-dialog-backdrop" @click.self="$emit('close')">
    <section class="log-export-dialog">
      <header>
        <h2>Export Logs</h2>
        <button type="button" @click="$emit('close')">
          <GIcon name="close" :size="16" />
        </button>
      </header>
      <p>{{ count }} logs will be exported from the current filtered result.</p>
      <div class="log-export-dialog__formats">
        <button v-for="item in formats" :key="item.value" type="button" :class="{ active: format === item.value }" @click="format = item.value">
          <GIcon :name="item.icon" :size="16" />
          <span>{{ item.label }}</span>
        </button>
      </div>
      <footer>
        <GButton variant="secondary" @click="$emit('close')">Cancel</GButton>
        <GButton variant="primary" icon="download" @click="$emit('export', format)">Export</GButton>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"
import type { LogExportFormat } from "../utils"

defineProps<{
  visible: boolean
  count: number
}>()

defineEmits<{
  close: []
  export: [format: LogExportFormat]
}>()

const format = ref<LogExportFormat>("json")
const formats: Array<{ value: LogExportFormat; label: string; icon: string }> = [
  { value: "json", label: "JSON", icon: "file-code" },
  { value: "ndjson", label: "NDJSON", icon: "code" },
  { value: "txt", label: "Text", icon: "file-text" },
]
</script>
