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
                    {{ t('settings.appearance.theme.eyebrow') }}
                  </p>
                  <h2 class="app-settings-card__title">
                    {{ t('settings.appearance.theme.title') }}
                  </h2>
                </div>
                <p class="app-settings-card__description">
                  {{ t('settings.appearance.theme.description') }}
                </p>
              </div>

              <div
                class="app-settings-segmented"
                role="radiogroup"
                :aria-label="t('settings.appearance.theme.title')"
              >
                <button
                  v-for="option in themeOptions"
                  :key="option.value"
                  type="button"
                  role="radio"
                  class="app-settings-segmented__option"
                  :class="{ 'app-settings-segmented__option--active': theme === option.value }"
                  :data-testid="`settings-theme-${option.value}`"
                  :aria-checked="theme === option.value"
                  :aria-pressed="theme === option.value"
                  @click="setTheme(option.value)"
                >
                  <span class="app-settings-segmented__icon">
                    <SIcon
                      :name="option.icon"
                      size="w-4 h-4"
                    />
                  </span>
                  <span class="app-settings-segmented__copy">
                    <span class="app-settings-segmented__title">{{ option.label }}</span>
                    <span class="app-settings-segmented__caption">{{ option.description }}</span>
                  </span>
                </button>
              </div>
              <p
                v-if="theme === 'system'"
                class="app-settings-segmented__resolved"
              >
                {{ systemResolvedHint }}
              </p>
            </Card>

            <Card
              variant="glass"
              class-name="app-settings-card app-settings-card--tight"
            >
              <div class="app-settings-card__header">
                <div>
                  <p class="app-settings-card__eyebrow">
                    {{ t('settings.appearance.flavor.eyebrow') }}
                  </p>
                  <h2 class="app-settings-card__title">
                    {{ t('settings.appearance.flavor.title') }}
                  </h2>
                </div>
                <p class="app-settings-card__description">
                  {{ t('settings.appearance.flavor.description') }}
                </p>
              </div>

              <div class="app-settings-flavor-grid">
                <button
                  v-for="option in flavorOptions"
                  :key="option.value"
                  type="button"
                  class="app-settings-flavor-card"
                  :class="{ 'app-settings-flavor-card--active': flavor === option.value }"
                  :data-testid="`settings-flavor-${option.value}`"
                  :aria-pressed="flavor === option.value"
                  @click="setFlavor(option.value)"
                >
                  <span
                    class="app-settings-flavor-card__preview"
                    :data-preview-flavor="option.value"
                    :style="flavorPreviewStyle(option.value)"
                  >
                    <span class="fp-surface">
                      <span class="fp-text">{{ PREVIEW_GLYPH_SAMPLE }}</span>
                      <span class="fp-muted">{{ PREVIEW_GLYPH_SAMPLE }}</span>
                      <i class="fp-accent" />
                    </span>
                  </span>
                  <span class="app-settings-flavor-card__copy">
                    <span class="app-settings-option__title">{{ option.label }}</span>
                    <span class="app-settings-option__caption">{{ option.description }}</span>
                  </span>
                  <span
                    v-if="flavor === option.value"
                    class="app-settings-flavor-card__footer"
                  >
                    <span
                      class="app-settings-flavor-card__dot"
                      aria-hidden="true"
                    />
                    <span class="app-settings-option__status app-settings-option__status--active">
                      {{ flavorStatusLabel(option) }}
                    </span>
                  </span>
                </button>
              </div>
            </Card>

            <Card
              variant="glass"
              class-name="app-settings-card app-settings-card--tight"
            >
              <div class="app-settings-card__header">
                <div>
                  <p class="app-settings-card__eyebrow">
                    {{ t('settings.appearance.accent.eyebrow') }}
                  </p>
                  <h2 class="app-settings-card__title">
                    {{ t('settings.appearance.accent.title') }}
                  </h2>
                </div>
                <p class="app-settings-card__description">
                  {{ t('settings.appearance.accent.description') }}
                </p>
              </div>

              <div
                class="app-settings-accent-grid"
                role="radiogroup"
                :aria-label="t('settings.appearance.accent.title')"
              >
                <button
                  v-for="option in accentOptions"
                  :key="option.value"
                  type="button"
                  role="radio"
                  class="app-settings-accent-option"
                  :class="{ 'app-settings-accent-option--active': accent === option.value }"
                  :aria-checked="accent === option.value"
                  :data-testid="`settings-accent-${option.value}`"
                  @click="setAccent(option.value)"
                >
                  <span
                    class="app-settings-accent-option__preview"
                    :data-preview-accent="option.value"
                    :style="accentPreviewStyle(option.value)"
                  >
                    <span class="fp-accent-button">{{ PREVIEW_GLYPH_SAMPLE }}</span>
                  </span>
                  <span class="app-settings-accent-option__copy">
                    <span class="app-settings-option__title">{{ option.label }}</span>
                    <span class="app-settings-option__caption">{{ option.description }}</span>
                  </span>
                  <span
                    v-if="accent === option.value"
                    class="app-settings-accent-option__dot"
                    aria-hidden="true"
                  />
                </button>
              </div>
            </Card>

            <Card
              variant="glass"
              class-name="app-settings-card app-settings-card--tight"
            >
              <div class="app-settings-card__header">
                <div>
                  <p class="app-settings-card__eyebrow">
                    {{ t('settings.appearance.typography.eyebrow') }}
                  </p>
                  <h2 class="app-settings-card__title">
                    {{ t('settings.appearance.typography.title') }}
                  </h2>
                </div>
                <p class="app-settings-card__description">
                  {{ t('settings.appearance.typography.description') }}
                </p>
              </div>

              <div class="app-settings-stack">
                <div class="app-settings-row app-settings-row--font">
                  <div class="app-settings-row__copy">
                    <h3 class="app-settings-row__title">
                      {{ t('settings.appearance.typography.uiLabel') }}
                    </h3>
                    <p class="app-settings-row__description">
                      {{ t('settings.appearance.typography.uiDescription') }}
                    </p>
                  </div>
                  <div class="app-settings-font-control">
                    <select
                      class="app-settings-font-select"
                      :value="uiSelectValue"
                      :aria-label="t('settings.appearance.typography.uiLabel')"
                      data-testid="settings-font-ui"
                      @change="onUiSelect(($event.target as HTMLSelectElement).value)"
                    >
                      <option value="__default__">
                        {{ t('settings.appearance.typography.systemDefault') }}
                      </option>
                      <option
                        v-for="preset in UI_FONT_PRESETS"
                        :key="preset"
                        :value="preset"
                      >
                        {{ preset }}
                      </option>
                      <option value="__custom__">
                        {{ t('settings.appearance.typography.custom') }}
                      </option>
                    </select>
                    <input
                      v-if="uiCustomActive"
                      :value="uiFont"
                      type="text"
                      class="app-settings-font-input"
                      :placeholder="t('settings.appearance.typography.customPlaceholder')"
                      :aria-label="t('settings.appearance.typography.uiLabel')"
                      data-testid="settings-font-ui-input"
                      @input="setUiFont(($event.target as HTMLInputElement).value)"
                    >
                    <p
                      class="app-settings-font-preview"
                      :style="{ fontFamily: uiPreviewFamily }"
                      aria-hidden="true"
                    >
                      {{ t('settings.appearance.typography.previewSampleUi') }}
                    </p>
                  </div>
                </div>

                <div class="app-settings-row app-settings-row--font">
                  <div class="app-settings-row__copy">
                    <h3 class="app-settings-row__title">
                      {{ t('settings.appearance.typography.codeLabel') }}
                    </h3>
                    <p class="app-settings-row__description">
                      {{ t('settings.appearance.typography.codeDescription') }}
                    </p>
                  </div>
                  <div class="app-settings-font-control">
                    <select
                      class="app-settings-font-select"
                      :value="codeSelectValue"
                      :aria-label="t('settings.appearance.typography.codeLabel')"
                      data-testid="settings-font-code"
                      @change="onCodeSelect(($event.target as HTMLSelectElement).value)"
                    >
                      <option value="__default__">
                        {{ t('settings.appearance.typography.systemDefault') }}
                      </option>
                      <option
                        v-for="preset in CODE_FONT_PRESETS"
                        :key="preset"
                        :value="preset"
                      >
                        {{ preset }}
                      </option>
                      <option value="__custom__">
                        {{ t('settings.appearance.typography.custom') }}
                      </option>
                    </select>
                    <input
                      v-if="codeCustomActive"
                      :value="codeFont"
                      type="text"
                      class="app-settings-font-input"
                      :placeholder="t('settings.appearance.typography.customPlaceholder')"
                      :aria-label="t('settings.appearance.typography.codeLabel')"
                      data-testid="settings-font-code-input"
                      @input="setCodeFont(($event.target as HTMLInputElement).value)"
                    >
                    <p
                      class="app-settings-font-preview app-settings-font-preview--mono"
                      :style="{ fontFamily: codePreviewFamily }"
                      aria-hidden="true"
                    >
                      {{ t('settings.appearance.typography.previewSampleCode') }}
                    </p>
                  </div>
                </div>

                <div class="app-settings-callout">
                  <SIcon
                    name="Info"
                    size="w-4 h-4"
                    class="mt-0.5 text-accent-primary"
                  />
                  <p>
                    {{ t('settings.appearance.typography.resetHint') }}
                  </p>
                </div>
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
                    <span class="app-settings-option__copy">
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
import { isCatppuccinFlavor, type AccentMode, type FlavorMode, type ThemeMode } from '@/utils/themeBootstrap'
import { CODE_FONT_PRESETS, UI_FONT_PRESETS } from '@/utils/fontPreferences'

