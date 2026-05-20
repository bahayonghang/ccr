<template>
  <div class="gemini-view stage-page">
    <div class="gemini-shell">
      <section class="gemini-hero animate-slide-up">
        <div class="gemini-hero__copy">
          <div class="gemini-hero__eyebrow">
            <span class="gemini-hero__pulse" />
            <span>{{ t('gemini.overview.hero.eyebrow') }}</span>
            <span class="gemini-hero__eyebrow-muted">{{ t('common.shell.tagline') }}</span>
          </div>

          <div class="gemini-title-row">
            <div class="gemini-brand-mark">
              <SIcon
                name="Sparkles"
                size="w-7 h-7"
              />
            </div>
            <div>
              <h1 class="gemini-title">
                Antigravity CLI
              </h1>
              <p class="gemini-subtitle">
                {{ t('gemini.overview.hero.subtitle') }}
              </p>
            </div>
          </div>

          <p class="gemini-description">
            {{ t('gemini.overview.hero.description') }}
          </p>

          <div class="gemini-tag-row">
            <span
              v-for="tag in heroTags"
              :key="tag.key"
              class="gemini-tag"
              :class="`gemini-tag--${tag.tone}`"
            >
              <SIcon
                :name="tag.icon"
                size="w-3.5 h-3.5"
              />
              {{ tag.label }}
            </span>
          </div>

          <div class="gemini-hero-actions">
            <RouterLink to="/antigravity/mcp">
              <Button
                variant="primary"
                size="md"
                class="gemini-action-button gemini-action-button--primary"
              >
                <SIcon
                  name="Server"
                  size="w-4 h-4"
                  class="mr-2"
                />
                {{ t('gemini.overview.hero.primaryAction') }}
              </Button>
            </RouterLink>
            <RouterLink to="/antigravity/slash-commands">
              <Button
                variant="glass"
                size="md"
                class="gemini-action-button"
              >
                <SIcon
                  name="Command"
                  size="w-4 h-4"
                  class="mr-2"
                />
                {{ t('gemini.overview.hero.secondaryAction') }}
              </Button>
            </RouterLink>
            <RouterLink to="/">
              <Button
                variant="ghost"
                size="md"
                class="gemini-action-button"
              >
                <SIcon
                  name="Home"
                  size="w-4 h-4"
                  class="mr-2"
                />
                {{ t('common.backToHome') }}
              </Button>
            </RouterLink>
          </div>
        </div>

        <Card
          variant="glass"
          padding="none"
          body-class="h-full"
          class="gemini-terminal-card"
        >
          <div class="gemini-terminal-card__header">
            <div>
              <p class="gemini-terminal-card__eyebrow">
                {{ t('gemini.overview.terminal.eyebrow') }}
              </p>
              <h2 class="gemini-terminal-card__title">
                {{ t('gemini.overview.terminal.title') }}
              </h2>
            </div>
            <div class="gemini-terminal-card__lights">
              <span />
              <span />
              <span />
            </div>
          </div>

          <div class="gemini-terminal-lines">
            <button
              v-for="snippet in terminalSnippets"
              :key="snippet.command"
              type="button"
              class="gemini-terminal-row group"
              :aria-label="t('gemini.overview.terminal.copyCommand', { command: snippet.command })"
              @click="copyCommand(snippet.command)"
            >
              <span class="gemini-terminal-row__prompt">$</span>
              <span class="gemini-terminal-row__body">
                <span class="gemini-terminal-row__label">{{ snippet.label }}</span>
                <code class="gemini-terminal-row__code">{{ snippet.command }}</code>
              </span>
              <span class="gemini-terminal-row__copy">
                <SIcon
                  name="Copy"
                  size="w-3.5 h-3.5"
                />
                {{ copiedCommand === snippet.command ? t('gemini.overview.terminal.copied') : t('gemini.overview.terminal.copy') }}
              </span>
            </button>
          </div>

          <div class="gemini-config-preview">
            <div
              v-for="item in configPreview"
              :key="item.label"
              class="gemini-config-preview__item"
            >
              <span>{{ item.label }}</span>
              <code>{{ item.value }}</code>
            </div>
          </div>
        </Card>
      </section>

      <PlatformUsageInsightPanel
        :spec="antigravityUsageSpec"
        :state="antigravityUsagePresentation"
        :loading="antigravityUsage.loading.value"
        :error="antigravityUsage.error.value"
        @refresh="antigravityUsage.refresh()"
      />

      <section
        class="gemini-module-section animate-slide-up"
        style="animation-delay: 120ms"
      >
        <div class="gemini-section-heading">
          <div>
            <p class="gemini-section-kicker">
              {{ t('gemini.overview.modules.eyebrow') }}
            </p>
            <h2 class="gemini-section-title">
              {{ t('gemini.overview.modules.title') }}
            </h2>
          </div>
          <p class="gemini-section-description">
            {{ t('gemini.overview.modules.subtitle') }}
          </p>
        </div>

        <div class="gemini-module-grid">
          <RouterLink
            v-for="module in moduleCards"
            :key="module.key"
            :to="module.to"
            class="gemini-module-link"
            :class="{ 'gemini-module-link--wide': module.spotlight }"
          >
            <Card
              variant="glass"
              padding="none"
              body-class="flex h-full flex-col"
              class="gemini-module-card"
              :class="[
                `gemini-module-card--${module.tone}`,
                { 'gemini-module-card--spotlight': module.spotlight },
              ]"
            >
              <span class="gemini-module-card__orbit" />
              <div class="gemini-module-card__topline">
                <div class="gemini-module-card__icon">
                  <SIcon
                    :name="module.icon"
                    size="w-5 h-5"
                  />
                </div>
                <span class="gemini-module-card__badge">{{ module.badge }}</span>
              </div>
              <div class="gemini-module-card__copy">
                <h3 class="gemini-module-card__title">
                  {{ module.title }}
                </h3>
                <p class="gemini-module-card__desc">
                  {{ module.description }}
                </p>
              </div>
              <div class="gemini-module-card__footer">
                <span>{{ module.hint }}</span>
                <strong>{{ module.status }}</strong>
              </div>
              <SIcon
                name="ArrowRight"
                size="w-4 h-4"
                class="gemini-module-card__arrow"
              />
            </Card>
          </RouterLink>
        </div>
      </section>

      <section
        class="gemini-quick-dock animate-slide-up"
        style="animation-delay: 220ms"
      >
        <Card
          v-for="card in quickInfoCards"
          :key="card.key"
          variant="glass"
          padding="none"
          body-class="h-full"
          class="gemini-quick-card"
        >
          <div class="gemini-quick-card__header">
            <div class="gemini-quick-card__icon">
              <SIcon
                :name="card.icon"
                size="w-4 h-4"
              />
            </div>
            <div>
              <p class="gemini-quick-card__kicker">
                {{ card.kicker }}
              </p>
              <h3 class="gemini-quick-card__title">
                {{ card.title }}
              </h3>
            </div>
          </div>
          <ul class="gemini-quick-card__list">
            <li
              v-for="item in card.items"
              :key="item"
            >
              <span class="gemini-quick-card__dot" />
              <span>{{ item }}</span>
            </li>
          </ul>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import PlatformUsageInsightPanel from '@/components/platform-usage/PlatformUsageInsightPanel.vue'
