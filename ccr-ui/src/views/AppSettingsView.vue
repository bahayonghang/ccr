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
                    <span class="app-settings-option__copy">
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

              <div class="app-settings-option-grid">
                <button
                  v-for="option in flavorOptions"
                  :key="option.value"
                  type="button"
                  class="app-settings-option"
                  :class="{ 'app-settings-option--active': flavor === option.value }"
                  :data-testid="`settings-flavor-${option.value}`"
                  :aria-pressed="flavor === option.value"
                  @click="setFlavor(option.value)"
                >
                  <div class="app-settings-option__meta">
                    <span
                      class="app-settings-option__swatch"
                      :style="{ background: option.preview }"
                    />
                    <span class="app-settings-option__copy">
                      <span class="app-settings-option__title">{{ option.label }}</span>
                      <span class="app-settings-option__caption">{{ option.description }}</span>
                    </span>
                  </div>
                  <span class="app-settings-option__status">
                    {{ flavorStatusLabel(option) }}
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
                  class="app-settings-accent-swatch"
                  :class="{ 'app-settings-accent-swatch--active': accent === option.value }"
                  :aria-checked="accent === option.value"
                  :aria-label="option.label"
                  :title="option.label"
                  :data-testid="`settings-accent-${option.value}`"
                  :style="{ '--accent-swatch-color': option.preview }"
                  @click="setAccent(option.value)"
                />
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

// 选项 UI 仍展示旧值域（子任务 C 统一重写）；store 写入路径会把旧值迁移进新值域。
type LegacyFlavorOptionValue = 'paper' | 'graphite' | 'latte' | 'frappe' | 'macchiato' | 'mocha'
type LegacyAccentOptionValue = 'sand' | 'amber' | 'rose' | 'slate'

interface FlavorOption {
  value: FlavorMode | LegacyFlavorOptionValue
  label: string
  description: string
  badge: string
  preview: string
}

interface AccentOption {
  value: AccentMode | LegacyAccentOptionValue
  label: string
  preview: string
}

const flavorOptions = computed<FlavorOption[]>(() => [
  {
    value: 'clay',
    label: t('settings.appearance.flavor.clay'),
    description: t('settings.appearance.flavor.clayDescription'),
    badge: t('settings.appearance.dayBadge'),
    preview: 'linear-gradient(135deg, #f4ede3 0%, #fbf6ee 60%, #fffaf3 100%)',
  },
  {
    value: 'paper',
    label: t('settings.appearance.flavor.paper'),
    description: t('settings.appearance.flavor.paperDescription'),
    badge: t('settings.appearance.dayBadge'),
    preview: 'linear-gradient(135deg, #fafafa 0%, #ffffff 60%, #f4f4f5 100%)',
  },
  {
    value: 'graphite',
    label: t('settings.appearance.flavor.graphite'),
    description: t('settings.appearance.flavor.graphiteDescription'),
    badge: t('settings.appearance.nightBadge'),
    preview: 'linear-gradient(135deg, #ececee 0%, #f6f6f8 60%, #2e3036 100%)',
  },
  {
    value: 'latte',
    label: t('settings.appearance.flavor.latte'),
    description: t('settings.appearance.flavor.latteDescription'),
    badge: t('settings.appearance.autoBadge'),
    preview: 'linear-gradient(135deg, #eff1f5 0%, #e6e9ef 58%, #7287fd 100%)',
  },
  {
    value: 'frappe',
    label: t('settings.appearance.flavor.frappe'),
    description: t('settings.appearance.flavor.frappeDescription'),
    badge: t('settings.appearance.autoBadge'),
    preview: 'linear-gradient(135deg, #303446 0%, #414559 58%, #babbf1 100%)',
  },
  {
    value: 'macchiato',
    label: t('settings.appearance.flavor.macchiato'),
    description: t('settings.appearance.flavor.macchiatoDescription'),
    badge: t('settings.appearance.autoBadge'),
    preview: 'linear-gradient(135deg, #24273a 0%, #363a4f 58%, #b7bdf8 100%)',
  },
  {
    value: 'mocha',
    label: t('settings.appearance.flavor.mocha'),
    description: t('settings.appearance.flavor.mochaDescription'),
    badge: t('settings.appearance.autoBadge'),
    preview: 'linear-gradient(135deg, #1e1e2e 0%, #313244 58%, #b4befe 100%)',
  },
])

