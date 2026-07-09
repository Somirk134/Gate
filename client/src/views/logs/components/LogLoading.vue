<template>
  <div class="log-loading">
    <div class="log-loading__sidebar">
      <GSkeleton
        v-for="i in 9"
        :key="i"
        variant="rect"
        height="28px"
        rounded="6px"
      />
    </div>
    <div class="log-loading__console">
      <GSkeleton
        v-for="i in 18"
        :key="i"
        variant="rect"
        height="18px"
        rounded="4px"
      />
    </div>
    <div class="log-loading__inspector">
      <GSkeleton
        variant="rect"
        height="28px"
        rounded="6px"
      />
      <GSkeleton
        v-for="i in 8"
        :key="i"
        variant="rect"
        height="16px"
        rounded="4px"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import GSkeleton from "@components/feedback/GSkeleton.vue"
</script>