import { usePlatformUsageInsight } from '@/composables/usePlatformUsageInsight'
import {
  buildPlatformUsageI18nLabels,
  buildPlatformUsageSpec,
} from '@/views/platform-usage/platformUsageSpecs'

type ModuleTone = 'gemini' | 'command' | 'capability' | 'plugin'
type TagTone = 'gemini' | 'command' | 'neutral' | 'capability'

interface HeroTag {
  key: string
  icon: string
  label: string
  tone: TagTone
}

interface ModuleCard {
  key: string
  to: string
  icon: string
  tone: ModuleTone
  title: string
  description: string
  badge: string
  hint: string
  status: string
  spotlight?: boolean
}

interface TerminalSnippet {
  label: string
  command: string
}

interface QuickInfoCard {
  key: string
  icon: string
  kicker: string
  title: string
  items: string[]
}

const { t } = useI18n({ useScope: 'global' })
const copiedCommand = ref<string | null>(null)
let copyResetTimer: number | undefined

const antigravityUsageLabels = computed(() => buildPlatformUsageI18nLabels(t))
const antigravityUsage = usePlatformUsageInsight({
  platform: 'gemini',
  labels: antigravityUsageLabels,
  tone: 'antigravity',
})
const antigravityUsageSpec = computed(() => buildPlatformUsageSpec(t, 'gemini'))
const antigravityUsagePresentation = computed(() => antigravityUsage.presentation.value)

