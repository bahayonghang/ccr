<template>
  <div class="droid-view stage-page">
    <AnimatedBackground
      contained
      variant="orbs"
    />

    <div class="droid-shell">
      <div class="droid-hero">
        <div class="droid-hero__row">
          <div class="droid-brand">
            <div class="droid-brand__icon">
              <SIcon
                name="Factory"
                size="w-8 h-8"
              />
            </div>
            <div>
              <h1 class="droid-brand__title">
                {{ $t('droid.overview.title') }}
              </h1>
              <p class="droid-brand__subtitle">
                {{ $t('droid.overview.description') }}
              </p>
            </div>
          </div>
          <RouterLink
            to="/"
            class="droid-home-link"
          >
            <SIcon
              name="Home"
              size="w-5 h-5"
            />
            <span class="font-medium">{{ $t('droid.overview.backToHome') }}</span>
          </RouterLink>
        </div>

        <div class="droid-tag-row">
          <span class="droid-tag droid-tag--success">
            {{ $t('droid.overview.tags.multiProvider') }}
          </span>
          <span class="droid-tag droid-tag--info">
            {{ $t('droid.overview.tags.customModels') }}
          </span>
          <span class="droid-tag droid-tag--primary">
            {{ $t('droid.overview.tags.mcp') }}
          </span>
          <span class="droid-tag droid-tag--warning">
            {{ $t('droid.overview.tags.coding') }}
          </span>
        </div>
      </div>

      <section class="droid-features">
        <h2 class="droid-section-title">
          <SIcon
            name="Boxes"
            size="w-7 h-7"
            class="text-accent-success"
          />
          {{ $t('droid.overview.features') }}
        </h2>
        <div class="droid-feature-grid">
          <RouterLink
            to="/droid/mcp"
            class="droid-feature-link"
          >
            <div class="droid-feature-card droid-feature-card--success">
              <div class="droid-feature-card__content">
                <div class="droid-feature-card__icon droid-feature-card__icon--success">
                  <SIcon
                    name="Server"
                    size="w-7 h-7"
                  />
                </div>
                <div class="droid-feature-card__body">
                  <h3 class="droid-feature-card__title">
                    {{ $t('droid.mcp.title') }}
                  </h3>
                  <p class="droid-feature-card__desc">
                    {{ $t('droid.mcp.description') }}
                  </p>
                  <span class="droid-feature-card__badge droid-feature-card__badge--success">
                    {{ $t('droid.mcp.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/agents"
            class="droid-feature-link"
          >
            <div class="droid-feature-card droid-feature-card--info">
              <div class="droid-feature-card__content">
                <div class="droid-feature-card__icon droid-feature-card__icon--info">
                  <SIcon
                    name="Bot"
                    size="w-7 h-7"
                  />
                </div>
                <div class="droid-feature-card__body">
                  <h3 class="droid-feature-card__title">
                    {{ $t('droid.agents.title') }}
                  </h3>
                  <p class="droid-feature-card__desc">
                    {{ $t('droid.agents.description') }}
                  </p>
                  <span class="droid-feature-card__badge droid-feature-card__badge--info">
                    {{ $t('droid.agents.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/plugins"
            class="droid-feature-link"
          >
            <div class="droid-feature-card droid-feature-card--primary">
              <div class="droid-feature-card__content">
                <div class="droid-feature-card__icon droid-feature-card__icon--primary">
                  <SIcon
                    name="Puzzle"
                    size="w-7 h-7"
                  />
                </div>
                <div class="droid-feature-card__body">
                  <h3 class="droid-feature-card__title">
                    {{ $t('droid.plugins.title') }}
                  </h3>
                  <p class="droid-feature-card__desc">
                    {{ $t('droid.plugins.description') }}
                  </p>
                  <span class="droid-feature-card__badge droid-feature-card__badge--primary">
                    {{ $t('droid.plugins.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/slash-commands"
            class="droid-feature-link"
          >
            <div class="droid-feature-card droid-feature-card--warning">
              <div class="droid-feature-card__content">
                <div class="droid-feature-card__icon droid-feature-card__icon--warning">
                  <SIcon
                    name="Command"
                    size="w-7 h-7"
                  />
                </div>
                <div class="droid-feature-card__body">
                  <h3 class="droid-feature-card__title">
                    {{ $t('droid.slashCommands.title') }}
                  </h3>
                  <p class="droid-feature-card__desc">
                    {{ $t('droid.slashCommands.description') }}
                  </p>
                  <span class="droid-feature-card__badge droid-feature-card__badge--warning">
                    {{ $t('droid.slashCommands.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/models"
            class="droid-feature-link"
          >
            <div class="droid-feature-card droid-feature-card--warning">
              <div class="droid-feature-card__content">
                <div class="droid-feature-card__icon droid-feature-card__icon--warning">
                  <SIcon
                    name="Cpu"
                    size="w-7 h-7"
                  />
                </div>
                <div class="droid-feature-card__body">
                  <h3 class="droid-feature-card__title">
                    Custom Models
                  </h3>
                  <p class="droid-feature-card__desc">
                    管理自定义 AI 模型配置
                  </p>
                  <span class="droid-feature-card__badge droid-feature-card__badge--warning">
                    多提供商支持
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/profiles"
            class="droid-feature-link"
          >
            <div class="droid-feature-card droid-feature-card--info">
              <div class="droid-feature-card__content">
                <div class="droid-feature-card__icon droid-feature-card__icon--info">
                  <SIcon
                    name="Settings"
                    size="w-7 h-7"
                  />
                </div>
                <div class="droid-feature-card__body">
                  <h3 class="droid-feature-card__title">
                    Profiles
                  </h3>
                  <p class="droid-feature-card__desc">
                    管理配置文件和环境设置
                  </p>
                  <span class="droid-feature-card__badge droid-feature-card__badge--info">
                    快速切换
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/droids"
            class="droid-feature-link"
          >
            <div class="droid-feature-card droid-feature-card--primary">
              <div class="droid-feature-card__content">
                <div class="droid-feature-card__icon droid-feature-card__icon--primary">
                  <SIcon
                    name="Bot"
                    size="w-7 h-7"
                  />
                </div>
                <div class="droid-feature-card__body">
                  <h3 class="droid-feature-card__title">
                    Droids (Subagents)
                  </h3>
                  <p class="droid-feature-card__desc">
                    管理自定义 AI Subagents
                  </p>
                  <span class="droid-feature-card__badge droid-feature-card__badge--primary">
                    Markdown 配置
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>
        </div>
      </section>

      <section class="droid-info-card">
        <div class="droid-info-card__content">
          <div class="droid-info-card__icon">
            <SIcon
              name="Info"
              size="w-6 h-6"
            />
          </div>
          <div class="droid-info-card__body">
            <h3 class="droid-info-card__title">
              {{ $t('droid.overview.featureTitle') }}
            </h3>
            <ul class="droid-info-list">
              <li class="droid-info-list__item">
                <span class="droid-info-list__bullet">•</span>
                <span>{{ $t('droid.overview.feature1') }}</span>
              </li>
              <li class="droid-info-list__item">
                <span class="droid-info-list__bullet">•</span>
                <span>{{ $t('droid.overview.feature2') }}</span>
              </li>
              <li class="droid-info-list__item">
                <span class="droid-info-list__bullet">•</span>
                <span>{{ $t('droid.overview.feature3') }}</span>
              </li>
              <li class="droid-info-list__item">
                <span class="droid-info-list__bullet">•</span>
                <span>{{ $t('droid.overview.feature4') }}</span>
              </li>
            </ul>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { RouterLink } from 'vue-router'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
</script>

<style scoped>
.droid-view {
  @apply relative min-h-full overflow-hidden;
}

.droid-shell {
  @apply relative z-10 mx-auto max-w-7xl p-6;
}

.droid-hero {
  @apply mb-8 space-y-6;
}

.droid-hero__row {
  @apply flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between;
}

.droid-brand {
  @apply flex items-start gap-4;
}

.droid-brand__icon {
  @apply flex h-16 w-16 shrink-0 items-center justify-center rounded-3xl border shadow-sm;

  color: var(--color-success);
  background: rgb(var(--color-success-rgb) / 10%);
  border-color: rgb(var(--color-success-rgb) / 20%);
}

.droid-brand__title {
  @apply mb-2 text-4xl font-bold tracking-tight text-text-primary md:text-5xl;
}

.droid-brand__subtitle {
  @apply max-w-2xl text-lg text-text-secondary;
}

.droid-home-link {
  @apply inline-flex min-h-[44px] items-center gap-2 self-start rounded-2xl border px-5 py-3 text-text-secondary transition-colors;

  border-color: var(--stage-border-medium);
  background: var(--stage-surface-medium);
}

.droid-home-link:hover {
  @apply text-text-primary;

  border-color: rgb(var(--color-success-rgb) / 30%);
  background: rgb(var(--color-bg-elevated-rgb) / 70%);
}

.droid-tag-row {
  @apply flex flex-wrap gap-3;
}

.droid-tag {
  @apply inline-flex min-h-[40px] items-center rounded-full border px-4 py-2 text-sm font-medium;
}

.droid-tag--success {
  @apply border-accent-success/20 bg-accent-success/10 text-accent-success;
}

.droid-tag--info {
  @apply border-accent-info/20 bg-accent-info/10 text-accent-info;
}

.droid-tag--primary {
  @apply border-accent-primary/20 bg-accent-primary/10 text-accent-primary;
}

.droid-tag--warning {
  @apply border-accent-warning/20 bg-accent-warning/10 text-accent-warning;
}

.droid-features {
  @apply mb-8;
}

.droid-section-title {
  @apply mb-6 flex items-center gap-3 text-2xl font-bold text-text-primary;
}

.droid-feature-grid {
  @apply grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3;
}

.droid-feature-link {
  @apply block;
}

.droid-feature-card {
  @apply h-full rounded-3xl border p-6 shadow-sm duration-200;

  border-color: var(--stage-border-medium);
  background: var(--stage-surface-medium);
  transition-property: border-color, transform, box-shadow;
}

.droid-feature-link:hover .droid-feature-card {
  transform: translateY(-2px);
}

.droid-feature-card--success:hover {
  border-color: rgb(var(--color-success-rgb) / 35%);
  box-shadow: 0 18px 36px rgb(var(--color-success-rgb) / 5%);
}

.droid-feature-card--info:hover {
  border-color: rgb(var(--color-info-rgb) / 35%);
  box-shadow: 0 18px 36px rgb(var(--color-info-rgb) / 5%);
}

.droid-feature-card--primary:hover {
  border-color: rgb(var(--color-accent-primary-rgb) / 35%);
  box-shadow: 0 18px 36px rgb(var(--color-accent-primary-rgb) / 5%);
}

.droid-feature-card--warning:hover {
  border-color: rgb(var(--color-warning-rgb) / 35%);
  box-shadow: 0 18px 36px rgb(var(--color-warning-rgb) / 5%);
}

.droid-feature-card__content {
  @apply flex items-start gap-4;
}

.droid-feature-card__icon {
  @apply rounded-2xl border p-3;
}

.droid-feature-card__icon--success {
  @apply border-accent-success/15 bg-accent-success/10 text-accent-success;
}

.droid-feature-card__icon--info {
  @apply border-accent-info/15 bg-accent-info/10 text-accent-info;
}

.droid-feature-card__icon--primary {
  @apply border-accent-primary/15 bg-accent-primary/10 text-accent-primary;
}

.droid-feature-card__icon--warning {
  @apply border-accent-warning/15 bg-accent-warning/10 text-accent-warning;
}

.droid-feature-card__body {
  @apply flex-1;
}

.droid-feature-card__title {
  @apply mb-2 text-xl font-bold text-text-primary;
}

.droid-feature-card__desc {
  @apply mb-3 text-sm text-text-secondary;
}

.droid-feature-card__badge {
  @apply inline-flex items-center rounded-full border px-3 py-1.5 text-xs font-bold;
}

.droid-feature-card__badge--success {
  @apply border-accent-success/20 bg-accent-success/10 text-accent-success;
}

.droid-feature-card__badge--info {
  @apply border-accent-info/20 bg-accent-info/10 text-accent-info;
}

.droid-feature-card__badge--primary {
  @apply border-accent-primary/20 bg-accent-primary/10 text-accent-primary;
}

.droid-feature-card__badge--warning {
  @apply border-accent-warning/20 bg-accent-warning/10 text-accent-warning;
}

.droid-info-card {
  @apply rounded-3xl border p-6 shadow-sm;

  border-color: var(--stage-border-medium);
  background: var(--stage-surface-strong);
}

.droid-info-card__content {
  @apply flex items-start gap-4;
}

.droid-info-card__icon {
  @apply rounded-2xl border p-3 text-accent-success;

  border-color: rgb(var(--color-success-rgb) / 15%);
  background: rgb(var(--color-success-rgb) / 10%);
}

.droid-info-card__body {
  @apply flex-1;
}

.droid-info-card__title {
  @apply mb-2 text-lg font-bold text-text-primary;
}

.droid-info-list {
  @apply space-y-2 text-sm text-text-secondary;
}

.droid-info-list__item {
  @apply flex items-start gap-2;
}

.droid-info-list__bullet {
  @apply text-lg leading-none text-accent-success;
}
</style>
