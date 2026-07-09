<!--
  ServerTag — 服务器标签 chip
  ------------------------------------------------------------------
  展示单个标签，支持点击移除（编辑态）与展示态。
-->
<template>
  <span
    class="server-tag"
    :class="{ 'server-tag--removable': removable }"
    :style="tagStyle"
  >
    <span class="server-tag__label">{{ name }}</span>
    <button
      v-if="removable"
      class="server-tag__remove"
      type="button"
      @click.stop="$emit('remove', name)"
    >
      <GIcon
        name="close"
        :size="10"
      />
    </button>
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"

const props = withDefaults(
  defineProps<{
    name: string
    color?: string
    removable?: boolean
  }>(),
  {
    color: "",
    removable: false,
  },
)

defineEmits<{ remove: [name: string] }>()

const tagStyle = computed(() => {
  if (!props.color) return {}
  return {
    background: `${props.color}1f`,
    color: props.color,
    borderColor: `${props.color}40`,
  }
})
</script>
