<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <div class="max-w-7xl mx-auto space-y-8">
      <!-- HEADER -->
      <header class="flex flex-col md:flex-row md:items-center justify-between gap-4 animate-slide-up">
        <div>
          <div class="mb-2 flex items-center gap-2 text-sm text-text-secondary">
            <RouterLink
              to="/claude-code"
              class="transition-colors hover:text-text-primary"
            >
              Claude Code
            </RouterLink>
            <ChevronRight class="w-3 h-3" />
            <span class="text-text-primary">{{ $t('claudeProfiles.breadcrumbProfiles') }}</span>
          </div>
          <h1 class="text-3xl font-bold font-display tracking-tight text-text-primary">
            {{ $t('claudeProfiles.title') }}
          </h1>
          <p class="mt-1 text-text-secondary">
            {{ $t('claudeProfiles.subtitle') }}
          </p>
        </div>
        <div class="flex items-center gap-3">
          <RouterLink to="/claude-code">
            <button class="flex min-h-[44px] items-center gap-2 rounded-xl border border-border-default bg-bg-surface px-4 py-2.5 text-sm text-text-secondary transition-colors hover:bg-bg-elevated hover:text-text-primary">
              <ArrowLeft class="w-4 h-4" /> {{ $t('claudeProfiles.back') }}
            </button>
          </RouterLink>
          <button
            type="button"
            class="flex min-h-[44px] items-center gap-2 rounded-xl border border-accent-secondary/30 bg-accent-secondary/10 px-4 py-2.5 text-sm font-medium text-accent-secondary transition-colors hover:bg-accent-secondary/15 focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
            @click="openAddForm()"
          >
            <Plus class="w-4 h-4" /> {{ $t('claudeProfiles.addProfile') }}
          </button>
        </div>
      </header>

      <!-- STATUS CARDS -->
      <div
        class="grid grid-cols-1 md:grid-cols-3 gap-4 animate-slide-up"
        style="animation-delay: 100ms"
      >
        <!-- 当前 Profile -->
        <div class="rounded-2xl border border-border-default/50 bg-bg-surface/75 p-5">
          <div class="flex items-center gap-3 mb-2">
            <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-accent-secondary/10 text-accent-secondary">
              <Zap class="w-5 h-5" />
            </div>
            <div>
              <p class="text-xs uppercase tracking-wider text-text-secondary">
                {{ $t('claudeProfiles.currentProfile') }}
              </p>
              <p class="text-lg font-semibold text-text-primary">
                {{ currentProfileName || $t('claudeProfiles.notSet') }}
              </p>
            </div>
          </div>
        </div>
        <!-- 总数 -->
        <div class="rounded-2xl border border-border-default/50 bg-bg-surface/75 p-5">
          <div class="flex items-center gap-3 mb-2">
            <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-accent-primary/10 text-accent-primary">
              <Layers class="w-5 h-5" />
            </div>
            <div>
              <p class="text-xs uppercase tracking-wider text-text-secondary">
                {{ $t('claudeProfiles.totalCount') }}
              </p>
              <p class="text-lg font-semibold text-text-primary">
                {{ profiles.length }}
              </p>
            </div>
          </div>
        </div>
        <!-- 快照范围 -->
        <div class="rounded-2xl border border-border-default/50 bg-bg-surface/75 p-5">
          <div class="flex items-center gap-3 mb-2">
            <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-accent-info/10 text-accent-info">
              <Package class="w-5 h-5" />
            </div>
            <div>
              <p class="text-xs uppercase tracking-wider text-text-secondary">
                {{ $t('claudeProfiles.snapshotScope') }}
              </p>
              <p class="text-sm font-semibold text-text-primary">
                {{ $t('claudeProfiles.snapshotScopeValue') }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- QUICK SWITCH -->
      <div
        v-if="profiles.length > 0"
        class="rounded-2xl border border-border-default/50 bg-bg-surface/75 p-5 animate-slide-up"
        style="animation-delay: 150ms"
      >
        <h3 class="mb-3 flex items-center gap-2 text-sm font-medium text-text-secondary">
          <RefreshCw class="w-4 h-4" /> {{ $t('claudeProfiles.quickSwitch') }}
        </h3>
        <div class="flex flex-wrap gap-3">
          <button
            v-for="profile in profiles"
            :key="profile.name"
            type="button"
            class="flex min-h-[44px] items-center gap-2 rounded-xl border px-4 py-2.5 text-sm font-medium transition-colors"
            :class="profile.is_current
              ? 'border-accent-secondary/35 bg-accent-secondary/10 text-accent-secondary shadow-sm'
              : 'border-border-default bg-bg-surface text-text-secondary hover:bg-bg-elevated hover:text-text-primary'"
            @click="handleApply(profile.name)"
          >
            <Check
              v-if="profile.is_current"
              class="w-3.5 h-3.5"
            />
            {{ profile.name }}
          </button>
        </div>
      </div>

      <!-- LOADING -->
      <div
        v-if="loading"
        class="flex items-center justify-center py-20"
      >
        <div class="h-8 w-8 rounded-full border-2 border-accent-secondary/30 border-t-accent-secondary animate-spin" />
      </div>

      <!-- EMPTY STATE -->
      <div
        v-else-if="profiles.length === 0"
        class="text-center py-20 animate-slide-up"
        style="animation-delay: 200ms"
      >
        <div class="mx-auto mb-6 flex h-20 w-20 items-center justify-center rounded-3xl border border-border-default/50 bg-bg-surface/75">
          <FolderOpen class="w-10 h-10 text-text-muted" />
        </div>
        <h3 class="mb-2 text-xl font-semibold text-text-primary">
          {{ $t('claudeProfiles.emptyTitle') }}
        </h3>
        <p class="mb-6 text-text-secondary">
          {{ $t('claudeProfiles.emptyDesc') }}
        </p>
        <button
          type="button"
          class="inline-flex min-h-[44px] items-center justify-center rounded-xl border border-accent-secondary/30 bg-accent-secondary/10 px-6 py-3 text-sm font-medium text-accent-secondary transition-colors hover:bg-accent-secondary/15 focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
          @click="openAddForm()"
        >
          <Plus class="w-4 h-4 inline mr-2" /> {{ $t('claudeProfiles.createProfile') }}
        </button>
      </div>

      <!-- PROFILE GRID -->
      <div
        v-else
        class="grid grid-cols-1 xl:grid-cols-2 gap-4 animate-slide-up"
        style="animation-delay: 200ms"
      >
        <div
          v-for="profile in profiles"
          :key="profile.id"
          class="group rounded-2xl border p-6 transition-[border-color,box-shadow,transform] hover:-translate-y-0.5 hover:shadow-lg"
          :class="profile.is_current
            ? 'border-accent-secondary/30 bg-accent-secondary/5 shadow-accent-secondary/5'
            : 'border-border-default/50 bg-bg-surface/75 hover:border-border-default'"
        >
          <!-- Card Header -->
          <div class="flex items-start justify-between mb-4">
            <div class="flex items-center gap-3">
              <div
                class="flex h-10 w-10 items-center justify-center rounded-xl"
                :class="profile.is_current ? 'bg-accent-secondary/10 text-accent-secondary' : 'bg-bg-elevated text-text-secondary'"
              >
                <User class="w-5 h-5" />
              </div>
              <div>
                <h3 class="flex items-center gap-2 font-semibold text-text-primary">
                  {{ profile.name }}
                  <span
                    v-if="profile.is_current"
                    class="rounded-full bg-accent-secondary/10 px-2 py-0.5 text-xs text-accent-secondary"
                  >
                    {{ $t('claudeProfiles.currentBadge') }}
                  </span>
                </h3>
                <p
                  v-if="profile.description"
                  class="text-sm text-text-secondary"
                >
                  {{ profile.description }}
                </p>
              </div>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                type="button"
                class="flex min-h-[44px] min-w-[44px] items-center justify-center rounded-lg text-text-secondary transition-colors hover:bg-bg-elevated hover:text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
                :title="$t('claudeProfiles.editTooltip')"
                @click="openEditForm(profile)"
              >
                <Pencil class="w-4 h-4" />
              </button>
              <button
                type="button"
                class="flex min-h-[44px] min-w-[44px] items-center justify-center rounded-lg text-text-secondary transition-colors hover:bg-accent-danger/10 hover:text-accent-danger focus:outline-none focus:ring-2 focus:ring-accent-danger/20"
                :title="$t('claudeProfiles.deleteTooltip')"
                @click="handleDelete(profile.name)"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- Snapshot Summary -->
          <div class="grid grid-cols-2 gap-2 mb-4">
            <div class="text-xs text-text-secondary">
              <span class="text-text-primary">{{ $t('claudeProfiles.mcpLabel') }}:</span> {{ $t('claudeProfiles.countUnit', { n: profile.snapshot_stats?.mcp_count ?? 0 }) }}
            </div>
            <div class="text-xs text-text-secondary">
              <span class="text-text-primary">{{ $t('claudeProfiles.stylesLabel') }}:</span> {{ $t('claudeProfiles.countUnit', { n: profile.snapshot_stats?.style_count ?? 0 }) }}
            </div>
            <div class="text-xs text-text-secondary">
              <span class="text-text-primary">{{ $t('claudeProfiles.updatedLabel') }}:</span> {{ formatDate(profile.updated_at) }}
            </div>
            <div class="text-xs text-text-secondary">
              <span class="text-text-primary">{{ $t('claudeProfiles.enabledLabel') }}:</span>
              <span :class="profile.enabled ? 'text-accent-success' : 'text-accent-danger'">
                {{ profile.enabled ? $t('claudeProfiles.yes') : $t('claudeProfiles.no') }}
              </span>
            </div>
          </div>

          <!-- Actions -->
          <button
            v-if="!profile.is_current"
            type="button"
            class="w-full rounded-xl border border-accent-secondary/25 bg-accent-secondary/10 px-4 py-2.5 text-sm font-medium text-accent-secondary transition-colors hover:bg-accent-secondary/15 focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
            @click="handleApply(profile.name)"
          >
            {{ $t('claudeProfiles.applyProfile') }}
          </button>
          <div
            v-else
            class="w-full rounded-xl border border-accent-secondary/15 bg-accent-secondary/5 px-4 py-2.5 text-center text-sm font-medium text-accent-secondary/70"
          >
            {{ $t('claudeProfiles.currentlyActive') }}
          </div>
        </div>
      </div>

      <!-- ADD/EDIT MODAL -->
      <BaseModal
        v-model="showForm"
        :title="isEditing ? $t('claudeProfiles.editProfileTitle') : $t('claudeProfiles.newProfileTitle')"
        :description="$t('claudeProfiles.subtitle')"
        size="lg"
        content-class="max-w-lg"
      >
        <div class="space-y-4">
          <div>
            <label
              for="claude-profile-name"
              class="mb-1.5 block text-sm font-medium text-text-secondary"
            >{{ $t('claudeProfiles.nameLabel') }}</label>
            <input
              id="claude-profile-name"
              v-model="form.name"
              type="text"
              :placeholder="$t('claudeProfiles.namePlaceholder')"
              class="w-full rounded-xl border border-border-default bg-bg-surface px-4 py-3 text-text-primary placeholder:text-text-muted transition-colors focus:border-accent-secondary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
            >
          </div>

          <div>
            <label
              for="claude-profile-description"
              class="mb-1.5 block text-sm font-medium text-text-secondary"
            >{{ $t('claudeProfiles.descLabel') }}</label>
            <input
              id="claude-profile-description"
              v-model="form.description"
              type="text"
              :placeholder="$t('claudeProfiles.descPlaceholder')"
              class="w-full rounded-xl border border-border-default bg-bg-surface px-4 py-3 text-text-primary placeholder:text-text-muted transition-colors focus:border-accent-secondary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
            >
          </div>

          <div>
            <label
              for="claude-profile-tags"
              class="mb-1.5 block text-sm font-medium text-text-secondary"
            >{{ $t('claudeProfiles.tagsLabel') }}</label>
            <input
              id="claude-profile-tags"
              v-model="form.tagsInput"
              type="text"
              :placeholder="$t('claudeProfiles.tagsPlaceholder')"
              class="w-full rounded-xl border border-border-default bg-bg-surface px-4 py-3 text-text-primary placeholder:text-text-muted transition-colors focus:border-accent-secondary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
            >
          </div>

          <div
            v-if="!isEditing"
            class="rounded-xl border border-border-default/50 bg-bg-surface/60 p-4"
          >
            <label
              for="claude-profile-snapshot-new"
              class="flex cursor-pointer items-start gap-3"
            >
              <input
                id="claude-profile-snapshot-new"
                v-model="form.snapshotFromCurrent"
                type="checkbox"
                class="mt-1 h-4 w-4 rounded border-border-default text-accent-secondary focus:ring-accent-secondary/30"
              >
              <span>
                <span class="block text-sm text-text-primary">{{ $t('claudeProfiles.snapshotFromCurrent') }}</span>
                <span class="mt-1 block text-xs text-text-secondary">{{ $t('claudeProfiles.snapshotHint') }}</span>
              </span>
            </label>
          </div>

          <div
            v-if="isEditing"
            class="rounded-xl border border-border-default/50 bg-bg-surface/60 p-4"
          >
            <label
              for="claude-profile-snapshot-edit"
              class="flex cursor-pointer items-center gap-3"
            >
              <input
                id="claude-profile-snapshot-edit"
                v-model="form.snapshotFromCurrent"
                type="checkbox"
                class="h-4 w-4 rounded border-border-default text-accent-secondary focus:ring-accent-secondary/30"
              >
              <span class="text-sm text-text-primary">{{ $t('claudeProfiles.reSnapshot') }}</span>
            </label>
          </div>

          <div class="flex items-center justify-end gap-3 border-t border-border-default/50 pt-4">
            <button
              type="button"
              class="min-h-[44px] rounded-xl border border-border-default bg-bg-surface px-5 py-2.5 text-sm text-text-secondary transition-colors hover:bg-bg-elevated hover:text-text-primary focus:outline-none focus:ring-2 focus:ring-accent-secondary/20"
              @click="showForm = false"
            >
              {{ $t('claudeProfiles.cancel') }}
            </button>
            <button
              type="button"
              class="min-h-[44px] rounded-xl border border-accent-secondary/30 bg-accent-secondary/10 px-5 py-2.5 text-sm font-medium text-accent-secondary transition-colors hover:bg-accent-secondary/15 focus:outline-none focus:ring-2 focus:ring-accent-secondary/20 disabled:cursor-not-allowed disabled:opacity-50"
              :disabled="!form.name.trim()"
              @click="handleSave()"
            >
              {{ isEditing ? $t('claudeProfiles.save') : $t('claudeProfiles.create') }}
            </button>
          </div>
        </div>
      </BaseModal>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import {
  ChevronRight, ArrowLeft, Plus, Zap, Layers, Package,
  RefreshCw, Check, FolderOpen, User, Pencil, Trash2
} from 'lucide-vue-next'
import {
  listClaudeProfiles,
  addClaudeProfile,
  updateClaudeProfile,
  deleteClaudeProfile,
  applyClaudeProfile,
} from '@/api'
import { getErrorMessage } from '@/types/api'
import type { ClaudeProfile, ClaudeProfilesResponse } from '@/types'
import BaseModal from '@/components/common/BaseModal.vue'

const { t } = useI18n()

// -- State --

const loading = ref(true)
const profiles = ref<ClaudeProfile[]>([])
const showForm = ref(false)
const isEditing = ref(false)
const editingName = ref('')

const form = reactive({
  name: '',
  description: '',
  tagsInput: '',
  snapshotFromCurrent: true,
})

// -- Derived State --

/** 当前激活的 Profile 名称（从 profiles 列表派生，避免冗余状态） */
const currentProfileName = computed(() =>
  profiles.value.find(p => p.is_current)?.name ?? null,
)

// -- Data Loading --

const loadProfiles = async () => {
  try {
    loading.value = true
    const data = await listClaudeProfiles<ClaudeProfilesResponse>()
    profiles.value = data.profiles || []
  } catch (error) {
    console.error('Failed to load profiles:', error) // eslint-disable-line no-console
  } finally {
    loading.value = false
  }
}

onMounted(loadProfiles)

// -- Helpers --

const formatDate = (dateStr: string): string => {
  try {
    const date = new Date(dateStr)
    return date.toLocaleDateString('zh-CN', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
  } catch {
    return dateStr
  }
}

// -- Form Actions --

const resetForm = () => {
  form.name = ''
  form.description = ''
  form.tagsInput = ''
  form.snapshotFromCurrent = true
}

const openAddForm = () => {
  resetForm()
  isEditing.value = false
  showForm.value = true
}

const openEditForm = (profile: ClaudeProfile) => {
  form.name = profile.name
  form.description = profile.description || ''
  form.tagsInput = profile.tags
    ? (() => { try { return JSON.parse(profile.tags).join(', ') } catch { return '' } })()
    : ''
  form.snapshotFromCurrent = false
  isEditing.value = true
  editingName.value = profile.name
  showForm.value = true
}

const handleSave = async () => {
  if (!form.name.trim()) return

  const tags = form.tagsInput
    .split(',')
    .map(t => t.trim())
    .filter(Boolean)

  try {
    if (isEditing.value) {
      await updateClaudeProfile(editingName.value, {
        name: form.name.trim(),
        description: form.description || undefined,
        tags: tags.length > 0 ? tags : undefined,
        snapshot_from_current: form.snapshotFromCurrent || undefined,
      })
    } else {
      await addClaudeProfile({
        name: form.name.trim(),
        description: form.description || undefined,
        tags: tags.length > 0 ? tags : undefined,
        snapshot_from_current: form.snapshotFromCurrent,
      })
    }

    showForm.value = false
    await loadProfiles()
  } catch (error) {
    alert(getErrorMessage(error, t('claudeProfiles.operationFailed')))
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('claudeProfiles.confirmDelete', { name }))) return
  try {
    await deleteClaudeProfile(name)
    await loadProfiles()
  } catch (error) {
    alert(getErrorMessage(error, t('claudeProfiles.deleteFailed')))
  }
}

const handleApply = async (name: string) => {
  if (!confirm(t('claudeProfiles.confirmApply', { name }))) return
  try {
    await applyClaudeProfile(name)
    await loadProfiles()
  } catch (error) {
    alert(getErrorMessage(error, t('claudeProfiles.applyFailed')))
  }
}
</script>
