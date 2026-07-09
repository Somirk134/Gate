<template>
  <section class="about-page" aria-labelledby="about-title">
    <div class="about-hero-grid">
      <article class="about-product">
        <div class="about-logo" aria-hidden="true">
          <img :src="appLogoUrl" alt="" />
        </div>

        <div class="about-product__content">
          <div class="about-title-row">
            <div>
              <p class="about-eyebrow">
                {{ t('about.eyebrow') }}
              </p>
              <h1 id="about-title">
                {{ t('common.appName') }}
              </h1>
            </div>
            <span class="about-version-badge">{{ versionBadge }}</span>
          </div>

          <p class="about-product__lead">
            {{ t('about.tagline') }}
          </p>

          <nav class="about-actions" :aria-label="t('about.linksLabel')">
            <a
              v-for="link in productLinks"
              :key="link.href"
              :href="link.href"
              target="_blank"
              rel="noopener noreferrer"
              class="about-action"
              :class="{ 'about-action--primary': link.primary }">
              <GIcon :name="link.icon" :size="15" />
              <span>{{ link.label }}</span>
            </a>
            <button
              type="button"
              class="about-action about-update-action"
              :class="{ 'about-update-action--active': updateInfo?.available }"
              :disabled="isUpdateBusy"
              @click="handleCheckUpdate">
              <GIcon :name="updateActionIcon" :size="15" :spin="isUpdateBusy" />
              <span>{{ updateActionLabel }}</span>
            </button>
            <button
              v-if="canDownloadUpdate"
              type="button"
              class="about-action about-update-action"
              :disabled="isUpdateBusy"
              @click="handleDownloadUpdate">
              <GIcon name="download" :size="15" :spin="updateStatus === 'downloading'" />
              <span>{{ t('about.update.download') }}</span>
            </button>
            <button
              v-if="canInstallUpdate"
              type="button"
              class="about-action about-update-action about-update-action--active"
              :disabled="isUpdateBusy"
              @click="handleInstallUpdate">
              <GIcon name="rocket" :size="15" :spin="updateStatus === 'installing'" />
              <span>{{ t('about.update.installAndRestart') }}</span>
            </button>
            <a
              v-if="canOpenReleasePage"
              :href="releasePageUrl"
              target="_blank"
              rel="noopener noreferrer"
              class="about-action about-update-action about-update-action--active">
              <GIcon name="external-link" :size="15" />
              <span>{{ t('about.update.openRelease') }}</span>
            </a>
          </nav>

          <p
            class="about-update-status"
            :class="`about-update-status--${updateStatusTone}`"
            aria-live="polite">
            {{ updateStatusText }}
          </p>
        </div>
      </article>

      <aside class="about-author" :aria-label="t('about.authorCardLabel')">
        <div class="about-author__avatar">
          <img :src="authorAvatarUrl" :alt="t('about.authorName')" />
          <i aria-hidden="true" />
        </div>

        <h2>{{ t('about.authorName') }}</h2>
        <p class="about-author__role">
          {{ t('about.authorRole') }}
        </p>
        <p class="about-author__quote">
          {{ t('about.authorQuote') }}
        </p>

        <div class="about-author__links">
          <a
            v-for="link in authorLinks"
            :key="link.href"
            :href="link.href"
            target="_blank"
            rel="noopener noreferrer"
            :title="link.label">
            <svg
              v-if="link.icon === 'gitee-brand'"
              class="about-author__brand-icon"
              viewBox="0 0 24 24"
              aria-hidden="true">
              <path
                d="M11.984 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.016 0zm6.09 5.333c.328 0 .593.266.592.593v1.482a.594.594 0 0 1-.593.592H9.777c-.982 0-1.778.796-1.778 1.778v5.63c0 .327.266.592.593.592h5.63c.982 0 1.778-.796 1.778-1.778v-.296a.593.593 0 0 0-.592-.593h-4.15a.592.592 0 0 1-.592-.592v-1.482a.593.593 0 0 1 .593-.592h6.815c.327 0 .593.265.593.592v3.408a4 4 0 0 1-4 4H5.926a.593.593 0 0 1-.593-.593V9.778a4.444 4.444 0 0 1 4.445-4.444h8.296Z" />
            </svg>
            <GIcon v-else :name="link.icon" :size="14" />
            <span>{{ link.label }}</span>
          </a>
        </div>
      </aside>
    </div>

    <div class="about-stats" :aria-label="t('about.versionSummary')">
      <article v-for="item in statItems" :key="item.label" class="about-stat">
        <div class="about-stat__top">
          <span>{{ item.label }}</span>
          <GIcon :name="item.icon" :size="16" />
        </div>
        <strong>{{ item.value }}</strong>
      </article>
    </div>

    <section class="about-section">
      <div class="about-section__heading">
        <div>
          <p class="about-section__kicker">
            {{ t('about.capabilitiesKicker') }}
          </p>
          <h2>{{ t('about.capabilitiesTitle') }}</h2>
        </div>
        <p>{{ t('about.capabilitiesSubtitle') }}</p>
      </div>

      <div class="about-features">
        <article v-for="item in capabilities" :key="item.title" class="about-feature">
          <span :class="['about-feature__icon', `about-feature__icon--${item.tone}`]">
            <GIcon :name="item.icon" :size="20" />
          </span>
          <h3>{{ item.title }}</h3>
          <p>{{ item.description }}</p>
        </article>
      </div>
    </section>

    <section class="about-release-grid">
      <article class="about-changelog">
        <div class="about-section__heading">
          <div>
            <p class="about-section__kicker">
              {{ t('about.releaseKicker') }}
            </p>
            <h2>{{ t('about.changelog') }}</h2>
          </div>
          <time datetime="2026-07">{{ t('about.releaseDate') }}</time>
        </div>

        <div class="about-release-card">
          <div class="about-release-card__header">
            <span>{{ t('about.releaseKicker') }}</span>
            <strong>{{ t('about.releaseTitle') }}</strong>
          </div>

          <ul class="about-release-list">
            <li v-for="item in releaseNotes" :key="item">
              <GIcon name="check-circle" :size="16" />
              <span>{{ item }}</span>
            </li>
          </ul>
        </div>
      </article>

      <aside class="about-stage" :aria-label="t('about.stageLabel')">
        <GIcon name="rocket" :size="22" />
        <p>{{ t('about.stageLabel') }}</p>
        <strong>{{ t('about.stageValue') }}</strong>
        <small>{{ t('about.stageDescription') }}</small>
        <a :href="GITHUB_ROADMAP_URL" target="_blank" rel="noopener noreferrer">
          {{ t('about.roadmap') }}
        </a>
      </aside>
    </section>

    <footer class="about-footer">
      <p>{{ t('about.copyright', { year: currentYear }) }}</p>
      <p>{{ t('about.builtWith') }}</p>
    </footer>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import appLogoUrl from '@repo-assets/logo/logo-ui.png'
