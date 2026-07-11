<template>
  <section v-if="topology?.nodes?.length" class="domain-topology">
    <header>
      <h3>{{ t('domains.topology.title') }}</h3>
      <span>{{ t('domains.topology.subtitle') }}</span>
    </header>
    <div class="domain-topology__canvas">
      <svg class="domain-topology__edges" aria-hidden="true">
        <line
          v-for="(edge, index) in topology.edges"
          :key="`${edge.from}-${edge.to}-${index}`"
          :x1="nodePosition(edge.from)?.x ?? 0"
          :y1="nodePosition(edge.from)?.y ?? 0"
          :x2="nodePosition(edge.to)?.x ?? 0"
          :y2="nodePosition(edge.to)?.y ?? 0" />
      </svg>
      <button
        v-for="node in topology.nodes"
        :key="node.id"
        type="button"
        class="domain-topology__node"
        :class="`is-${node.type}`"
        :data-node-id="node.id"
        @click="emit('navigate', node.route)">
        <GIcon :name="iconFor(node.type)" :size="14" />
        <span>{{ node.label }}</span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { DomainTopologyResponse } from '../../types'

const props = defineProps<{
  topology: DomainTopologyResponse | null
}>()

const emit = defineEmits<{
  navigate: [route: string]
}>()

const { t } = useI18n()
const positions = ref<Record<string, { x: number; y: number }>>({})
let layoutTimer: number | null = null

function iconFor(type: string) {
  switch (type) {
    case 'project':
      return 'projects'
    case 'tunnel':
      return 'router'
    case 'certificate':
      return 'shield-check'
    case 'https':
      return 'shield-check'
    default:
      return 'globe'
  }
}

function measurePositions() {
  const next: Record<string, { x: number; y: number }> = {}
  const canvas = document.querySelector('.domain-topology__canvas')
  if (!canvas) return
  const canvasRect = canvas.getBoundingClientRect()
  canvas.querySelectorAll<HTMLElement>('.domain-topology__node').forEach((element) => {
    const id = element.dataset.nodeId
    if (!id) return
    const rect = element.getBoundingClientRect()
    next[id] = {
      x: rect.left - canvasRect.left + rect.width / 2,
      y: rect.top - canvasRect.top + rect.height / 2,
    }
  })
  positions.value = next
}

function scheduleMeasure() {
  if (layoutTimer) window.clearTimeout(layoutTimer)
  layoutTimer = window.setTimeout(() => {
    measurePositions()
    layoutTimer = null
  }, 60)
}

function nodePosition(id: string) {
  return positions.value[id]
}

onMounted(() => {
  scheduleMeasure()
  window.addEventListener('resize', scheduleMeasure)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', scheduleMeasure)
  if (layoutTimer) window.clearTimeout(layoutTimer)
})

watch(
  () => props.topology,
  () => scheduleMeasure(),
  { deep: true },
)
</script>

<style scoped>
.domain-topology {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  background: var(--bg-surface);
}

.domain-topology header {
  margin-bottom: var(--space-3);
}

.domain-topology header h3 {
  margin: 0;
}

.domain-topology header span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.domain-topology__canvas {
  position: relative;
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  min-height: 72px;
}

.domain-topology__edges {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.domain-topology__edges line {
  stroke: var(--border-strong);
  stroke-width: 1.5;
  opacity: 0.55;
}

.domain-topology__node {
  position: relative;
  z-index: 1;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-full);
  padding: 8px 12px;
  background: var(--bg-input);
  cursor: pointer;
  transition: border-color var(--duration-fast), transform var(--duration-fast);
}

.domain-topology__node:hover {
  border-color: var(--color-primary);
  transform: translateY(-1px);
}

.domain-topology__node.is-domain { color: var(--color-primary); }
.domain-topology__node.is-tunnel { color: var(--text-primary); }
.domain-topology__node.is-project { color: var(--color-success); }
.domain-topology__node.is-certificate,
.domain-topology__node.is-https { color: var(--color-warning); }
</style>
