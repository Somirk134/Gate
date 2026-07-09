<template>
  <GIconButton
    class="setting-reset"
    name="refresh"
    size="sm"
    :disabled="!modified"
    :tooltip="modified ? '重置此设置' : '当前为默认值'"
    @click="emit('reset')" />
</template>

<script setup lang="ts">
import GIconButton from '@components/base/GIconButton.vue'

defineProps<{
  modified: boolean
}>()

const emit = defineEmits<{
  reset: []
}>()
</script>