const heroTags = computed<HeroTag[]>(() => [
  {
    key: 'mcp',
    icon: 'Server',
    label: t('gemini.overview.tags.mcp'),
    tone: 'gemini',
  },
  {
    key: 'commands',
    icon: 'Command',
    label: t('gemini.overview.tags.commands'),
    tone: 'command',
  },
  {
    key: 'settings',
    icon: 'Settings',
    label: t('gemini.overview.tags.settings'),
    tone: 'neutral',
  },
  {
    key: 'boundary',
    icon: 'ShieldCheck',
    label: t('gemini.overview.tags.boundary'),
    tone: 'capability',
  },
])

const terminalSnippets = computed<TerminalSnippet[]>(() => [
  {
    label: t('gemini.overview.terminal.helpLabel'),
    command: 'agy --help',
  },
  {
    label: t('gemini.overview.terminal.versionLabel'),
    command: 'agy --version',
  },
  {
    label: t('gemini.overview.terminal.importLabel'),
    command: 'agy plugin import gemini',
  },
])

const configPreview = computed(() => [
  {
    label: t('gemini.overview.terminal.settingsPath'),
    value: '~/.gemini/antigravity-cli/settings.json',
  },
  {
    label: t('gemini.overview.terminal.mcpPath'),
    value: '~/.gemini/antigravity-cli/mcp_config.json',
  },
  {
    label: t('gemini.overview.terminal.skillsPath'),
    value: '~/.gemini/antigravity-cli/skills',
  },
  {
    label: t('gemini.overview.terminal.workspacePath'),
    value: '.agents/{mcp_config.json,skills}',
  },
])

const moduleCards = computed<ModuleCard[]>(() => [
  {
    key: 'mcp',
    to: '/antigravity/mcp',
    icon: 'Server',
    tone: 'gemini',
    spotlight: true,
    title: t('gemini.mcp.title'),
    description: t('gemini.overview.modules.mcpDescription'),
    badge: t('gemini.overview.modules.supportedBadge'),
    hint: t('gemini.overview.modules.mcpHint'),
    status: t('gemini.overview.modules.mcpStatus'),
  },
  {
    key: 'slash-commands',
    to: '/antigravity/slash-commands',
    icon: 'Command',
    tone: 'command',
    spotlight: true,
    title: t('gemini.slashCommands.title'),
    description: t('gemini.overview.modules.commandsDescription'),
    badge: t('gemini.overview.modules.supportedBadge'),
    hint: t('gemini.overview.modules.commandsHint'),
    status: t('gemini.overview.modules.commandsStatus'),
  },
  {
    key: 'agents',
    to: '/antigravity/agents',
    icon: 'Bot',
    tone: 'capability',
    title: t('gemini.agents.title'),
    description: t('gemini.overview.modules.agentsDescription'),
    badge: t('gemini.overview.modules.boundaryBadge'),
    hint: t('gemini.overview.modules.agentsHint'),
    status: t('gemini.overview.modules.agentsStatus'),
  },
  {
    key: 'plugins',
    to: '/antigravity/plugins',
    icon: 'Puzzle',
    tone: 'plugin',
    title: t('gemini.plugins.title'),
    description: t('gemini.overview.modules.pluginsDescription'),
    badge: t('gemini.overview.modules.boundaryBadge'),
    hint: t('gemini.overview.modules.pluginsHint'),
    status: t('gemini.overview.modules.pluginsStatus'),
  },
])