type SectionKey = 'appearance' | 'language' | 'shell' | 'diagnostics'

// 预览卡的字形样例（装饰性，非文案）。
const PREVIEW_GLYPH_SAMPLE = 'Aa'

const { t } = useI18n()
const shellPreferencesStore = useShellPreferencesStore()
const {
  accent,
  closeToTray,
  codeFont,
  confirmBeforeExit,
  effectiveTheme,
  flavor,
  locale,
  openPanelOnTrayClick,
  perfTelemetryEnabled,
  resolvedFlavor,
  sidebarWidth,
  theme,
  uiFont,
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
  },
  {
    value: 'dark' as ThemeMode,
    icon: 'Moon',
    label: t('theme.dark'),
    description: t('settings.appearance.darkDescription'),
  },
  {
    value: 'system' as ThemeMode,
    icon: 'Monitor',
    label: t('theme.system'),
    description: t('settings.appearance.systemDescription'),
  },
])

const systemResolvedHint = computed(() => translateWithFallback(
  t,
  'settings.appearance.theme.resolvedHint',
  `Resolved now: {resolved}`,
  { resolved: t(`theme.${effectiveTheme.value}`) },
))

interface FlavorOption {
  value: FlavorMode
  label: string
  description: string
}

interface AccentOption {
  value: AccentMode
  label: string
  description: string
}

