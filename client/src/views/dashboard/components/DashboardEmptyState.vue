<!--
  DashboardEmptyState — 空状态
  ------------------------------------------------------------------
  无项目时的引导插画与操作入口。
-->
<template>
  <GCard
    variant="plain"
    padding="lg"
    class="dash-empty"
  >
    <div class="dash-empty__inner">
      <div class="dash-empty__art">
        <GIcon
          name="rocket"
          :size="40"
        />
        <span class="dash-empty__art-ring" />
        <span class="dash-empty__art-ring dash-empty__art-ring--2" />
      </div>
      <h3 class="dash-empty__title">
        {{ title }}
      </h3>
      <p class="dash-empty__desc">
        {{ description }}
      </p>
      <div class="dash-empty__actions">
        <GButton
          variant="primary"
          icon="plus"
          @click="$emit('create')"
        >
          {{ createText }}
        </GButton>
        <GButton
          variant="secondary"
          icon="plug"
          @click="$emit('connect')"
        >
          {{ connectText }}
        </GButton>
      </div>
    </div>
  </GCard>
</template>

<script setup lang="ts">
import GCard from "@components/base/GCard.vue"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"

withDefaults(
  defineProps<{
    title?: string
    description?: string
    createText?: string
    connectText?: string
  }>(),
  {
    title: "开启你的第一个项目",
    description: "Gate 让你一键将本地服务暴露到公网。创建项目或连接服务器，即刻开始。",
    createText: "创建第一个项目",
    connectText: "连接服务器",
  },
)

defineEmits<{ create: []; connect: [] }>()
</script>

<style scoped>
.dash-empty__inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: var(--space-10) var(--space-6);
  gap: var(--space-3);
}
.dash-empty__art {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 88px;
  height: 88px;
  border-radius: var(--radius-full);
  background: var(--color-primary-muted);
  color: var(--color-primary);
  margin-bottom: var(--space-2);
}
.dash-empty__art-ring {
  position: absolute;
  inset: -8px;
  border: 1px solid var(--color-border-accent);
  border-radius: var(--radius-full);
  opacity: 0.4;
}
.dash-empty__art-ring--2 {
  inset: -18px;
  opacity: 0.2;
}
.dash-empty__title {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
}
.dash-empty__desc {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  max-width: 380px;
  line-height: var(--leading-relaxed);
}
.dash-empty__actions {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-top: var(--space-2);
}
</style>