const flavorLabelMap = computed<Record<string, string>>(() => ({
  clay: t('settings.appearance.flavor.clay'),
  paper: t('settings.appearance.flavor.paper'),
  graphite: t('settings.appearance.flavor.graphite'),
  latte: t('settings.appearance.flavor.latte'),
  frappe: t('settings.appearance.flavor.frappe'),
  macchiato: t('settings.appearance.flavor.macchiato'),
  mocha: t('settings.appearance.flavor.mocha'),
}))

const resolvedFlavorLabel = computed(() => flavorLabelMap.value[resolvedFlavor.value])

const flavorStatusLabel = (option: FlavorOption): string => {
  if (flavor.value !== option.value) {
    return option.badge
  }

  if (isCatppuccinFlavor(option.value) && resolvedFlavor.value !== option.value) {
    return `${t('settings.active')} · ${resolvedFlavorLabel.value}`
  }

  return t('settings.active')
}

const accentOptions = computed<AccentOption[]>(() => [
  { value: 'clay', label: t('settings.appearance.accent.clay'), preview: '#d97757' },
  { value: 'sand', label: t('settings.appearance.accent.sand'), preview: '#b99666' },
  { value: 'sage', label: t('settings.appearance.accent.sage'), preview: '#5b8a62' },
  { value: 'sky', label: t('settings.appearance.accent.sky'), preview: '#7d97b6' },
  { value: 'mauve', label: t('settings.appearance.accent.mauve'), preview: '#9c7f94' },
  { value: 'amber', label: t('settings.appearance.accent.amber'), preview: '#bc8540' },
  { value: 'rose', label: t('settings.appearance.accent.rose'), preview: '#c76953' },
  { value: 'slate', label: t('settings.appearance.accent.slate'), preview: '#857367' },
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

// 旧值域选项值经 store 写入路径迁移到新值域（migrateFlavorValue/migrateAccentValue）。
const setFlavor = (nextFlavor: FlavorOption['value']) => {
  shellPreferencesStore.setFlavor(nextFlavor as FlavorMode)
}

const setAccent = (nextAccent: AccentOption['value']) => {
  shellPreferencesStore.setAccent(nextAccent as AccentMode)
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

  border-color: rgb(var(--color-border-default-rgb) / 56%);
  background: rgb(var(--color-bg-base-rgb) / 72%);
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

  border-color: rgb(var(--color-border-default-rgb) / 44%);
  background: rgb(var(--color-bg-elevated-rgb) / 72%);
  color: var(--color-text-secondary);
}

.app-settings-nav__button:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  background: rgb(var(--color-bg-surface-rgb) / 82%);
}

.app-settings-nav__button--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 30%);
  background: rgb(var(--color-accent-primary-rgb) / 10%);
  box-shadow: inset 2px 0 0 var(--color-accent-primary);
  color: var(--color-text-primary);
}

.app-settings-nav__icon {
  @apply mt-0.5 flex h-8 w-8 flex-none items-center justify-center rounded-md border;

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

  border-color: rgb(var(--color-border-default-rgb) / 44%);
  background: rgb(var(--color-bg-elevated-rgb) / 82%);
}

.app-settings-option:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  box-shadow: var(--shadow-sm);
}

.app-settings-option--active {
  border-color: rgb(var(--color-accent-primary-rgb) / 34%);
  background: rgb(var(--color-accent-primary-rgb) / 8%);
  box-shadow: inset 0 0 0 1px rgb(var(--color-accent-primary-rgb) / 12%);
}

.app-settings-option__meta {
  @apply flex min-w-0 items-start gap-3;
}

.app-settings-option__copy {
  @apply min-w-0;
}

.app-settings-option__icon {
  @apply flex h-9 w-9 flex-none items-center justify-center rounded-md border;

  border-color: rgb(var(--color-border-default-rgb) / 46%);
  background: rgb(var(--color-bg-base-rgb) / 82%);
  color: var(--color-text-primary);
}

.app-settings-option__icon--plain {
  @apply text-sm font-semibold tracking-normal;
}