import authorAvatarUrl from '@repo-assets/icon/头像.jpg'
import { useService } from '@/composables/useService'
import {
  GITEE_REPOSITORY_URL,
  GITHUB_LICENSE_URL,
  GITHUB_REPOSITORY_URL,
  GITHUB_ROADMAP_URL,
} from '@/constants'
import { UPDATE_SERVICE } from '@/services/tokens'
import type { UpdateInfo, UpdateStatus } from '@/updates'

const APP_VERSION = '0.1.0'
const BUILD_NUMBER = '2026.0704.1'

const { t } = useI18n()
const updateService = useService(UPDATE_SERVICE)
const currentYear = new Date().getFullYear()
const versionBadge = computed(() => t('about.heroBadge', { version: APP_VERSION }))
const updateStatus = ref<UpdateStatus>(updateService.getStatus())
const updateInfo = ref<UpdateInfo | null>(null)
const updateError = ref('')
const isUpdateBusy = computed(() =>
  ['checking', 'downloading', 'installing'].includes(updateStatus.value),
)
const canDownloadUpdate = computed(() => {
  const info = updateInfo.value
  return Boolean(info?.available && info.installable && updateStatus.value === 'available')
})
const canInstallUpdate = computed(() => {
  const info = updateInfo.value
  return Boolean(info?.available && info.installable && updateStatus.value === 'ready')
})
const canOpenReleasePage = computed(() => {
  const info = updateInfo.value
  return Boolean(info?.available && !info.installable && info.url)
})
const releasePageUrl = computed(() => updateInfo.value?.url ?? `${GITHUB_REPOSITORY_URL}/releases`)
const updateActionIcon = computed(() => {
  if (isUpdateBusy.value) return 'loader'
  if (updateInfo.value?.available) return 'check-circle'
  return 'refresh'
})
const updateActionLabel = computed(() => {
  if (updateStatus.value === 'checking') return t('about.update.checking')
  if (updateStatus.value === 'downloading') return t('about.update.downloading')
  if (updateStatus.value === 'installing') return t('about.update.installing')
  return t('about.update.check')
})
const updateStatusTone = computed(() => {
  if (updateStatus.value === 'error') return 'error'
  if (updateInfo.value?.available) return 'success'
  if (updateInfo.value && !updateInfo.value.available) return 'muted'
  return 'info'
})
const updateStatusText = computed(() => {
  if (updateError.value) return updateError.value
  if (updateStatus.value === 'checking') return t('about.update.statusChecking')
  if (updateStatus.value === 'downloading') return t('about.update.statusDownloading')
  if (updateStatus.value === 'ready') return t('about.update.statusReady')
  if (updateStatus.value === 'installing') return t('about.update.statusInstalling')

  if (updateInfo.value?.available) {
    return t('about.update.statusAvailable', {
      version: updateInfo.value.version ?? t('about.update.unknownVersion'),
    })
  }

  if (updateInfo.value && !updateInfo.value.available) {
    return t('about.update.statusLatest')
  }

  return t('about.update.statusIdle')
})

