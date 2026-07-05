<template>
  <div class="about-page">
    <div class="about-hero">
      <div class="about-logo">
        <svg viewBox="0 0 64 64" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 24L32 8l24 16v28a4 4 0 01-4 4H12a4 4 0 01-4-4z"/><polyline points="24 56 24 32 40 32 40 56"/></svg>
      </div>
      <h1 class="about-name">{{ t('common.appName') }}</h1>
      <p class="about-version">v0.1.0</p>
      <p class="about-build">Build 2026.0704.1</p>
    </div>

    <div class="about-links">
      <a href="https://github.com" target="_blank" class="about-link">
        <svg viewBox="0 0 18 18" fill="currentColor"><path d="M9 1C4.58 1 1 4.58 1 9c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0017 9c0-4.42-3.58-8-8-8z"/></svg>
        {{ t('common.github') }}
      </a>
      <a href="https://gate.dev" target="_blank" class="about-link">
        <svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="9" cy="9" r="7"/><line x1="2" y1="9" x2="16" y2="9"/><path d="M9 2a10 10 0 010 14"/><path d="M9 2a10 10 0 000 14"/></svg>
        {{ t('common.website') }}
      </a>
      <a href="#" class="about-link">
        <svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="14" height="14" rx="2"/><line x1="5" y1="5" x2="5" y2="13"/><line x1="5" y1="9" x2="13" y2="9"/><line x1="13" y1="5" x2="13" y2="7"/></svg>
        {{ t('common.license') }}
      </a>
    </div>

    <div class="about-divider"></div>

    <section class="about-section">
      <h2 class="about-section-title">{{ t('about.changelog') }}</h2>
      <div class="changelog">
        <div class="changelog-item">
          <div class="changelog-version">v0.1.0 <span class="changelog-date">— July 2026</span></div>
          <ul class="changelog-list">
            <li>{{ t('about.initialRelease') }}</li>
            <li v-for="(feature, i) in t('about.features')" :key="i">{{ feature }}</li>
          </ul>
        </div>
      </div>
    </section>

    <div class="about-divider"></div>

    <section class="about-section">
      <h2 class="about-section-title">{{ t('about.acknowledgements') }}</h2>
      <p class="about-ack-text">
        {{ t('about.builtWith') }}<br />
        {{ t('about.iconsFont') }}<br />
        {{ t('about.inspiredBy') }}
      </p>
    </section>

    <div class="about-divider"></div>

    <p class="about-copyright">{{ t('about.copyright') }}</p>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<style scoped>
.about-page { max-width: 560px; margin: 0 auto; text-align: center; padding-top: var(--space-8); }

.about-hero { margin-bottom: var(--space-8); }
.about-logo { width: 64px; height: 64px; margin: 0 auto var(--space-4); color: var(--accent-primary); }
.about-logo svg { width: 100%; height: 100%; }
.about-name { font-size: 28px; font-weight: 700; color: var(--text-primary); letter-spacing: var(--tracking-tight); margin-bottom: var(--space-1); }
.about-version { font-size: var(--text-lg); font-weight: 500; color: var(--text-secondary); font-family: var(--font-mono); }
.about-build { font-size: var(--text-sm); color: var(--text-muted); font-family: var(--font-mono); margin-top: 2px; }

.about-links { display: flex; justify-content: center; gap: var(--space-4); margin-bottom: var(--space-8); }
.about-link { display: flex; align-items: center; gap: var(--space-2); padding: var(--space-2) var(--space-4); background: var(--bg-surface); border: 1px solid var(--border-default); border-radius: var(--radius-md); color: var(--text-secondary); font-size: var(--text-sm); font-weight: 500; text-decoration: none; transition: all var(--duration-micro) var(--ease-out); }
.about-link:hover { background: var(--bg-surface-hover); border-color: var(--border-strong); color: var(--text-primary); }
.about-link svg { width: 14px; height: 14px; }

.about-divider { height: 1px; background: var(--border-subtle); margin: var(--space-6) 0; }

.about-section { text-align: left; }
.about-section-title { font-size: var(--text-lg); font-weight: 600; color: var(--text-primary); margin-bottom: var(--space-3); }

.changelog-item { margin-bottom: var(--space-4); }
.changelog-version { font-size: var(--text-base); font-weight: 600; color: var(--text-primary); font-family: var(--font-mono); }
.changelog-date { font-weight: 400; color: var(--text-muted); }
.changelog-list { margin-top: var(--space-2); padding-left: var(--space-5); }
.changelog-list li { font-size: var(--text-base); color: var(--text-secondary); line-height: var(--leading-relaxed); }

.about-ack-text { font-size: var(--text-base); color: var(--text-secondary); line-height: var(--leading-relaxed); }

.about-copyright { font-size: var(--text-xs); color: var(--text-muted); padding-bottom: var(--space-6); }
</style>
