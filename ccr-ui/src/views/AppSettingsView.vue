<template>
  <div class="app-settings-view">
    <div class="app-settings-shell">
      <header class="app-settings-hero">
        <div class="app-settings-hero__intro">
          <div class="app-settings-hero__icon">
            <SIcon
              name="SlidersHorizontal"
              size="w-6 h-6"
            />
          </div>
          <div class="space-y-2">
            <p class="app-settings-hero__eyebrow">
              {{ t('settings.eyebrow') }}
            </p>
            <div>
              <h1 class="app-settings-hero__title">
                {{ t('settings.title') }}
              </h1>
              <p class="app-settings-hero__description">
                {{ t('settings.description') }}
              </p>
            </div>
          </div>
        </div>

        <div class="app-settings-summary">
          <span class="app-settings-summary__pill">
            {{ runtimeLabel }}
          </span>
          <span
            v-if="runtimeVersion"
            class="app-settings-summary__pill app-settings-summary__pill--mono"
          >
            v{{ runtimeVersion }}
          </span>
          <span class="app-settings-summary__pill">
            {{ themeSummaryLabel }}
          </span>
          <span class="app-settings-summary__pill">
            {{ localeLabel }}
          </span>
          <span class="app-settings-summary__pill app-settings-summary__pill--mono">
            {{ sidebarWidth }}px
          </span>
        </div>
      </header>

      <div class="app-settings-layout">
        <aside class="app-settings-nav">
          <div class="app-settings-nav__inner">
            <button
              v-for="section in sections"
              :key="section.key"
              type="button"
              class="app-settings-nav__button"
              :class="{ 'app-settings-nav__button--active': activeSection === section.key }"
              :data-testid="`settings-section-${section.key}`"
              @click="scrollToSection(section.key)"
            >
              <span class="app-settings-nav__icon">
                <SIcon
                  :name="section.icon"
                  size="w-4 h-4"
                />
              </span>
              <span>
                <span class="app-settings-nav__title">{{ section.title }}</span>
                <span class="app-settings-nav__caption">{{ section.caption }}</span>
              </span>
            </button>
          </div>
        </aside>

        <div class="app-settings-content">
          <section :ref="setSectionRef('appearance')">
            <Card
              variant="glass"
              class-name="app-settings-card"
            >
              <div class="app-settings-card__header">
                <div>
                  <p class="app-settings-card__eyebrow">
                    {{ t('settings.appearance.eyebrow') }}
                  </p>
                  <h2 class="app-settings-card__title">
                    {{ t('settings.appearance.title') }}
                  </h2>
                </div>
                <p class="app-settings-card__description">
                  {{ t('settings.appearance.description') }}
                </p>
              </div>

              <div class="app-settings-option-grid">
                <button
                  v-for="option in themeOptions"
                  :key="option.value"
                  type="button"
                  class="app-settings-option"
                  :class="{ 'app-settings-option--active': theme === option.value }"
                  :data-testid="`settings-theme-${option.value}`"
                  :aria-pressed="theme === option.value"
                  @click="setTheme(option.value)"
                >
                  <div class="app-settings-option__meta">
                    <span class="app-settings-option__icon">
                      <SIcon
                        :name="option.icon"
                        size="w-4 h-4"
                      />
                    </span>
                    <span>
                      <span class="app-settings-option__title">{{ option.label }}</span>
                      <span class="app-settings-option__caption">{{ option.description }}</span>
                    </span>
                  </div>
                  <span class="app-settings-option__status">
                    {{ theme === option.value ? t('settings.active') : option.badge }}
                  </span>
                </button>
              </div>
            </Card>
          </section>

          <section :ref="setSectionRef('language')">
            <Card
              variant="glass"
              class-name="app-settings-card"
            >
              <div class="app-settings-card__header">
                <div>
                  <p class="app-settings-card__eyebrow">
                    {{ t('settings.language.eyebrow') }}
                  </p>
                  <h2 class="app-settings-card__title">
                    {{ t('settings.language.title') }}
                  </h2>
                </div>
                <p class="app-settings-card__description">
                  {{ t('settings.language.description') }}
                </p>
              </div>

              <div class="app-settings-option-grid app-settings-option-grid--compact">
                <button
                  v-for="option in languageOptions"
                  :key="option.value"
                  type="button"
                  class="app-settings-option"
                  :class="{ 'app-settings-option--active': locale === option.value }"
                  :data-testid="`settings-language-${option.value}`"
                  :aria-pressed="locale === option.value"
                  @click="setLocalePreference(option.value)"
                >
                  <div class="app-settings-option__meta">
                    <span class="app-settings-option__icon app-settings-option__icon--plain">
                      {{ option.flag }}
                    </span>
                    <span>
                      <span class="app-settings-option__title">{{ option.label }}</span>
                      <span class="app-settings-option__caption">{{ option.description }}</span>
                    </span>
                  </div>
                  <span class="app-settings-option__status">
                    {{ locale === option.value ? t('settings.active') : t('settings.language.instant') }}
                  </span>
                </button>
              </div>
            </Card>
          </section>

          <section :ref="setSectionRef('shell')">
            <Card
              variant="glass"
              class-name="app-settings-card"
            >
              <div class="app-settings-card__header">
                <div>
                  <p class="app-settings-card__eyebrow">
                    {{ t('settings.shell.eyebrow') }}
                  </p>
                  <h2 class="app-settings-card__title">
                    {{ t('settings.shell.title') }}
                  </h2>
                </div>
                <p class="app-settings-card__description">
                  {{ t('settings.shell.description') }}
                </p>
              </div>

              <div class="app-settings-stack">
                <div class="app-settings-row">
                  <div class="app-settings-row__copy">
                    <h3 class="app-settings-row__title">
                      {{ t('settings.shell.exitConfirmTitle') }}
                    </h3>
                    <p class="app-settings-row__description">
                      {{ t('settings.shell.exitConfirmDescription') }}
                    </p>
                  </div>
                  <button
                    type="button"
                    role="switch"
                    class="app-settings-switch"
                    :class="{ 'app-settings-switch--active': confirmBeforeExit }"
                    :aria-checked="confirmBeforeExit"
                    data-testid="settings-confirm-exit-toggle"
                    @click="toggleConfirmBeforeExit"
                  >
                    <span class="app-settings-switch__track" />
                    <span class="app-settings-switch__thumb" />
                    <span class="app-settings-switch__label">
                      {{ confirmBeforeExit ? t('settings.enabled') : t('settings.disabled') }}
                    </span>
                  </button>
                </div>

                <div class="app-settings-row">
                  <div class="app-settings-row__copy">
                    <h3 class="app-settings-row__title">
                      {{ t('settings.shell.closeToTrayTitle') }}
                    </h3>
                    <p class="app-settings-row__description">
                      {{ t('settings.shell.closeToTrayDescription') }}
                    </p>
                  </div>
                  <button
                    type="button"
                    role="switch"
                    class="app-settings-switch"
                    :class="{ 'app-settings-switch--active': closeToTray }"
                    :aria-checked="closeToTray"
                    data-testid="settings-close-to-tray-toggle"
                    @click="toggleCloseToTray"
                  >
                    <span class="app-settings-switch__track" />
                    <span class="app-settings-switch__thumb" />
                    <span class="app-settings-switch__label">
                      {{ closeToTray ? t('settings.enabled') : t('settings.disabled') }}
                    </span>
                  </button>
                </div>

                <div class="app-settings-row">
                  <div class="app-settings-row__copy">
                    <h3 class="app-settings-row__title">
                      {{ t('settings.shell.openPanelOnTrayClickTitle') }}
                    </h3>
                    <p class="app-settings-row__description">
                      {{ t('settings.shell.openPanelOnTrayClickDescription') }}
                    </p>
                  </div>
                  <button
                    type="button"
                    role="switch"
                    class="app-settings-switch"
                    :class="{ 'app-settings-switch--active': openPanelOnTrayClick }"
                    :aria-checked="openPanelOnTrayClick"
                    data-testid="settings-open-panel-on-tray-click-toggle"
                    @click="toggleOpenPanelOnTrayClick"
                  >
                    <span class="app-settings-switch__track" />
                    <span class="app-settings-switch__thumb" />
                    <span class="app-settings-switch__label">
                      {{ openPanelOnTrayClick ? t('settings.enabled') : t('settings.disabled') }}
                    </span>
                  </button>
                </div>

                <div class="app-settings-row app-settings-row--slider">
                  <div class="app-settings-row__copy">
                    <h3 class="app-settings-row__title">
                      {{ t('settings.shell.sidebarWidthTitle') }}
                    </h3>
                    <p class="app-settings-row__description">
                      {{ t('settings.shell.sidebarWidthDescription') }}
                    </p>
                  </div>
                  <div class="app-settings-slider">
                    <input
                      v-model.number="sidebarWidthModel"
                      type="range"
                      min="200"
                      max="480"
                      step="8"
                      class="app-settings-slider__control"
                      data-testid="settings-sidebar-width-slider"
                    >
                    <div class="app-settings-slider__meta">
                      <span>200</span>
                      <strong>{{ sidebarWidth }}px</strong>
                      <span>480</span>
                    </div>
                  </div>
                </div>

                <div class="app-settings-row">
                  <div class="app-settings-row__copy">
                    <h3 class="app-settings-row__title">
                      {{ t('settings.shell.resetLayoutTitle') }}
                    </h3>
                    <p class="app-settings-row__description">
                      {{ t('settings.shell.resetLayoutDescription') }}
                    </p>
                  </div>
                  <Button
                    variant="secondary"
                    surface="status"
                    density="compact"
                    motion="subtle"
                    v-bind="{ 'data-testid': 'settings-reset-layout' }"
                    @click="resetLayout"
                  >
                    <template #leading>
                      <SIcon
                        name="RotateCw"
                        size="w-4 h-4"
                      />
                    </template>
                    {{ t('settings.shell.resetLayoutAction') }}
                  </Button>
                </div>
              </div>
            </Card>
          </section>

          <section :ref="setSectionRef('diagnostics')">
            <Card
              variant="glass"
              class-name="app-settings-card"
            >
              <div class="app-settings-card__header">
                <div>
                  <p class="app-settings-card__eyebrow">
                    {{ t('settings.diagnostics.eyebrow') }}
                  </p>
                  <h2 class="app-settings-card__title">
                    {{ t('settings.diagnostics.title') }}
                  </h2>
                </div>
                <p class="app-settings-card__description">
                  {{ t('settings.diagnostics.description') }}
                </p>
              </div>

              <div class="app-settings-stack">
                <div class="app-settings-row">
                  <div class="app-settings-row__copy">
                    <h3 class="app-settings-row__title">
                      {{ t('settings.diagnostics.perfTitle') }}
                    </h3>
                    <p class="app-settings-row__description">
                      {{ t('settings.diagnostics.perfDescription') }}
                    </p>
                  </div>
                  <button
                    type="button"
                    role="switch"
                    class="app-settings-switch"
                    :class="{ 'app-settings-switch--active': perfTelemetryEnabled }"
                    :aria-checked="perfTelemetryEnabled"
                    data-testid="settings-perf-toggle"
                    @click="togglePerfTelemetry"
                  >
                    <span class="app-settings-switch__track" />
                    <span class="app-settings-switch__thumb" />
                    <span class="app-settings-switch__label">
                      {{ perfTelemetryEnabled ? t('settings.enabled') : t('settings.disabled') }}
                    </span>
                  </button>
                </div>

                <div class="app-settings-callout">
                  <SIcon
                    name="Info"
                    size="w-4 h-4"
                    class="mt-0.5 text-accent-primary"
                  />
                  <p>
                    {{ t('settings.diagnostics.restartNote') }}
                  </p>
                </div>
              </div>
            </Card>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, type ComponentPublicInstance } from 'vue'
