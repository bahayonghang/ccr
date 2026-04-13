<template>
  <OpenCodePageShell
    title="Skills hub"
    description="复用现有 Skills Hub，但默认锁定 OpenCode 平台，并把官方 skill 搜索路径、兼容目录和权限语义直接放在页头。"
    icon="BookOpen"
    badge="OpenCode"
    tone="lime"
  >
    <template #meta>
      <span class="opencode-skills__chip">
        Search path precedence
      </span>
      <span class="opencode-skills__chip">
        Compatibility-aware
      </span>
      <span class="opencode-skills__chip">
        Permission prompts from skill metadata
      </span>
    </template>

    <section class="grid gap-4 xl:grid-cols-[minmax(0,1.45fr)_360px]">
      <Card
        variant="glass"
        class="opencode-skills__brief"
      >
        <div class="opencode-skills__brief-grid">
          <article
            v-for="card in primerCards"
            :key="card.title"
            class="opencode-skills__info-card"
          >
            <div class="opencode-skills__info-title">
              {{ card.title }}
            </div>
            <p class="opencode-skills__info-copy">
              {{ card.copy }}
            </p>
          </article>
        </div>
      </Card>

      <Card
        variant="glass"
        class="opencode-skills__paths"
      >
        <div class="opencode-skills__section-label">
          Resolved locations
        </div>
        <div class="opencode-skills__path-list">
          <article
            v-for="entry in resolvedLocations"
            :key="entry.label"
            class="opencode-skills__path-card"
          >
            <div class="opencode-skills__path-header">
              <strong>{{ entry.label }}</strong>
              <span>{{ entry.scope }}</span>
            </div>
            <code>{{ entry.path }}</code>
          </article>
        </div>
      </Card>
    </section>

    <SkillsView
      forced-platform="opencode"
      route-base-path="/opencode/skills"
      page-title="OpenCode Skills"
      page-description="默认聚焦 OpenCode 平台的 Skills 库存、探索与来源。"
      page-badge="OpenCode"
      hide-page-header
    />
  </OpenCodePageShell>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { listOpenCodeSkillLocations } from '@/api'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import Card from '@/components/ui/Card.vue'
import SkillsView from '@/views/skills/SkillsView.vue'
import type { OpenCodeSkillLocation } from '@/types/opencode'
import { isTauriRuntime } from '@/utils/tauriRuntime'

const discoveredLocations = ref<OpenCodeSkillLocation[]>([])

const primerCards = [
  {
    title: 'Official behavior',
    copy: 'OpenCode 会先读取自己的 skill 目录，再兼容常见的 Claude / Agents 目录，让同一套技能资产可以跨 CLI 复用。',
  },
  {
    title: 'Permission model',
    copy: 'Skill frontmatter 和技能内容会影响工具调用、命令模板与执行提示；高风险动作仍应通过 OpenCode 的权限/确认层处理。',
  },
  {
    title: 'Why this wrapper exists',
    copy: '这里不复制 Skills Hub，而是把通用库存、探索和来源管理直接嵌到 OpenCode 语境里，减少维护面并保持平台一致性。',
  },
]

const fallbackLocations = [
  { label: 'OpenCode local', scope: 'project', path: '.opencode/skills' },
  { label: 'OpenCode global', scope: 'user', path: '~/.config/opencode/skills' },
  { label: 'Claude compatibility', scope: 'user', path: '~/.claude/skills' },
  { label: 'Agents compatibility', scope: 'user', path: '~/.agents/skills' },
]

const resolvedLocations = computed(() => {
  if (discoveredLocations.value.length === 0) {
    return fallbackLocations
  }

  return discoveredLocations.value.map((location) => ({
    label: location.kind
      .split('-')
      .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
      .join(' '),
    scope: location.scope,
    path: location.path,
  }))
})

onMounted(async () => {
  if (!isTauriRuntime()) return

  try {
    discoveredLocations.value = await listOpenCodeSkillLocations()
  } catch {
    discoveredLocations.value = []
  }
})
</script>

<style scoped>
.opencode-skills__chip {
  @apply inline-flex items-center rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.16em] text-text-secondary;
}

.opencode-skills__brief,
.opencode-skills__paths {
  @apply p-5;
}

.opencode-skills__brief-grid {
  @apply grid gap-3 md:grid-cols-3;
}

.opencode-skills__info-card {
  @apply rounded-3xl border border-white/10 bg-white/5 p-4;
}

.opencode-skills__info-title {
  @apply text-xs font-semibold uppercase tracking-[0.16em] text-lime-200;
}

.opencode-skills__info-copy {
  @apply mt-3 text-sm leading-7 text-text-secondary;
}

.opencode-skills__section-label {
  @apply text-xs font-semibold uppercase tracking-[0.16em] text-text-muted;
}

.opencode-skills__path-list {
  @apply mt-4 flex flex-col gap-3;
}

.opencode-skills__path-card {
  @apply rounded-3xl border border-white/10 bg-white/5 p-4;
}

.opencode-skills__path-header {
  @apply flex items-center justify-between gap-3 text-sm text-text-primary;
}

.opencode-skills__path-header span {
  @apply rounded-full border border-lime-400/20 bg-lime-400/10 px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.16em] text-lime-200;
}

.opencode-skills__path-card code {
  @apply mt-3 block break-all rounded-2xl bg-black/20 px-3 py-2 text-xs text-text-secondary;
}
</style>
