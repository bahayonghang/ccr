<template>
  <PageShell class="claude-view">
    <template #header>
      <PageHeader
        :title="$t('claudeCode.title')"
        :eyebrow="$t('claudeCode.hero.eyebrow')"
        :description="$t('claudeCode.subtitle')"
      >
        <template #actions>
          <RouterLink to="/claude-code/auth">
            <Button>
              <SIcon
                name="KeyRound"
                size="w-4 h-4"
                class="mr-2"
              />
              {{ $t('claudeCode.hero.primaryAction') }}
            </Button>
          </RouterLink>
          <RouterLink to="/claude-code/profiles">
            <Button variant="ghost">
              <SIcon
                name="Settings"
                size="w-4 h-4"
                class="mr-2"
              />
              {{ $t('claudeCode.hero.secondaryAction') }}
            </Button>
          </RouterLink>
        </template>
      </PageHeader>
    </template>

    <div class="claude-console">
      <p class="claude-console__line">
        <span>$</span> {{ currentConfigCommand }}
      </p>
      <p class="claude-console__status">
        {{ $t('claudeCode.hero.consoleStatus') }}
      </p>
      <div class="claude-console__chips">
        <span
          v-for="chip in heroChips"
          :key="chip"
          class="claude-chip"
        >
          {{ chip }}
        </span>
      </div>
    </div>

      <section
        class="claude-section animate-slide-up claude-section--observer"
        :style="{ animationDelay: '80ms' }"
        aria-label="Claude Code usage insight"
      >
        <UsageInsightPanel />
      </section>

      <section
        class="claude-tag-row animate-slide-up"
        :style="{ animationDelay: '100ms' }"
        aria-label="Claude Code capabilities"
      >
        <span
          v-for="feature in featureTags"
          :key="feature.label"
          class="claude-tag"
          :class="feature.className"
        >
          <SIcon
            :name="feature.icon"
            size="w-3 h-3"
          />
          {{ feature.label }}
        </span>
      </section>

      <section
        class="claude-section animate-slide-up"
        :style="{ animationDelay: '180ms' }"
      >
        <div class="claude-section-heading">
          <SIcon
            name="Zap"
            size="w-5 h-5"
            class="claude-section-heading__icon"
          />
          <div>
            <p class="claude-section-heading__eyebrow">
              {{ $t('claudeCode.modules.title') }}
            </p>
            <h2 class="claude-section-heading__title">
              {{ $t('claudeCode.modules.coreTitle') }}
            </h2>
          </div>
          <div class="claude-section-heading__rule" />
        </div>

        <div class="claude-core-grid">
          <RouterLink
            v-for="module in coreModules"
            :key="module.to"
            :to="module.to"
            class="group h-full"
          >
            <Card
              variant="glass"
              hover
              class="claude-action-card"
              :class="module.cardClass"
            >
              <SIcon
                :name="module.icon"
                size="w-20 h-20 lg:w-24 lg:h-24"
                class="claude-action-card__watermark"
              />
              <div class="claude-action-card__icon">
                <SIcon
                  :name="module.icon"
                  size="w-6 h-6"
                />
              </div>
              <div class="claude-action-card__copy">
                <div class="claude-action-card__title-row">
                  <h3 class="claude-action-card__title">
                    {{ module.title }}
                  </h3>
                  <span class="claude-action-card__badge">
                    {{ module.badge }}
                  </span>
                </div>
                <p class="claude-action-card__desc">
                  {{ module.desc }}
                </p>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <section
        class="claude-section animate-slide-up"
        :style="{ animationDelay: '260ms' }"
      >
        <div class="claude-section-heading">
          <SIcon
            name="Boxes"
            size="w-5 h-5"
            class="claude-section-heading__icon"
          />
          <div>
            <p class="claude-section-heading__eyebrow">
              {{ $t('claudeCode.modules.extensionsEyebrow') }}
            </p>
            <h2 class="claude-section-heading__title">
              {{ $t('claudeCode.modules.extensionsTitle') }}
            </h2>
          </div>
          <div class="claude-section-heading__rule" />
        </div>

        <div class="claude-extension-grid">
          <RouterLink
            v-for="module in extensionModules"
            :key="module.to"
            :to="module.to"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="claude-extension-card"
            >
              <div class="claude-extension-card__header">
                <div
                  class="claude-extension-card__icon"
                  :class="module.iconClass"
                >
                  <SIcon
                    :name="module.icon"
                    size="w-5 h-5"
                  />
                </div>
                <span class="claude-extension-card__eyebrow">{{ module.badge }}</span>
              </div>
              <div>
                <h3 class="claude-extension-card__title">
                  {{ module.title }}
                </h3>
                <p class="claude-extension-card__desc">
                  {{ module.desc }}
                </p>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <section
        class="claude-bottom-grid animate-slide-up"
        :style="{ animationDelay: '340ms' }"
      >
        <Card
          variant="glass"
          class="claude-panel"
        >
          <h3 class="claude-panel__title">
            <SIcon
              name="Terminal"
              size="w-5 h-5"
              class="text-accent-warning"
            />
            {{ $t('claudeCode.quickActions.commonCommands') }}
          </h3>
          <p class="claude-panel__desc">
            {{ $t('claudeCode.quickActions.commonCommandsDesc') }}
          </p>
          <div class="claude-stack">
            <button
              v-for="cmd in commonCommands"
              :key="cmd.cmd"
              type="button"
              class="claude-command-row group"
              @click="copyCommand(cmd.cmd)"
            >
              <span class="claude-command-row__label">{{ cmd.label }}</span>
              <span class="claude-command-row__command">
                <code class="claude-command-row__code">{{ cmd.cmd }}</code>
                <SIcon
                  name="Copy"
                  size="w-3 h-3"
                  class="claude-command-row__copy"
                />
              </span>
            </button>
          </div>
        </Card>

        <div class="claude-resource-column">
          <Card
            variant="glass"
            class="claude-panel"
          >
            <h3 class="claude-panel__title">
              <SIcon
                name="BookOpen"
                size="w-5 h-5"
                class="text-accent-secondary"
              />
              {{ $t('claudeCode.quickActions.resources') }}
            </h3>
            <div class="claude-resource-grid">
              <a
                v-for="res in resources"
                :key="res.url"
                :href="res.url"
                target="_blank"
                rel="noreferrer"
                class="claude-resource-link group"
              >
                <SIcon
                  :name="res.icon"
                  size="w-4 h-4"
                  class="claude-resource-link__icon"
                />
                <span class="claude-resource-link__label">{{ res.label }}</span>
                <SIcon
                  name="ExternalLink"
                  size="w-3 h-3"
                  class="claude-resource-link__external"
                />
              </a>
            </div>
          </Card>

          <Card
            variant="outline"
            class="claude-tip-card"
          >
            <div class="claude-tip-card__content">
              <SIcon
                name="Info"
                size="w-5 h-5"
                class="claude-tip-card__icon"
              />
              <div class="claude-tip-card__copy">
                <h4 class="claude-tip-card__title">
                  {{ $t('claudeCode.tips.title') }}
                </h4>
                <ul class="claude-tip-card__list">
                  <li>{{ $t('claudeCode.tips.tip1') }}</li>
                  <li>{{ $t('claudeCode.tips.tip2') }}</li>
                </ul>
              </div>
            </div>
          </Card>
        </div>
      </section>
  </PageShell>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import Button from '@/components/ui/Button.vue'
