<template>
  <section
    class="about-page"
    aria-labelledby="about-title"
  >
    <header class="about-hero">
      <div class="about-hero__identity">
        <div
          class="about-logo"
          aria-hidden="true"
        >
          <GIcon
            name="home"
            :size="34"
            :stroke-width="1.9"
          />
        </div>
        <div class="about-hero__copy">
          <p class="about-eyebrow">
            {{ t("about.eyebrow") }}
          </p>
          <h1 id="about-title">
            {{ t("common.appName") }}
          </h1>
          <p>{{ t("about.tagline") }}</p>
        </div>
      </div>

      <nav
        class="about-actions"
        :aria-label="t('about.linksLabel')"
      >
        <a
          v-for="link in productLinks"
          :key="link.href"
          :href="link.href"
          target="_blank"
          rel="noopener noreferrer"
          class="about-action"
        >
          <GIcon
            :name="link.icon"
            :size="15"
          />
          <span>{{ link.label }}</span>
          <GIcon
            name="external-link"
            :size="13"
          />
        </a>
      </nav>
    </header>

    <div
      class="about-meta"
      :aria-label="t('about.versionSummary')"
    >
      <div
        v-for="item in versionItems"
        :key="item.label"
        class="about-meta__item"
      >
        <span class="about-meta__icon"><GIcon
          :name="item.icon"
          :size="16"
        /></span>
        <span class="about-meta__label">{{ item.label }}</span>
        <strong>{{ item.value }}</strong>
      </div>
    </div>

    <main class="about-content">
      <section class="about-section about-section--intro">
        <div>
          <p class="about-section__kicker">
            {{ t("about.summaryKicker") }}
          </p>
          <h2>{{ t("about.summaryTitle") }}</h2>
        </div>
        <p>{{ t("about.summaryBody") }}</p>
      </section>

      <section class="about-section">
        <div class="about-section__heading">
          <div>
            <p class="about-section__kicker">
              {{ t("about.capabilitiesKicker") }}
            </p>
            <h2>{{ t("about.capabilitiesTitle") }}</h2>
          </div>
          <span>{{ t("about.alphaBadge") }}</span>
        </div>

        <div class="about-capabilities">
          <article
            v-for="item in capabilities"
            :key="item.title"
            class="about-card"
          >
            <span class="about-card__icon"><GIcon
              :name="item.icon"
              :size="18"
            /></span>
            <h3>{{ item.title }}</h3>
            <p>{{ item.description }}</p>
          </article>
        </div>
      </section>

      <section class="about-section about-section--split">
        <div>
          <div class="about-section__heading">
            <div>
              <p class="about-section__kicker">
                {{ t("about.releaseKicker") }}
              </p>
              <h2>{{ t("about.changelog") }}</h2>
            </div>
            <time datetime="2026-07">{{ t("about.releaseDate") }}</time>
          </div>

          <ol class="about-release">
            <li
              v-for="item in releaseNotes"
              :key="item"
            >
              <span aria-hidden="true" />
              <p>{{ item }}</p>
            </li>
          </ol>
        </div>

        <aside
          class="about-status"
          :aria-label="t('about.stageLabel')"
        >
          <span class="about-status__icon"><GIcon
            name="rocket"
            :size="18"
          /></span>
          <p>{{ t("about.stageLabel") }}</p>
          <strong>{{ t("about.stageValue") }}</strong>
          <small>{{ t("about.stageDescription") }}</small>
        </aside>
      </section>

      <section class="about-section about-section--credits">
        <div>
          <p class="about-section__kicker">
            {{ t("about.creditsKicker") }}
          </p>
          <h2>{{ t("about.acknowledgements") }}</h2>
        </div>

        <div class="about-stack">
          <span
            v-for="item in techStack"
            :key="item"
          >{{ item }}</span>
        </div>

        <div>
          <p class="about-credit-text">
            {{ t("about.creditLine") }}
          </p>
          <p class="about-copyright">
            {{ t("about.copyright") }}
          </p>
        </div>
      </section>
    </main>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue"
