<!--
  WelcomeCard — 欢迎卡片
  ------------------------------------------------------------------
  显示欢迎语、当前时间、随机开发者语录、当前版本。
  未来可根据用户名动态显示。
-->
<template>
  <GCard variant="plain" padding="lg" class="welcome-card">
    <div class="welcome-card__inner">
      <div class="welcome-card__left">
        <div class="welcome-card__greeting">
          <span class="welcome-card__wave">{{ greetingEmoji }}</span>
          <span class="welcome-card__hello">{{ greeting }}</span>
          <span class="welcome-card__user">{{ username }}</span>
        </div>
        <div class="welcome-card__time">
          <GIcon name="clock" :size="14" />
          <span class="welcome-card__time-text">{{ timeText }}</span>
          <span class="welcome-card__time-date">{{ dateText }}</span>
        </div>
        <div class="welcome-card__quote">
          <GIcon name="sparkles" :size="14" class="welcome-card__quote-icon" />
          <span class="welcome-card__quote-text">{{ quote }}</span>
        </div>
      </div>
      <div class="welcome-card__right">
        <div class="welcome-card__version">
          <GIcon name="rocket" :size="16" />
          <span class="welcome-card__version-label">Gate</span>
          <span class="welcome-card__version-num">{{ version }}</span>
        </div>
        <div class="welcome-card__stats">
          <div class="welcome-card__stat">
            <span class="welcome-card__stat-value">{{ runningCount }}</span>
            <span class="welcome-card__stat-label">运行中</span>
          </div>
          <div class="welcome-card__stat-divider" />
          <div class="welcome-card__stat">
            <span class="welcome-card__stat-value">{{ serverCount }}</span>
            <span class="welcome-card__stat-label">服务器</span>
          </div>
        </div>
      </div>
    </div>
  </GCard>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GCard from '@components/base/GCard.vue'
import GIcon from '@components/icons/GIcon.vue'
import { useDashboardClock } from '../composables/useDashboardClock'

withDefaults(
  defineProps<{
    username?: string
    quote: string
    version: string
    runningCount: number
    serverCount: number
  }>(),
  {
    username: '开发者',
  },
)

const { timeText, dateText, greeting } = useDashboardClock()

const greetingEmoji = computed(() => {
  const h = new Date().getHours()
  if (h < 6) return '🌙'
  if (h < 12) return '🌅'
  if (h < 18) return '☀️'
  return '🌆'
})
</script>

<style scoped>
.welcome-card {
  position: relative;
  overflow: hidden;
  background: linear-gradient(135deg, var(--bg-card) 0%, var(--bg-surface-hover) 100%);
}
.welcome-card::before {
  content: '';
  position: absolute;
  top: -40%;
  right: -10%;
  width: 360px;
  height: 360px;
  background: radial-gradient(circle, var(--color-primary-muted) 0%, transparent 70%);
  pointer-events: none;
}
.welcome-card__inner {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-6);
  position: relative;
  z-index: 1;
}
.welcome-card__left {
  min-width: 0;
  flex: 1;
}
.welcome-card__greeting {
  display: flex;
  align-items: baseline;
  gap: var(--space-2);
  flex-wrap: wrap;
}
.welcome-card__wave {
  font-size: var(--text-xl);
}
.welcome-card__hello {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  letter-spacing: var(--tracking-tight);
}
.welcome-card__user {
  font-size: var(--text-lg);
  font-weight: var(--weight-medium);
  color: var(--color-primary);
}
.welcome-card__time {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}
.welcome-card__time-text {
  font-family: var(--font-mono);
  font-variant-numeric: tabular-nums;
  font-weight: var(--weight-medium);
  color: var(--text-primary);
}
.welcome-card__time-date {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}
.welcome-card__quote {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  margin-top: var(--space-4);
  max-width: 520px;
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  line-height: var(--leading-relaxed);
  font-style: italic;
}
.welcome-card__quote-icon {
  color: var(--color-warning);
  flex-shrink: 0;
  margin-top: 2px;
}
.welcome-card__right {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: var(--space-4);
  flex-shrink: 0;
}
.welcome-card__version {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-1) var(--space-3);
  background: var(--color-primary-muted);
  border: 1px solid var(--color-border-accent);
  border-radius: var(--radius-full);
  color: var(--color-primary);
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
}
.welcome-card__version-num {
  font-family: var(--font-mono);
}
.welcome-card__stats {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}
.welcome-card__stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}
.welcome-card__stat-value {
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  line-height: 1;
}
.welcome-card__stat-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wider);
}
.welcome-card__stat-divider {
  width: 1px;
  height: 28px;
  background: var(--border-subtle);
}

@media (max-width: 760px) {
  .welcome-card__inner {
    flex-direction: column;
  }
  .welcome-card__right {
    align-items: flex-start;
  }
}
</style>