const productLinks = computed(() => [
  {
    label: t('common.github'),
    href: GITHUB_REPOSITORY_URL,
    icon: 'github',
    primary: true,
  },
  {
    label: t('common.website'),
    href: 'https://gate.dev',
    icon: 'globe',
    primary: false,
  },
  {
    label: t('common.license'),
    href: GITHUB_LICENSE_URL,
    icon: 'file-code',
    primary: false,
  },
])

const authorLinks = computed(() => [
  {
    label: t('common.github'),
    href: GITHUB_REPOSITORY_URL,
    icon: 'github',
  },
  {
    label: t('common.gitee'),
    href: GITEE_REPOSITORY_URL,
    icon: 'gitee-brand',
  },
  {
    label: t('about.email'),
    href: 'mailto:15035267995@163.com',
    icon: 'mail',
  },
])

const statItems = computed(() => [
  {
    label: t('about.stat.build'),
    value: BUILD_NUMBER,
    icon: 'terminal',
  },
  {
    label: t('about.stat.engine'),
    value: 'Rust & Tauri',
    icon: 'cpu',
  },
  {
    label: t('about.stat.channel'),
    value: t('about.stat.alpha'),
    icon: 'rocket',
  },
  {
    label: t('about.stat.license'),
    value: 'MIT License',
    icon: 'shield-check',
  },
])

const capabilities = computed(() => [
  {
    icon: 'servers',
    tone: 'blue',
    title: t('about.capability.selfHosted.title'),
    description: t('about.capability.selfHosted.description'),
  },
  {
    icon: 'monitor',
    tone: 'indigo',
    title: t('about.capability.desktop.title'),
    description: t('about.capability.desktop.description'),
  },
  {
    icon: 'terminal',
    tone: 'amber',
    title: t('about.capability.runtime.title'),
    description: t('about.capability.runtime.description'),
  },
  {
    icon: 'layout-grid',
    tone: 'green',
    title: t('about.capability.openSource.title'),
    description: t('about.capability.openSource.description'),
  },
])

