<template>
    <div class="loading-layer">
        <div class="loading-backdrop" />
        <div class="loading-content">
            <div class="loading-spinner">
                <GIcon name="loader" :size="24" spin />
            </div>
            <div class="loading-message">{{ loading.globalMessage || 'Loading...' }}</div>
            <div v-if="loading.currentTask?.progress !== undefined" class="loading-progress">
                <div class="progress-track">
                    <div
                        class="progress-fill"
                        :style="{ width: (loading.currentTask.progress * 100) + '%' }"
                    />
                </div>
                <span class="progress-text">{{ Math.round(loading.currentTask.progress * 100) }}%</span>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useLoadingStore } from "@stores"
import GIcon from "@components/icons/GIcon.vue"

const loading = useLoadingStore()
</script>

<style scoped>
.loading-layer {
    position: fixed;
    inset: 0;
    z-index: var(--z-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    gap: var(--space-4);
    animation: fadeIn var(--duration-fast) var(--ease-out);
}

.loading-backdrop {
    position: absolute;
    inset: 0;
    background: var(--color-overlay);
    backdrop-filter: blur(2px);
}

.loading-content {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-4);
    padding: var(--space-8) var(--space-10);
    background: var(--bg-surface-raised);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-floating);
    animation: scaleIn var(--duration-standard) var(--ease-out);
}

.loading-spinner {
    color: var(--color-primary);
}

.loading-message {
    font-size: var(--text-sm);
    color: var(--text-secondary);
}

.loading-progress {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    width: 200px;
}

.progress-track {
    flex: 1;
    height: 4px;
    background: var(--bg-surface-hover);
    border-radius: var(--radius-full);
    overflow: hidden;
}

.progress-fill {
    height: 100%;
    background: var(--color-primary);
    border-radius: var(--radius-full);
    transition: width var(--duration-base) var(--ease-out);
}

.progress-text {
    font-size: var(--text-xs);
    font-family: var(--font-mono);
    color: var(--text-tertiary);
    min-width: 32px;
    text-align: right;
}

@keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}

@keyframes scaleIn {
    from {
        opacity: 0;
        transform: scale(0.92);
    }
    to {
        opacity: 1;
        transform: scale(1);
    }
}
</style>