const quickInfoCards = computed<QuickInfoCard[]>(() => [
  {
    key: 'paths',
    icon: 'Workflow',
    kicker: t('gemini.overview.quick.pathsKicker'),
    title: t('gemini.overview.quick.pathsTitle'),
    items: [
      t('gemini.overview.quick.pathMcp'),
      t('gemini.overview.quick.pathCommands'),
      t('gemini.overview.quick.pathSkills'),
    ],
  },
  {
    key: 'config',
    icon: 'FolderOpen',
    kicker: t('gemini.overview.quick.configKicker'),
    title: t('gemini.overview.quick.configTitle'),
    items: [
      t('gemini.overview.quick.configSettings'),
      t('gemini.overview.quick.configProjectCommands'),
      t('gemini.overview.quick.configUserCommands'),
    ],
  },
  {
    key: 'tips',
    icon: 'Lightbulb',
    kicker: t('gemini.overview.quick.tipsKicker'),
    title: t('gemini.overview.quick.tipsTitle'),
    items: [
      t('gemini.overview.quick.tipSafeCommands'),
      t('gemini.overview.quick.tipBoundaries'),
      t('gemini.overview.quick.tipNoBackendChange'),
    ],
  },
])

const copyCommand = async (command: string) => {
  if (!navigator.clipboard?.writeText) return

  await navigator.clipboard.writeText(command)
  copiedCommand.value = command

  if (copyResetTimer) {
    window.clearTimeout(copyResetTimer)
  }

  copyResetTimer = window.setTimeout(() => {
    if (copiedCommand.value === command) {
      copiedCommand.value = null
    }
  }, 1600)
}

onBeforeUnmount(() => {
  if (copyResetTimer) {
    window.clearTimeout(copyResetTimer)
  }
})
</script>

<style scoped>
.gemini-view {
  @apply relative min-h-full overflow-hidden p-5 md:p-8 lg:p-10;

  background:
    radial-gradient(circle at 18% 12%, rgb(var(--color-info-rgb) / 8%) 0, transparent 28rem),
    radial-gradient(circle at 82% 8%, rgb(var(--color-warning-rgb) / 7%) 0, transparent 24rem),
    linear-gradient(145deg, rgb(var(--color-bg-base-rgb) / 96%), rgb(var(--color-bg-surface-rgb) / 90%));
}

.gemini-shell {
  @apply relative z-10 mx-auto max-w-7xl space-y-7;
}

.gemini-hero {
  @apply grid gap-5 lg:grid-cols-[minmax(0,1.08fr)_minmax(22rem,0.92fr)] lg:items-stretch;
}

.gemini-hero__copy,
.gemini-terminal-card,
.gemini-module-card,
.gemini-quick-card {
  border: 1px solid var(--stage-border-soft);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 76%), rgb(var(--color-bg-surface-rgb) / 58%)),
    var(--stage-surface-soft);
  box-shadow:
    0 24px 60px rgb(var(--color-bg-base-rgb) / 22%),
    inset 0 1px 0 rgb(255 251 245 / 10%);
}

.gemini-hero__copy {
  @apply relative overflow-hidden rounded-[2rem] p-6 md:p-8;
}

.gemini-hero__copy::before {
  content: '';
  position: absolute;
  inset: 0;
  background:
    linear-gradient(115deg, rgb(var(--color-platform-gemini-rgb) / 18%), transparent 34%),
    repeating-linear-gradient(90deg, transparent 0 4.5rem, rgb(var(--color-platform-gemini-rgb) / 5%) 4.5rem 4.55rem);
  mask-image: linear-gradient(90deg, #000, transparent 82%);
  pointer-events: none;
}

.gemini-hero__eyebrow,
.gemini-section-kicker,
.gemini-terminal-card__eyebrow,
.gemini-quick-card__kicker {
  @apply relative z-10 flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.22em];

  color: var(--stage-text-muted);
}

.gemini-hero__pulse {
  @apply h-2 w-2 rounded-full;

  background: var(--platform-gemini);
  box-shadow: 0 0 16px rgb(var(--color-platform-gemini-rgb) / 65%);
}

