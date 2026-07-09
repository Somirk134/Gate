<template>
  <aside class="log-source-tree">
    <div class="log-source-tree__header">
      <span>Sources</span>
      <strong>{{ total }}</strong>
    </div>

    <div class="log-source-tree__nodes">
      <div
        v-for="root in sources"
        :key="root.id"
        class="log-source-tree__group"
      >
        <div
          class="log-source-node"
          :class="{ 'log-source-node--active': selected === root.id }"
        >
          <button
            type="button"
            class="log-source-node__toggle"
            @click.stop="expanded = !expanded"
          >
            <GIcon
              :name="expanded ? 'chevron-down' : 'chevron-right'"
              :size="13"
            />
          </button>
          <button
            type="button"
            class="log-source-node__label"
            @click="$emit('select', root.id)"
          >
            <GIcon
              :name="root.icon"
              :size="15"
            />
            <span>{{ root.label }}</span>
            <strong>{{ total }}</strong>
          </button>
        </div>

        <div
          v-if="expanded"
          class="log-source-tree__children"
        >
          <button
            v-for="child in root.children"
            :key="child.id"
            type="button"
            class="log-source-node log-source-node--child"
            :class="{ 'log-source-node--active': selected === child.id }"
            @click="$emit('select', child.id)"
          >
            <span class="log-source-node__toggle" />
            <GIcon
              :name="child.icon"
              :size="14"
            />
            <span>{{ child.label }}</span>
            <em v-if="child.reserved">soon</em>
            <strong>{{ getCount(child.id) }}</strong>
          </button>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import type { LogSource, LogSourceNode } from "../types"

const props = defineProps<{
  sources: LogSourceNode[]
  selected: LogSource | "ALL"
  counts: Record<LogSource, number>
  total: number
}>()

defineEmits<{ select: [value: LogSource | "ALL"] }>()

const expanded = ref(true)

function getCount(source: LogSource | "ALL"): number {
  return source === "ALL" ? props.total : props.counts[source] ?? 0
}
</script>