import { useI18n } from "vue-i18n"
import GIcon from "@components/icons/GIcon.vue"

const APP_VERSION = "0.1.0"
const BUILD_NUMBER = "2026.0704.1"

const { t } = useI18n()

const productLinks = computed(() => [
  {
    label: t("common.github"),
    href: "https://github.com/gate/gate",
    icon: "github",
  },
  {
    label: t("common.website"),
    href: "https://gate.dev",
    icon: "globe",
  },
  {
    label: t("common.license"),
    href: "https://github.com/gate/gate/blob/main/LICENSE",
    icon: "file-text",
  },
])

const versionItems = computed(() => [
  {
    label: t("about.versionLabel"),
    value: `v${APP_VERSION}`,
    icon: "package",
  },
  {
    label: t("about.buildLabel"),
    value: BUILD_NUMBER,
    icon: "git-commit",
  },
  {
    label: t("about.channelLabel"),
    value: t("about.channelValue"),
    icon: "radio",
  },
  {
    label: t("about.licenseLabel"),
    value: "MIT",
    icon: "shield-check",
  },
])

const capabilities = computed(() => [
  {
    icon: "servers",
    title: t("about.capability.selfHosted.title"),
    description: t("about.capability.selfHosted.description"),
  },
  {
    icon: "layout-grid",
    title: t("about.capability.desktop.title"),
    description: t("about.capability.desktop.description"),
  },
  {
    icon: "activity",
    title: t("about.capability.runtime.title"),
    description: t("about.capability.runtime.description"),
  },
  {
    icon: "code",
    title: t("about.capability.openSource.title"),
    description: t("about.capability.openSource.description"),
  },
])

const releaseNotes = computed(() => [
  t("about.releaseNote.clientShell"),
  t("about.releaseNote.tunnelWorkflow"),
  t("about.releaseNote.monitoring"),
  t("about.releaseNote.docs"),
])

const techStack = computed(() => [
  "Tauri",
  "Vue 3",
  "Rust",
  "Naive UI",
  "Lucide",
  "MIT License",
])
</script>

<style scoped>
.about-page {
  width: min(100%, 1040px);
  min-height: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
  margin: 0 auto;
  color: var(--text-primary);
}

.about-hero {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-5);
  padding: var(--space-6);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background:
    linear-gradient(135deg, rgba(91, 141, 239, 0.13), transparent 44%),
    var(--bg-surface);
}

.about-hero__identity {
  min-width: 0;
  display: grid;
  grid-template-columns: 58px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-4);
}

.about-logo {
  width: 58px;
  height: 58px;
  display: grid;
  place-items: center;
  border: 1px solid rgba(91, 141, 239, 0.35);
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.03);
}

.about-hero__copy {
  min-width: 0;
}

