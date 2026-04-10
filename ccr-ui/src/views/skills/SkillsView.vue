<template>
  <div class="flex flex-col gap-5 px-4 py-4">
    <PageHeaderCard
      title="Skills Hub"
      description="统一管理 Skills 库存、探索、平台和来源。"
      badge="Workspace"
      icon="Package"
      tone="secondary"
    />
    <AsyncStatePanel
      v-if="runtimeUnavailable"
      state="runtime-unavailable"
      :title="runtimeCopy.title"
      :description="runtimeCopy.description"
      :action-label="runtimeCopy.actionLabel"
      action-icon="ArrowLeft"
      @action="$router.push('/')"
    />
    <template v-else>
      <div class="flex flex-wrap gap-2 rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-2">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="inline-flex items-center gap-2 rounded-2xl px-4 py-2.5 text-sm"
          :class="activeTab===tab.id?'border border-accent-primary/30 bg-accent-primary/10 text-text-primary':'text-text-secondary'"
          @click="setTab(tab.id)"
        >
          <SIcon
            :name="tab.icon"
            size="w-4 h-4"
          /><span>{{ tab.label }}</span><strong class="rounded-full bg-bg-base/70 px-2 py-0.5 text-xs">{{ tab.count }}</strong>
        </button>
      </div>

      <section
        v-if="activeTab==='library'"
        class="grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)]"
      >
        <aside class="flex flex-col gap-4">
          <div class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4">
            <div class="mb-3 flex items-center justify-between">
              <h2 class="text-xs uppercase tracking-[0.16em] text-text-muted">
                Filters
              </h2><button
                class="text-xs text-text-muted"
                @click="resetLibraryFilters"
              >
                Reset
              </button>
            </div>
            <div class="flex flex-col gap-3">
              <input
                v-model="librarySearch"
                aria-label="Search installed skills"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                placeholder="search"
              >
              <select
                v-model="filters.platform"
                aria-label="Filter skills by platform"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
              >
                <option value="all">
                  All platforms
                </option><option
                  v-for="platform in platforms"
                  :key="platform.id"
                  :value="platform.id"
                >
                  {{ platform.displayName }}
                </option>
              </select>
              <select
                v-model="filters.origin"
                aria-label="Filter skills by origin"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
              >
                <option value="all">
                  All origins
                </option><option
                  v-for="origin in originOptions"
                  :key="origin"
                  :value="origin"
                >
                  {{ origin }}
                </option>
              </select>
            </div>
          </div>
          <div class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4">
            <div class="mb-3 flex items-center justify-between">
              <h2 class="text-xs uppercase tracking-[0.16em] text-text-muted">
                Library
              </h2><span class="text-xs text-text-muted">{{ filteredSkills?.length ?? 0 }}</span>
            </div>
            <div class="flex max-h-[70vh] flex-col gap-3 overflow-auto">
              <button
                v-for="skill in filteredSkills || []"
                :key="skill.id"
                class="rounded-2xl border p-3 text-left"
                :class="selectedSkill?.id===skill.id?'border-accent-primary/35 bg-accent-primary/10':'border-border-default/55 bg-bg-base/35'"
                @click="handleSelectSkill(skill.id)"
              >
                <div class="mb-2 flex items-center justify-between gap-3">
                  <strong class="truncate text-sm text-text-primary">{{ skill.name }}</strong><span class="rounded-full bg-bg-overlay/70 px-2 py-0.5 text-xs text-text-secondary">{{ skill.installCount }}</span>
                </div>
                <p class="line-clamp-2 text-xs text-text-muted">
                  {{ skill.description || 'No description.' }}
                </p>
              </button>
            </div>
          </div>
          <div
            v-if="(onboardingCandidates || []).length"
            class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4"
          >
            <div class="mb-3 flex items-center justify-between">
              <h2 class="text-xs uppercase tracking-[0.16em] text-text-muted">
                Onboarding
              </h2><span class="text-xs text-text-muted">{{ onboardingCandidates.length }}</span>
            </div>
            <div
              v-for="candidate in onboardingCandidates"
              :key="candidate.skillId"
              class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3"
            >
              <div class="mb-2">
                <strong class="text-sm text-text-primary">{{ candidate.name }}</strong><p class="text-xs text-text-muted">
                  {{ candidate.reason }}
                </p>
              </div>
              <button
                data-testid="onboarding-import"
                class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                @click="importOnboardingCandidate(candidate)"
              >
                Import
              </button>
            </div>
          </div>
        </aside>
        <div class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4">
          <div
            v-if="selectedSkill"
            class="flex flex-col gap-4"
          >
            <div class="flex flex-wrap items-start justify-between gap-3">
              <div>
                <h2 class="text-lg font-semibold text-text-primary">
                  {{ selectedSkill.name }}
                </h2><p class="mt-1 text-sm text-text-muted">
                  {{ selectedSkill.description || 'No description.' }}
                </p>
              </div>
              <div class="flex gap-2">
                <button
                  class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                  :disabled="mutationLoading || selectedPlatforms.length===0"
                  @click="syncSelectedSkill"
                >
                  Sync selected
                </button>
                <button
                  class="rounded-xl border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger"
                  :disabled="mutationLoading"
                  @click="removeSelectedSkill"
                >
                  Remove
                </button>
              </div>
            </div>
            <div class="grid gap-3 md:grid-cols-2">
              <div class="rounded-2xl bg-bg-base/35 p-3">
                <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">Origin</span><strong class="mt-1 block text-sm text-text-primary">{{ selectedSkill.origin }}</strong>
              </div>
              <div class="rounded-2xl bg-bg-base/35 p-3">
                <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">Source</span><strong class="mt-1 block truncate text-sm text-text-primary">{{ selectedSkill.lifecycle.sourceLabel || selectedSkill.lifecycle.sourceRef || 'N/A' }}</strong>
              </div>
            </div>
            <div class="grid gap-3 lg:grid-cols-2">
              <div class="rounded-2xl bg-bg-base/35 p-3">
                <h3 class="mb-2 text-xs uppercase tracking-[0.16em] text-text-muted">
                  Targets
                </h3><div class="flex flex-col gap-2">
                  <div
                    v-for="target in selectedSkill.targets"
                    :key="target.id"
                    class="rounded-2xl border border-border-default/55 bg-bg-overlay/50 p-3"
                  >
                    <strong class="text-sm text-text-primary">{{ target.platformName }}</strong><p class="mt-1 text-xs text-text-muted">
                      {{ target.targetPath }}
                    </p>
                  </div>
                </div>
              </div>
              <div class="rounded-2xl bg-bg-base/35 p-3">
                <h3 class="mb-2 text-xs uppercase tracking-[0.16em] text-text-muted">
                  Content
                </h3><pre class="max-h-[340px] overflow-auto rounded-2xl bg-bg-overlay/70 p-4 text-xs text-text-primary">{{ selectedContent }}</pre>
              </div>
            </div>
          </div>
          <div
            v-else
            class="flex min-h-[380px] flex-col items-center justify-center gap-3 text-center"
          >
            <SIcon
              name="BookOpen"
              size="w-10 h-10"
              class="text-text-muted"
            /><h2 class="text-lg font-semibold text-text-primary">
              Select a skill
            </h2>
          </div>
        </div>
      </section>

      <section
        v-else-if="activeTab==='explore'"
        class="grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)]"
      >
        <aside class="flex flex-col gap-4">
          <div class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4">
            <div class="mb-3 flex items-center justify-between">
              <h2 class="text-xs uppercase tracking-[0.16em] text-text-muted">
                Targets
              </h2><button
                class="text-xs text-text-muted"
                @click="selectDetectedPlatforms"
              >
                Detected
              </button>
            </div><div class="flex flex-col gap-2">
              <label
                v-for="platform in platforms"
                :key="platform.id"
                class="grid grid-cols-[auto_1fr_auto] items-center gap-3 rounded-2xl border px-3 py-2 text-sm"
                :class="platform.detected?'border-border-default/55 bg-bg-base/35 text-text-primary':'border-border-default/35 bg-bg-base/20 text-text-muted opacity-50'"
              ><input
                v-model="selectedPlatforms"
                type="checkbox"
                :value="platform.id"
                :disabled="!platform.detected"
              ><span>{{ platform.displayName }}</span><strong>{{ platform.installedCount }}</strong></label>
            </div>
          </div>
          <div class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4">
            <h2 class="mb-3 text-xs uppercase tracking-[0.16em] text-text-muted">
              npx
            </h2><p class="text-sm text-text-primary">
              {{ npxCapabilities?.available ? `ready · ${npxCapabilities.version || 'unknown'}` : 'missing' }}
            </p><p class="mt-1 break-all text-xs text-text-muted">
              {{ npxCapabilities?.path || 'Not detected' }}
            </p>
          </div>
          <div class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4">
            <h2 class="mb-3 text-xs uppercase tracking-[0.16em] text-text-muted">
              Manual
            </h2><div class="flex flex-col gap-3">
              <input
                v-model="manualGithub"
                aria-label="GitHub skill repository"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                placeholder="owner/repo"
              ><button
                class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                :disabled="!manualGithub.trim() || selectedPlatforms.length===0"
                @click="openInstallReview('github', manualGithub.trim())"
              >
                Review GitHub
              </button><input
                v-model="manualNpxPackage"
                aria-label="npx package"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                placeholder="vercel-labs/agent-skills"
              ><input
                v-model="manualNpxSkills"
                aria-label="npx skills to install"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                placeholder="skill-a,skill-b"
              ><button
                class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                :disabled="!manualNpxPackage.trim() || selectedPlatforms.length===0"
                @click="openInstallReview('npx', manualNpxPackage.trim(), parseSelectedSkills(manualNpxSkills))"
              >
                Review npx
              </button>
            </div>
          </div>
        </aside>
        <div class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4">
          <div class="mb-4 flex gap-3">
            <input
              v-model="exploreQuery"
              aria-label="Search skills marketplace"
              class="flex-1 rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
              placeholder="Search skills.sh"
              @keydown.enter="reloadMarketplace(true)"
            ><button
              class="rounded-xl border border-border-default/55 bg-bg-base/45 px-4 py-2 text-sm text-text-primary"
              @click="reloadMarketplace(true)"
            >
              {{ exploreQuery ? 'Search' : 'Trending' }}
            </button>
          </div>
          <div
            v-if="marketplaceLoading"
            class="flex min-h-[360px] items-center justify-center"
          >
            <div class="loading-spinner w-8 h-8 border-accent-primary/30 border-t-accent-primary" />
          </div>
          <div
            v-else
            class="grid gap-3 lg:grid-cols-2 2xl:grid-cols-3"
          >
            <article
              v-for="item in marketplace.items"
              :key="item.package"
              class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4"
            >
              <div class="mb-2 flex items-start justify-between gap-3">
                <div>
                  <strong class="block text-sm text-text-primary">{{ item.skill || item.repo }}</strong><p class="text-xs text-text-muted">
                    {{ item.owner }}/{{ item.repo }}
                  </p>
                </div><span class="rounded-full bg-bg-overlay/70 px-2 py-0.5 text-[11px] text-text-secondary">★ {{ item.stars || 0 }}</span>
              </div><p class="mb-4 line-clamp-3 text-sm text-text-muted">
                {{ item.description || 'No description.' }}
              </p><div class="flex items-center justify-between gap-3">
                <a
                  :href="item.skillsShUrl"
                  target="_blank"
                  rel="noreferrer"
                  class="text-sm text-accent-primary"
                >Open</a><button
                  class="rounded-xl border border-accent-primary/30 bg-accent-primary/10 px-3 py-2 text-sm text-text-primary"
                  :disabled="selectedPlatforms.length===0"
                  @click="openInstallReview('marketplace', item.package, item.skill ? [item.skill] : [])"
                >
                  Install review
                </button>
              </div>
            </article>
          </div>
        </div>
      </section>

      <section
        v-else-if="activeTab==='platforms'"
        class="grid gap-4 md:grid-cols-2 2xl:grid-cols-3"
      >
        <article
          v-for="platform in platforms"
          :key="platform.id"
          class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4"
        >
          <div class="mb-4 flex items-start justify-between gap-3">
            <div>
              <h2 class="text-base font-semibold text-text-primary">
                {{ platform.displayName }}
              </h2><p class="text-xs text-text-muted">
                {{ platform.id }}
              </p>
            </div><span
              class="rounded-full px-2 py-0.5 text-[11px]"
              :class="platform.detected ? 'bg-success/10 text-success' : 'bg-danger/10 text-danger'"
            >{{ platform.detected ? 'Detected' : 'Missing' }}</span>
          </div><div class="grid gap-3">
            <div class="rounded-2xl bg-bg-base/35 p-3">
              <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">Skills dir</span><strong class="mt-1 block break-all text-sm text-text-primary">{{ platform.globalSkillsDir }}</strong>
            </div><div class="rounded-2xl bg-bg-base/35 p-3">
              <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">Install strategy</span><strong class="mt-1 block text-sm text-text-primary">{{ platform.installStrategy || 'managedcopy' }}</strong>
            </div><div class="rounded-2xl bg-bg-base/35 p-3">
              <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">npx agent key</span><strong class="mt-1 block text-sm text-text-primary">{{ platform.npxAgentKey || 'fallback' }}</strong>
            </div>
          </div>
        </article>
      </section>

      <section
        v-else
        class="flex flex-col gap-4"
      >
        <div class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4">
          <h2 class="mb-4 text-xs uppercase tracking-[0.16em] text-text-muted">
            Add source
          </h2><div class="grid gap-3 lg:grid-cols-2">
            <div class="flex flex-col gap-3">
              <input
                v-model="manualGitSource"
                aria-label="Git source URL"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                placeholder="https://github.com/owner/repo"
              ><button
                class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                :disabled="mutationLoading || !manualGitSource.trim()"
                @click="addGitRepository"
              >
                Add Git source
              </button>
            </div><div class="flex flex-col gap-3">
              <input
                v-model="manualLocalSource"
                aria-label="Local source directory"
                class="rounded-2xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                placeholder="D:/skills/repo"
              ><button
                class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                :disabled="mutationLoading || !manualLocalSource.trim()"
                @click="addLocalRepository"
              >
                Add local source
              </button>
            </div>
          </div>
        </div>
        <article
          v-for="source in sources"
          :key="source.id"
          class="rounded-3xl border border-border-default/55 bg-bg-elevated/60 p-4"
        >
          <div class="mb-4 flex flex-wrap items-start justify-between gap-3">
            <div>
              <h2 class="text-base font-semibold text-text-primary">
                {{ source.name }}
              </h2><p class="mt-1 text-sm text-text-muted">
                {{ source.location }}
              </p>
            </div><div class="flex gap-2">
              <button
                class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                :disabled="mutationLoading"
                @click="syncRepository(source.id)"
              >
                Sync
              </button><button
                class="rounded-xl border border-danger/30 bg-danger/10 px-3 py-2 text-sm text-danger"
                :disabled="mutationLoading"
                @click="removeRepository(source.id)"
              >
                Remove
              </button>
            </div>
          </div><div class="mt-4 flex flex-wrap gap-2">
            <button
              v-for="skill in source.skills"
              :key="`${source.id}:${skill.id}`"
              class="rounded-full border border-border-default/55 bg-bg-base/45 px-3 py-1.5 text-xs text-text-primary"
              :disabled="selectedPlatforms.length===0"
              @click="openSourceSkillReview(source.id, skill.id)"
            >
              {{ skill.name }}
            </button>
          </div>
        </article>
      </section>

      <Transition name="fade">
        <div
          v-if="reviewDrawerOpen"
          class="fixed inset-0 z-50 flex justify-end bg-bg-overlay/40 backdrop-blur-md"
          @click="closeReviewDrawer"
        >
          <aside
            class="flex h-full w-full max-w-[560px] flex-col gap-4 overflow-auto border-l border-border-default/55 bg-bg-elevated/90 p-5"
            @click.stop
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <h2 class="text-lg font-semibold text-text-primary">
                  Install review
                </h2><p class="mt-1 text-sm text-text-muted">
                  {{ installReview?.source.resolvedName || pendingInstall.sourceRef }}
                </p>
              </div><button
                class="rounded-xl border border-border-default/55 bg-bg-base/45 px-3 py-2 text-sm text-text-primary"
                @click="closeReviewDrawer"
              >
                <SIcon
                  name="X"
                  size="w-4 h-4"
                />
              </button>
            </div>
            <template v-if="installReview">
              <div class="grid gap-3 md:grid-cols-2">
                <div class="rounded-2xl bg-bg-base/40 p-4">
                  <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">Kind</span><strong class="mt-1 block text-sm text-text-primary">{{ installReview.source.sourceKind }}</strong>
                </div><div class="rounded-2xl bg-bg-base/40 p-4">
                  <span class="block text-[11px] uppercase tracking-[0.14em] text-text-muted">Resolved dir</span><strong class="mt-1 block text-sm text-text-primary">{{ installReview.source.resolvedDirName }}</strong>
                </div>
              </div>
              <div
                v-if="installReview.warnings.length>0"
                class="rounded-2xl border border-warning/30 bg-warning/10 p-4"
              >
                <ul class="list-disc space-y-1 pl-4 text-sm text-text-primary">
                  <li
                    v-for="warning in installReview.warnings"
                    :key="warning"
                  >
                    {{ warning }}
                  </li>
                </ul>
              </div>
              <div class="rounded-2xl bg-bg-base/40 p-4">
                <h3 class="mb-3 text-xs uppercase tracking-[0.16em] text-text-muted">
                  Targets
                </h3><div class="flex flex-col gap-2">
                  <label
                    v-for="platform in platforms"
                    :key="platform.id"
                    class="grid grid-cols-[auto_1fr_auto] items-center gap-3 rounded-2xl border px-3 py-2"
                    :class="platform.detected?'border-border-default/55 bg-bg-overlay/50 text-text-primary':'border-border-default/35 bg-bg-base/20 text-text-muted opacity-50'"
                  ><input
                    v-model="selectedPlatforms"
                    type="checkbox"
                    :value="platform.id"
                    :disabled="!platform.detected"
                    @change="refreshInstallReview"
                  ><span>{{ platform.displayName }}</span><strong class="text-xs">{{ platform.npxAgentKey || platform.installStrategy || 'managedcopy' }}</strong></label>
                </div>
              </div>
              <article
                v-for="preview in installReview.commandPreviews"
                :key="`${preview.kind}:${preview.command}`"
                class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4"
              >
                <div class="mb-2 flex items-center justify-between gap-3">
                  <strong class="text-sm text-text-primary">{{ preview.label }}</strong><span class="rounded-full bg-bg-overlay/70 px-2 py-0.5 text-[11px] text-text-secondary">{{ preview.kind }}</span>
                </div><pre class="overflow-auto rounded-2xl bg-bg-overlay/70 p-4 text-xs text-text-primary">{{ preview.command }}</pre>
              </article>
              <div class="flex justify-end gap-2">
                <button
                  class="rounded-xl border border-border-default/55 bg-bg-base/45 px-4 py-2 text-sm text-text-primary"
                  @click="closeReviewDrawer"
                >
                  Cancel
                </button><button
                  class="rounded-xl border border-accent-primary/30 bg-accent-primary/10 px-4 py-2 text-sm text-text-primary"
                  :disabled="selectedPlatforms.length===0 || mutationLoading"
                  @click="confirmInstall"
                >
                  Confirm install
                </button>
              </div>
            </template>
          </aside>
        </div>
      </Transition>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import AsyncStatePanel from '@/components/ui/AsyncStatePanel.vue'
