<template>
  <section :id="`settings-group-${group.id}`" class="setting-group">
    <header class="setting-group__header">
      <div>
        <h3>{{ group.label }}</h3>
        <p v-if="group.description">{{ group.description }}</p>
      </div>
      <span>{{ group.items.length }}</span>
    </header>

    <div class="setting-group__items">
      <SettingItemView
        v-for="item in group.items"
        :key="item.id"
        :item="item"
        :value="values[item.key]"
        :error="validationErrors[item.key]"
        :selected="selectedSettingId === item.id"
        :modified="dirtyKeys.includes(item.key)"
        :highlight-query="highlightQuery"
        :action-statuses="actionStatuses"
        @select="emit('select', item)"
        @reset="emit('reset', item)"
        @run-action="emit('run-action', $event)"
        @update:value="emit('update:value', item, $event)"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import type { SettingActionStatus, SettingGroup, SettingItem, SettingValue } from "../types"
import SettingItemView from "./SettingItem.vue"

defineProps<{
  group: SettingGroup
  values: Record<string, SettingValue>
  validationErrors: Record<string, string | undefined>
  dirtyKeys: string[]
  selectedSettingId: string | null
  highlightQuery: string
  actionStatuses: Record<string, SettingActionStatus>
}>()

const emit = defineEmits<{
  "update:value": [item: SettingItem, value: SettingValue]
  select: [item: SettingItem]
  reset: [item: SettingItem]
  "run-action": [actionId: string]
}>()
</script>