const releaseNotes = computed(() => [
  t('about.releaseNote.clientShell'),
  t('about.releaseNote.tunnelWorkflow'),
  t('about.releaseNote.monitoring'),
  t('about.releaseNote.docs'),
])

async function handleCheckUpdate() {
  updateError.value = ''

  try {
    updateInfo.value = await updateService.check()
  } catch (error) {
    updateInfo.value = null
    updateError.value = getUpdateErrorMessage(error)
  } finally {
    updateStatus.value = updateService.getStatus()
  }
}

async function handleDownloadUpdate() {
  updateError.value = ''
  updateStatus.value = 'downloading'

  try {
    await updateService.download()
  } catch (error) {
    updateError.value = getUpdateErrorMessage(error)
  } finally {
    updateStatus.value = updateService.getStatus()
  }
}

async function handleInstallUpdate() {
  updateError.value = ''
  updateStatus.value = 'installing'

  try {
    await updateService.install()
    await updateService.restart()
  } catch (error) {
    updateError.value = getUpdateErrorMessage(error)
    updateStatus.value = updateService.getStatus()
  }
}

function getUpdateErrorMessage(error: unknown) {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return t('about.update.statusFailed')
}
</script>

<style scoped>
.about-page {
  width: min(100%, 1080px);
  min-height: 100%;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
  margin: 0 auto;
  color: var(--text-primary);
}

.about-hero-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  align-items: stretch;
  gap: var(--space-5);
}

.about-product,
.about-author,
.about-stat,
.about-feature,
.about-release-card,
.about-stage {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: rgba(24, 26, 32, 0.72);
  box-shadow: var(--shadow-xs);
}

.about-product {
  position: relative;
  min-width: 0;
  display: grid;
  grid-template-columns: 104px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-5);
  min-height: 384px;
  padding: var(--space-6);
  overflow: hidden;
  background:
    linear-gradient(135deg, rgba(91, 141, 239, 0.2), transparent 46%),
    linear-gradient(160deg, rgba(95, 179, 255, 0.08), transparent 58%), var(--bg-surface);
}

.about-logo {
  width: 104px;
  aspect-ratio: 1;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: 0;
  background:
    radial-gradient(circle at 52% 44%, rgba(56, 189, 248, 0.2), transparent 58%),
    radial-gradient(circle at 42% 62%, rgba(167, 243, 208, 0.12), transparent 60%);
}

.about-logo img {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: contain;
  filter: drop-shadow(0 16px 26px rgba(56, 189, 248, 0.16));
}

.about-product__content,
.about-title-row > div {
  min-width: 0;
}

.about-title-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.about-eyebrow,
.about-section__kicker {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  line-height: var(--leading-normal);
}

.about-product h1 {
  margin-top: var(--space-1);
  color: var(--text-primary);
  font-size: var(--text-3xl);
  font-weight: var(--weight-bold);
  letter-spacing: 0;
  line-height: var(--leading-tight);
}

.about-version-badge {
  flex: 0 0 auto;
  max-width: 160px;
  min-height: 26px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 var(--space-3);
  border: 1px solid rgba(91, 141, 239, 0.28);
  border-radius: var(--radius-sm);
  background: var(--color-primary-muted);
  color: var(--color-primary-hover);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  white-space: nowrap;
}

.about-product__lead {
  max-width: 560px;
  margin-top: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-md);
  line-height: var(--leading-relaxed);
}

.about-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-5);
}

.about-action,
.about-author__links a,
.about-stage a {
  min-height: 34px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: 0 var(--space-3);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: rgba(17, 19, 24, 0.64);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  line-height: var(--leading-normal);
  text-decoration: none;
  transition:
    background-color var(--transition-fast),
    border-color var(--transition-fast),
    color var(--transition-fast),
    transform var(--transition-fast);
}

button.about-action {
  font: inherit;
  cursor: pointer;
}

.about-action:hover,
.about-author__links a:hover,
.about-stage a:hover {
  border-color: var(--border-strong);
  background: var(--bg-surface-hover);
  color: var(--text-primary);
  transform: translateY(-1px);
}