import PageHeaderCard from '@/components/PageHeaderCard.vue'
import SIcon from '@/components/ui/SIcon.vue'
import { useUIStore } from '@/stores/ui'
import { useUnifiedSkills } from '@/composables/useUnifiedSkills'
import type { Platform, SkillOrigin, SkillsInstallRequest, SkillsTab } from '@/types/skills'
import { getRuntimeUnavailableCopy } from '@/utils/runtimeState'
import { handleSkillsChangedPayload } from './skillsWatcher'
import { isTauriRuntime } from '@/utils/tauriRuntime'

const route = useRoute()
const router = useRouter()
const uiStore = useUIStore()
const { initialize, refresh, loadMarketplace, loadOnboardingCandidates, loadNpxCapabilities, addGitSource, addLocalSourceRecord, prepareInstall, install, syncSkill, removeSkillRecord, syncSource, removeSource, ensureDetail, ensureContent, selectSkill, selectedSkill, filteredSkills, selectedInstallation, platforms, sources, marketplace, onboardingCandidates, importFromLocal, stats, filters, routeState, installReview, npxCapabilities, marketplaceLoading, mutationLoading } = useUnifiedSkills()
const runtimeUnavailable = computed(() => !isTauriRuntime())
const runtimeCopy = computed(() => getRuntimeUnavailableCopy('skills'))
const activeTab = computed<SkillsTab>(() => routeState.value.tab || 'library')
const tabs = computed(() => [{ id: 'library' as SkillsTab, label: 'Library', icon: 'LibraryBig', count: stats.value.logicalSkills }, { id: 'explore' as SkillsTab, label: 'Explore', icon: 'Store', count: marketplace.value.total }, { id: 'platforms' as SkillsTab, label: 'Platforms', icon: 'Cpu', count: platforms.value.length }, { id: 'sources' as SkillsTab, label: 'Sources', icon: 'FolderGit2', count: sources.value.length }])
const originOptions: SkillOrigin[] = ['marketplace', 'github', 'repo', 'local', 'npx', 'unknown']
const selectedPlatforms = ref<Platform[]>([])
const librarySearch = ref('')
const exploreQuery = ref('')
const manualGithub = ref('')
const manualNpxPackage = ref('')
const manualNpxSkills = ref('')
const manualGitSource = ref('')
const manualLocalSource = ref('')
const reviewDrawerOpen = ref(false)
const pendingInstall = ref<SkillsInstallRequest>({ sourceKind: 'marketplace', sourceRef: '', targetPlatforms: [] })
const selectedContent = ref('')
let stopSkillsEvent: null | (() => void) = null
let searchTimer = 0

