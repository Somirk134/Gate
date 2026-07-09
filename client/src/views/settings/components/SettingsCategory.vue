<template>
  <section
    :id="`settings-category-${category.id}`"
    class="settings-category"
  >
    <header class="settings-category__header">
      <div class="settings-category__icon">
        <GIcon
          :name="category.icon"
          :size="18"
        />
      </div>
      <div>
        <h2>{{ category.label }}</h2>
        <p>{{ category.description }}</p>
      </div>
      <span>{{ itemCount }}</span>
    </header>

    <SettingGroup
      v-for="group in category.groups"
      :key="group.id"
      :group="group"
      :values="values"
      :validation-errors="validationErrors"
      :dirty-keys="dirtyKeys"
      :selected-setting-id="selectedSettingId"
      :highlight-query="highlightQuery"
      :action-statuses="actionStatuses"
      @select="emit('select', $event)"
      @reset="emit('reset', $event)"
      @run-action="emit('run-action', $event)"
      @update:value="handleUpdateValue"
    />
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import type { SettingActionStatus, SettingCategory, SettingItem, SettingValue } from "../types"
import SettingGroup from "./SettingGroup.vue"

const props = defineProps<{
  category: SettingCategory
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

const itemCount = computed(() => props.category.groups.reduce((total, group) => total + group.items.length, 0))

function handleUpdateValue(item: SettingItem, value: SettingValue) {
  emit("update:value", item, value)
}
</script>