.gemini-hero__eyebrow-muted {
  @apply hidden rounded-full px-2 py-1 normal-case tracking-normal md:inline-flex;

  color: var(--stage-chip-neutral-text);
  background: var(--stage-chip-neutral-bg);
  border: 1px solid var(--stage-chip-neutral-border);
}

.gemini-title-row {
  @apply relative z-10 mt-6 flex items-center gap-4;
}

.gemini-brand-mark {
  @apply flex h-14 w-14 shrink-0 items-center justify-center rounded-2xl border shadow-lg backdrop-blur-md;

  color: var(--platform-gemini);
  background: rgb(var(--color-platform-gemini-rgb) / 12%);
  border-color: rgb(var(--color-platform-gemini-rgb) / 22%);
  box-shadow: 0 18px 42px rgb(var(--color-platform-gemini-rgb) / 14%);
}

.gemini-title {
  @apply text-[2.45rem] font-semibold leading-none tracking-[-0.055em] md:text-[3.4rem];

  font-family: var(--font-brand);
  color: var(--stage-text-primary);
}

.gemini-subtitle {
  @apply mt-2 text-sm font-semibold uppercase tracking-[0.2em];

  color: var(--platform-gemini);
}

.gemini-description {
  @apply relative z-10 mt-5 max-w-2xl text-base leading-7 md:text-lg;

  color: var(--stage-text-secondary);
}

.gemini-tag-row {
  @apply relative z-10 mt-6 flex flex-wrap gap-2.5;
}

.gemini-tag {
  @apply inline-flex items-center gap-2 rounded-full border px-3 py-1.5 text-xs font-semibold;
}

.gemini-tag--gemini {
  color: var(--platform-gemini);
  background: rgb(var(--color-platform-gemini-rgb) / 10%);
  border-color: rgb(var(--color-platform-gemini-rgb) / 22%);
}

.gemini-tag--command {
  color: var(--color-accent-secondary);
  background: rgb(var(--color-accent-secondary-rgb) / 10%);
  border-color: rgb(var(--color-accent-secondary-rgb) / 22%);
}

.gemini-tag--neutral {
  color: var(--stage-chip-neutral-text);
  background: var(--stage-chip-neutral-bg);
  border-color: var(--stage-chip-neutral-border);
}

.gemini-tag--capability {
  color: var(--color-warning);
  background: rgb(var(--color-warning-rgb) / 10%);
  border-color: rgb(var(--color-warning-rgb) / 22%);
}

.gemini-hero-actions {
  @apply relative z-10 mt-7 flex flex-wrap gap-3;
}

.gemini-action-button {
  @apply gap-0;
}

.gemini-action-button--primary {
  --ui-button-shadow: 0 18px 36px rgb(var(--color-platform-gemini-rgb) / 18%);

  color: var(--color-text-inverted);
  border-color: rgb(var(--color-platform-gemini-rgb) / 22%);
  background: linear-gradient(180deg, var(--platform-gemini), rgb(var(--color-platform-gemini-rgb) / 82%));
}

.gemini-action-button--primary:hover:not(:disabled) {
  background: linear-gradient(180deg, rgb(var(--color-platform-gemini-rgb) / 92%), var(--platform-gemini));
}

.gemini-terminal-card {
  @apply relative overflow-hidden rounded-[2rem] p-5;
}

.gemini-terminal-card::before {
  content: '';
  position: absolute;
  inset: auto -20% -45% 12%;
  height: 18rem;
  background: radial-gradient(circle, rgb(var(--color-platform-gemini-rgb) / 18%) 0, transparent 65%);
  pointer-events: none;
}

.gemini-terminal-card__header {
  @apply relative z-10 flex items-start justify-between gap-4;
}

.gemini-terminal-card__title {
  @apply mt-1 text-xl font-semibold tracking-[-0.03em];

  color: var(--stage-text-primary);
}

.gemini-terminal-card__lights {
  @apply flex gap-1.5 rounded-full border px-2 py-1.5;

  background: var(--stage-surface-soft);
  border-color: var(--stage-border-soft);
}

.gemini-terminal-card__lights span {
  @apply h-2 w-2 rounded-full;

  background: var(--stage-text-muted);
}

.gemini-terminal-card__lights span:first-child {
  background: var(--color-danger);
}

