<template>
  <div class="claude-view stage-page">
    <!-- Enhanced Animated Background -->
    <AnimatedBackground
      contained
      variant="aurora"
    />

    <div class="claude-shell">
      <!-- HEADER -->
      <header class="claude-header animate-slide-up">
        <div class="claude-header__brand">
          <div class="claude-header__icon">
            <SIcon
              name="Code2"
              size="w-8 h-8"
              class="text-accent-primary"
            />
          </div>
          <div>
            <h1 class="claude-header__title">
              Claude Code
            </h1>
            <p class="claude-header__subtitle">
              {{ $t('claudeCode.subtitle') }}
            </p>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <RouterLink to="/claude-code/auth">
            <Button
              variant="ghost"
              class="group"
            >
              <SIcon
                name="KeyRound"
                size="w-4 h-4"
                class="mr-2 group-hover:text-accent-primary transition-colors"
              />
              Auth
            </Button>
          </RouterLink>

          <RouterLink to="/">
            <Button
              variant="ghost"
              class="group"
            >
              <SIcon
                name="Home"
                size="w-4 h-4"
                class="mr-2 group-hover:text-accent-primary transition-colors"
              />
              {{ $t('claudeCode.backToHome') }}
            </Button>
          </RouterLink>
        </div>
      </header>

      <!-- FEATURES TAGS -->
      <div
        class="claude-tag-row animate-slide-up"
        style="animation-delay: 100ms"
      >
        <div class="claude-tag claude-tag--primary">
          <SIcon
            name="Zap"
            size="w-3 h-3"
          /> {{ $t('claudeCode.features.mcpSupport') }}
        </div>
        <div class="claude-tag claude-tag--secondary">
          <SIcon
            name="Bot"
            size="w-3 h-3"
          /> {{ $t('claudeCode.features.aiAgents') }}
        </div>
        <div class="claude-tag claude-tag--info">
          <SIcon
            name="Cloud"
            size="w-3 h-3"
          /> {{ $t('claudeCode.features.cloudSync') }}
        </div>
      </div>

      <!-- CORE MODULES (Grid) -->
      <section
        class="claude-core-grid animate-slide-up"
      >
        <RouterLink
          to="/claude-code/profiles"
          class="group h-full"
        >
          <Card
            variant="glass"
            hover
            glow
            class="claude-feature-card claude-feature-card--primary"
          >
            <!-- Decorative bg icon -->
            <SIcon
              name="Settings"
              size="w-24 h-24 lg:w-28 lg:h-28"
              class="claude-feature-card__watermark claude-feature-card__watermark--primary"
            />

            <div class="claude-feature-card__icon claude-feature-card__icon--primary">
              <SIcon
                name="Settings"
                size="w-6 h-6"
                class="text-accent-primary"
              />
            </div>
            <div class="relative z-10">
              <div class="claude-feature-card__title-row">
                <h3 class="claude-feature-card__title claude-feature-card__title--primary">
                  {{ $t('claudeCode.modules.configManagement.title') }}
                </h3>
                <span class="claude-feature-card__badge claude-feature-card__badge--primary">
                  {{ $t('claudeCode.modules.configManagement.badge') }}
                </span>
              </div>
              <p class="claude-feature-card__desc">
                {{ $t('claudeCode.modules.configManagement.desc') }}
              </p>
            </div>
          </Card>
        </RouterLink>

        <RouterLink
          to="/sync"
          class="group h-full"
        >
          <Card
            variant="glass"
            hover
            glow
            class="claude-feature-card claude-feature-card--secondary"
          >
            <!-- Decorative bg icon -->
            <SIcon
              name="Cloud"
              size="w-24 h-24 lg:w-28 lg:h-28"
              class="claude-feature-card__watermark claude-feature-card__watermark--secondary"
            />

            <div class="claude-feature-card__icon claude-feature-card__icon--secondary">
              <SIcon
                name="Cloud"
                size="w-6 h-6"
                class="text-accent-secondary"
              />
            </div>
            <div class="relative z-10">
              <div class="claude-feature-card__title-row">
                <h3 class="claude-feature-card__title claude-feature-card__title--secondary">
                  {{ $t('claudeCode.modules.cloudSync.title') }}
                </h3>
                <span class="claude-feature-card__badge claude-feature-card__badge--secondary">
                  {{ $t('claudeCode.modules.cloudSync.badge') }}
                </span>
              </div>
              <p class="claude-feature-card__desc">
                {{ $t('claudeCode.modules.cloudSync.desc') }}
              </p>
            </div>
          </Card>
        </RouterLink>
      </section>

      <!-- EXTENDED MODULES -->
      <section class="animate-slide-up">
        <div class="claude-section-heading">
          <SIcon
            name="Boxes"
            size="w-5 h-5"
            class="claude-section-heading__icon"
          />
          <h2 class="claude-section-heading__title">
            {{ $t('claudeCode.modules.extensionsTitle') }}
          </h2>
          <div class="claude-section-heading__rule" />
        </div>

        <div class="claude-extension-grid">
          <RouterLink
            to="/mcp-manager"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="claude-extension-card"
            >
              <div class="claude-extension-card__header">
                <div class="claude-extension-card__icon claude-extension-card__icon--info">
                  <SIcon
                    name="Server"
                    size="w-5 h-5"
                  />
                </div>
                <span class="claude-extension-card__eyebrow">MCP</span>
              </div>
              <div>
                <h3 class="claude-extension-card__title claude-extension-card__title--info">
                  {{ $t('claudeCode.modules.mcpServers.title') }}
                </h3>
                <p class="claude-extension-card__desc">
                  {{ $t('claudeCode.modules.mcpServers.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>

          <RouterLink
            to="/agents"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="claude-extension-card"
            >
              <div class="claude-extension-card__header">
                <div class="claude-extension-card__icon claude-extension-card__icon--success">
                  <SIcon
                    name="Users"
                    size="w-5 h-5"
                  />
                </div>
                <span class="claude-extension-card__eyebrow">Agent</span>
              </div>
              <div>
                <h3 class="claude-extension-card__title claude-extension-card__title--success">
                  {{ $t('claudeCode.modules.agents.title') }}
                </h3>
                <p class="claude-extension-card__desc">
                  {{ $t('claudeCode.modules.agents.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>

          <RouterLink
            to="/plugins"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="claude-extension-card"
            >
              <div class="claude-extension-card__header">
                <div class="claude-extension-card__icon claude-extension-card__icon--primary">
                  <SIcon
                    name="Puzzle"
                    size="w-5 h-5"
                  />
                </div>
                <span class="claude-extension-card__eyebrow">Plug</span>
              </div>
              <div>
                <h3 class="claude-extension-card__title claude-extension-card__title--primary">
                  {{ $t('claudeCode.modules.plugins.title') }}
                </h3>
                <p class="claude-extension-card__desc">
                  {{ $t('claudeCode.modules.plugins.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>

          <RouterLink
            to="/slash-commands"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="claude-extension-card"
            >
              <div class="claude-extension-card__header">
                <div class="claude-extension-card__icon claude-extension-card__icon--warning">
                  <SIcon
                    name="Terminal"
                    size="w-5 h-5"
                  />
                </div>
                <span class="claude-extension-card__eyebrow">CMD</span>
              </div>
              <div>
                <h3 class="claude-extension-card__title claude-extension-card__title--warning">
                  {{ $t('claudeCode.modules.slashCommands.title') }}
                </h3>
                <p class="claude-extension-card__desc">
                  {{ $t('claudeCode.modules.slashCommands.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>

          <RouterLink
            to="/claude-code/settings"
            class="group"
          >
            <Card
              variant="elevated"
              hover
              class="claude-extension-card"
            >
              <div class="claude-extension-card__header">
                <div class="claude-extension-card__icon claude-extension-card__icon--secondary">
                  <SIcon
                    name="SlidersHorizontal"
                    size="w-5 h-5"
                  />
                </div>
                <span class="claude-extension-card__eyebrow">CFG</span>
              </div>
              <div>
                <h3 class="claude-extension-card__title claude-extension-card__title--secondary">
                  {{ $t('claudeCode.modules.settings.title') }}
                </h3>
                <p class="claude-extension-card__desc">
                  {{ $t('claudeCode.modules.settings.desc') }}
                </p>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- BOTTOM GRID (Commands & Tips) -->
      <section
        class="claude-bottom-grid animate-slide-up"
        style="animation-delay: 500ms"
      >
        <!-- Common Commands -->
        <Card
          variant="glass"
          class="claude-panel"
        >
          <h3 class="claude-panel__title">
            <SIcon
              name="Terminal"
              size="w-5 h-5"
              class="text-accent-primary"
            />
            {{ $t('claudeCode.quickActions.commonCommands') }}
          </h3>
          <div class="claude-stack">
            <div
              v-for="cmd in commonCommands"
              :key="cmd.cmd"
              class="claude-command-row"
              @click="copyCommand(cmd.cmd)"
            >
              <span class="claude-command-row__label">{{ cmd.label }}</span>
              <div class="flex items-center gap-2">
                <code class="claude-command-row__code">{{ cmd.cmd }}</code>
                <SIcon
                  name="Copy"
                  size="w-3 h-3"
                  class="claude-command-row__copy"
                />
              </div>
            </div>
          </div>
        </Card>

        <!-- Resources & Tips -->
        <div class="space-y-6">
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
                class="claude-resource-link"
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
                  <li>{{ $t('claudeCode.tips.tip3') }}</li>
                </ul>
              </div>
            </div>
          </Card>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'

const { t } = useI18n()

const commonCommands = computed(() => [
   { label: t('claudeCode.quickActions.viewCurrentConfig'), cmd: 'ccr current' },
   { label: t('claudeCode.quickActions.switchConfig'), cmd: 'ccr switch' },
   { label: t('claudeCode.quickActions.listAllConfigs'), cmd: 'ccr list' },
])

const resources = computed(() => [
   { label: t('claudeCode.quickActions.officialDocs'), url: 'https://docs.anthropic.com/claude/docs', icon: 'BookOpen' },
   { label: t('claudeCode.quickActions.mcpProtocol'), url: 'https://modelcontextprotocol.io', icon: 'Server' },
   { label: t('claudeCode.quickActions.sdkRepo'), url: 'https://github.com/anthropics/anthropic-sdk-typescript', icon: 'Github' },
])

const copyCommand = (cmd: string) => {
  navigator.clipboard.writeText(cmd)
  // Toast notification would go here
}
</script>

<style scoped>
.claude-view {
  @apply relative min-h-full overflow-hidden p-6 lg:p-10;
}

.claude-shell {
  @apply relative z-10 mx-auto max-w-7xl space-y-10;
}

.claude-header {
  @apply flex flex-col justify-between gap-6 md:flex-row md:items-center;
}

.claude-header__brand {
  @apply flex items-center gap-5;
}

.claude-header__icon {
  @apply flex h-16 w-16 items-center justify-center rounded-2xl border border-accent-primary/30 shadow-lg backdrop-blur-md;

  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 14%), rgb(var(--color-accent-secondary-rgb) / 10%));
}

.claude-header__title {
  @apply mb-1 text-[2.6rem] font-semibold tracking-[-0.04em] text-text-primary;

  font-family: var(--font-brand);
}

.claude-header__subtitle {
  color: var(--stage-text-secondary);

  @apply text-lg;
}

.claude-tag-row {
  @apply flex flex-wrap gap-3;
}

.claude-tag {
  @apply flex items-center gap-2 rounded-full px-3 py-1.5 text-xs font-semibold;

  letter-spacing: 0.04em;
}

.claude-tag--primary {
  @apply border border-accent-primary/30 bg-accent-primary/15 text-accent-primary;
}

.claude-tag--secondary {
  @apply border border-accent-secondary/30 bg-accent-secondary/10 text-accent-secondary;
}

.claude-tag--info {
  @apply border border-accent-info/30 bg-accent-info/10 text-accent-info;
}

.claude-core-grid {
  @apply grid grid-cols-1 gap-6 md:grid-cols-2;
}

.claude-feature-card {
  @apply relative flex h-full items-start gap-5 overflow-hidden p-6;
}

.claude-feature-card--primary {
  @apply border-accent-primary/20 hover:border-accent-primary/40;
}

.claude-feature-card--secondary {
  @apply border-accent-secondary/20 hover:border-accent-secondary/40;
}

.claude-feature-card__watermark {
  @apply absolute bottom-4 right-4 rotate-12 transition-colors;
}

.claude-feature-card__watermark--primary {
  @apply text-accent-primary/5 group-hover:text-accent-primary/10;
}

.claude-feature-card__watermark--secondary {
  @apply text-accent-secondary/5 group-hover:text-accent-secondary/10;
}

.claude-feature-card__icon {
  @apply flex h-12 w-12 shrink-0 items-center justify-center rounded-xl border transition-transform duration-300 group-hover:scale-110;
}

.claude-feature-card__icon--primary {
  @apply border-accent-primary/30 bg-accent-primary/15;
}

.claude-feature-card__icon--secondary {
  @apply border-accent-secondary/30 bg-accent-secondary/15;
}

.claude-feature-card__title-row {
  @apply mb-2 flex items-center gap-2;
}

.claude-feature-card__title {
  color: var(--stage-text-primary);

  @apply text-xl font-bold transition-colors;
}

.claude-feature-card__title--primary {
  @apply group-hover:text-accent-primary;
}

.claude-feature-card__title--secondary {
  @apply group-hover:text-accent-secondary;
}

.claude-feature-card__badge {
  @apply rounded px-2 py-0.5 font-bold;

  font-size: 10px;
}

.claude-feature-card__badge--primary {
  @apply border border-accent-primary/30 bg-accent-primary/20 text-accent-primary;
}

.claude-feature-card__badge--secondary {
  @apply border border-accent-secondary/30 bg-accent-secondary/20 text-accent-secondary;
}

.claude-feature-card__desc {
  color: var(--stage-text-secondary);

  @apply leading-relaxed;
}

.claude-section-heading {
  @apply mb-6 flex items-center gap-3;
}

.claude-section-heading__title {
  color: var(--stage-text-muted);

  @apply text-sm font-semibold tracking-[0.08em];
}

.claude-section-heading__icon {
  color: var(--stage-text-muted);
}

.claude-section-heading__rule {
  @apply h-px flex-1 bg-border-subtle;
}

.claude-extension-grid {
  @apply grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-6;
}

.claude-extension-card {
  @apply flex h-full flex-col gap-4 p-4;
}

.claude-extension-card__header {
  @apply flex items-center justify-between;
}

.claude-extension-card__icon {
  @apply flex h-10 w-10 items-center justify-center rounded-lg transition-transform group-hover:scale-110;
}

.claude-extension-card__icon--info {
  @apply bg-accent-info/10 text-accent-info;
}

.claude-extension-card__icon--success {
  @apply bg-accent-success/10 text-accent-success;
}

.claude-extension-card__icon--danger {
  @apply bg-accent-danger/10 text-accent-danger;
}

.claude-extension-card__icon--primary {
  @apply bg-accent-primary/10 text-accent-primary;
}

.claude-extension-card__icon--warning {
  @apply bg-accent-warning/10 text-accent-warning;
}

.claude-extension-card__icon--secondary {
  @apply bg-accent-secondary/10 text-accent-secondary;
}

.claude-extension-card__eyebrow {
  color: var(--stage-text-muted);
  background: var(--stage-chip-neutral-bg);
  border: 1px solid var(--stage-chip-neutral-border);

  @apply rounded px-2 py-1 font-semibold;

  font-size: 10px;
}

.claude-extension-card__title {
  color: var(--stage-text-primary);

  @apply mb-1 font-bold transition-colors;
}

.claude-extension-card__title--info {
  @apply group-hover:text-accent-info;
}

.claude-extension-card__title--success {
  @apply group-hover:text-accent-success;
}

.claude-extension-card__title--danger {
  @apply group-hover:text-accent-danger;
}

.claude-extension-card__title--primary {
  @apply group-hover:text-accent-primary;
}

.claude-extension-card__title--warning {
  @apply group-hover:text-accent-warning;
}

.claude-extension-card__title--secondary {
  @apply group-hover:text-accent-secondary;
}

.claude-extension-card__desc {
  color: var(--stage-text-muted);

  @apply line-clamp-2 text-xs;
}

.claude-bottom-grid {
  @apply grid grid-cols-1 gap-6 lg:grid-cols-2;
}

.claude-panel {
  @apply p-6;
}

.claude-panel__title {
  color: var(--stage-text-primary);

  @apply mb-4 flex items-center gap-2 text-lg font-semibold;
}

.claude-stack {
  @apply space-y-3;
}

.claude-command-row {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply flex cursor-copy items-center justify-between rounded-xl p-3 transition-colors hover:border-accent-primary/20;
}

.claude-command-row__label {
  color: var(--stage-text-secondary);

  @apply text-sm font-medium;
}

.claude-command-row__code {
  color: var(--stage-text-muted);
  background: var(--stage-chip-neutral-bg);
  border: 1px solid var(--stage-chip-neutral-border);

  @apply rounded px-2 py-1 text-xs font-mono transition-colors group-hover:text-accent-primary;
}

.claude-command-row__copy {
  color: var(--stage-text-muted);

  @apply opacity-0 transition-opacity group-hover:opacity-100;
}

.claude-resource-grid {
  @apply grid grid-cols-2 gap-3;
}

.claude-resource-link {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);

  @apply flex items-center gap-3 rounded-xl p-3 transition-colors;
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
  @apply border-accent-info/20 bg-accent-info/10 p-4;
}

.claude-tip-card__content {
  @apply flex gap-3;
}

.claude-tip-card__icon {
  @apply mt-0.5 shrink-0 text-accent-info;
}

.claude-tip-card__copy {
  @apply space-y-2;
}

.claude-tip-card__title {
  @apply text-sm font-semibold text-accent-info;
}

.claude-tip-card__list {
  color: var(--stage-text-secondary);

  @apply list-inside list-disc space-y-1 text-xs;
}
</style>
