<template>
  <div class="skills-migration-view">
    <div class="skills-migration-view__shell">
      <section class="skills-migration-view__hero">
        <div class="skills-migration-view__badge">
          Skills migration
        </div>
        <h1 class="skills-migration-view__title">
          Skills 已从 CCR UI 下线
        </h1>
        <p class="skills-migration-view__lead">
          CCR UI 现在只保留 CLI 配置管理主线，不再内置 skills 安装、市场和来源管理。
        </p>
        <p class="skills-migration-view__copy">
          之后请改用独立应用
          <a
            href="https://github.com/iamzhihuix/skills-manage"
            target="_blank"
            rel="noreferrer"
            class="skills-migration-view__link"
          >skills-manage</a>
          处理 skills。
        </p>

        <div class="skills-migration-view__actions">
          <a
            href="https://github.com/iamzhihuix/skills-manage"
            target="_blank"
            rel="noreferrer"
            class="skills-migration-view__primary"
          >
            打开 skills-manage
          </a>
          <RouterLink
            to="/configs"
            class="skills-migration-view__secondary"
          >
            返回配置管理
          </RouterLink>
        </div>
      </section>

      <section class="skills-migration-view__grid">
        <article class="skills-migration-view__card">
          <h2>为什么下线</h2>
          <p>
            这一层功能和 CCR UI 的核心定位不一致。继续保留会让路由、状态和桌面后端持续膨胀。
          </p>
        </article>

        <article class="skills-migration-view__card">
          <h2>现在去哪里</h2>
          <p>
            skills 的浏览、安装和管理统一改到 skills-manage。CCR UI 只负责 CLI 配置、运行态和数据面。
          </p>
        </article>

        <article class="skills-migration-view__card">
          <h2>怎么开始</h2>
          <p>
            截至 2026-04-23，上游最新 release 是 v0.8.0，发布日期 2026-04-20。Windows 先按仓库说明源码运行。
          </p>
        </article>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
</script>

<style scoped>
.skills-migration-view {
  @apply min-h-full px-4 py-6 sm:px-6;
}

.skills-migration-view__shell {
  @apply mx-auto flex max-w-5xl flex-col gap-6;
}

.skills-migration-view__hero {
  @apply rounded-[32px] border border-border-default/60 bg-bg-elevated/70 p-8;
}

.skills-migration-view__badge {
  @apply inline-flex items-center rounded-full border border-accent-primary/20 bg-accent-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-accent-primary;
}

.skills-migration-view__title {
  @apply mt-4 text-3xl font-semibold tracking-[-0.04em] text-text-primary;
}

.skills-migration-view__lead {
  @apply mt-4 max-w-3xl text-base leading-7 text-text-primary;
}

.skills-migration-view__copy {
  @apply mt-3 max-w-3xl text-sm leading-7 text-text-secondary;
}

.skills-migration-view__link {
  @apply font-semibold text-accent-primary hover:underline;
}

.skills-migration-view__actions {
  @apply mt-6 flex flex-wrap gap-3;
}

.skills-migration-view__primary,
.skills-migration-view__secondary {
  @apply inline-flex min-h-[44px] items-center justify-center rounded-2xl px-5 py-3 text-sm font-semibold transition-colors;
}

.skills-migration-view__primary {
  @apply bg-accent-primary text-white hover:bg-accent-primary/90;
}

.skills-migration-view__secondary {
  @apply border border-border-default/60 bg-bg-base/50 text-text-primary hover:bg-bg-surface/70;
}

.skills-migration-view__grid {
  @apply grid gap-4 md:grid-cols-3;
}

.skills-migration-view__card {
  @apply rounded-[28px] border border-border-default/50 bg-bg-surface/70 p-5;
}

.skills-migration-view__card h2 {
  @apply text-sm font-semibold text-text-primary;
}

.skills-migration-view__card p {
  @apply mt-3 text-sm leading-7 text-text-secondary;
}
</style>