import { storeToRefs } from 'pinia'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useI18n } from 'vue-i18n'
import { getEnvironmentName, getTauriVersion, isTauriEnvironment } from '@/api/runtime/environment'
import { translateWithFallback } from '@/i18n/formatMessage'
import { useShellPreferencesStore } from '@/stores/shellPreferences'
import type { ThemeMode } from '@/utils/themeBootstrap'

type SectionKey = 'appearance' | 'language' | 'shell' | 'diagnostics'

const { t } = useI18n()
const shellPreferencesStore = useShellPreferencesStore()
const {
  closeToTray,
  confirmBeforeExit,
  effectiveTheme,
  locale,
  openPanelOnTrayClick,
  perfTelemetryEnabled,
  sidebarWidth,
  theme,
} =
  storeToRefs(shellPreferencesStore)

const runtimeVersion = ref<string | null>(null)
const activeSection = ref<SectionKey>('appearance')
const sectionElements = ref<Record<SectionKey, HTMLElement | null>>({
  appearance: null,
  language: null,
  shell: null,
  diagnostics: null,
})

const sections = computed(() => [
  {
    key: 'appearance' as const,
    icon: 'Sun',
    title: t('settings.appearance.title'),
    caption: t('settings.appearance.navCaption'),
  },
  {
    key: 'language' as const,
    icon: 'Languages',
    title: t('settings.language.title'),
    caption: t('settings.language.navCaption'),
  },
  {
    key: 'shell' as const,
    icon: 'PanelLeftOpen',
    title: t('settings.shell.title'),
    caption: t('settings.shell.navCaption'),
  },
  {
    key: 'diagnostics' as const,
    icon: 'Activity',
    title: t('settings.diagnostics.title'),
    caption: t('settings.diagnostics.navCaption'),
  },
])

