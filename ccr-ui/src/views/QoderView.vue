<template>
  <div class="qoder-view stage-page">
    <!-- 🎨 彩色背景装饰 -->
    <!-- Standard Animated Background -->
    <AnimatedBackground
      contained
      variant="mesh"
    />

    <div class="qoder-shell">
      <!-- Hero 区域 -->
      <div class="qoder-hero">
        <div class="qoder-hero__row">
          <div class="qoder-brand">
            <div class="glass-card qoder-brand__icon">
              <SIcon
                name="Workflow"
                size="w-10 h-10"
                class="text-accent-success"
              />
            </div>
            <div>
              <h1 class="qoder-brand__title">
                {{ $t('qoder.overview.title') }}
              </h1>
              <p class="qoder-brand__subtitle">
                {{ $t('qoder.overview.description') }}
              </p>
            </div>
          </div>
          <RouterLink
            to="/"
            class="glass-card qoder-home-link"
          >
            <SIcon
              name="Home"
              size="w-5 h-5"
              class="text-text-muted"
            />
            <span class="font-medium">{{ $t('qoder.overview.backToHome') }}</span>
          </RouterLink>
        </div>

        <!-- 特性标签 -->
        <div class="qoder-tag-row">
          <span class="glass-card qoder-tag qoder-tag--success">
            {{ $t('qoder.overview.tags.workflow') }}
          </span>
          <span class="glass-card qoder-tag qoder-tag--info">
            {{ $t('qoder.overview.tags.orchestration') }}
          </span>
          <span class="glass-card qoder-tag qoder-tag--primary">
            {{ $t('qoder.overview.tags.mcp') }}
          </span>
          <span class="glass-card qoder-tag qoder-tag--secondary">
            {{ $t('qoder.overview.tags.agent') }}
          </span>
        </div>
      </div>

      <!-- 功能模块网格 -->
      <div class="qoder-features">
        <h2 class="qoder-section-title">
          <SIcon
            name="Boxes"
            size="w-7 h-7"
            class="text-accent-success"
          />
          {{ $t('qoder.overview.features') }}
        </h2>
        <div class="qoder-feature-grid">
          <RouterLink
            to="/qoder/mcp"
            class="qoder-feature-link"
          >
            <div class="glass-card qoder-feature-card">
              <div class="qoder-feature-card__content">
                <div class="qoder-feature-card__icon qoder-feature-card__icon--success">
                  <SIcon
                    name="Server"
                    size="w-7 h-7"
                    class="text-accent-success"
                  />
                </div>
                <div class="qoder-feature-card__body">
                  <h3 class="qoder-feature-card__title">
                    {{ $t('qoder.mcp.title') }}
                  </h3>
                  <p class="qoder-feature-card__desc">
                    {{ $t('qoder.mcp.description') }}
                  </p>
                  <span class="qoder-feature-card__badge qoder-feature-card__badge--success">
                    {{ $t('qoder.mcp.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/qoder/subagents"
            class="qoder-feature-link"
          >
            <div class="glass-card qoder-feature-card">
              <div class="qoder-feature-card__content">
                <div class="qoder-feature-card__icon qoder-feature-card__icon--info">
                  <SIcon
                    name="Bot"
                    size="w-7 h-7"
                    class="text-accent-info"
                  />
                </div>
                <div class="qoder-feature-card__body">
                  <h3 class="qoder-feature-card__title">
                    {{ $t('qoder.agents.title') }}
                  </h3>
                  <p class="qoder-feature-card__desc">
                    {{ $t('qoder.agents.description') }}
                  </p>
                  <span class="qoder-feature-card__badge qoder-feature-card__badge--info">
                    {{ $t('qoder.agents.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/qoder/hooks"
            class="qoder-feature-link"
          >
            <div class="glass-card qoder-feature-card">
              <div class="qoder-feature-card__content">
                <div class="qoder-feature-card__icon qoder-feature-card__icon--primary">
                  <SIcon
                    name="Puzzle"
                    size="w-7 h-7"
                    class="text-accent-primary"
                  />
                </div>
                <div class="qoder-feature-card__body">
                  <h3 class="qoder-feature-card__title">
                    {{ $t('qoder.plugins.title') }}
                  </h3>
                  <p class="qoder-feature-card__desc">
                    {{ $t('qoder.plugins.description') }}
                  </p>
                  <span class="qoder-feature-card__badge qoder-feature-card__badge--primary">
                    {{ $t('qoder.plugins.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/qoder/commands"
            class="qoder-feature-link"
          >
            <div class="glass-card qoder-feature-card">
              <div class="qoder-feature-card__content">
                <div class="qoder-feature-card__icon qoder-feature-card__icon--secondary">
                  <SIcon
                    name="Command"
                    size="w-7 h-7"
                    class="text-accent-secondary"
                  />
                </div>
                <div class="qoder-feature-card__body">
                  <h3 class="qoder-feature-card__title">
                    {{ $t('qoder.slashCommands.title') }}
                  </h3>
                  <p class="qoder-feature-card__desc">
                    {{ $t('qoder.slashCommands.description') }}
                  </p>
                  <span class="qoder-feature-card__badge qoder-feature-card__badge--secondary">
                    {{ $t('qoder.slashCommands.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>
        </div>
      </div>

      <!-- 提示卡片 -->
      <div class="glass-card qoder-info-card">
        <div class="qoder-info-card__content">
          <div class="qoder-info-card__icon">
            <SIcon
              name="Info"
              size="w-6 h-6"
            />
          </div>
          <div class="qoder-info-card__body">
            <h3 class="qoder-info-card__title">
              {{ $t('qoder.overview.featureTitle') }}
            </h3>
            <ul class="qoder-info-list">
              <li class="qoder-info-list__item">
                <span class="qoder-info-list__bullet">•</span>
                <span>{{ $t('qoder.overview.feature1') }}</span>
              </li>
              <li class="qoder-info-list__item">
                <span class="qoder-info-list__bullet">•</span>
                <span>{{ $t('qoder.overview.feature2') }}</span>
              </li>
              <li class="qoder-info-list__item">
                <span class="qoder-info-list__bullet">•</span>
                <span>{{ $t('qoder.overview.feature3') }}</span>
              </li>
              <li class="qoder-info-list__item">
                <span class="qoder-info-list__bullet">•</span>
                <span>{{ $t('qoder.overview.feature4') }}</span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { RouterLink } from 'vue-router'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
</script>

<style scoped>
.qoder-view {
  @apply relative min-h-full overflow-hidden;
}

.qoder-shell {
  @apply relative z-10 mx-auto max-w-7xl p-6;
}

.qoder-hero {
  @apply mb-8;
}

.qoder-hero__row {
  @apply mb-6 flex items-center justify-between gap-6;
}

.qoder-brand {
  @apply flex items-center gap-4;
}

.qoder-brand__icon {
  @apply rounded-3xl p-4;
}

.qoder-brand__title {
  @apply mb-2 text-4xl font-bold text-text-primary md:text-5xl;
}

.qoder-brand__subtitle {
  @apply text-lg text-text-secondary;
}

.qoder-home-link {
  @apply flex items-center gap-2 px-5 py-3 text-text-secondary transition-colors duration-300;

  background: var(--stage-surface-medium);
  border: 1px solid var(--stage-border-soft);
}

.qoder-home-link:hover {
  @apply text-text-primary;

  background: rgb(var(--color-bg-overlay-rgb) / 70%);
}

.qoder-tag-row {
  @apply mb-6 flex flex-wrap gap-3;
}

.qoder-tag {
  @apply rounded-full px-4 py-2 text-sm font-medium;
}

.qoder-tag--success {
  @apply bg-accent-success/10 text-accent-success;
}

.qoder-tag--info {
  @apply bg-accent-info/10 text-accent-info;
}

.qoder-tag--primary {
  @apply bg-accent-primary/10 text-accent-primary;
}

.qoder-tag--secondary {
  @apply bg-accent-secondary/10 text-accent-secondary;
}

.qoder-features {
  @apply mb-8;
}

.qoder-section-title {
  @apply mb-6 flex items-center gap-3 text-2xl font-bold text-text-primary;
}

.qoder-feature-grid {
  @apply grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3;
}

.qoder-feature-link {
  @apply block;
}

.qoder-feature-card {
  @apply h-full p-6 duration-300;

  background: var(--stage-surface-medium);
  border: 1px solid var(--stage-border-soft);
  transition-property: transform, box-shadow, border-color;
}

.qoder-feature-link:hover .qoder-feature-card {
  transform: translateY(-0.25rem);
  border-color: var(--stage-border-medium);
}

.qoder-feature-card__content {
  @apply flex items-start gap-4;
}

.qoder-feature-card__icon {
  @apply rounded-2xl p-3;
}

.qoder-feature-card__icon--success {
  @apply bg-accent-success/15;
}

.qoder-feature-card__icon--info {
  @apply bg-accent-info/15;
}

.qoder-feature-card__icon--primary {
  @apply bg-accent-primary/15;
}

.qoder-feature-card__icon--secondary {
  @apply bg-accent-secondary/15;
}

.qoder-feature-card__body {
  @apply flex-1;
}

.qoder-feature-card__title {
  @apply mb-2 text-xl font-bold text-text-primary;
}

.qoder-feature-card__desc {
  @apply mb-3 text-sm text-text-secondary;
}

.qoder-feature-card__badge {
  @apply inline-flex items-center rounded-full px-3 py-1.5 text-xs font-bold;
}

.qoder-feature-card__badge--success {
  @apply bg-accent-success/15 text-accent-success;
}

.qoder-feature-card__badge--info {
  @apply bg-accent-info/15 text-accent-info;
}

.qoder-feature-card__badge--primary {
  @apply bg-accent-primary/15 text-accent-primary;
}

.qoder-feature-card__badge--secondary {
  @apply bg-accent-secondary/15 text-accent-secondary;
}

.qoder-info-card {
  background: var(--stage-surface-strong);
  border: 1px solid var(--stage-border-medium);

  @apply p-6;
}

.qoder-info-card__content {
  @apply flex items-start gap-4;
}

.qoder-info-card__icon {
  @apply rounded-2xl p-3;

  color: #10b981;
  background: rgb(16 185 129 / 15%);
}

.qoder-info-card__body {
  @apply flex-1;
}

.qoder-info-card__title {
  @apply mb-2 text-lg font-bold text-text-primary;
}

.qoder-info-list {
  @apply space-y-2 text-sm text-text-secondary;
}

.qoder-info-list__item {
  @apply flex items-start gap-2;
}

.qoder-info-list__bullet {
  @apply text-lg;
}
</style>
