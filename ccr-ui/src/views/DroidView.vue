<template>
  <div class="min-h-full relative overflow-hidden">
    <AnimatedBackground
      contained
      variant="orbs"
    />

    <div class="relative z-10 mx-auto max-w-7xl p-6">
      <div class="mb-8 space-y-6">
        <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div class="flex items-start gap-4">
            <div class="flex h-16 w-16 shrink-0 items-center justify-center rounded-3xl border border-accent-success/20 bg-accent-success/10 text-accent-success shadow-sm">
              <SIcon
                name="Factory"
                size="w-8 h-8"
              />
            </div>
            <div>
              <h1 class="mb-2 text-4xl font-bold tracking-tight text-text-primary md:text-5xl">
                {{ $t('droid.overview.title') }}
              </h1>
              <p class="max-w-2xl text-lg text-text-secondary">
                {{ $t('droid.overview.description') }}
              </p>
            </div>
          </div>
          <RouterLink
            to="/"
            class="inline-flex min-h-[44px] items-center gap-2 self-start rounded-2xl border border-border-default/60 bg-bg-surface/80 px-5 py-3 text-text-secondary transition-colors hover:border-accent-success/30 hover:bg-bg-elevated/70 hover:text-text-primary"
          >
            <SIcon
              name="Home"
              size="w-5 h-5"
            />
            <span class="font-medium">{{ $t('droid.overview.backToHome') }}</span>
          </RouterLink>
        </div>

        <div class="flex flex-wrap gap-3">
          <span class="inline-flex min-h-[40px] items-center rounded-full border border-accent-success/20 bg-accent-success/10 px-4 py-2 text-sm font-medium text-accent-success">
            {{ $t('droid.overview.tags.multiProvider') }}
          </span>
          <span class="inline-flex min-h-[40px] items-center rounded-full border border-accent-info/20 bg-accent-info/10 px-4 py-2 text-sm font-medium text-accent-info">
            {{ $t('droid.overview.tags.customModels') }}
          </span>
          <span class="inline-flex min-h-[40px] items-center rounded-full border border-accent-primary/20 bg-accent-primary/10 px-4 py-2 text-sm font-medium text-accent-primary">
            {{ $t('droid.overview.tags.mcp') }}
          </span>
          <span class="inline-flex min-h-[40px] items-center rounded-full border border-accent-warning/20 bg-accent-warning/10 px-4 py-2 text-sm font-medium text-accent-warning">
            {{ $t('droid.overview.tags.coding') }}
          </span>
        </div>
      </div>

      <section class="mb-8">
        <h2 class="mb-6 flex items-center gap-3 text-2xl font-bold text-text-primary">
          <SIcon
            name="Boxes"
            size="w-7 h-7"
            class="text-accent-success"
          />
          {{ $t('droid.overview.features') }}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <RouterLink
            to="/droid/mcp"
            class="group block"
          >
            <div class="h-full rounded-3xl border border-border-default/50 bg-bg-surface/75 p-6 shadow-sm transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-accent-success/35 hover:shadow-lg hover:shadow-accent-success/5">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl border border-accent-success/15 bg-accent-success/10 p-3 text-accent-success">
                  <SIcon
                    name="Server"
                    size="w-7 h-7"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    {{ $t('droid.mcp.title') }}
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    {{ $t('droid.mcp.description') }}
                  </p>
                  <span class="inline-flex items-center rounded-full border border-accent-success/20 bg-accent-success/10 px-3 py-1.5 text-xs font-bold text-accent-success">
                    {{ $t('droid.mcp.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/agents"
            class="group block"
          >
            <div class="h-full rounded-3xl border border-border-default/50 bg-bg-surface/75 p-6 shadow-sm transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-accent-info/35 hover:shadow-lg hover:shadow-accent-info/5">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl border border-accent-info/15 bg-accent-info/10 p-3 text-accent-info">
                  <SIcon
                    name="Bot"
                    size="w-7 h-7"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    {{ $t('droid.agents.title') }}
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    {{ $t('droid.agents.description') }}
                  </p>
                  <span class="inline-flex items-center rounded-full border border-accent-info/20 bg-accent-info/10 px-3 py-1.5 text-xs font-bold text-accent-info">
                    {{ $t('droid.agents.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/plugins"
            class="group block"
          >
            <div class="h-full rounded-3xl border border-border-default/50 bg-bg-surface/75 p-6 shadow-sm transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-accent-primary/35 hover:shadow-lg hover:shadow-accent-primary/5">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl border border-accent-primary/15 bg-accent-primary/10 p-3 text-accent-primary">
                  <SIcon
                    name="Puzzle"
                    size="w-7 h-7"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    {{ $t('droid.plugins.title') }}
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    {{ $t('droid.plugins.description') }}
                  </p>
                  <span class="inline-flex items-center rounded-full border border-accent-primary/20 bg-accent-primary/10 px-3 py-1.5 text-xs font-bold text-accent-primary">
                    {{ $t('droid.plugins.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/slash-commands"
            class="group block"
          >
            <div class="h-full rounded-3xl border border-border-default/50 bg-bg-surface/75 p-6 shadow-sm transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-accent-warning/35 hover:shadow-lg hover:shadow-accent-warning/5">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl border border-accent-warning/15 bg-accent-warning/10 p-3 text-accent-warning">
                  <SIcon
                    name="Command"
                    size="w-7 h-7"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    {{ $t('droid.slashCommands.title') }}
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    {{ $t('droid.slashCommands.description') }}
                  </p>
                  <span class="inline-flex items-center rounded-full border border-accent-warning/20 bg-accent-warning/10 px-3 py-1.5 text-xs font-bold text-accent-warning">
                    {{ $t('droid.slashCommands.badge') }}
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/models"
            class="group block"
          >
            <div class="h-full rounded-3xl border border-border-default/50 bg-bg-surface/75 p-6 shadow-sm transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-accent-warning/35 hover:shadow-lg hover:shadow-accent-warning/5">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl border border-accent-warning/15 bg-accent-warning/10 p-3 text-accent-warning">
                  <SIcon
                    name="Cpu"
                    size="w-7 h-7"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    Custom Models
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    管理自定义 AI 模型配置
                  </p>
                  <span class="inline-flex items-center rounded-full border border-accent-warning/20 bg-accent-warning/10 px-3 py-1.5 text-xs font-bold text-accent-warning">
                    多提供商支持
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/profiles"
            class="group block"
          >
            <div class="h-full rounded-3xl border border-border-default/50 bg-bg-surface/75 p-6 shadow-sm transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-accent-info/35 hover:shadow-lg hover:shadow-accent-info/5">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl border border-accent-info/15 bg-accent-info/10 p-3 text-accent-info">
                  <SIcon
                    name="Settings"
                    size="w-7 h-7"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    Profiles
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    管理配置文件和环境设置
                  </p>
                  <span class="inline-flex items-center rounded-full border border-accent-info/20 bg-accent-info/10 px-3 py-1.5 text-xs font-bold text-accent-info">
                    快速切换
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/droid/droids"
            class="group block"
          >
            <div class="h-full rounded-3xl border border-border-default/50 bg-bg-surface/75 p-6 shadow-sm transition-[border-color,transform,box-shadow] duration-200 hover:-translate-y-0.5 hover:border-accent-primary/35 hover:shadow-lg hover:shadow-accent-primary/5">
              <div class="flex items-start gap-4">
                <div class="rounded-2xl border border-accent-primary/15 bg-accent-primary/10 p-3 text-accent-primary">
                  <SIcon
                    name="Bot"
                    size="w-7 h-7"
                  />
                </div>
                <div class="flex-1">
                  <h3 class="mb-2 text-xl font-bold text-text-primary">
                    Droids (Subagents)
                  </h3>
                  <p class="mb-3 text-sm text-text-secondary">
                    管理自定义 AI Subagents
                  </p>
                  <span class="inline-flex items-center rounded-full border border-accent-primary/20 bg-accent-primary/10 px-3 py-1.5 text-xs font-bold text-accent-primary">
                    Markdown 配置
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>
        </div>
      </section>

      <section class="rounded-3xl border border-border-default/50 bg-bg-surface/80 p-6 shadow-sm">
        <div class="flex items-start gap-4">
          <div class="rounded-2xl border border-accent-success/15 bg-accent-success/10 p-3 text-accent-success">
            <SIcon
              name="Info"
              size="w-6 h-6"
            />
          </div>
          <div class="flex-1">
            <h3 class="mb-2 text-lg font-bold text-text-primary">
              {{ $t('droid.overview.featureTitle') }}
            </h3>
            <ul class="space-y-2 text-sm text-text-secondary">
              <li class="flex items-start gap-2">
                <span class="text-lg leading-none text-accent-success">•</span>
                <span>{{ $t('droid.overview.feature1') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg leading-none text-accent-success">•</span>
                <span>{{ $t('droid.overview.feature2') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg leading-none text-accent-success">•</span>
                <span>{{ $t('droid.overview.feature3') }}</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg leading-none text-accent-success">•</span>
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