const runtimeLabel = computed(() => (
  getEnvironmentName() === 'tauri'
    ? t('settings.summary.runtimeDesktop')
    : t('settings.summary.runtimeWeb')
))

const localeLabel = computed(() => (
  locale.value === 'en-US' ? t('language.english') : t('language.chinese')
))

const themeSummaryLabel = computed(() => {
  if (theme.value === 'system') {
    const resolvedLabel = t(`theme.${effectiveTheme.value}`)
    return translateWithFallback(
      t,
      'settings.appearance.systemSummary',
      `${t('theme.system')} · {resolved}`,
      { resolved: resolvedLabel },
    )
  }

  return t(`theme.${theme.value}`)
})

const themeOptions = computed(() => [
  {
    value: 'light' as ThemeMode,
    icon: 'Sun',
    label: t('theme.light'),
    description: t('settings.appearance.lightDescription'),
    badge: t('settings.appearance.dayBadge'),
  },
  {
    value: 'dark' as ThemeMode,
    icon: 'Moon',
    label: t('theme.dark'),
    description: t('settings.appearance.darkDescription'),
    badge: t('settings.appearance.nightBadge'),
  },
  {
    value: 'system' as ThemeMode,
    icon: 'Monitor',
    label: t('theme.system'),
    description: t('settings.appearance.systemDescription'),
    badge: t('settings.appearance.autoBadge'),
  },
])

