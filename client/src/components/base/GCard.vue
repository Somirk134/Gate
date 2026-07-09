<!--
  GCard — 统一卡片基础组件
  ------------------------------------------------------------------
  用途：所有卡片类容器的基底。业务卡片（ProjectCard/TunnelCard 等）
  均基于 GCard 组合，不重复写容器样式。

  Props:
    variant     plain(默认) | outlined | elevated | interactive
    padding     none | sm | md | lg
    hoverable   是否启用 hover 高亮（legacy，等价 interactive）
    clickable   是否可点击（带 hover 抬升 + cursor pointer）

  Slots:
    default     主体内容
    header      头部（带底分隔线）
    footer      底部（带顶分隔线）
-->
<template>
  <component
    :is="clickable || hoverable ? 'button' : 'div'"
    class="g-card"
    :class="[
      `g-card--${variant}`,
      `g-card--pad-${padding}`,
      { 'g-card--clickable': clickable, 'g-card--hoverable': hoverable },
    ]"
    :type="clickable ? 'button' : undefined">
    <header v-if="$slots.header" class="g-card__header">
      <slot name="header" />
    </header>

    <div class="g-card__body">
      <slot />
    </div>

    <footer v-if="$slots.footer" class="g-card__footer">
      <slot name="footer" />
    </footer>
  </component>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'plain' | 'outlined' | 'elevated' | 'interactive'
    padding?: 'none' | 'sm' | 'md' | 'lg'
    hoverable?: boolean
    clickable?: boolean
  }>(),
  {
    variant: 'plain',
    padding: 'md',
    hoverable: false,
    clickable: false,
  },
)
</script>

<style scoped>
.g-card {
  display: flex;
  flex-direction: column;
  width: 100%;
  background: var(--bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-card);
  color: var(--text-primary);
  text-align: left;
  font-family: var(--font-ui);
  transition:
    border-color var(--duration-fast) var(--ease-out),
    background-color var(--duration-fast) var(--ease-out),
    box-shadow var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out);
}

/* ── Variants ── */
.g-card--plain {
  background: var(--bg-card);
  border-color: var(--color-border);
}
.g-card--outlined {
  background: transparent;
  border-color: var(--color-border-subtle);
}
.g-card--elevated {
  border-color: var(--color-border-subtle);
  box-shadow: var(--shadow-sm);
}
.g-card--interactive {
  border-color: var(--color-border);
  cursor: pointer;
}
.g-card--interactive:hover {
  background: var(--bg-surface-hover);
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-hover);
  transform: translateY(-1px);
}
.g-card--interactive:active {
  transform: translateY(0);
}

/* 兼容 hoverable */
.g-card--hoverable {
  cursor: pointer;
}
.g-card--hoverable:hover {
  background: var(--bg-surface-hover);
  border-color: var(--color-border-strong);
}

/* ── Padding ── */
.g-card--pad-none .g-card__body {
  padding: 0;
}
.g-card--pad-sm .g-card__body {
  padding: var(--space-3);
}
.g-card--pad-md .g-card__body {
  padding: var(--space-4);
}
.g-card--pad-lg .g-card__body {
  padding: var(--space-6);
}

.g-card--pad-none .g-card__header {
  padding: 0;
}
.g-card--pad-sm .g-card__header {
  padding: var(--space-3);
}
.g-card--pad-md .g-card__header {
  padding: var(--space-3) var(--space-4);
}
.g-card--pad-lg .g-card__header {
  padding: var(--space-4) var(--space-6);
}

.g-card--pad-none .g-card__footer {
  padding: 0;
}
.g-card--pad-sm .g-card__footer {
  padding: var(--space-3);
}
.g-card--pad-md .g-card__footer {
  padding: var(--space-3) var(--space-4);
}
.g-card--pad-lg .g-card__footer {
  padding: var(--space-4) var(--space-6);
}

/* ── Header / Footer ── */
.g-card__header {
  border-bottom: 1px solid var(--color-border-subtle);
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
}
.g-card__footer {
  border-top: 1px solid var(--color-border-subtle);
}
.g-card__body {
  flex: 1;
}
</style>