.about-action--primary {
  border-color: transparent;
  background: var(--color-primary);
  color: var(--color-primary-fg);
  box-shadow: 0 8px 18px rgba(91, 141, 239, 0.22);
}

.about-action--primary:hover {
  border-color: transparent;
  background: var(--color-primary-hover);
  color: var(--color-primary-fg);
}

.about-update-action--active {
  border-color: rgba(47, 209, 124, 0.3);
  background: rgba(47, 209, 124, 0.12);
  color: var(--color-success);
}

.about-action:disabled {
  cursor: wait;
  opacity: 0.72;
  transform: none;
}

.about-update-status {
  min-height: 20px;
  margin-top: var(--space-3);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: var(--leading-relaxed);
}

.about-update-status--success {
  color: var(--color-success);
}

.about-update-status--error {
  color: var(--color-error);
}

.about-update-status--muted {
  color: var(--text-tertiary);
}

.about-author {
  min-width: 0;
  min-height: 384px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-5);
  text-align: center;
  background:
    linear-gradient(180deg, rgba(91, 141, 239, 0.08), transparent 42%), rgba(24, 26, 32, 0.62);
}

.about-author__avatar {
  position: relative;
  width: 76px;
  aspect-ratio: 1;
  display: grid;
  place-items: center;
  overflow: hidden;
  border: 1px solid rgba(91, 141, 239, 0.3);
  border-radius: var(--radius-full);
  background:
    linear-gradient(145deg, rgba(91, 141, 239, 0.26), rgba(47, 209, 124, 0.16)), var(--bg-input);
}

.about-author__avatar img {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: cover;
}

.about-author__avatar i {
  position: absolute;
  right: 5px;
  bottom: 5px;
  width: 14px;
  aspect-ratio: 1;
  border: 2px solid var(--bg-surface);
  border-radius: var(--radius-full);
  background: var(--color-success);
  box-shadow: 0 0 12px rgba(47, 209, 124, 0.38);
}

.about-author h2 {
  margin-top: var(--space-4);
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.about-author__role {
  margin-top: var(--space-1);
  color: var(--color-primary-hover);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
}

.about-author__quote {
  max-width: 260px;
  margin-top: var(--space-3);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.about-author__links {
  width: 100%;
  max-width: 520px;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-2);
  margin-top: var(--space-5);
  padding-top: 0;
}

.about-author__links a {
  min-width: 0;
  min-height: 36px;
  padding: 0 var(--space-2);
}

.about-author__brand-icon {
  width: 14px;
  height: 14px;
  flex: 0 0 auto;
  fill: #c71d23;
}

.about-author__links span,
.about-action span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.about-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
}

.about-stat {
  min-width: 0;
  min-height: 86px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-4);
  transition:
    background-color var(--transition-fast),
    border-color var(--transition-fast);
}

.about-stat:hover {
  border-color: var(--border-default);
  background: var(--bg-surface-hover);
}

.about-stat__top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
}

.about-stat__top svg {
  color: var(--text-tertiary);
}

.about-stat strong {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  line-height: var(--leading-normal);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.about-section {
  padding-top: var(--space-4);
}

.about-section__heading {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--space-5);
  margin-bottom: var(--space-4);
}

.about-section__heading h2 {
  margin-top: var(--space-1);
  color: var(--text-primary);
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  line-height: var(--leading-tight);
  letter-spacing: 0;
}

.about-section__heading > p {
  max-width: 360px;
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
  text-align: right;
}