const languageOptions = computed(() => [
  {
    value: 'zh-CN',
    flag: 'CN',
    label: t('language.chinese'),
    description: t('settings.language.chineseDescription'),
  },
  {
    value: 'en-US',
    flag: 'US',
    label: t('language.english'),
    description: t('settings.language.englishDescription'),
  },
])

const sidebarWidthModel = computed({
  get: () => sidebarWidth.value,
  set: (nextWidth: number) => {
    shellPreferencesStore.updateSidebarWidth(nextWidth)
  },
})

const setSectionRef = (key: SectionKey) => (element: Element | ComponentPublicInstance | null) => {
  sectionElements.value[key] = element instanceof HTMLElement ? element : null
}

const scrollToSection = async (key: SectionKey) => {
  activeSection.value = key
  await nextTick()
  sectionElements.value[key]?.scrollIntoView({
    behavior: 'smooth',
    block: 'start',
  })
}

const setTheme = (nextTheme: ThemeMode) => {
  shellPreferencesStore.setTheme(nextTheme)
}

const setLocalePreference = async (nextLocale: string) => {
  await shellPreferencesStore.setLocalePreference(nextLocale)
}

const toggleConfirmBeforeExit = async () => {
  await shellPreferencesStore.setConfirmBeforeExit(!confirmBeforeExit.value)
}

const toggleCloseToTray = async () => {
  await shellPreferencesStore.setCloseToTray(!closeToTray.value)
}

const toggleOpenPanelOnTrayClick = async () => {
  await shellPreferencesStore.setOpenPanelOnTrayClick(!openPanelOnTrayClick.value)
}

const togglePerfTelemetry = () => {
  shellPreferencesStore.setPerfTelemetryPreference(!perfTelemetryEnabled.value)
}

const resetLayout = () => {
  shellPreferencesStore.resetLayout()
}

onMounted(async () => {
  await shellPreferencesStore.hydrateRuntimePreferences()

  if (isTauriEnvironment()) {
    runtimeVersion.value = await getTauriVersion()
  }
})
</script>

<style scoped>
.app-settings-view {
  @apply min-h-full;
}

.app-settings-shell {
  @apply mx-auto flex max-w-[1440px] flex-col gap-6;
}

