<template>
  <div class="gemini-view stage-page">
    <!-- Background Mesh -->
    <!-- Standard Animated Background -->
    <AnimatedBackground
      contained
      variant="complex"
    />

    <div class="gemini-shell">
      <!-- HEADER -->
      <section class="animate-slide-up">
        <div class="gemini-header">
          <div class="gemini-brand">
            <div class="gemini-brand__icon">
              <SIcon
                name="Sparkles"
                size="w-8 h-8"
              />
            </div>
            <div>
              <h1 class="gemini-brand__title">
                Gemini CLI
              </h1>
              <p class="gemini-brand__subtitle">
                {{ $t('gemini.overview.description') }}
              </p>
            </div>
          </div>

          <RouterLink to="/">
            <Button
              variant="glass"
              class="gemini-home-button"
            >
              <SIcon
                name="Home"
                size="w-4 h-4"
              />
              {{ $t('common.backToHome') }}
            </Button>
          </RouterLink>
        </div>

        <!-- Tags -->
        <div class="gemini-tag-row">
          <span class="gemini-tag gemini-tag--gemini">
            <SIcon
              name="Server"
              size="w-3 h-3"
            /> MCP Server
          </span>
          <span class="gemini-tag gemini-tag--success">
            <SIcon
              name="Bot"
              size="w-3 h-3"
            /> Smart Agents
          </span>
          <span class="gemini-tag gemini-tag--claude">
            <SIcon
              name="Puzzle"
              size="w-3 h-3"
            /> Plugin System
          </span>
          <span class="gemini-tag gemini-tag--qoder">
            <SIcon
              name="Sparkles"
              size="w-3 h-3"
            /> Multimodal
          </span>
        </div>
      </section>

      <!-- MODULES -->
      <section class="animate-slide-up">
        <div class="gemini-section-heading">
          <SIcon
            name="Boxes"
            size="w-5 h-5"
            class="gemini-section-heading__icon"
          />
          <h2 class="gemini-section-heading__title">
            {{ $t('common.modules') }}
          </h2>
          <div class="gemini-section-heading__rule" />
        </div>

        <div class="gemini-module-grid">
          <!-- MCP -->
          <RouterLink
            to="/gemini-cli/mcp"
            class="gemini-module-link"
          >
            <Card
              variant="glass"
              hover
              glow
              class="gemini-module-card"
            >
              <div class="gemini-module-card__icon gemini-module-card__icon--gemini">
                <SIcon
                  name="Server"
                  size="w-6 h-6"
                />
              </div>
              <div>
                <h3 class="gemini-module-card__title gemini-module-card__title--gemini">
                  {{ $t('gemini.mcp.title') }}
                </h3>
                <p class="gemini-module-card__desc">
                  {{ $t('gemini.mcp.subtitle') }}
                </p>
                <span class="gemini-module-card__badge gemini-module-card__badge--gemini">MCP</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Agents -->
          <RouterLink
            to="/gemini-cli/agents"
            class="gemini-module-link"
          >
            <Card
              variant="glass"
              hover
              glow
              class="gemini-module-card"
            >
              <div class="gemini-module-card__icon gemini-module-card__icon--success">
                <SIcon
                  name="Bot"
                  size="w-6 h-6"
                />
              </div>
              <div>
                <h3 class="gemini-module-card__title gemini-module-card__title--success">
                  {{ $t('gemini.agents.title') }}
                </h3>
                <p class="gemini-module-card__desc">
                  {{ $t('gemini.agents.subtitle') }}
                </p>
                <span class="gemini-module-card__badge gemini-module-card__badge--success">AI Agents</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Plugins -->
          <RouterLink
            to="/gemini-cli/plugins"
            class="gemini-module-link"
          >
            <Card
              variant="glass"
              hover
              glow
              class="gemini-module-card"
            >
              <div class="gemini-module-card__icon gemini-module-card__icon--claude">
                <SIcon
                  name="Puzzle"
                  size="w-6 h-6"
                />
              </div>
              <div>
                <h3 class="gemini-module-card__title gemini-module-card__title--claude">
                  {{ $t('gemini.plugins.title') }}
                </h3>
                <p class="gemini-module-card__desc">
                  {{ $t('gemini.plugins.subtitle') }}
                </p>
                <span class="gemini-module-card__badge gemini-module-card__badge--claude">Plugins</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Slash Commands -->
          <RouterLink
            to="/gemini-cli/slash-commands"
            class="gemini-module-link gemini-module-link--wide"
          >
            <Card
              variant="glass"
              hover
              glow
              class="gemini-module-card"
            >
              <div class="gemini-module-card__icon gemini-module-card__icon--qoder">
                <SIcon
                  name="Command"
                  size="w-6 h-6"
                />
              </div>
              <div>
                <h3 class="gemini-module-card__title gemini-module-card__title--qoder">
                  {{ $t('gemini.slashCommands.title') }}
                </h3>
                <p class="gemini-module-card__desc">
                  {{ $t('gemini.slashCommands.subtitle') }}
                </p>
                <span class="gemini-module-card__badge gemini-module-card__badge--qoder">Commands</span>
              </div>
            </Card>
          </RouterLink>
        </div>
      </section>

      <!-- INFO -->
      <section
        class="animate-slide-up"
        style="animation-delay: 200ms"
      >
        <Card
          variant="glass"
          class="gemini-info-card"
        >
          <div class="gemini-info-card__content">
            <div class="gemini-info-card__icon">
              <SIcon
                name="Info"
                size="w-6 h-6"
              />
            </div>
            <div class="gemini-info-card__body">
              <h3 class="gemini-info-card__title">
                💡 {{ $t('gemini.overview.welcome') }}
              </h3>
              <div class="gemini-info-grid">
                <div class="gemini-info-feature">
                  <div class="gemini-info-dot" />
                  <span class="gemini-info-feature__text">{{ $t('gemini.overview.feature1') }}</span>
                </div>
                <div class="gemini-info-feature">
                  <div class="gemini-info-dot" />
                  <span class="gemini-info-feature__text">{{ $t('gemini.overview.feature2') }}</span>
                </div>
                <div class="gemini-info-feature">
                  <div class="gemini-info-dot" />
                  <span class="gemini-info-feature__text">{{ $t('gemini.overview.feature3') }}</span>
                </div>
                <div class="gemini-info-feature">
                  <div class="gemini-info-dot" />
                  <span class="gemini-info-feature__text">{{ $t('gemini.overview.feature4') }}</span>
                </div>
              </div>
            </div>
          </div>
        </Card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { RouterLink } from 'vue-router'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