const flavorOptions = computed<FlavorOption[]>(() => [
  {
    value: 'neutral',
    label: t('settings.appearance.flavor.neutral'),
    description: t('settings.appearance.flavor.neutralDescription'),
  },
  {
    value: 'clay',
    label: t('settings.appearance.flavor.clay'),
    description: t('settings.appearance.flavor.clayDescription'),
  },
  {
    value: 'catppuccin',
    label: t('settings.appearance.flavor.catppuccin'),
    description: t('settings.appearance.flavor.catppuccinDescription'),
  },
])

const resolvedFlavorLabelMap = computed<Record<string, string>>(() => ({
  neutral: t('settings.appearance.flavor.neutral'),
  clay: t('settings.appearance.flavor.clay'),
  latte: t('settings.appearance.flavor.resolvedLatte'),
  mocha: t('settings.appearance.flavor.resolvedMocha'),
}))

const resolvedFlavorLabel = computed(() => resolvedFlavorLabelMap.value[resolvedFlavor.value])

const flavorStatusLabel = (option: FlavorOption): string => {
  if (isCatppuccinFlavor(option.value) && resolvedFlavor.value !== option.value) {
    return `${t('settings.active')} · ${resolvedFlavorLabel.value}`
  }

  return t('settings.active')
}

// 预览令牌子集静态复制自 tokens.css（与 tokens.css 同步）：
// neutral/clay 取 :root 与 [data-theme='dark'] 两套；catppuccin 的 light/dark
// 分别取 [data-resolved-flavor='latte'] 与 html:root[data-resolved-flavor='mocha'] 语义块。
// accent 条引用实时的 var(--color-accent-primary)，不随预览覆写。
interface SurfacePreviewTokens {
  base: string
  elevated: string
  surface: string
  text: string
  muted: string
}