.app-settings-hero {
  @apply flex flex-col gap-5 rounded-[2rem] border px-6 py-6 lg:flex-row lg:items-end lg:justify-between;

  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 94%), rgb(var(--color-bg-surface-rgb) / 88%));
  border-color: rgb(var(--color-border-default-rgb) / 48%);
  box-shadow:
    0 24px 48px rgb(73 54 40 / 10%),
    inset 0 1px 0 rgb(255 251 245 / 14%);
}

.app-settings-hero__intro {
  @apply flex items-start gap-4;
}

.app-settings-hero__icon {
  @apply flex h-14 w-14 items-center justify-center rounded-[1.4rem] border shadow-sm;

  color: var(--color-accent-primary);
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  background:
    radial-gradient(circle at top, rgb(var(--color-accent-primary-rgb) / 20%), transparent 65%),
    rgb(var(--color-bg-base-rgb) / 78%);
}

.app-settings-hero__eyebrow,
.app-settings-card__eyebrow {
  @apply text-[11px] font-semibold uppercase tracking-[0.22em];

  color: var(--color-text-muted);
}

.app-settings-hero__title {
  @apply text-[2rem] font-semibold tracking-[-0.06em] sm:text-[2.4rem];

  color: var(--color-text-primary);
}

.app-settings-hero__description,
.app-settings-card__description {
  @apply max-w-[56ch] text-sm leading-6;

  color: var(--color-text-secondary);
}

.app-settings-summary {
  @apply flex flex-wrap items-center gap-2.5;
}

.app-settings-summary__pill {
  @apply inline-flex items-center rounded-full border px-3 py-1.5 text-[11px] font-semibold tracking-[0.14em] uppercase;

  border-color: rgb(var(--color-border-default-rgb) / 56%);
  background: rgb(var(--color-bg-base-rgb) / 72%);
  color: var(--color-text-secondary);
}

.app-settings-summary__pill--mono {
  @apply font-mono tracking-[0.08em];
}

.app-settings-layout {
  @apply grid gap-5 xl:grid-cols-[260px_minmax(0,1fr)];
}

.app-settings-nav {
  @apply xl:sticky xl:top-6 xl:self-start;
}

.app-settings-nav__inner {
  @apply flex gap-2 overflow-x-auto pb-1 xl:flex-col xl:overflow-visible;
}

.app-settings-nav__button {
  @apply flex min-h-[72px] min-w-[220px] items-start gap-3 rounded-[1.4rem] border px-4 py-3 text-left transition-[border-color,background-color,box-shadow,transform] duration-200 xl:min-w-0;

  border-color: rgb(var(--color-border-default-rgb) / 44%);
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
  color: var(--color-text-secondary);
}

.app-settings-nav__button:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--color-accent-primary-rgb) / 16%);
  background: rgb(var(--color-bg-surface-rgb) / 84%);
}

.app-settings-nav__button--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 22%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 96%), rgb(var(--color-bg-surface-rgb) / 88%));
  box-shadow: 0 16px 30px rgb(var(--color-accent-primary-rgb) / 8%);
  color: var(--color-text-primary);
}

.app-settings-nav__icon {
  @apply mt-0.5 flex h-9 w-9 flex-none items-center justify-center rounded-2xl border;

  border-color: rgb(var(--color-border-default-rgb) / 46%);
  background: rgb(var(--color-bg-base-rgb) / 78%);
}

.app-settings-nav__title,
.app-settings-option__title,
.app-settings-row__title,
.app-settings-card__title {
  @apply block text-sm font-semibold;

  color: var(--color-text-primary);
}

.app-settings-nav__caption,
.app-settings-option__caption {
  @apply mt-1 block text-xs leading-5;

  color: var(--color-text-secondary);
}

.app-settings-content {
  @apply flex flex-col gap-5;
}

.app-settings-card {
  @apply p-5 sm:p-6;
}

.app-settings-card__header {
  @apply flex flex-col gap-3 border-b border-border-default/40 pb-5 lg:flex-row lg:items-end lg:justify-between;
}

.app-settings-option-grid {
  @apply mt-5 grid gap-3 xl:grid-cols-3;
}

.app-settings-option-grid--compact {
  @apply xl:grid-cols-2;
}

