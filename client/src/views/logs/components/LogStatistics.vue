<template>
  <section class="log-statistics" aria-label="Log statistics">
    <div v-for="item in items" :key="item.label" class="log-stat">
      <div class="log-stat__icon" :class="item.tone">
        <GIcon :name="item.icon" :size="15" />
      </div>
      <div class="log-stat__body">
        <span class="log-stat__label">{{ item.label }}</span>
        <strong>{{ item.value }}</strong>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import type { LogStatistics as LogStatisticsState } from "../types"

const props = defineProps<{ statistics: LogStatisticsState }>()

const items = computed(() => [
  { label: "Log Count", value: props.statistics.total, icon: "logs", tone: "is-info" },
  { label: "Error Count", value: props.statistics.error + props.statistics.fatal, icon: "alert-circle", tone: "is-error" },
  { label: "Warning Count", value: props.statistics.warning, icon: "alert-triangle", tone: "is-warn" },
  { label: "Info Count", value: props.statistics.info, icon: "info-circle", tone: "is-info" },
  { label: "Today", value: props.statistics.today, icon: "calendar", tone: "is-success" },
])
</script>