.gemini-terminal-card__lights span:nth-child(2) {
  background: var(--color-warning);
}

.gemini-terminal-card__lights span:nth-child(3) {
  background: var(--color-success);
}

.gemini-terminal-lines {
  @apply relative z-10 mt-5 space-y-2;
}

.gemini-terminal-row {
  @apply flex w-full cursor-copy items-center gap-3 rounded-2xl border p-3 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-primary/30;

  background: rgb(var(--color-bg-base-rgb) / 34%);
  border-color: var(--stage-border-soft);
}

.gemini-terminal-row:hover,
.gemini-terminal-row:focus-visible {
  border-color: rgb(var(--color-platform-gemini-rgb) / 30%);
  background: rgb(var(--color-platform-gemini-rgb) / 8%);
  transform: translateY(-1px);
}

.gemini-terminal-row__prompt {
  @apply flex h-7 w-7 shrink-0 items-center justify-center rounded-lg font-mono text-sm font-bold;

  color: var(--platform-gemini);
  background: rgb(var(--color-platform-gemini-rgb) / 12%);
}

.gemini-terminal-row__body {
  @apply min-w-0 flex-1;
}

.gemini-terminal-row__label {
  @apply block text-xs;

  color: var(--stage-text-muted);
}

.gemini-terminal-row__code {
  @apply mt-0.5 block truncate font-mono text-sm;

  color: var(--stage-text-primary);
}

.gemini-terminal-row__copy {
  @apply inline-flex shrink-0 items-center gap-1.5 rounded-full border px-2 py-1 text-[0.68rem] font-semibold opacity-70 transition-opacity group-hover:opacity-100;

  color: var(--stage-text-secondary);
  background: var(--stage-chip-neutral-bg);
  border-color: var(--stage-chip-neutral-border);
}

.gemini-config-preview {
  @apply relative z-10 mt-4 grid gap-2 sm:grid-cols-2 lg:grid-cols-1 xl:grid-cols-2;
}

.gemini-config-preview__item {
  @apply rounded-2xl border p-3;

  background: var(--stage-surface-soft);
  border-color: var(--stage-border-soft);
}

.gemini-config-preview__item span {
  @apply block text-xs font-semibold uppercase tracking-[0.14em];

  color: var(--stage-text-muted);
}

.gemini-config-preview__item code {
  @apply mt-1 block truncate font-mono text-xs;

  color: var(--stage-text-primary);
}

.gemini-module-section,
.gemini-quick-dock {
  @apply space-y-5;
}

.gemini-section-heading {
  @apply flex flex-col gap-3 md:flex-row md:items-end md:justify-between;
}

.gemini-section-title {
  @apply mt-1 text-2xl font-semibold tracking-[-0.04em];

  color: var(--stage-text-primary);
}

.gemini-section-description {
  @apply max-w-2xl text-sm leading-6;

  color: var(--stage-text-secondary);
}

.gemini-module-grid {
  @apply grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4;
}

.gemini-module-link {
  @apply block h-full focus-visible:outline-none;
}

.gemini-module-link:focus-visible .gemini-module-card {
  outline: 2px solid rgb(var(--color-platform-gemini-rgb) / 52%);
  outline-offset: 3px;
}

.gemini-module-link--wide {
  @apply lg:col-span-2;
}

.gemini-module-card {
  --module-color: var(--platform-gemini);
  --module-rgb: var(--color-platform-gemini-rgb);

  @apply relative flex h-full min-h-[15rem] flex-col rounded-[1.6rem] p-5 transition-all duration-300;
}

.gemini-module-card--command {
  --module-color: var(--color-accent-secondary);
  --module-rgb: var(--color-accent-secondary-rgb);
}

.gemini-module-card--capability {
  --module-color: var(--color-warning);
  --module-rgb: var(--color-warning-rgb);
}

.gemini-module-card--plugin {
  --module-color: var(--color-info);
  --module-rgb: var(--color-info-rgb);
}

.gemini-module-card:hover {
  border-color: rgb(var(--module-rgb) / 32%);
  box-shadow:
    0 24px 50px rgb(var(--module-rgb) / 12%),
    inset 0 1px 0 rgb(255 251 245 / 12%);
  transform: translateY(-3px);
}