.app-settings-option {
  @apply flex min-h-[136px] flex-col justify-between rounded-[1.4rem] border p-4 text-left transition-[border-color,background-color,box-shadow,transform] duration-200;

  border-color: rgb(var(--color-border-default-rgb) / 44%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 82%), rgb(var(--color-bg-surface-rgb) / 72%));
}

.app-settings-option:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  box-shadow: 0 18px 32px rgb(73 54 40 / 10%);
}

.app-settings-option--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  box-shadow: 0 18px 34px rgb(var(--color-accent-primary-rgb) / 10%);
}

.app-settings-option__meta {
  @apply flex items-start gap-3;
}

.app-settings-option__icon {
  @apply flex h-10 w-10 flex-none items-center justify-center rounded-2xl border;

  border-color: rgb(var(--color-border-default-rgb) / 46%);
  background: rgb(var(--color-bg-base-rgb) / 82%);
  color: var(--color-text-primary);
}

.app-settings-option__icon--plain {
  @apply text-sm font-semibold tracking-[0.18em];
}

.app-settings-option__status {
  @apply inline-flex w-fit items-center rounded-full border px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em];

  border-color: rgb(var(--color-border-default-rgb) / 52%);
  background: rgb(var(--color-bg-base-rgb) / 68%);
  color: var(--color-text-muted);
}

.app-settings-option--active .app-settings-option__status {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: var(--color-accent-primary);
}

.app-settings-stack {
  @apply mt-5 flex flex-col gap-4;
}

.app-settings-row {
  @apply flex flex-col gap-4 rounded-[1.4rem] border px-4 py-4 lg:flex-row lg:items-center lg:justify-between;

  border-color: rgb(var(--color-border-default-rgb) / 44%);
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
}

.app-settings-row--slider {
  @apply items-stretch;
}

.app-settings-row__copy {
  @apply max-w-[56ch];
}

.app-settings-row__description,
.app-settings-callout {
  @apply text-sm leading-6;

  color: var(--color-text-secondary);
}

.app-settings-switch {
  @apply inline-flex items-center gap-3 rounded-full border px-3 py-2 text-xs font-semibold uppercase tracking-[0.14em] transition-[border-color,background-color,box-shadow] duration-200;

  border-color: rgb(var(--color-border-default-rgb) / 56%);
  background: rgb(var(--color-bg-base-rgb) / 78%);
  color: var(--color-text-muted);
}

.app-settings-switch--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
}

.app-settings-switch__track {
  @apply relative h-6 w-10 rounded-full;

  background: rgb(var(--color-border-default-rgb) / 55%);
}

.app-settings-switch--active .app-settings-switch__track {
  background: rgb(var(--color-accent-primary-rgb) / 24%);
}

.app-settings-switch__thumb {
  @apply absolute ml-0.5 h-5 w-5 rounded-full border transition-transform duration-200;

  transform: translateX(0);
  border-color: rgb(var(--color-border-default-rgb) / 62%);
  background: rgb(var(--color-bg-elevated-rgb) / 100%);
}

.app-settings-switch--active .app-settings-switch__thumb {
  transform: translateX(16px);
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
}

.app-settings-switch__label {
  @apply min-w-[4.75rem] text-left;
}

.app-settings-slider {
  @apply w-full max-w-[360px] rounded-[1.2rem] border px-4 py-3;

  border-color: rgb(var(--color-border-default-rgb) / 46%);
  background: rgb(var(--color-bg-base-rgb) / 74%);
}

.app-settings-slider__control {
  @apply w-full accent-accent-primary;
}

.app-settings-slider__meta {
  @apply mt-2 flex items-center justify-between text-xs;

  color: var(--color-text-muted);
}

.app-settings-slider__meta strong {
  color: var(--color-text-primary);
}

.app-settings-callout {
  @apply flex items-start gap-2 rounded-[1.2rem] border px-4 py-3;

  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  background: rgb(var(--color-accent-primary-rgb) / 6%);
}

@media (width <= 1279px) {
  .app-settings-nav {
    @apply sticky top-0 z-10;
  }

  .app-settings-nav__inner {
    @apply rounded-[1.4rem] border p-2;

    border-color: rgb(var(--color-border-default-rgb) / 44%);
    background: rgb(var(--color-bg-base-rgb) / 86%);
    backdrop-filter: blur(12px);
  }

  .app-settings-nav__button {
    @apply min-h-[64px] min-w-[180px];
  }
}
</style>