function normalizeRouteState(query: Record<string, unknown>) { return { tab: (query.tab === 'explore' || query.tab === 'platforms' || query.tab === 'sources' ? query.tab : 'library') as SkillsTab, selected: typeof query.selected === 'string' ? query.selected : null, mode: 'view' as const, platform: typeof query.platform === 'string' ? query.platform : 'all', origin: (typeof query.origin === 'string' ? query.origin : 'all') as SkillOrigin | 'all', q: typeof query.q === 'string' ? query.q : '', page: 1, source: null } }
function syncRoute(extra: Record<string, string | null>) { const next: Record<string, string> = {}; for (const [key, value] of Object.entries({ ...route.query, ...extra })) { if (typeof value === 'string' && value.trim()) next[key] = value } void router.replace({ path: '/skills', query: next }) }
function setTab(tab: SkillsTab) { routeState.value.tab = tab; syncRoute({ tab: tab === 'library' ? null : tab }); if (tab === 'explore') void reloadMarketplace(false) }
function resetLibraryFilters() { librarySearch.value = ''; filters.value.search = ''; filters.value.platform = 'all'; filters.value.origin = 'all'; filters.value.source = 'all'; filters.value.tags = []; syncRoute({ q: null, platform: null, origin: null }) }
function selectDetectedPlatforms() { selectedPlatforms.value = platforms.value.filter((item) => item.detected).map((item) => item.id) }
async function handleSelectSkill(skillId: string) { selectSkill(skillId, null); syncRoute({ selected: skillId }); await ensureDetail(skillId, true); const content = await ensureContent(skillId, selectedInstallation.value?.id ?? null, true); selectedContent.value = content?.raw ?? '' }
async function syncSelectedSkill() { if (!selectedSkill.value || selectedPlatforms.value.length === 0) return; try { await syncSkill({ skillId: selectedSkill.value.id, installationId: selectedInstallation.value?.id, targetPlatforms: selectedPlatforms.value, force: true }); uiStore.showSuccess(`Synced ${selectedSkill.value.name}`); await refresh(activeTab.value === 'explore') } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
async function removeSelectedSkill() { if (!selectedSkill.value) return; try { await removeSkillRecord(selectedSkill.value.id); uiStore.showSuccess(`Removed ${selectedSkill.value.name}`); selectSkill(null, null); selectedContent.value = ''; syncRoute({ selected: null }) } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
async function importOnboardingCandidate(candidate: { name: string; platformIds: string[]; installationPaths: string[] }) { try { await importFromLocal({ sourcePath: candidate.installationPaths[0], agents: candidate.platformIds, skillName: candidate.name }); uiStore.showSuccess(`Imported ${candidate.name}`) } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
function parseSelectedSkills(value: string) { return value.split(',').map((item) => item.trim()).filter(Boolean) }
async function openInstallReview(sourceKind: SkillsInstallRequest['sourceKind'], sourceRef: string, selectedSkills: string[] = []) { pendingInstall.value = { sourceKind, sourceRef, sourceSkillId: selectedSkills[0], selectedSkills, targetPlatforms: [...selectedPlatforms.value], scope: 'global', copyMode: true, allMode: false }; try { await prepareInstall(pendingInstall.value); reviewDrawerOpen.value = true } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
async function openSourceSkillReview(sourceId: string, skillId: string) { await openInstallReview('source', sourceId, [skillId]) }
async function refreshInstallReview() { if (!reviewDrawerOpen.value || !pendingInstall.value.sourceRef) return; pendingInstall.value = { ...pendingInstall.value, targetPlatforms: [...selectedPlatforms.value] }; try { await prepareInstall(pendingInstall.value) } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
async function confirmInstall() { pendingInstall.value = { ...pendingInstall.value, targetPlatforms: [...selectedPlatforms.value] }; try { await install(pendingInstall.value); uiStore.showSuccess(`Installed ${installReview.value?.source.resolvedName || pendingInstall.value.sourceRef}`); closeReviewDrawer(); await refresh(activeTab.value === 'explore') } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
function closeReviewDrawer() { reviewDrawerOpen.value = false }
async function reloadMarketplace(force: boolean) { routeState.value.q = exploreQuery.value.trim(); syncRoute({ q: routeState.value.q || null, tab: 'explore' }); await loadMarketplace(force) }
async function addGitRepository() { try { await addGitSource(manualGitSource.value.trim()); manualGitSource.value = ''; uiStore.showSuccess('Git source added') } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
async function addLocalRepository() { try { await addLocalSourceRecord(manualLocalSource.value.trim()); manualLocalSource.value = ''; uiStore.showSuccess('Local source added') } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
async function syncRepository(sourceId: string) { try { await syncSource(sourceId); uiStore.showSuccess('Source synced') } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
async function removeRepository(sourceId: string) { try { await removeSource(sourceId); uiStore.showSuccess('Source removed') } catch (error) { uiStore.showError(error instanceof Error ? error.message : String(error)) } }
watch(() => route.query, (query) => { const normalized = normalizeRouteState(query as Record<string, unknown>); routeState.value = normalized; filters.value.platform = normalized.platform; filters.value.origin = normalized.origin as SkillOrigin | 'all'; filters.value.search = normalized.q; librarySearch.value = normalized.q; if (normalized.selected) void handleSelectSkill(normalized.selected) }, { immediate: true })
watch(librarySearch, (value) => { window.clearTimeout(searchTimer); searchTimer = window.setTimeout(() => { filters.value.search = value.trim(); syncRoute({ q: value.trim() || null }) }, 250) })

onMounted(async () => { if (runtimeUnavailable.value) return; await initialize(activeTab.value === 'explore'); await Promise.all([loadNpxCapabilities?.(true) ?? Promise.resolve(null), loadOnboardingCandidates(true)]); if (selectedPlatforms.value.length === 0) selectDetectedPlatforms(); if (isTauriRuntime()) { const { listen } = await import('@tauri-apps/api/event'); stopSkillsEvent = await listen('skills-changed', async (event) => { await handleSkillsChangedPayload(event.payload as { affectsInventory?: boolean; affectsSources?: boolean; affectsMarketplace?: boolean }, { currentTab: activeTab.value, loadOnboardingCandidates, refresh }) }) } })
onUnmounted(() => { stopSkillsEvent?.(); stopSkillsEvent = null })
</script>