import Card from '@/components/ui/Card.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { copyText } from '@/utils/clipboard'

// 用量洞察面板较重，按需异步加载，避免拖慢 Claude Code 首屏
const UsageInsightPanel = defineAsyncComponent({
  loader: () => import('@/components/claude-observer/UsageInsightPanel.vue'),
  suspensible: false,
})

const { t } = useI18n()
const currentConfigCommand = 'ccr current'
const switchConfigCommand = 'ccr switch'
const listConfigsCommand = 'ccr list'

const heroChips = computed(() => [
  t('claudeCode.features.configManagement'),
  t('claudeCode.features.authReady'),
  t('claudeCode.features.localSettings'),
])

const featureTags = computed(() => [
  { label: t('claudeCode.features.mcpSupport'), icon: 'Server', className: 'claude-tag--primary' },
  { label: t('claudeCode.features.aiAgents'), icon: 'Bot', className: 'claude-tag--secondary' },
  { label: t('claudeCode.features.slashCommands'), icon: 'Terminal', className: 'claude-tag--warning' },
  { label: t('claudeCode.features.localSettings'), icon: 'SlidersHorizontal', className: 'claude-tag--neutral' },
])

const coreModules = computed(() => [
  {
    to: '/claude-code/profiles',
    icon: 'Settings',
    title: t('claudeCode.modules.profiles.title'),
    desc: t('claudeCode.modules.profiles.desc'),
    badge: t('claudeCode.modules.profiles.badge'),
    cardClass: 'claude-action-card--primary',
  },
  {
    to: '/claude-code/auth',
    icon: 'KeyRound',
    title: t('claudeCode.modules.auth.title'),
    desc: t('claudeCode.modules.auth.desc'),
    badge: t('claudeCode.modules.auth.badge'),
    cardClass: 'claude-action-card--warning',
  },
  {
    to: '/claude-code/settings',
    icon: 'SlidersHorizontal',
    title: t('claudeCode.modules.settings.title'),
    desc: t('claudeCode.modules.settings.desc'),
    badge: t('claudeCode.modules.settings.badge'),
    cardClass: 'claude-action-card--secondary',
  },
])

