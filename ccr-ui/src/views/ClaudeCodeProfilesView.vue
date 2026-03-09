<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <div class="max-w-7xl mx-auto space-y-8">
      <!-- HEADER -->
      <header class="flex flex-col md:flex-row md:items-center justify-between gap-4 animate-slide-up">
        <div>
          <div class="flex items-center gap-2 text-white/50 text-sm mb-2">
            <RouterLink
              to="/claude-code"
              class="hover:text-white/80 transition-colors"
            >
              Claude Code
            </RouterLink>
            <ChevronRight class="w-3 h-3" />
            <span class="text-white/80">{{ $t('claudeProfiles.breadcrumbProfiles') }}</span>
          </div>
          <h1 class="text-3xl font-bold font-display tracking-tight text-gradient-brand">
            {{ $t('claudeProfiles.title') }}
          </h1>
          <p class="text-white/60 mt-1">
            {{ $t('claudeProfiles.subtitle') }}
          </p>
        </div>
        <div class="flex items-center gap-3">
          <RouterLink to="/claude-code">
            <button class="px-4 py-2 rounded-xl bg-white/5 border border-white/10 text-white/70 hover:bg-white/10 hover:text-white transition-all text-sm flex items-center gap-2">
              <ArrowLeft class="w-4 h-4" /> {{ $t('claudeProfiles.back') }}
            </button>
          </RouterLink>
          <button
            class="px-4 py-2 rounded-xl bg-[#FF6B35]/20 border border-[#FF6B35]/30 text-[#FF6B35] hover:bg-[#FF6B35]/30 transition-all text-sm font-medium flex items-center gap-2"
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
        <div class="rounded-2xl bg-white/5 border border-white/10 backdrop-blur-md p-5">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-10 h-10 rounded-xl bg-[#FF6B35]/20 flex items-center justify-center">
              <Zap class="w-5 h-5 text-[#FF6B35]" />
            </div>
            <div>
              <p class="text-white/50 text-xs uppercase tracking-wider">
                {{ $t('claudeProfiles.currentProfile') }}
              </p>
              <p class="text-white font-semibold text-lg">
                {{ currentProfileName || $t('claudeProfiles.notSet') }}
              </p>
            </div>
          </div>
        </div>
        <!-- 总数 -->
        <div class="rounded-2xl bg-white/5 border border-white/10 backdrop-blur-md p-5">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
              <Layers class="w-5 h-5 text-purple-400" />
            </div>
            <div>
              <p class="text-white/50 text-xs uppercase tracking-wider">
                {{ $t('claudeProfiles.totalCount') }}
              </p>
              <p class="text-white font-semibold text-lg">
                {{ profiles.length }}
              </p>
            </div>
          </div>
        </div>
        <!-- 快照范围 -->
        <div class="rounded-2xl bg-white/5 border border-white/10 backdrop-blur-md p-5">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center">
              <Package class="w-5 h-5 text-cyan-400" />
            </div>
            <div>
              <p class="text-white/50 text-xs uppercase tracking-wider">
                {{ $t('claudeProfiles.snapshotScope') }}
              </p>
              <p class="text-white font-semibold text-sm">
                {{ $t('claudeProfiles.snapshotScopeValue') }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- QUICK SWITCH -->
      <div
        v-if="profiles.length > 0"
        class="rounded-2xl bg-white/5 border border-white/10 backdrop-blur-md p-5 animate-slide-up"
        style="animation-delay: 150ms"
      >
        <h3 class="text-white/70 text-sm font-medium mb-3 flex items-center gap-2">
          <RefreshCw class="w-4 h-4" /> {{ $t('claudeProfiles.quickSwitch') }}
        </h3>
        <div class="flex flex-wrap gap-3">
          <button
            v-for="profile in profiles"
            :key="profile.name"
            class="px-4 py-2 rounded-xl text-sm font-medium transition-all flex items-center gap-2"
            :class="profile.is_current
              ? 'bg-[#FF6B35]/20 border border-[#FF6B35]/40 text-[#FF6B35] shadow-lg shadow-[#FF6B35]/10'
              : 'bg-white/5 border border-white/10 text-white/70 hover:bg-white/10 hover:text-white'"
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
        <div class="w-8 h-8 border-2 border-[#FF6B35]/30 border-t-[#FF6B35] rounded-full animate-spin" />
      </div>

      <!-- EMPTY STATE -->
      <div
        v-else-if="profiles.length === 0"
        class="text-center py-20 animate-slide-up"
        style="animation-delay: 200ms"
      >
        <div class="w-20 h-20 rounded-3xl bg-white/5 border border-white/10 flex items-center justify-center mx-auto mb-6">
          <FolderOpen class="w-10 h-10 text-white/20" />
        </div>
        <h3 class="text-white/80 text-xl font-semibold mb-2">
          {{ $t('claudeProfiles.emptyTitle') }}
        </h3>
        <p class="text-white/50 mb-6">
          {{ $t('claudeProfiles.emptyDesc') }}
        </p>
        <button
          class="px-6 py-3 rounded-xl bg-[#FF6B35]/20 border border-[#FF6B35]/30 text-[#FF6B35] hover:bg-[#FF6B35]/30 transition-all font-medium"
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
          class="rounded-2xl border backdrop-blur-md p-6 transition-all hover:shadow-lg group"
          :class="profile.is_current
            ? 'bg-[#FF6B35]/5 border-[#FF6B35]/30 shadow-[#FF6B35]/5'
            : 'bg-white/5 border-white/10 hover:border-white/20'"
        >
          <!-- Card Header -->
          <div class="flex items-start justify-between mb-4">
            <div class="flex items-center gap-3">
              <div
                class="w-10 h-10 rounded-xl flex items-center justify-center"
                :class="profile.is_current ? 'bg-[#FF6B35]/20' : 'bg-white/10'"
              >
                <User
                  class="w-5 h-5"
                  :class="profile.is_current ? 'text-[#FF6B35]' : 'text-white/50'"
                />
              </div>
              <div>
                <h3 class="text-white font-semibold flex items-center gap-2">
                  {{ profile.name }}
                  <span
                    v-if="profile.is_current"
                    class="text-xs px-2 py-0.5 rounded-full bg-[#FF6B35]/20 text-[#FF6B35]"
                  >
                    {{ $t('claudeProfiles.currentBadge') }}
                  </span>
                </h3>
                <p
                  v-if="profile.description"
                  class="text-white/50 text-sm"
                >
                  {{ profile.description }}
                </p>
              </div>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                class="p-2 rounded-lg hover:bg-white/10 text-white/50 hover:text-white transition-all"
                :title="$t('claudeProfiles.editTooltip')"
                @click="openEditForm(profile)"
              >
                <Pencil class="w-4 h-4" />
              </button>
              <button
                class="p-2 rounded-lg hover:bg-red-500/10 text-white/50 hover:text-red-400 transition-all"
                :title="$t('claudeProfiles.deleteTooltip')"
                @click="handleDelete(profile.name)"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- Snapshot Summary -->
          <div class="grid grid-cols-2 gap-2 mb-4">
            <div class="text-xs text-white/40">
              <span class="text-white/60">{{ $t('claudeProfiles.mcpLabel') }}:</span> {{ $t('claudeProfiles.countUnit', { n: profile.snapshot_stats?.mcp_count ?? 0 }) }}
            </div>
            <div class="text-xs text-white/40">
              <span class="text-white/60">{{ $t('claudeProfiles.stylesLabel') }}:</span> {{ $t('claudeProfiles.countUnit', { n: profile.snapshot_stats?.style_count ?? 0 }) }}
            </div>
            <div class="text-xs text-white/40">
              <span class="text-white/60">{{ $t('claudeProfiles.updatedLabel') }}:</span> {{ formatDate(profile.updated_at) }}
            </div>
            <div class="text-xs text-white/40">
              <span class="text-white/60">{{ $t('claudeProfiles.enabledLabel') }}:</span>
              <span :class="profile.enabled ? 'text-green-400' : 'text-red-400'">
                {{ profile.enabled ? $t('claudeProfiles.yes') : $t('claudeProfiles.no') }}
              </span>
            </div>
          </div>

          <!-- Actions -->
          <button
            v-if="!profile.is_current"
            class="w-full px-4 py-2.5 rounded-xl bg-[#FF6B35]/10 border border-[#FF6B35]/20 text-[#FF6B35] hover:bg-[#FF6B35]/20 transition-all text-sm font-medium"
            @click="handleApply(profile.name)"
          >
            {{ $t('claudeProfiles.applyProfile') }}
          </button>
          <div
            v-else
            class="w-full px-4 py-2.5 rounded-xl bg-[#FF6B35]/5 border border-[#FF6B35]/10 text-[#FF6B35]/60 text-sm font-medium text-center"
          >
            {{ $t('claudeProfiles.currentlyActive') }}
          </div>
        </div>
      </div>

      <!-- ADD/EDIT MODAL -->
      <Teleport to="body">
        <div
          v-if="showForm"
          class="fixed inset-0 z-50 flex items-center justify-center p-4"
        >
          <div
            class="absolute inset-0 bg-black/60 backdrop-blur-sm"
            @click="showForm = false"
          />
          <div class="relative w-full max-w-lg rounded-3xl bg-[#1a1025] border border-white/10 p-8 shadow-2xl">
            <h2 class="text-xl font-bold text-white mb-6">
              {{ isEditing ? $t('claudeProfiles.editProfileTitle') : $t('claudeProfiles.newProfileTitle') }}
            </h2>

            <!-- Name -->
            <div class="mb-4">
              <label class="block text-white/60 text-sm mb-1.5">{{ $t('claudeProfiles.nameLabel') }}</label>
              <input
                v-model="form.name"
                type="text"
                :placeholder="$t('claudeProfiles.namePlaceholder')"
                class="w-full px-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-white placeholder-white/30 focus:border-[#FF6B35]/50 focus:outline-none transition-colors"
              >
            </div>

            <!-- Description -->
            <div class="mb-4">
              <label class="block text-white/60 text-sm mb-1.5">{{ $t('claudeProfiles.descLabel') }}</label>
              <input
                v-model="form.description"
                type="text"
                :placeholder="$t('claudeProfiles.descPlaceholder')"
                class="w-full px-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-white placeholder-white/30 focus:border-[#FF6B35]/50 focus:outline-none transition-colors"
              >
            </div>

            <!-- Tags -->
            <div class="mb-4">
              <label class="block text-white/60 text-sm mb-1.5">{{ $t('claudeProfiles.tagsLabel') }}</label>
              <input
                v-model="form.tagsInput"
                type="text"
                :placeholder="$t('claudeProfiles.tagsPlaceholder')"
                class="w-full px-4 py-2.5 rounded-xl bg-white/5 border border-white/10 text-white placeholder-white/30 focus:border-[#FF6B35]/50 focus:outline-none transition-colors"
              >
            </div>

            <!-- Snapshot from current -->
            <div
              v-if="!isEditing"
              class="mb-6"
            >
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="form.snapshotFromCurrent"
                  type="checkbox"
                  class="w-4 h-4 rounded accent-[#FF6B35]"
                >
                <span class="text-white/80 text-sm">{{ $t('claudeProfiles.snapshotFromCurrent') }}</span>
              </label>
              <p class="text-white/40 text-xs mt-1 ml-7">
                {{ $t('claudeProfiles.snapshotHint') }}
              </p>
            </div>

            <!-- Re-snapshot for edit -->
            <div
              v-if="isEditing"
              class="mb-6"
            >
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="form.snapshotFromCurrent"
                  type="checkbox"
                  class="w-4 h-4 rounded accent-[#FF6B35]"
                >
                <span class="text-white/80 text-sm">{{ $t('claudeProfiles.reSnapshot') }}</span>
              </label>
            </div>

            <!-- Actions -->
            <div class="flex items-center justify-end gap-3">
              <button
                class="px-5 py-2.5 rounded-xl bg-white/5 border border-white/10 text-white/70 hover:bg-white/10 transition-all text-sm"
                @click="showForm = false"
              >
                {{ $t('claudeProfiles.cancel') }}
              </button>
              <button
                class="px-5 py-2.5 rounded-xl bg-[#FF6B35]/20 border border-[#FF6B35]/30 text-[#FF6B35] hover:bg-[#FF6B35]/30 transition-all text-sm font-medium disabled:opacity-50"
                :disabled="!form.name.trim()"
                @click="handleSave()"
              >
                {{ isEditing ? $t('claudeProfiles.save') : $t('claudeProfiles.create') }}
              </button>
            </div>
          </div>
        </div>
      </Teleport>
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