const FLAVOR_PREVIEW_TOKENS: Record<FlavorMode, { light: SurfacePreviewTokens; dark: SurfacePreviewTokens }> = {
  neutral: {
    light: { base: '#e8e9ec', elevated: '#f2f3f5', surface: '#fbfcfd', text: '#191b20', muted: '#5f646e' },
    dark: { base: '#131316', elevated: '#1a1b1f', surface: '#22242a', text: '#f2f3f5', muted: '#9ba1ab' },
  },
  clay: {
    light: { base: '#ebe1d0', elevated: '#f5eee1', surface: '#fefaf2', text: '#31241c', muted: '#715d4c' },
    dark: { base: '#17120f', elevated: '#221b18', surface: '#2a221e', text: '#f3eadf', muted: '#b9a695' },
  },
  catppuccin: {
    light: { base: '#e6e9ef', elevated: '#eff1f5', surface: '#fafbfe', text: '#2e3043', muted: '#6c6f85' },
    dark: { base: '#11111b', elevated: '#1e1e2e', surface: '#313244', text: '#fafbff', muted: '#a6adc8' },
  },
}

// accent 预览令牌静态复制自 tokens.css accent 块（与 tokens.css 同步）：
// light/dark 取 [data-accent] 两套；latte/mocha 取 Catppuccin 作用域 accent 映射。
interface AccentPreviewTokens {
  bg: string
  contrast: string
}

const ACCENT_PREVIEW_TOKENS: Record<AccentMode, Record<'light' | 'dark' | 'latte' | 'mocha', AccentPreviewTokens>> = {
  clay: {
    light: { bg: '#cf6239', contrast: '#fff8f2' },
    dark: { bg: '#e8835b', contrast: '#1d1207' },
    latte: { bg: '#fe640b', contrast: '#1e1e2e' },
    mocha: { bg: '#fab387', contrast: '#11111b' },
  },
  sage: {
    light: { bg: '#5b8a62', contrast: '#fff8f2' },
    dark: { bg: '#6fbf73', contrast: '#0e1a0c' },
    latte: { bg: '#40a02b', contrast: '#1e1e2e' },
    mocha: { bg: '#a6e3a1', contrast: '#11111b' },
  },
  sky: {
    light: { bg: '#5a7ba6', contrast: '#fff8f2' },
    dark: { bg: '#6ea8e8', contrast: '#0b1521' },
    latte: { bg: '#1e66f5', contrast: '#eff1f5' },
    mocha: { bg: '#89b4fa', contrast: '#11111b' },
  },
  mauve: {
    light: { bg: '#8a6d94', contrast: '#fff8f2' },
    dark: { bg: '#b78fe0', contrast: '#190f22' },
    latte: { bg: '#8839ef', contrast: '#eff1f5' },
    mocha: { bg: '#cba6f7', contrast: '#11111b' },
  },
}

const flavorPreviewStyle = (flavorValue: FlavorMode): Record<string, string> => {
  const tokens = FLAVOR_PREVIEW_TOKENS[flavorValue][effectiveTheme.value]
  return {
    '--fp-bg-base': tokens.base,
    '--fp-bg-elevated': tokens.elevated,
    '--fp-bg-surface': tokens.surface,
    '--fp-text-primary': tokens.text,
    '--fp-text-muted': tokens.muted,
  }
}