const extensionModules = computed(() => [
  {
    to: '/mcp-manager',
    icon: 'Server',
    title: t('claudeCode.modules.mcpServers.title'),
    desc: t('claudeCode.modules.mcpServers.desc'),
    badge: t('claudeCode.modules.mcpServers.badge'),
    iconClass: 'claude-extension-card__icon--info',
  },
  {
    to: '/agents',
    icon: 'Users',
    title: t('claudeCode.modules.agents.title'),
    desc: t('claudeCode.modules.agents.desc'),
    badge: t('claudeCode.modules.agents.badge'),
    iconClass: 'claude-extension-card__icon--success',
  },
  {
    to: '/plugins',
    icon: 'Puzzle',
    title: t('claudeCode.modules.plugins.title'),
    desc: t('claudeCode.modules.plugins.desc'),
    badge: t('claudeCode.modules.plugins.badge'),
    iconClass: 'claude-extension-card__icon--primary',
  },
  {
    to: '/slash-commands',
    icon: 'Terminal',
    title: t('claudeCode.modules.slashCommands.title'),
    desc: t('claudeCode.modules.slashCommands.desc'),
    badge: t('claudeCode.modules.slashCommands.badge'),
    iconClass: 'claude-extension-card__icon--warning',
  },
])

const commonCommands = computed(() => [
  { label: t('claudeCode.quickActions.viewCurrentConfig'), cmd: currentConfigCommand },
  { label: t('claudeCode.quickActions.switchConfig'), cmd: switchConfigCommand },
  { label: t('claudeCode.quickActions.listAllConfigs'), cmd: listConfigsCommand },
])

const resources = computed(() => [
  { label: t('claudeCode.quickActions.officialDocs'), url: 'https://docs.anthropic.com/en/docs/claude-code', icon: 'BookOpen' },
  { label: t('claudeCode.quickActions.mcpProtocol'), url: 'https://modelcontextprotocol.io', icon: 'Server' },
  { label: t('claudeCode.quickActions.settingsReference'), url: 'https://docs.anthropic.com/en/docs/claude-code/settings', icon: 'SlidersHorizontal' },
])

