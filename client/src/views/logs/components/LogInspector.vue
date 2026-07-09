<template>
  <aside class="log-inspector">
    <template v-if="log">
      <header class="log-inspector__header">
        <div>
          <span class="log-inspector__eyebrow">{{ log.source }} / {{ log.module }}</span>
          <h2>{{ log.level }}</h2>
        </div>
        <button
          type="button"
          class="log-inspector__level"
          :class="`is-${log.level.toLowerCase()}`"
        >
          {{ log.level }}
        </button>
      </header>

      <div class="log-inspector__message">
        {{ log.message }}
      </div>

      <div class="log-inspector__tabs">
        <button
          v-for="tab in tabs"
          :key="tab"
          type="button"
          :class="{ active: activeTab === tab }"
          @click="activeTab = tab"
        >
          {{ tab }}
        </button>
      </div>

      <section
        v-if="activeTab === 'Context'"
        class="log-inspector__section"
      >
        <dl>
          <template
            v-for="item in contextItems"
            :key="item.label"
          >
            <dt>{{ item.label }}</dt>
            <dd>{{ item.value }}</dd>
          </template>
        </dl>
      </section>

      <section
        v-else-if="activeTab === 'Metadata'"
        class="log-inspector__section"
      >
        <pre>{{ JSON.stringify(log.metadata, null, 2) }}</pre>
      </section>

      <section
        v-else-if="activeTab === 'Stack'"
        class="log-inspector__section"
      >
        <pre>{{ stackText }}</pre>
      </section>

      <section
        v-else
        class="log-inspector__section"
      >
        <pre>{{ log.raw }}</pre>
      </section>
    </template>

    <div
      v-else
      class="log-inspector__empty"
    >
      <GIcon
        name="panel-right-open"
        :size="28"
      />
      <span>Select a log line to inspect details</span>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import type { LogItem } from "../types"
import { formatLogTime } from "../utils"

const props = defineProps<{ log: LogItem | null }>()
const activeTab = ref<"Context" | "Metadata" | "Stack" | "Raw">("Context")
const tabs = ["Context", "Metadata", "Stack", "Raw"] as const

const contextItems = computed(() => {
  if (!props.log) return []
  return [
    { label: "Time", value: formatLogTime(props.log.timestamp) },
    { label: "TraceId", value: props.log.traceId ?? "-" },
    { label: "RequestId", value: props.log.requestId ?? "-" },
    { label: "Project", value: props.log.projectName ?? "-" },
    { label: "Tunnel", value: props.log.tunnelName ?? "-" },
    { label: "Host", value: props.log.context.host },
    { label: "Thread", value: props.log.context.thread },
    { label: "Session", value: props.log.context.sessionId },
  ]
})

const stackText = computed(() => props.log?.stack?.join("\n") ?? "Stack is reserved for this log entry.")

watch(
  () => props.log?.id,
  () => {
    activeTab.value = "Context"
  },
)
</script>