const accentPreviewStyle = (accentValue: AccentMode): Record<string, string> => {
  const contextKey = resolvedFlavor.value === 'latte' || resolvedFlavor.value === 'mocha'
    ? resolvedFlavor.value
    : effectiveTheme.value
  const tokens = ACCENT_PREVIEW_TOKENS[accentValue][contextKey]
  return {
    '--fp-accent-bg': tokens.bg,
    '--fp-accent-contrast': tokens.contrast,
  }
}

const accentOptions = computed<AccentOption[]>(() => [
  { value: 'clay', label: t('settings.appearance.accent.clay'), description: t('settings.appearance.accent.clayDescription') },
  { value: 'sage', label: t('settings.appearance.accent.sage'), description: t('settings.appearance.accent.sageDescription') },
  { value: 'sky', label: t('settings.appearance.accent.sky'), description: t('settings.appearance.accent.skyDescription') },
  { value: 'mauve', label: t('settings.appearance.accent.mauve'), description: t('settings.appearance.accent.mauveDescription') },
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

const setFlavor = (nextFlavor: FlavorMode) => {
  shellPreferencesStore.setFlavor(nextFlavor)
}

const setAccent = (nextAccent: AccentMode) => {
  shellPreferencesStore.setAccent(nextAccent)
}

// 预设清单集中在 @/utils/fontPreferences；自定义输入不受清单限制（空串=系统默认，回内置栈）。
// 存值非空且不在预设内时，强制展开自定义输入回填。
const uiCustomActive = ref(uiFont.value !== '' && !UI_FONT_PRESETS.includes(uiFont.value))
const codeCustomActive = ref(codeFont.value !== '' && !CODE_FONT_PRESETS.includes(codeFont.value))

const uiSelectValue = computed(() => {
  if (uiCustomActive.value) return '__custom__'
  if (uiFont.value === '') return '__default__'
  return UI_FONT_PRESETS.includes(uiFont.value) ? uiFont.value : '__custom__'
})

const codeSelectValue = computed(() => {
  if (codeCustomActive.value) return '__custom__'
  if (codeFont.value === '') return '__default__'
  return CODE_FONT_PRESETS.includes(codeFont.value) ? codeFont.value : '__custom__'
})

const uiPreviewFamily = computed(() =>
  uiFont.value ? `"${uiFont.value}", var(--font-sans-base)` : 'var(--font-sans-base)',
)

const codePreviewFamily = computed(() =>
  codeFont.value ? `"${codeFont.value}", var(--font-mono-base)` : 'var(--font-mono-base)',
)

const onUiSelect = (value: string) => {
  if (value === '__custom__') {
    uiCustomActive.value = true
    return
  }
  uiCustomActive.value = false
  shellPreferencesStore.setUiFont(value === '__default__' ? '' : value)
}

const onCodeSelect = (value: string) => {
  if (value === '__custom__') {
    codeCustomActive.value = true
    return
  }
  codeCustomActive.value = false
  shellPreferencesStore.setCodeFont(value === '__default__' ? '' : value)
}

const setUiFont = (value: string) => {
  shellPreferencesStore.setUiFont(value)
}

const setCodeFont = (value: string) => {
  shellPreferencesStore.setCodeFont(value)
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
  @apply mx-auto flex max-w-[1440px] flex-col gap-5;
}

.app-settings-hero {
  @apply flex flex-col gap-4 rounded-lg border px-5 py-5 lg:flex-row lg:items-center lg:justify-between;

  background: var(--surface-card-bg);
  border-color: var(--surface-card-border);
  box-shadow: var(--shadow-sm);
}

.app-settings-hero__intro {
  @apply flex items-start gap-4;
}

.app-settings-hero__icon {
  @apply flex h-12 w-12 items-center justify-center rounded-lg border;

  color: var(--color-accent-primary);
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
}

.app-settings-hero__eyebrow,
.app-settings-card__eyebrow {
  @apply text-[11px] font-semibold tracking-[0.08em];

  color: var(--color-text-muted);
}

.app-settings-hero__title {
  @apply text-[1.7rem] font-semibold tracking-[-0.03em] sm:text-[2rem];

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
  @apply inline-flex items-center rounded-md border px-2.5 py-1.5 text-[11px] font-semibold;

  border-color: var(--color-border-default);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
}

.app-settings-summary__pill--mono {
  @apply font-mono tracking-normal;
}

.app-settings-layout {
  @apply grid min-w-0 grid-cols-1 gap-5 xl:grid-cols-[260px_minmax(0,1fr)];
}

.app-settings-nav {
  @apply xl:sticky xl:top-6 xl:self-start;
}

.app-settings-nav__inner {
  @apply flex min-w-0 gap-2 overflow-x-auto pb-1 xl:flex-col xl:overflow-visible;
}

.app-settings-nav__button {
  @apply flex min-h-[64px] min-w-[220px] items-start gap-3 rounded-lg border px-3.5 py-3 text-left transition-[border-color,background-color,box-shadow,transform] duration-200 xl:min-w-0;

  border-color: var(--color-border-default);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
}

.app-settings-nav__button:hover {
  transform: translateY(-1px);
  border-color: var(--color-border-strong);
  background: var(--color-bg-surface);
}

.app-settings-nav__button--active {
  border-color: var(--color-accent-primary);
  background: var(--color-bg-surface);
  box-shadow: inset 2px 0 0 var(--color-accent-primary);
  color: var(--color-text-primary);
}

.app-settings-nav__icon {
  @apply mt-0.5 flex h-8 w-8 flex-none items-center justify-center rounded-md border;

  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
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
  overflow-wrap: anywhere;
}

.app-settings-content {
  @apply flex min-w-0 flex-col gap-5;
}

.app-settings-content > section {
  @apply flex min-w-0 flex-col gap-4;
}

.app-settings-card {
  @apply min-w-0 p-4 sm:p-5;
}

.app-settings-card__header {
  @apply flex max-w-[72ch] flex-col gap-2 border-b border-border-default/40 pb-4;
}

.app-settings-option-grid {
  @apply mt-4 grid min-w-0 grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3;
}

.app-settings-option-grid--compact {
  @apply xl:grid-cols-2;
}

.app-settings-option {
  @apply flex min-h-[112px] min-w-0 flex-col justify-between rounded-lg border p-3.5 text-left transition-[border-color,background-color,box-shadow,transform] duration-200;

  border-color: var(--color-border-default);
  background: var(--color-bg-elevated);
}

.app-settings-option:hover {
  transform: translateY(-1px);
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-sm);
}

.app-settings-option--active {
  border-color: var(--color-accent-primary);
  background: var(--color-bg-surface);
  box-shadow: inset 0 0 0 1px var(--color-accent-primary);
}

.app-settings-option__meta {
  @apply flex min-w-0 items-start gap-3;
}

.app-settings-option__copy {
  @apply min-w-0;
}

.app-settings-option__icon {
  @apply flex h-9 w-9 flex-none items-center justify-center rounded-md border;

  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
  color: var(--color-text-primary);
}

.app-settings-option__icon--plain {
  @apply text-sm font-semibold tracking-normal;
}

.app-settings-option__status {
  @apply inline-flex w-fit items-center rounded-md border px-2 py-1 text-[10px] font-semibold;

  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
  color: var(--color-text-muted);
}

.app-settings-option__status--active,
.app-settings-option--active .app-settings-option__status {
  border-color: var(--color-border-accent);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
}

.app-settings-card--tight {
  @apply p-4 sm:p-5;
}

/* --- 主题分段控件：radiogroup + 实心选中段 --- */
.app-settings-segmented {
  @apply mt-4 grid min-w-0 grid-cols-1 gap-2 rounded-lg border p-1.5 sm:grid-cols-3;

  border-color: var(--color-border-default);
  background: var(--color-bg-base);
}

.app-settings-segmented__option {
  @apply flex min-w-0 items-start gap-3 rounded-md border border-transparent px-3 py-2.5 text-left transition-[border-color,background-color,box-shadow] duration-200;

  color: var(--color-text-secondary);
}

.app-settings-segmented__option:hover {
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
}

.app-settings-segmented__option--active {
  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
  box-shadow: var(--shadow-sm);
  color: var(--color-text-primary);
}

.app-settings-segmented__icon {
  @apply mt-0.5 flex h-7 w-7 flex-none items-center justify-center rounded-md border;

  border-color: var(--color-border-default);
  background: var(--color-bg-elevated);
}

.app-settings-segmented__option--active .app-settings-segmented__icon {
  border-color: var(--color-border-accent);
  color: var(--color-accent-primary);
}

.app-settings-segmented__copy {
  @apply min-w-0;
}

.app-settings-segmented__title {
  @apply block text-sm font-semibold;

  color: var(--color-text-primary);
}

.app-settings-segmented__caption {
  @apply mt-0.5 block text-xs leading-5;

  color: var(--color-text-muted);
}

.app-settings-segmented__resolved {
  @apply mt-3 text-xs font-semibold;

  color: var(--color-text-muted);
}

/* --- flavor 真实 token 预览卡：预览元素上的 --fp-* 令牌子集为 tokens.css 的静态副本 --- */
.app-settings-flavor-grid {
  @apply mt-4 grid min-w-0 grid-cols-1 gap-3 md:grid-cols-3;
}

.app-settings-flavor-card {
  @apply relative flex min-w-0 flex-col gap-3 rounded-lg border p-3.5 text-left transition-[border-color,background-color,box-shadow,transform] duration-200;

  border-color: var(--color-border-default);
  background: var(--color-bg-elevated);
}

.app-settings-flavor-card:hover {
  transform: translateY(-1px);
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-sm);
}