const copyCommand = (cmd: string) => {
  void copyText(cmd)
}
</script>

<style scoped>
.claude-view {
  background: var(--color-bg-elevated);
}

.claude-console {
  padding: 1rem 1.25rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: 12px;
  background: var(--color-bg-surface);
  font-family: var(--font-mono);
}

.claude-console__line {
  margin: 0;
  font-size: 0.875rem;
  color: var(--color-text-primary);
}

.claude-console__line span {
  margin-right: 0.5rem;
  color: var(--color-text-muted);
}

.claude-console__status {
  margin: 0.5rem 0 0;
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--color-text-secondary);
}

.claude-console__chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-top: 0.75rem;
}

.claude-chip {
  padding: 0.25rem 0.625rem;
  border: 1px solid var(--color-border-subtle);
  border-radius: 8px;
  background: var(--color-bg-elevated);
  color: var(--color-text-secondary);
  font-size: 0.6875rem;
  font-weight: 500;
}

.claude-tag-row {
  @apply flex flex-wrap gap-3;
}

.claude-tag {
  @apply flex items-center gap-2 rounded-lg border px-3 py-1.5 text-xs font-medium;
}

.claude-tag--primary {
  @apply border-accent-info/30 bg-accent-info/10 text-accent-info;
}

.claude-tag--secondary {
  @apply border-accent-success/30 bg-accent-success/10 text-accent-success;
}

.claude-tag--warning {
  @apply border-accent-warning/30 bg-accent-warning/10 text-accent-warning;
}

.claude-tag--neutral {
  color: var(--stage-text-secondary);
  background: var(--stage-chip-neutral-bg);
  border-color: var(--stage-chip-neutral-border);
}

.claude-section {
  @apply space-y-5;
}

.claude-section-heading {
  @apply flex items-center gap-3;
}

.claude-section-heading__eyebrow {
  color: var(--stage-text-muted);

  @apply text-xs font-medium;
}

.claude-section-heading__title {
  color: var(--stage-text-primary);

  @apply text-xl font-semibold;
}

.claude-section-heading__icon {
  @apply text-accent-warning;
}

.claude-section-heading__rule {
  @apply h-px flex-1 bg-border-subtle;
}

.claude-core-grid {
  @apply grid grid-cols-1 gap-4 lg:grid-cols-3;
}

.claude-action-card {
  @apply relative flex h-full min-h-[190px] flex-col justify-between gap-5 overflow-hidden p-5 transition-transform duration-300 group-hover:-translate-y-1;

  background: var(--color-bg-surface);
}

.claude-action-card--primary {
  @apply border-accent-info/20 hover:border-accent-info/40;
}

.claude-action-card--warning {
  @apply border-accent-warning/20 hover:border-accent-warning/40;
}

.claude-action-card--secondary {
  @apply border-accent-secondary/20 hover:border-accent-secondary/40;
}

.claude-action-card__watermark {
  @apply absolute bottom-3 right-3 rotate-12 text-text-disabled transition-colors group-hover:text-text-ghost;
}

.claude-action-card__icon {
  border-color: var(--stage-border-soft);
  background: var(--stage-surface-soft);

  @apply flex h-12 w-12 items-center justify-center rounded-xl border text-accent-warning;
}

.claude-action-card__copy {
  @apply relative z-10;
}

.claude-action-card__title-row {
  @apply mb-2 flex flex-wrap items-center gap-2;
}

.claude-action-card__title {
  color: var(--stage-text-primary);

  @apply text-xl font-bold;
}

.claude-action-card__badge {
  @apply rounded border border-accent-warning/25 bg-accent-warning/10 px-2 py-0.5 text-[10px] font-bold uppercase tracking-[0.08em] text-accent-warning;
}

.claude-action-card__desc {
  color: var(--stage-text-secondary);

  @apply text-sm leading-6;
}

.claude-extension-grid {
  @apply grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4;
}

