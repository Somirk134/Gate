<!--
  GBadge — 统一徽章/标签组件
  ------------------------------------------------------------------
  用途：状态标记、协议标记、计数徽章。统一所有小标签视觉。

  Props:
    variant   颜色语义：neutral | primary | success | warning | error | info
    type      形态：dot(圆点) | solid(实心) | soft(柔和底) | outline(描边)
    size      sm | md
    dot       快捷：仅显示状态圆点（带脉冲）

  用法：
    <GBadge variant="success" type="soft">Running</GBadge>
    <GBadge variant="primary" type="solid">HTTP</GBadge>
    <GBadge variant="success" dot />          <!-- 在线圆点 -->
-->
<template>
  <span
    v-if="!dot"
    class="g-badge"
    :class="[`g-badge--${variant}`, `g-badge--${type}`, `g-badge--${size}`]"
  >
    <span
      v-if="showDot"
      class="g-badge__dot"
      :class="{ 'g-badge__dot--pulse': pulse }"
    />
    <slot />
  </span>

  <span
    v-else
    class="g-badge-dot"
    :class="[`g-badge-dot--${variant}`, { 'g-badge-dot--pulse': pulse }]"
  />
</template>

<script setup lang="ts">
import { computed } from "vue"

const props = withDefaults(
  defineProps<{
    variant?: "neutral" | "primary" | "success" | "warning" | "error" | "info"
    type?: "dot" | "solid" | "soft" | "outline"
    size?: "sm" | "md"
    dot?: boolean
    pulse?: boolean
  }>(),
  {
    variant: "neutral",
    type: "soft",
    size: "md",
    dot: false,
    pulse: false,
  },
)

const showDot = computed(() => props.type === "dot")
</script>

<style scoped>
.g-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-family: var(--font-ui);
  font-weight: var(--weight-medium);
  line-height: 1;
  white-space: nowrap;
  border-radius: var(--radius-badge);
  border: 1px solid transparent;
  vertical-align: middle;
}

.g-badge--sm {
  font-size: var(--text-xs);
  padding: 2px var(--space-2);
  height: 18px;
}
.g-badge--md {
  font-size: var(--text-sm);
  padding: 3px var(--space-2);
  height: 22px;
}

/* ── Variant color maps ── */
/* soft */
.g-badge--neutral.soft,
.g-badge--neutral.g-badge--soft {
  background: var(--status-offline-bg);
  color: var(--text-secondary);
}
.g-badge--primary.g-badge--soft {
  background: var(--color-primary-muted);
  color: var(--color-primary-hover);
}
.g-badge--success.g-badge--soft {
  background: var(--color-success-muted);
  color: var(--color-success);
}
.g-badge--warning.g-badge--soft {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}
.g-badge--error.g-badge--soft {
  background: var(--color-error-muted);
  color: var(--color-error);
}
.g-badge--info.g-badge--soft {
  background: var(--color-info-muted);
  color: var(--color-info);
}

/* solid */
.g-badge--neutral.g-badge--solid {
  background: var(--status-offline);
  color: var(--color-text-on-primary);
}
.g-badge--primary.g-badge--solid {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}
.g-badge--success.g-badge--solid {
  background: var(--color-success);
  color: var(--color-success-fg);
}
.g-badge--warning.g-badge--solid {
  background: var(--color-warning);
  color: var(--color-warning-fg);
}
.g-badge--error.g-badge--solid {
  background: var(--color-error);
  color: var(--color-error-fg);
}
.g-badge--info.g-badge--solid {
  background: var(--color-info);
  color: var(--color-info-fg);
}

/* outline */
.g-badge--neutral.g-badge--outline {
  border-color: var(--color-border-strong);
  color: var(--text-secondary);
}
.g-badge--primary.g-badge--outline { border-color: var(--color-primary); color: var(--color-primary); }
.g-badge--success.g-badge--outline { border-color: var(--color-success); color: var(--color-success); }
.g-badge--warning.g-badge--outline { border-color: var(--color-warning); color: var(--color-warning); }
.g-badge--error.g-badge--outline   { border-color: var(--color-error);   color: var(--color-error); }
.g-badge--info.g-badge--outline    { border-color: var(--color-info);    color: var(--color-info); }

/* ── 内嵌圆点（type=dot 时） ── */
.g-badge__dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  flex-shrink: 0;
}
.g-badge__dot--pulse {
  animation: g-pulse 1.5s var(--ease-in-out) infinite;
}
.g-badge--neutral .g-badge__dot  { background: var(--status-offline); }
.g-badge--primary .g-badge__dot  { background: var(--color-primary); }
.g-badge--success .g-badge__dot  { background: var(--color-success); }
.g-badge--warning .g-badge__dot  { background: var(--color-warning); }
.g-badge--error   .g-badge__dot  { background: var(--color-error); }
.g-badge--info    .g-badge__dot  { background: var(--color-info); }

/* ── 纯圆点徽章（dot prop） ── */
.g-badge-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
  flex-shrink: 0;
  vertical-align: middle;
}
.g-badge-dot--pulse {
  animation: g-pulse 1.5s var(--ease-in-out) infinite;
}
.g-badge-dot--neutral { background: var(--status-offline); }
.g-badge-dot--primary { background: var(--color-primary); }
.g-badge-dot--success { background: var(--color-success); }
.g-badge-dot--warning { background: var(--color-warning); }
.g-badge-dot--error   { background: var(--color-error); }
.g-badge-dot--info    { background: var(--color-info); }
</style>