.app-settings-flavor-card--active {
  border-color: var(--color-accent-primary);
  background: var(--color-bg-surface);
  box-shadow: inset 0 0 0 1px var(--color-accent-primary);
}

.app-settings-flavor-card__preview {
  @apply flex h-20 items-center justify-center rounded-md border;

  border-color: var(--color-border-subtle);
  background: var(--fp-bg-base);
}

.fp-surface {
  @apply flex items-center gap-2 rounded border px-3 py-2;

  border-color: var(--color-border-subtle);
  background: var(--fp-bg-surface);
  box-shadow: var(--shadow-sm);
}

.fp-text {
  @apply text-sm font-semibold;

  color: var(--fp-text-primary);
}

.fp-muted {
  @apply text-xs;

  color: var(--fp-text-muted);
}

.fp-accent {
  @apply h-3.5 w-6 rounded-sm;

  background: var(--color-accent-primary);
}

.app-settings-flavor-card__copy {
  @apply min-w-0;
}

.app-settings-flavor-card__footer {
  @apply flex items-center gap-2;
}

.app-settings-flavor-card__dot,
.app-settings-accent-option__dot {
  @apply h-2 w-2 flex-none rounded-full;

  background: var(--color-accent-primary);
}

.app-settings-accent-option__dot {
  @apply absolute right-3 top-3;
}

