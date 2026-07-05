<template>
  <div class="log-empty">
    <div class="log-empty__icon">
      <GIcon name="terminal" :size="44" />
    </div>
    <h2>Log Center</h2>
    <p>
      Central debugging workspace for System, Client, Server, Project, Tunnel and future Plugin logs.
    </p>
    <GButton variant="primary" icon="plus" @click="$emit('generate')">Generate Test Logs</GButton>
  </div>
</template>

<script setup lang="ts">
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"

defineEmits<{ generate: [] }>()
</script>