</script>

<style scoped>
.gemini-view {
  @apply relative min-h-full overflow-hidden p-6 lg:p-10;
}

.gemini-shell {
  @apply relative z-10 mx-auto max-w-7xl space-y-8;
}

.gemini-header {
  @apply mb-8 flex flex-col justify-between gap-6 md:flex-row md:items-center;
}

.gemini-brand {
  @apply flex items-center gap-5;
}

.gemini-brand__icon {
  @apply flex h-16 w-16 items-center justify-center rounded-2xl border shadow-lg backdrop-blur-md;

  color: var(--platform-gemini);
  background: rgb(var(--color-platform-gemini-rgb) / 10%);
  border-color: rgb(var(--color-platform-gemini-rgb) / 20%);
}

.gemini-brand__title {
  @apply text-4xl font-bold tracking-tight;

  font-family: MapleBright, 'Microsoft YaHei UI', system-ui, sans-serif;
  background: linear-gradient(90deg, var(--platform-gemini) 0%, var(--platform-qoder) 55%, var(--color-success) 100%);
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.gemini-brand__subtitle {
  color: var(--stage-text-secondary);
  @apply mt-1 max-w-xl text-lg;
}

.gemini-home-button {
  @apply gap-2;
}

.gemini-tag-row {
  @apply flex flex-wrap gap-3;
}

.gemini-tag {
  @apply flex items-center gap-2 rounded-full px-3 py-1.5 text-xs font-bold uppercase;

  letter-spacing: 0.1em;
}

.gemini-tag--gemini {
  color: var(--platform-gemini);
  background: rgb(var(--color-platform-gemini-rgb) / 10%);
  border: 1px solid rgb(var(--color-platform-gemini-rgb) / 20%);
}

.gemini-tag--success {
  @apply border border-accent-success/20 bg-accent-success/10 text-accent-success;
}

.gemini-tag--claude {
  @apply border border-platform-claude/20 bg-platform-claude/10 text-platform-claude;
}

.gemini-tag--qoder {
  color: var(--platform-qoder);
  background: rgb(var(--color-platform-qoder-rgb) / 10%);
  border: 1px solid rgb(var(--color-platform-qoder-rgb) / 20%);
}

.gemini-section-heading {
  @apply mb-6 flex items-center gap-3;
}

.gemini-section-heading__icon {
  color: var(--platform-gemini);
}

.gemini-section-heading__title {
  color: var(--stage-text-muted);
  @apply text-lg font-bold uppercase;

  letter-spacing: 0.2em;
}

.gemini-section-heading__rule {
  @apply h-px flex-1 bg-border-subtle;
}

.gemini-module-grid {
  @apply grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3;
}

.gemini-module-link {
  @apply block h-full;
}

.gemini-module-link--wide {
  @apply md:col-span-2 lg:col-span-1;
}

.gemini-module-card {
  @apply flex h-full items-start gap-4 p-6;
}

.gemini-module-card__icon {
  @apply rounded-xl p-3 transition-transform duration-300;
}

.gemini-module-link:hover .gemini-module-card__icon {
  transform: scale(1.1);
}

.gemini-module-card__icon--gemini {
  color: var(--platform-gemini);
  background: rgb(var(--color-platform-gemini-rgb) / 10%);
}

.gemini-module-card__icon--success {
  @apply bg-accent-success/10 text-accent-success;
}

.gemini-module-card__icon--claude {
  @apply bg-platform-claude/10 text-platform-claude;
}

.gemini-module-card__icon--qoder {
  color: var(--platform-qoder);
  background: rgb(var(--color-platform-qoder-rgb) / 10%);
}

.gemini-module-card__title {
  color: var(--stage-text-primary);
  @apply mb-1 text-lg font-bold transition-colors;
}

.gemini-module-card__desc {
  color: var(--stage-text-secondary);
  @apply mb-3 text-sm leading-relaxed;
}

.gemini-module-card__badge {
  @apply inline-flex rounded border px-2 py-1 font-bold uppercase;

  font-size: 10px;
  letter-spacing: 0.1em;
}

.gemini-module-link:hover .gemini-module-card__title--gemini {
  color: var(--platform-gemini);
}

.gemini-module-link:hover .gemini-module-card__title--success {
  @apply text-accent-success;
}

.gemini-module-link:hover .gemini-module-card__title--claude {
  @apply text-platform-claude;
}

.gemini-module-link:hover .gemini-module-card__title--qoder {
  color: var(--platform-qoder);
}

.gemini-module-card__badge--gemini {
  color: var(--platform-gemini);
  background: rgb(var(--color-platform-gemini-rgb) / 10%);
  border-color: rgb(var(--color-platform-gemini-rgb) / 20%);
}

.gemini-module-card__badge--success {
  @apply border-accent-success/20 bg-accent-success/10 text-accent-success;
}

.gemini-module-card__badge--claude {
  @apply border-platform-claude/20 bg-platform-claude/10 text-platform-claude;
}

.gemini-module-card__badge--qoder {
  color: var(--platform-qoder);
  background: rgb(var(--color-platform-qoder-rgb) / 10%);
  border-color: rgb(var(--color-platform-qoder-rgb) / 20%);
}

.gemini-info-card {
  @apply p-6;
}

.gemini-info-card__content {
  @apply flex items-start gap-4;
}

.gemini-info-card__icon {
  @apply shrink-0 rounded-xl p-3;

  color: var(--platform-gemini);
  background: rgb(var(--color-platform-gemini-rgb) / 10%);
}

.gemini-info-card__body {
  @apply space-y-4;
}

.gemini-info-card__title {
  color: var(--stage-text-primary);
  @apply text-lg font-bold;
}

.gemini-info-grid {
  @apply grid grid-cols-1 gap-4 md:grid-cols-2;
}

.gemini-info-feature {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);
  @apply flex items-center gap-3 rounded-lg p-3;
}

.gemini-info-dot {
  @apply h-2 w-2 rounded-full;

  background: var(--platform-gemini);
  box-shadow: 0 0 8px var(--platform-gemini);
}

.gemini-info-feature__text {
  color: var(--stage-text-secondary);
  @apply text-sm;
}
</style>
