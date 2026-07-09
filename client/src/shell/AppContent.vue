<template>
  <main class="app-content">
    <router-view v-slot="{ Component, route }">
      <transition
        name="page"
        mode="out-in"
        appear
      >
        <keep-alive :include="cachedViews">
          <component
            :is="Component"
            :key="route.path"
          />
        </keep-alive>
      </transition>
    </router-view>

    <!-- Empty State (fallback) -->
    <div
      v-if="!routeHasComponent"
      class="content-empty"
    >
      <GEmptyState
        title="No Content"
        description="This page is not available yet."
      />
    </div>
  </main>
</template>

<script setup lang="ts">
import { ref, computed } from "vue"
import { useRoute } from "vue-router"
import GEmptyState from "@components/feedback/GEmptyState.vue"

const route = useRoute()

const cachedViews = ref<string[]>(['dashboard', 'projects', 'settings'])

const routeHasComponent = computed(() => {
    return route.matched.length > 0
})
</script>

<style scoped>
.app-content {
    flex: 1;
    min-width: 0;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--space-6);
    position: relative;
    box-sizing: border-box;
}

.content-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
}

/* ── Page Transition ── */
.page-enter-active,
.page-leave-active {
    transition: opacity var(--duration-standard) var(--ease-out),
                transform var(--duration-standard) var(--ease-out);
}

.page-enter-from {
    opacity: 0;
    transform: translateY(8px);
}

.page-leave-to {
    opacity: 0;
    transform: translateY(-4px);
}
</style>