.gemini-module-card__orbit {
  @apply absolute inset-x-4 top-4 h-px origin-left scale-x-50 rounded-full opacity-50 transition-all duration-300;

  background: linear-gradient(90deg, rgb(var(--module-rgb) / 72%), transparent);
}

.gemini-module-card:hover .gemini-module-card__orbit {
  @apply scale-x-100 opacity-100;
}

.gemini-module-card__topline {
  @apply relative z-10 flex items-center justify-between gap-3;
}

.gemini-module-card__icon {
  @apply flex h-11 w-11 items-center justify-center rounded-2xl border transition-transform duration-300;

  color: var(--module-color);
  background: rgb(var(--module-rgb) / 12%);
  border-color: rgb(var(--module-rgb) / 18%);
}

.gemini-module-card:hover .gemini-module-card__icon {
  transform: scale(1.06) rotate(-2deg);
}

.gemini-module-card__badge {
  @apply rounded-full border px-2.5 py-1 text-[0.68rem] font-semibold uppercase tracking-[0.12em];

  color: var(--module-color);
  background: rgb(var(--module-rgb) / 10%);
  border-color: rgb(var(--module-rgb) / 18%);
}

.gemini-module-card__copy {
  @apply relative z-10 mt-5 flex-1;
}

.gemini-module-card__title {
  @apply text-xl font-semibold tracking-[-0.035em];

  color: var(--stage-text-primary);
}

.gemini-module-card__desc {
  @apply mt-2 text-sm leading-6;

  color: var(--stage-text-secondary);
}

.gemini-module-card__footer {
  @apply relative z-10 mt-5 flex items-center justify-between gap-3 rounded-2xl border p-3 text-xs;

  color: var(--stage-text-muted);
  background: rgb(var(--color-bg-base-rgb) / 28%);
  border-color: var(--stage-border-soft);
}

.gemini-module-card__footer strong {
  @apply shrink-0 font-semibold;

  color: var(--module-color);
}

.gemini-module-card__arrow {
  @apply absolute bottom-5 right-5 opacity-0 transition-all duration-300;

  color: var(--module-color);
  transform: translateX(-0.35rem);
}

.gemini-module-card:hover .gemini-module-card__arrow {
  @apply opacity-100;

  transform: translateX(0);
}

.gemini-quick-dock {
  @apply grid grid-cols-1 gap-4 lg:grid-cols-3;
}

.gemini-quick-card {
  @apply rounded-[1.6rem] p-5;
}

.gemini-quick-card__header {
  @apply flex items-start gap-3;
}

.gemini-quick-card__icon {
  @apply flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl border;

  color: var(--platform-gemini);
  background: rgb(var(--color-platform-gemini-rgb) / 10%);
  border-color: rgb(var(--color-platform-gemini-rgb) / 18%);
}

.gemini-quick-card__title {
  @apply mt-1 text-base font-semibold;

  color: var(--stage-text-primary);
}

.gemini-quick-card__list {
  @apply mt-4 space-y-3;
}

.gemini-quick-card__list li {
  @apply flex gap-3 text-sm leading-6;

  color: var(--stage-text-secondary);
}

.gemini-quick-card__dot {
  @apply mt-2.5 h-1.5 w-1.5 shrink-0 rounded-full;

  background: var(--platform-gemini);
  box-shadow: 0 0 10px rgb(var(--color-platform-gemini-rgb) / 46%);
}

@media (width <= 767px) {
  .gemini-title {
    @apply text-[2.25rem];
  }

  .gemini-terminal-row {
    @apply items-start;
  }

  .gemini-terminal-row__copy {
    @apply sr-only;
  }
}

@media (prefers-reduced-motion: reduce) {
  .gemini-terminal-row,
  .gemini-module-card,
  .gemini-module-card__icon,
  .gemini-module-card__arrow,
  .gemini-module-card__orbit {
    transition: none;
  }

  .gemini-terminal-row:hover,
  .gemini-module-card:hover {
    transform: none;
  }
}
</style>