.about-section__heading time {
  flex: 0 0 auto;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.about-features {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.about-feature {
  min-width: 0;
  min-height: 176px;
  padding: var(--space-5);
  transition:
    background-color var(--transition-fast),
    border-color var(--transition-fast),
    transform var(--transition-fast);
}

.about-feature:hover {
  border-color: var(--border-default);
  background: var(--bg-surface-hover);
  transform: translateY(-1px);
}

.about-feature__icon {
  width: 40px;
  aspect-ratio: 1;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.about-feature__icon--blue {
  color: var(--color-info);
}

.about-feature__icon--indigo {
  color: var(--color-primary-hover);
}

.about-feature__icon--amber {
  color: var(--color-warning);
}

.about-feature__icon--green {
  color: var(--color-success);
}

.about-feature h3 {
  margin-top: var(--space-4);
  color: var(--text-primary);
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  line-height: var(--leading-tight);
  letter-spacing: 0;
}

.about-feature p {
  margin-top: var(--space-2);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.about-release-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 280px;
  gap: var(--space-5);
  padding-top: var(--space-4);
}

.about-changelog {
  min-width: 0;
}

.about-release-card {
  padding: var(--space-5);
}

.about-release-card__header {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-4);
}

.about-release-card__header span {
  min-height: 22px;
  display: inline-flex;
  align-items: center;
  padding: 0 var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--color-primary-muted);
  color: var(--color-primary-hover);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  font-weight: var(--weight-bold);
}

.about-release-card__header strong {
  color: var(--text-primary);
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
}

.about-release-list {
  display: grid;
  gap: var(--space-3);
  list-style: none;
}

.about-release-list li {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: start;
  gap: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.about-release-list svg {
  margin-top: 2px;
  color: var(--color-success);
}

.about-stage {
  position: relative;
  min-width: 0;
  align-self: start;
  display: flex;
  flex-direction: column;
  min-height: 100%;
  padding: var(--space-5);
  overflow: hidden;
  border-color: rgba(245, 184, 75, 0.24);
  background:
    linear-gradient(180deg, rgba(245, 184, 75, 0.14), rgba(245, 184, 75, 0.04)), var(--bg-surface);
  color: var(--color-warning);
}

.about-stage > svg {
  width: 42px;
  height: 42px;
  padding: 10px;
  border: 1px solid rgba(245, 184, 75, 0.28);
  border-radius: var(--radius-md);
  background: rgba(245, 184, 75, 0.14);
}

.about-stage p {
  margin-top: var(--space-4);
  color: var(--color-warning);
  font-size: var(--text-xs);
  font-weight: var(--weight-bold);
}

.about-stage strong {
  margin-top: var(--space-1);
  color: var(--text-primary);
  font-size: var(--text-xl);
  font-weight: var(--weight-bold);
  line-height: var(--leading-tight);
}

.about-stage small {
  margin-top: var(--space-3);
  color: rgba(245, 230, 195, 0.78);
  font-size: var(--text-sm);
  line-height: var(--leading-relaxed);
}

.about-stage a {
  width: 100%;
  margin-top: auto;
  padding: 0 var(--space-3);
  border-color: rgba(245, 184, 75, 0.28);
  background: rgba(245, 184, 75, 0.1);
  color: var(--color-warning-hover);
}

.about-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-5) 0 var(--space-8);
  border-top: 1px solid var(--border-subtle);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

@media (max-width: 980px) {
  .about-hero-grid,
  .about-release-grid {
    grid-template-columns: 1fr;
  }

  .about-product,
  .about-author {
    min-height: auto;
  }

  .about-stage {
    min-height: 260px;
  }
}

@media (max-width: 760px) {
  .about-product,
  .about-section__heading,
  .about-footer {
    grid-template-columns: 1fr;
  }

  .about-product {
    display: flex;
    flex-direction: column;
  }

  .about-title-row {
    flex-direction: column;
    gap: var(--space-3);
  }

  .about-stats,
  .about-features {
    grid-template-columns: 1fr;
  }

  .about-section__heading {
    align-items: flex-start;
  }

  .about-section__heading > p {
    max-width: none;
    text-align: left;
  }

  .about-footer {
    align-items: flex-start;
    flex-direction: column;
  }
}

@media (max-width: 520px) {
  .about-page {
    gap: var(--space-4);
  }

  .about-product,
  .about-author,
  .about-feature,
  .about-release-card,
  .about-stage {
    padding: var(--space-4);
  }

  .about-action {
    width: 100%;
  }
}
</style>