.claude-extension-card {
  @apply flex h-full min-h-[150px] flex-col justify-between gap-4 p-4;

  background: var(--color-bg-surface);
}

.claude-extension-card__header {
  @apply flex items-center justify-between;
}

.claude-extension-card__icon {
  @apply flex h-10 w-10 items-center justify-center rounded-lg transition-transform group-hover:scale-105;
}

.claude-extension-card__icon--info {
  @apply bg-accent-info/10 text-accent-info;
}

.claude-extension-card__icon--success {
  @apply bg-accent-success/10 text-accent-success;
}

.claude-extension-card__icon--primary {
  @apply bg-accent-primary/10 text-accent-primary;
}

.claude-extension-card__icon--warning {
  @apply bg-accent-warning/10 text-accent-warning;
}

.claude-extension-card__eyebrow {
  color: var(--stage-text-muted);
  background: var(--stage-chip-neutral-bg);
  border: 1px solid var(--stage-chip-neutral-border);

  @apply rounded px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.08em];
}

.claude-extension-card__title {
  color: var(--stage-text-primary);

  @apply mb-1 font-bold transition-colors group-hover:text-accent-warning;
}

.claude-extension-card__desc {
  color: var(--stage-text-muted);

  @apply text-xs leading-5;
}

.claude-bottom-grid {
  @apply grid grid-cols-1 gap-6 lg:grid-cols-2;
}

.claude-resource-column {
  @apply space-y-6;
}

.claude-panel {
  @apply p-6;

  background: var(--color-bg-surface);
}

.claude-panel__title {
  color: var(--stage-text-primary);

  @apply mb-2 flex items-center gap-2 text-lg font-semibold;
}

.claude-panel__desc {
  color: var(--stage-text-muted);

  @apply mb-4 text-sm;
}

.claude-stack {
  @apply space-y-3;
}

.claude-command-row {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply flex w-full cursor-copy items-center justify-between gap-3 rounded-xl p-3 text-left transition-colors hover:border-accent-warning/25;
}

.claude-command-row__label {
  color: var(--stage-text-secondary);

  @apply text-sm font-medium;
}

.claude-command-row__command {
  @apply flex shrink-0 items-center gap-2;
}

.claude-command-row__code {
  color: var(--stage-text-muted);
  background: var(--stage-chip-neutral-bg);
  border: 1px solid var(--stage-chip-neutral-border);

  @apply rounded px-2 py-1 text-xs font-mono transition-colors group-hover:text-accent-warning;
}

.claude-command-row__copy {
  color: var(--stage-text-muted);

  @apply opacity-0 transition-opacity group-hover:opacity-100;
}

.claude-resource-grid {
  @apply grid grid-cols-1 gap-3 sm:grid-cols-3 lg:grid-cols-1 xl:grid-cols-3;
}

.claude-resource-link {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply flex items-center gap-3 rounded-xl p-3 transition-colors hover:border-accent-secondary/25;
}

.claude-resource-link__icon {
  color: var(--stage-text-muted);

  @apply transition-colors group-hover:text-accent-secondary;
}

.claude-resource-link__label {
  color: var(--stage-text-secondary);

  @apply text-sm font-medium transition-colors;
}

.claude-resource-link__external {
  color: var(--stage-text-muted);

  @apply ml-auto opacity-0 group-hover:opacity-100;
}

.claude-tip-card {
  @apply border-accent-warning/20 bg-accent-warning/10 p-4;
}

.claude-tip-card__content {
  @apply flex gap-3;
}

.claude-tip-card__icon {
  @apply mt-0.5 shrink-0 text-accent-warning;
}

.claude-tip-card__copy {
  @apply space-y-2;
}

.claude-tip-card__title {
  @apply text-sm font-semibold text-accent-warning;
}

.claude-tip-card__list {
  color: var(--stage-text-secondary);

  @apply list-inside list-disc space-y-1 text-xs leading-5;
}
</style>