.about-eyebrow,
.about-section__kicker {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.about-hero h1 {
  margin-top: var(--space-1);
  color: var(--text-primary);
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
  line-height: var(--leading-tight);
}

.about-hero__copy p:last-child {
  max-width: 660px;
  margin-top: var(--space-2);
  color: var(--text-secondary);
  font-size: var(--text-md);
  line-height: var(--leading-relaxed);
}

.about-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.about-action {
  height: 34px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0 var(--space-3);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: rgba(15, 16, 19, 0.28);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  text-decoration: none;
  transition:
    background-color var(--transition-fast),
    border-color var(--transition-fast),
    color var(--transition-fast);
}

.about-action:hover {
  border-color: var(--border-strong);
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.about-meta {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
}

.about-meta__item {
  min-width: 0;
  min-height: 72px;
  display: grid;
  grid-template-columns: 32px minmax(0, 1fr);
  grid-template-rows: auto auto;
  align-content: center;
  gap: 2px var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.about-meta__icon {
  grid-row: 1 / 3;
  width: 32px;
  height: 32px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--color-primary);
}

.about-meta__label {
  min-width: 0;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.about-meta strong {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.about-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-6);
  padding-bottom: var(--space-8);
}

.about-section {
  padding-top: var(--space-1);
  border-top: 1px solid var(--border-subtle);
}

.about-section--intro {
  display: grid;
  grid-template-columns: minmax(220px, 0.42fr) minmax(0, 1fr);
  gap: var(--space-6);
  align-items: start;
}

.about-section h2 {
  margin-top: var(--space-1);
  color: var(--text-primary);
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
  line-height: var(--leading-tight);
}

.about-section--intro > p,
.about-credit-text {
  color: var(--text-secondary);
  font-size: var(--text-md);
  line-height: var(--leading-relaxed);
}

.about-section__heading {
  display: flex;
  align-items: end;
  justify-content: space-between;
  gap: var(--space-4);
  margin-bottom: var(--space-4);
}

.about-section__heading > span,
.about-section__heading time {
  flex: 0 0 auto;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.about-capabilities {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
}

.about-card {
  min-width: 0;
  min-height: 156px;
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.about-card__icon,
.about-status__icon {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-sm);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.about-card h3 {
  margin-top: var(--space-3);
  color: var(--text-primary);
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.about-card p {
  margin-top: var(--space-2);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.about-section--split {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 280px;
  gap: var(--space-5);
}

.about-release {
  display: grid;
  gap: var(--space-3);
  list-style: none;
}

.about-release li {
  display: grid;
  grid-template-columns: 22px minmax(0, 1fr);
  gap: var(--space-3);
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.about-release li span {
  position: relative;
  width: 22px;
  min-height: 22px;
}

.about-release li span::before {
  content: "";
  position: absolute;
  top: 8px;
  left: 8px;
  width: 7px;
  height: 7px;
  border-radius: var(--radius-full);
  background: var(--color-primary);
}

.about-release li span::after {
  content: "";
  position: absolute;
  top: 19px;
  left: 11px;
  width: 1px;
  height: calc(100% + var(--space-3));
  background: var(--border-subtle);
}

.about-release li:last-child span::after {
  display: none;
}

.about-status {
  min-width: 0;
  align-self: start;
  padding: var(--space-4);
  border: 1px solid rgba(245, 184, 75, 0.24);
  border-radius: var(--radius-md);
  background: var(--color-warning-muted);
}

.about-status__icon {
  background: rgba(245, 184, 75, 0.18);
  color: var(--color-warning);
}

.about-status p {
  margin-top: var(--space-3);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.about-status strong {
  display: block;
  margin-top: 2px;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
}

.about-status small {
  display: block;
  margin-top: var(--space-2);
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
}

.about-section--credits {
  display: grid;
  grid-template-columns: minmax(180px, 0.32fr) minmax(0, 0.36fr) minmax(0, 1fr);
  gap: var(--space-5);
  align-items: start;
}

.about-stack {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.about-stack span {
  height: 26px;
  display: inline-flex;
  align-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  background: var(--bg-surface);
  color: var(--text-secondary);
  padding: 0 var(--space-2);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.about-copyright {
  margin-top: var(--space-3);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

@media (max-width: 980px) {
  .about-hero,
  .about-section--split,
  .about-section--intro,
  .about-section--credits {
    grid-template-columns: 1fr;
  }

  .about-actions {
    justify-content: flex-start;
  }

  .about-meta,
  .about-capabilities {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .about-status {
    width: 100%;
  }
}

@media (max-width: 620px) {
  .about-page {
    gap: var(--space-4);
  }

  .about-hero {
    padding: var(--space-4);
  }

  .about-hero__identity {
    grid-template-columns: 1fr;
  }

  .about-meta,
  .about-capabilities {
    grid-template-columns: 1fr;
  }

  .about-action {
    width: 100%;
    justify-content: center;
  }
}
</style>