/* --- accent 实心按钮预览 --- */
.app-settings-accent-grid {
  @apply mt-4 grid min-w-0 grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4;
}

.app-settings-accent-option {
  @apply relative flex min-w-0 flex-col gap-3 rounded-lg border p-3.5 text-left transition-[border-color,background-color,box-shadow,transform] duration-200;

  border-color: var(--color-border-default);
  background: var(--color-bg-elevated);
}

.app-settings-accent-option:hover {
  transform: translateY(-1px);
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-sm);
}

.app-settings-accent-option--active {
  border-color: var(--color-accent-primary);
  background: var(--color-bg-surface);
  box-shadow: inset 0 0 0 1px var(--color-accent-primary);
}

.app-settings-accent-option__preview {
  @apply flex h-14 items-center justify-center rounded-md border;

  border-color: var(--color-border-subtle);
  background: var(--color-bg-surface);
}

.fp-accent-button {
  @apply rounded-md px-3.5 py-1.5 text-xs font-semibold;

  background: var(--fp-accent-bg);
  color: var(--fp-accent-contrast);
}

.app-settings-accent-option__copy {
  @apply min-w-0;
}

.app-settings-stack {
  @apply mt-5 flex min-w-0 flex-col gap-4;
}

.app-settings-row {
  @apply flex min-w-0 flex-col gap-4 rounded-lg border px-4 py-4 lg:flex-row lg:items-center lg:justify-between;

  border-color: var(--color-border-default);
  background: var(--color-bg-elevated);
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
  @apply relative inline-flex items-center gap-3 rounded-lg border px-3 py-2 text-xs font-semibold transition-[border-color,background-color,box-shadow] duration-200;

  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
  color: var(--color-text-muted);
}