.app-settings-option__status {
  @apply inline-flex w-fit items-center rounded-md border px-2 py-1 text-[10px] font-semibold;

  border-color: rgb(var(--color-border-default-rgb) / 52%);
  background: rgb(var(--color-bg-base-rgb) / 68%);
  color: var(--color-text-muted);
}

.app-settings-option--active .app-settings-option__status {
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  color: var(--color-accent-primary);
}

.app-settings-option__swatch {
  @apply flex h-9 w-9 flex-none items-center justify-center rounded-md border;

  border-color: rgb(var(--color-border-default-rgb) / 46%);
  box-shadow: inset 0 0 0 1px rgb(0 0 0 / 8%);
}

.app-settings-card--tight {
  @apply p-4 sm:p-5;
}

.app-settings-accent-grid {
  @apply mt-4 flex flex-wrap gap-2.5;
}

.app-settings-accent-swatch {
  @apply relative h-9 w-9 cursor-pointer rounded-md border transition-[transform,border-color,box-shadow] duration-200;

  background: var(--accent-swatch-color);
  border-color: rgb(var(--color-border-default-rgb) / 46%);
  box-shadow:
    var(--inner-glow),
    0 1px 2px rgb(0 0 0 / 6%);
}

.app-settings-accent-swatch:hover {
  transform: translateY(-1px);
  border-color: rgb(var(--color-border-default-rgb) / 72%);
}

.app-settings-accent-swatch:focus-visible {
  outline: none;
  box-shadow:
    var(--inner-glow),
    0 0 0 3px rgb(var(--color-accent-primary-rgb) / 30%);
}

.app-settings-accent-swatch--active {
  transform: translateY(-1px);
  border-color: var(--color-accent-primary);
  box-shadow:
    inset 0 0 0 1px rgb(0 0 0 / 12%),
    0 0 0 2px rgb(var(--color-accent-primary-rgb) / 34%);
}

.app-settings-stack {
  @apply mt-5 flex min-w-0 flex-col gap-4;
}

.app-settings-row {
  @apply flex min-w-0 flex-col gap-4 rounded-lg border px-4 py-4 lg:flex-row lg:items-center lg:justify-between;

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
  @apply relative inline-flex items-center gap-3 rounded-lg border px-3 py-2 text-xs font-semibold transition-[border-color,background-color,box-shadow] duration-200;

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
  @apply absolute left-3 top-1/2 h-5 w-5 rounded-full border transition-transform duration-200;

  transform: translateY(-50%);
  border-color: rgb(var(--color-border-default-rgb) / 62%);
  background: rgb(var(--color-bg-elevated-rgb) / 100%);
}

.app-settings-switch--active .app-settings-switch__thumb {
  transform: translate(16px, -50%);
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
}

.app-settings-switch__label {
  @apply min-w-[4.75rem] text-left;
}

.app-settings-slider {
  @apply w-full max-w-[360px] rounded-lg border px-4 py-3;

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
  @apply flex items-start gap-2 rounded-lg border px-4 py-3;

  border-color: rgb(var(--color-accent-primary-rgb) / 18%);
  background: rgb(var(--color-accent-primary-rgb) / 6%);
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

  border-color: rgb(var(--color-border-default-rgb) / 56%);
  background: rgb(var(--color-bg-base-rgb) / 78%);
  color: var(--color-text-primary);
}

.app-settings-font-select:focus-visible,
.app-settings-font-input:focus-visible {
  outline: none;
  border-color: rgb(var(--color-accent-primary-rgb) / 40%);
  box-shadow: 0 0 0 3px rgb(var(--color-accent-primary-rgb) / 18%);
}

.app-settings-font-preview {
  @apply truncate rounded-lg border px-3 py-2 text-sm;

  border-color: rgb(var(--color-border-default-rgb) / 40%);
  background: rgb(var(--color-bg-elevated-rgb) / 60%);
  color: var(--color-text-secondary);
}

.app-settings-font-preview--mono {
  @apply tracking-normal;
}

@media (width <= 1279px) {
  .app-settings-nav {
    @apply sticky top-0 z-10;
  }

  .app-settings-nav__inner {
    @apply rounded-lg border p-2;

    border-color: rgb(var(--color-border-default-rgb) / 44%);
    background: rgb(var(--color-bg-base-rgb) / 86%);
    backdrop-filter: var(--surface-status-blur);
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
