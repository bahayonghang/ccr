<template>
  <div class="qwen-view stage-page">
    <!-- Background Mesh -->
    <!-- Standard Animated Background -->
    <AnimatedBackground
      contained
      variant="complex"
    />

    <div class="qwen-shell">
      <!-- HEADER -->
      <section class="animate-slide-up">
        <div class="qwen-header">
          <div class="qwen-brand">
            <div class="qwen-brand__icon">
              <SIcon
                name="Zap"
                size="w-8 h-8"
              />
            </div>
            <div>
              <h1 class="qwen-brand__title">
                {{ $t('qwen.overview.title') }}
              </h1>
              <p class="qwen-brand__subtitle">
                {{ $t('qwen.overview.description') }}
              </p>
            </div>
          </div>

          <RouterLink to="/">
            <Button
              variant="glass"
              class="qwen-home-button"
            >
              <SIcon
                name="Home"
                size="w-4 h-4"
              />
              {{ $t('qwen.overview.backToHome') }}
            </Button>
          </RouterLink>
        </div>

        <!-- Tags -->
        <div class="qwen-tag-row">
          <span class="qwen-tag qwen-tag--purple">
            <SIcon
              name="Bot"
              size="w-3 h-3"
            /> {{ $t('qwen.overview.tags.model') }}
          </span>
          <span class="qwen-tag qwen-tag--pink">
            <SIcon
              name="MessageSquare"
              size="w-3 h-3"
            /> {{ $t('qwen.overview.tags.dialog') }}
          </span>
          <span class="qwen-tag qwen-tag--amber">
            <SIcon
              name="Server"
              size="w-3 h-3"
            /> {{ $t('qwen.overview.tags.mcp') }}
          </span>
          <span class="qwen-tag qwen-tag--red">
            <SIcon
              name="Globe"
              size="w-3 h-3"
            /> {{ $t('qwen.overview.tags.chinese') }}
          </span>
        </div>
      </section>

      <!-- MODULES -->
      <section class="animate-slide-up">
        <div class="qwen-section-heading">
          <SIcon
            name="Boxes"
            size="w-5 h-5"
            class="qwen-section-heading__icon"
          />
          <h2 class="qwen-section-heading__title">
            {{ $t('qwen.overview.features') }}
          </h2>
          <div class="qwen-section-heading__rule" />
        </div>

        <div class="qwen-module-grid">
          <!-- MCP -->
          <RouterLink
            to="/qwen/mcp"
            class="qwen-module-link"
          >
            <Card
              variant="glass"
              hover
              glow
              class="qwen-module-card"
            >
              <div class="qwen-module-card__icon qwen-module-card__icon--purple">
                <SIcon
                  name="Server"
                  size="w-6 h-6"
                />
              </div>
              <div>
                <h3 class="qwen-module-card__title qwen-module-card__title--purple">
                  {{ $t('qwen.mcp.title') }}
                </h3>
                <p class="qwen-module-card__desc">
                  {{ $t('qwen.mcp.description') }}
                </p>
                <span class="qwen-module-card__badge qwen-module-card__badge--purple">{{ $t('qwen.mcp.badge') }}</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Agents -->
          <RouterLink
            to="/qwen/agents"
            class="qwen-module-link"
          >
            <Card
              variant="glass"
              hover
              glow
              class="qwen-module-card"
            >
              <div class="qwen-module-card__icon qwen-module-card__icon--pink">
                <SIcon
                  name="Bot"
                  size="w-6 h-6"
                />
              </div>
              <div>
                <h3 class="qwen-module-card__title qwen-module-card__title--pink">
                  {{ $t('qwen.agents.title') }}
                </h3>
                <p class="qwen-module-card__desc">
                  {{ $t('qwen.agents.description') }}
                </p>
                <span class="qwen-module-card__badge qwen-module-card__badge--pink">{{ $t('qwen.agents.badge') }}</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Plugins -->
          <RouterLink
            to="/qwen/plugins"
            class="qwen-module-link"
          >
            <Card
              variant="glass"
              hover
              glow
              class="qwen-module-card"
            >
              <div class="qwen-module-card__icon qwen-module-card__icon--amber">
                <SIcon
                  name="Puzzle"
                  size="w-6 h-6"
                />
              </div>
              <div>
                <h3 class="qwen-module-card__title qwen-module-card__title--amber">
                  {{ $t('qwen.plugins.title') }}
                </h3>
                <p class="qwen-module-card__desc">
                  {{ $t('qwen.plugins.description') }}
                </p>
                <span class="qwen-module-card__badge qwen-module-card__badge--amber">{{ $t('qwen.plugins.badge') }}</span>
              </div>
            </Card>
          </RouterLink>

          <!-- Slash Commands -->
          <RouterLink
            to="/qwen/slash-commands"
            class="qwen-module-link qwen-module-link--wide"
          >
            <Card
              variant="glass"
              hover
              glow
              class="qwen-module-card"
            >
              <div class="qwen-module-card__icon qwen-module-card__icon--red">
                <SIcon
                  name="Command"
                  size="w-6 h-6"
                />
              </div>
              <div>
                <h3 class="qwen-module-card__title qwen-module-card__title--red">
                  {{ $t('qwen.slashCommands.title') }}
                </h3>
                <p class="qwen-module-card__desc">
                  {{ $t('qwen.slashCommands.description') }}
                </p>
                <span class="qwen-module-card__badge qwen-module-card__badge--red">{{ $t('qwen.slashCommands.badge') }}</span>
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
          class="qwen-info-card"
        >
          <div class="qwen-info-card__content">
            <div class="qwen-info-card__icon">
              <SIcon
                name="Info"
                size="w-6 h-6"
              />
            </div>
            <div class="qwen-info-card__body">
              <h3 class="qwen-info-card__title">
                💡 {{ $t('qwen.overview.featureTitle') }}
              </h3>
              <div class="qwen-info-grid">
                <div class="qwen-info-feature">
                  <div class="qwen-info-dot" />
                  <span class="qwen-info-feature__text">{{ $t('qwen.overview.feature1') }}</span>
                </div>
                <div class="qwen-info-feature">
                  <div class="qwen-info-dot" />
                  <span class="qwen-info-feature__text">{{ $t('qwen.overview.feature2') }}</span>
                </div>
                <div class="qwen-info-feature">
                  <div class="qwen-info-dot" />
                  <span class="qwen-info-feature__text">{{ $t('qwen.overview.feature3') }}</span>
                </div>
                <div class="qwen-info-feature">
                  <div class="qwen-info-dot" />
                  <span class="qwen-info-feature__text">{{ $t('qwen.overview.feature4') }}</span>
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
.qwen-view {
  @apply relative min-h-full overflow-hidden p-6 lg:p-10;
}