.app-settings-switch--active {
  border-color: var(--color-border-accent);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  color: var(--color-accent-primary);
}

.app-settings-switch__track {
  @apply relative h-6 w-10 rounded-full;

  background: var(--color-bg-overlay);
}

.app-settings-switch--active .app-settings-switch__track {
  background: rgb(var(--color-accent-primary-rgb) / 30%);
}

.app-settings-switch__thumb {
  @apply absolute left-3 top-1/2 h-5 w-5 rounded-full border transition-transform duration-200;

  transform: translateY(-50%);
  border-color: var(--color-border-strong);
  background: var(--color-bg-surface);
}

.app-settings-switch--active .app-settings-switch__thumb {
  transform: translate(16px, -50%);
  border-color: var(--color-border-accent);
}

.app-settings-switch__label {
  @apply min-w-[4.75rem] text-left;
}

.app-settings-slider {
  @apply w-full max-w-[360px] rounded-lg border px-4 py-3;

  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
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
  @apply flex items-start gap-2 rounded-lg border px-4 py-3;

  border-color: var(--color-border-accent);
  background: rgb(var(--color-accent-primary-rgb) / 8%);
}

.app-settings-row--font {
  @apply items-stretch;
}

.app-settings-font-control {
  @apply flex w-full flex-col gap-2 lg:max-w-[360px];
}

.app-settings-font-select,
.app-settings-font-input {
  @apply w-full rounded-lg border px-3 py-2 text-sm;

  border-color: var(--color-border-default);
  background: var(--color-bg-surface);
  color: var(--color-text-primary);
}

.app-settings-font-select:focus-visible,
.app-settings-font-input:focus-visible {
  outline: none;
  border-color: var(--color-accent-primary);
  box-shadow: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 18%);
}

.app-settings-font-preview {
  @apply truncate rounded-lg border px-3 py-2 text-sm;

  border-color: var(--color-border-subtle);
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
}

.app-settings-font-preview--mono {
  @apply tracking-normal;
}

@media (width <= 1279px) {
  /* z-20：内容区 Card 内层为 relative z-10，sticky 导航必须压过它，否则滚动时卡片盖住导航。 */
  .app-settings-nav {
    @apply sticky top-0 z-20;
  }

  /* sticky 导航必须不透明：接入 surface 契约的 inline 档（现为不透明配方），
     修复滚动时下方文字从 86% alpha 底透出的问题。 */
  .app-settings-nav__inner {
    @apply rounded-lg border p-2;

    border-color: var(--surface-status-border);
    background: var(--surface-status-bg);
    backdrop-filter: var(--surface-status-blur);
    box-shadow: var(--surface-status-shadow);
  }

  .app-settings-nav__button {
    @apply min-h-[64px] min-w-[180px];
  }
}

@media (width <= 479px) {
  .app-settings-nav__inner {
    @apply grid grid-cols-2 overflow-visible;
  }

  .app-settings-nav__button {
    @apply min-w-0;
  }
}
</style>