.qwen-shell {
  @apply relative z-10 mx-auto max-w-7xl space-y-8;
}

.qwen-header {
  @apply mb-8 flex flex-col justify-between gap-6 md:flex-row md:items-center;
}

.qwen-brand {
  @apply flex items-center gap-5;
}

.qwen-brand__icon {
  @apply flex h-16 w-16 items-center justify-center rounded-2xl border shadow-lg backdrop-blur-md;

  color: rgb(147 51 234);
  background: rgb(168 85 247 / 10%);
  border-color: rgb(168 85 247 / 20%);
}

.qwen-brand__title {
  @apply text-4xl font-bold tracking-tight;

  font-family: MapleBright, 'Microsoft YaHei UI', system-ui, sans-serif;
  background: linear-gradient(90deg, rgb(168 85 247) 0%, rgb(236 72 153) 55%, rgb(245 158 11) 100%);
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.qwen-brand__subtitle {
  color: var(--stage-text-secondary);
  @apply mt-1 max-w-xl text-lg;
}

.qwen-home-button {
  @apply gap-2;
}

.qwen-tag-row {
  @apply flex flex-wrap gap-3;
}

.qwen-tag {
  @apply flex items-center gap-2 rounded-full px-3 py-1.5 text-xs font-bold uppercase;

  letter-spacing: 0.1em;
}

.qwen-tag--purple {
  color: rgb(168 85 247);
  background: rgb(168 85 247 / 10%);
  border: 1px solid rgb(168 85 247 / 20%);
}

.qwen-tag--pink {
  color: rgb(236 72 153);
  background: rgb(236 72 153 / 10%);
  border: 1px solid rgb(236 72 153 / 20%);
}

.qwen-tag--amber {
  color: rgb(245 158 11);
  background: rgb(245 158 11 / 10%);
  border: 1px solid rgb(245 158 11 / 20%);
}

.qwen-tag--red {
  color: rgb(239 68 68);
  background: rgb(239 68 68 / 10%);
  border: 1px solid rgb(239 68 68 / 20%);
}

.qwen-section-heading {
  @apply mb-6 flex items-center gap-3;
}

.qwen-section-heading__icon {
  color: rgb(168 85 247);
}

.qwen-section-heading__title {
  color: var(--stage-text-muted);
  @apply text-lg font-bold uppercase;

  letter-spacing: 0.2em;
}

.qwen-section-heading__rule {
  @apply h-px flex-1 bg-border-subtle;
}

.qwen-module-grid {
  @apply grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3;
}

.qwen-module-link {
  @apply block h-full;
}

.qwen-module-link--wide {
  @apply md:col-span-2 lg:col-span-1;
}

.qwen-module-card {
  @apply flex h-full items-start gap-4 p-6;
}

.qwen-module-card__icon {
  @apply rounded-xl p-3 transition-transform duration-300;
}

.qwen-module-link:hover .qwen-module-card__icon {
  transform: scale(1.1);
}

.qwen-module-card__icon--purple {
  color: rgb(168 85 247);
  background: rgb(168 85 247 / 10%);
}

.qwen-module-card__icon--pink {
  color: rgb(236 72 153);
  background: rgb(236 72 153 / 10%);
}

.qwen-module-card__icon--amber {
  color: rgb(245 158 11);
  background: rgb(245 158 11 / 10%);
}

.qwen-module-card__icon--red {
  color: rgb(239 68 68);
  background: rgb(239 68 68 / 10%);
}

.qwen-module-card__title {
  color: var(--stage-text-primary);
  @apply mb-1 text-lg font-bold transition-colors;
}

.qwen-module-link:hover .qwen-module-card__title--purple {
  color: rgb(168 85 247);
}

.qwen-module-link:hover .qwen-module-card__title--pink {
  color: rgb(236 72 153);
}

.qwen-module-link:hover .qwen-module-card__title--amber {
  color: rgb(245 158 11);
}

.qwen-module-link:hover .qwen-module-card__title--red {
  color: rgb(239 68 68);
}

.qwen-module-card__desc {
  color: var(--stage-text-secondary);
  @apply mb-3 text-sm leading-relaxed;
}

.qwen-module-card__badge {
  @apply inline-flex rounded border px-2 py-1 font-bold uppercase;

  font-size: 10px;
  letter-spacing: 0.1em;
}

.qwen-module-card__badge--purple {
  color: rgb(168 85 247);
  background: rgb(168 85 247 / 10%);
  border-color: rgb(168 85 247 / 20%);
}

.qwen-module-card__badge--pink {
  color: rgb(236 72 153);
  background: rgb(236 72 153 / 10%);
  border-color: rgb(236 72 153 / 20%);
}

.qwen-module-card__badge--amber {
  color: rgb(245 158 11);
  background: rgb(245 158 11 / 10%);
  border-color: rgb(245 158 11 / 20%);
}

.qwen-module-card__badge--red {
  color: rgb(239 68 68);
  background: rgb(239 68 68 / 10%);
  border-color: rgb(239 68 68 / 20%);
}

.qwen-info-card {
  @apply p-6;
}

.qwen-info-card__content {
  @apply flex items-start gap-4;
}

.qwen-info-card__icon {
  @apply shrink-0 rounded-xl p-3;

  color: rgb(168 85 247);
  background: rgb(168 85 247 / 10%);
}

.qwen-info-card__body {
  @apply space-y-4;
}

.qwen-info-card__title {
  color: var(--stage-text-primary);
  @apply text-lg font-bold;
}

.qwen-info-grid {
  @apply grid grid-cols-1 gap-4 md:grid-cols-2;
}

.qwen-info-feature {
  background: var(--stage-surface-soft);
  border: 1px solid var(--stage-border-soft);
  @apply flex items-center gap-3 rounded-lg p-3;
}

.qwen-info-dot {
  @apply h-2 w-2 rounded-full;

  background: rgb(168 85 247);
  box-shadow: 0 0 8px rgb(168 85 247 / 50%);
}

.qwen-info-feature__text {
  color: var(--stage-text-secondary);
  @apply text-sm;
}
</style>
