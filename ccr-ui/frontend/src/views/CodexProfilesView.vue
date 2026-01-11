<template>
  <div class="min-h-screen bg-bg-base p-6">
    <div class="max-w-[1800px] mx-auto">
      <Breadcrumb
        :items="[
          { label: $t('common.home'), path: '/', icon: Home },
          { label: 'Codex', path: '/codex', icon: Boxes },
          { label: $t('codex.profiles.breadcrumb'), path: '/codex/profiles', icon: Settings }
        ]"
        module-color="#ec4899"
      />

      <div class="grid grid-cols-[auto_1fr] gap-6 mt-6">
        <CollapsibleSidebar module="codex" />

        <main class="flex flex-col gap-6 w-full min-w-0">
          <!-- Header Section -->
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <div class="p-2 rounded-xl bg-platform-codex/10">
                <Settings class="w-6 h-6 text-platform-codex" />
              </div>
              <div>
                <h1 class="text-2xl font-bold text-text-primary">
                  {{ $t('codex.profiles.title') }}
                </h1>
                <p class="text-sm text-text-secondary mt-1">
                  {{ $t('codex.profiles.subtitle') }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <RouterLink
                to="/codex"
                class="btn btn-secondary"
              >
                <ArrowLeft class="w-4 h-4" />
                <span>{{ $t('codex.profiles.backToCodex') }}</span>
              </RouterLink>

              <button
                class="btn btn-primary"
                @click="handleAdd"
              >
                <Plus class="w-4 h-4" />
                {{ $t('codex.profiles.addProfile') }}
              </button>
            </div>
          </div>

          <!-- Status Cards -->
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <!-- Current Config -->
            <GuofengCard
              :gradient-border="true"
              glow-color="warning"
              class="relative overflow-hidden group"
            >
              <div class="flex items-center gap-4">
                <div class="p-3 rounded-xl bg-yellow-500/10 text-yellow-500 group-hover:scale-110 transition-transform duration-300">
                  <Zap class="w-6 h-6" />
                </div>
                <div>
                  <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-1">
                    {{ $t('codex.status.currentConfig') }}
                  </p>
                  <p class="text-xl font-bold text-text-primary truncate">
                    {{ currentProfile || $t('codex.status.notSet') }}
                  </p>
                </div>
              </div>
            </GuofengCard>

            <!-- Total Profiles -->
            <GuofengCard
              :interactive="true"
              glow-color="primary"
              class="group"
            >
              <div class="flex items-center gap-4">
                <div class="p-3 rounded-xl bg-indigo-500/10 text-indigo-500 group-hover:scale-110 transition-transform duration-300">
                  <Layers class="w-6 h-6" />
                </div>
                <div>
                  <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-1">
                    {{ $t('codex.status.totalProfiles') }}
                  </p>
                  <p class="text-xl font-bold text-text-primary">
                    {{ profiles.length }}
                  </p>
                </div>
              </div>
            </GuofengCard>

            <!-- Config Mode -->
            <GuofengCard
              :interactive="true"
              :glow-color="currentConfigMode === 'official' ? 'success' : 'secondary'"
              class="group"
            >
              <div class="flex items-center gap-4">
                <div 
                  class="p-3 rounded-xl transition-colors duration-300 group-hover:scale-110 transition-transform"
                  :class="currentConfigMode === 'official' ? 'bg-emerald-500/10 text-emerald-500' : 'bg-pink-500/10 text-pink-500'"
                >
                  <component
                    :is="currentConfigMode === 'official' ? Globe : Server"
                    class="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-1">
                    {{ $t('codex.status.configMode') }}
                  </p>
                  <p class="text-xl font-bold text-text-primary">
                    {{ currentConfigMode === 'official' ? $t('codex.profiles.officialConfig') : $t('codex.profiles.customRelay') }}
                  </p>
                </div>
              </div>
            </GuofengCard>
          </div>

          <!-- Quick Switch -->
          <GuofengCard
            v-if="profiles.length > 0"
            padding="lg"
          >
            <div class="flex items-center gap-2 mb-4">
              <Shuffle class="w-5 h-5 text-platform-codex" />
              <h3 class="text-base font-semibold text-text-primary">
                {{ $t('codex.profiles.quickSwitch') }}
              </h3>
            </div>
            <div class="flex flex-wrap gap-3">
              <button
                v-for="profile in profiles"
                :key="profile.name"
                class="group relative px-4 py-2.5 rounded-xl font-medium text-sm transition-all duration-300 border flex items-center gap-2.5"
                :class="[
                  profile.name === currentProfile
                    ? 'bg-platform-codex/10 border-platform-codex/50 text-platform-codex shadow-[0_0_15px_rgba(245,158,11,0.2)]'
                    : 'bg-bg-surface border-border-default text-text-secondary hover:border-platform-codex/30 hover:bg-bg-overlay'
                ]"
                @click="handleApply(profile.name)"
              >
                <Star
                  v-if="isOfficialConfig(profile)"
                  class="w-3.5 h-3.5"
                  :class="profile.name === currentProfile ? 'text-platform-codex' : 'text-yellow-500'"
                />
                <span>{{ profile.name }}</span>
                <div 
                  v-if="profile.name === currentProfile" 
                  class="flex items-center justify-center w-4 h-4 rounded-full bg-platform-codex text-white text-[10px]"
                >
                  <Check
                    class="w-2.5 h-2.5"
                    stroke-width="3"
                  />
                </div>
              </button>
            </div>
          </GuofengCard>

          <!-- Profile List Title -->
          <div class="flex items-center justify-between">
            <h2 class="text-xl font-bold text-text-primary flex items-center gap-2">
              <ListFilter class="w-5 h-5 text-platform-codex" />
              {{ $t('codex.profiles.listTitle') }}
            </h2>
          </div>
            
          <!-- Loading State -->
          <div
            v-if="loading"
            class="flex justify-center py-20"
          >
            <div class="w-12 h-12 rounded-full border-4 border-transparent border-t-accent-primary border-r-accent-secondary animate-spin" />
          </div>

          <!-- Empty State -->
          <div
            v-else-if="profiles.length === 0"
            class="empty-state bg-bg-elevated rounded-2xl border border-border-subtle"
          >
            <div class="p-4 rounded-full bg-bg-surface mb-4">
              <Boxes class="w-8 h-8 text-text-muted" />
            </div>
            <p class="text-text-secondary">
              {{ $t('codex.profiles.emptyState') }}
            </p>
          </div>

          <!-- Profile Grid -->
          <div
            v-else
            class="grid grid-cols-1 xl:grid-cols-2 gap-4"
          >
            <GuofengCard 
              v-for="profile in profiles" 
              :key="profile.name"
              class="group relative overflow-hidden transition-all duration-300 hover:-translate-y-1 hover:shadow-xl"
              :class="{ 'ring-1 ring-platform-codex/50': currentProfile && profile.name === currentProfile }"
              :glow-color="currentProfile && profile.name === currentProfile ? 'warning' : 'primary'"
              padding="lg"
            >
              <!-- Active Indicator Background -->
              <div 
                v-if="currentProfile && profile.name === currentProfile"
                class="absolute top-0 right-0 w-32 h-32 bg-gradient-to-bl from-platform-codex/10 to-transparent -mr-8 -mt-8 rounded-bl-full pointer-events-none"
              />

              <div class="relative z-10">
                <div class="flex items-start justify-between gap-4 mb-4">
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                      <h3 class="text-lg font-bold font-mono text-text-primary truncate">
                        {{ profile.name }}
                      </h3>
                      <span 
                        v-if="currentProfile && profile.name === currentProfile"
                        class=" badge badge-primary"
                      >
                        {{ $t('codex.profiles.currentBadge') }}
                      </span>
                      <span 
                        v-else-if="profile.enabled === false"
                        class="badge badge-danger"
                      >
                        {{ $t('codex.states.disabled') }}
                      </span>
                      <span 
                        v-else
                        class="badge badge-success"
                      >
                        {{ $t('codex.states.enabled') }}
                      </span>
                    </div>
                    <p
                      v-if="profile.description"
                      class="text-sm text-text-secondary line-clamp-1"
                    >
                      {{ profile.description }}
                    </p>
                  </div>
                   
                  <!-- Actions -->
                  <div class="flex items-center gap-1 opacity-100 xl:opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                    <button 
                      class="p-2 rounded-lg hover:bg-bg-overlay text-accent-success transition-colors"
                      :title="$t('codex.profiles.apply')"
                      @click.stop="handleApply(profile.name)"
                    >
                      <Check class="w-4 h-4" />
                    </button>
                    <button 
                      class="p-2 rounded-lg hover:bg-bg-overlay text-accent-primary transition-colors"
                      :title="$t('codex.actions.edit')"
                      @click.stop="handleEdit(profile.name)"
                    >
                      <Edit2 class="w-4 h-4" />
                    </button>
                    <button 
                      class="p-2 rounded-lg hover:bg-bg-overlay text-accent-danger transition-colors"
                      :title="$t('codex.actions.delete')"
                      @click.stop="handleDelete(profile.name)"
                    >
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </div>

                <!-- Info Grid -->
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-y-3 gap-x-6 text-sm">
                  <div class="flex flex-col gap-1">
                    <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.baseUrl') }}
                    </span>
                    <code class="font-mono text-text-primary truncate px-2 py-1 rounded bg-bg-surface border border-border-subtle">
                      {{ profile.base_url }}
                    </code>
                  </div>

                  <div class="flex flex-col gap-1">
                    <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                      {{ $t('codex.profiles.fields.model') }}
                    </span>
                    <div class="flex items-center gap-2">
                      <span class="font-mono text-accent-primary bg-accent-primary/5 px-2 py-0.5 rounded">
                        {{ profile.model }}
                      </span>
                    </div>
                  </div>
                </div>
                 
                <div
                  v-if="profile.tags?.length || profile.provider"
                  class="mt-4 flex items-center justify-between border-t border-border-subtle pt-3"
                >
                  <div class="flex flex-wrap gap-1.5">
                    <span 
                      v-if="profile.provider"
                      class="px-2 py-0.5 rounded-md text-xs font-medium bg-bg-surface text-text-secondary border border-border-subtle"
                    >
                      {{ profile.provider }}
                    </span>
                    <span 
                      v-for="tag in profile.tags" 
                      :key="tag"
                      class="px-2 py-0.5 rounded-md text-xs font-medium bg-bg-surface text-text-muted border border-border-subtle"
                    >
                      #{{ tag }}
                    </span>
                  </div>
                   
                  <div
                    v-if="profile.extra && Object.keys(profile.extra).length > 0"
                    class="text-xs text-text-muted font-mono bg-bg-surface px-2 py-1 rounded"
                  >
                    +{{ Object.keys(profile.extra).length }} extras
                  </div>
                </div>
              </div>
            </GuofengCard>
          </div>
            
          <!-- Add/Edit Modal -->
          <div
            v-if="showForm"
            class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50"
          >
            <GuofengCard
              class="w-full max-w-3xl max-h-[90vh] overflow-y-auto !p-0 shadow-2xl animate-in zoom-in-95 duration-200"
              :padding="'none'"
            >
              <!-- Modal Header -->
              <div class="px-6 py-4 border-b border-border-subtle flex items-center justify-between sticky top-0 bg-bg-elevated/95 backdrop-blur z-10">
                <h2 class="text-xl font-bold text-text-primary">
                  {{ editingName ? $t('codex.profiles.editProfile') : $t('codex.profiles.addProfile') }}
                </h2>
                <button
                  class="p-1 rounded-lg hover:bg-bg-overlay text-text-muted transition-colors"
                  @click="handleCloseForm"
                >
                  <X class="w-5 h-5" />
                </button>
              </div>

              <!-- Modal Content -->
              <div class="p-6 space-y-6">
                <!-- Use generic grid for form -->
                <div class="grid grid-cols-1 gap-6">
                  <!-- Name & Desc -->
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="space-y-1.5">
                      <label class="text-sm font-semibold text-text-secondary">
                        {{ $t('codex.profiles.fields.name') }} <span class="text-red-500">*</span>
                      </label>
                      <input
                        v-model="form.name"
                        :disabled="!!editingName"
                        type="text"
                        class="input"
                        :placeholder="$t('codex.profiles.placeholders.name')"
                      >
                    </div>
                    <div class="space-y-1.5">
                      <label class="text-sm font-semibold text-text-secondary">
                        {{ $t('codex.profiles.fields.description') }}
                      </label>
                      <input
                        v-model="form.description"
                        type="text"
                        class="input"
                        :placeholder="$t('codex.profiles.placeholders.description')"
                      >
                    </div>
                  </div>
                       
                  <!-- URL & Token -->
                  <div class="space-y-1.5">
                    <label class="text-sm font-semibold text-text-secondary">
                      {{ $t('codex.profiles.fields.baseUrl') }} <span class="text-red-500">*</span>
                    </label>
                    <input
                      v-model="form.base_url"
                      type="text"
                      class="input font-mono text-sm"
                      :placeholder="$t('codex.profiles.placeholders.baseUrl')"
                    >
                  </div>

                  <div class="space-y-1.5">
                    <label class="text-sm font-semibold text-text-secondary">
                      {{ $t('codex.profiles.fields.authToken') }} <span class="text-red-500">*</span>
                    </label>
                    <input
                      v-model="form.auth_token"
                      type="password"
                      class="input font-mono text-sm"
                      :placeholder="$t('codex.profiles.placeholders.authToken')"
                    >
                  </div>

                  <!-- Models -->
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="space-y-1.5">
                      <label class="text-sm font-semibold text-text-secondary">
                        {{ $t('codex.profiles.fields.model') }} <span class="text-red-500">*</span>
                      </label>
                      <input
                        v-model="form.model"
                        type="text"
                        class="input font-mono text-sm"
                        :placeholder="$t('codex.profiles.placeholders.model')"
                      >
                    </div>
                    <div class="space-y-1.5">
                      <label class="text-sm font-semibold text-text-secondary">
                        {{ $t('codex.profiles.fields.smallFastModel') }}
                      </label>
                      <input
                        v-model="form.small_fast_model"
                        type="text"
                        class="input font-mono text-sm"
                        :placeholder="$t('codex.profiles.placeholders.smallFastModel')"
                      >
                    </div>
                  </div>
                       
                  <!-- Metadata -->
                  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div class="space-y-1.5">
                      <label class="text-sm font-semibold text-text-secondary">
                        {{ $t('codex.profiles.fields.provider') }}
                      </label>
                      <input
                        v-model="form.provider"
                        type="text"
                        class="input"
                        :placeholder="$t('codex.profiles.placeholders.provider')"
                      >
                    </div>
                    <div class="space-y-1.5">
                      <label class="text-sm font-semibold text-text-secondary">
                        {{ $t('codex.profiles.fields.providerType') }}
                      </label>
                      <input
                        v-model="form.provider_type"
                        type="text"
                        class="input"
                        :placeholder="$t('codex.profiles.placeholders.providerType')"
                      >
                    </div>
                    <div class="space-y-1.5">
                      <label class="text-sm font-semibold text-text-secondary">
                        {{ $t('codex.profiles.fields.tags') }}
                      </label>
                      <input
                        v-model="tagsText"
                        type="text"
                        class="input"
                        :placeholder="$t('codex.profiles.placeholders.tags')"
                      >
                    </div>
                  </div>
                       
                  <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface border border-border-subtle">
                    <input
                      id="profileEnabled"
                      v-model="form.enabled"
                      type="checkbox"
                      class="w-5 h-5 rounded border-border-default text-accent-primary focus:ring-accent-primary/20"
                    >
                    <label
                      for="profileEnabled"
                      class="text-sm font-medium text-text-primary cursor-pointer select-none"
                    >
                      {{ $t('codex.profiles.fields.enabled') }}
                    </label>
                  </div>
                       
                  <!-- Extra JSON -->
                  <div class="space-y-1.5">
                    <label class="text-sm font-semibold text-text-secondary flex justify-between">
                      <span>{{ $t('codex.profiles.fields.extraJson') }}</span>
                      <span class="text-xs font-normal text-text-muted">{{ $t('codex.profiles.extraHint') }}</span>
                    </label>
                    <textarea
                      v-model="extraText"
                      rows="6"
                      class="input font-mono text-xs leading-relaxed"
                      :placeholder="$t('codex.profiles.placeholders.extraJson')"
                    />
                  </div>
                </div>
              </div>

              <!-- Footer -->
              <div class="px-6 py-4 border-t border-border-subtle flex justify-end gap-3 bg-bg-surface/50">
                <button
                  class="btn btn-secondary"
                  @click="handleCloseForm"
                >
                  {{ $t('codex.actions.cancel') }}
                </button>
                <button 
                  class="btn btn-primary"
                  :disabled="saving"
                  @click="handleSave"
                >
                  <span
                    v-if="saving"
                    class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2"
                  />
                  {{ saving ? $t('codex.states.saving') : $t('codex.actions.save') }}
                </button>
              </div>
            </GuofengCard>
          </div>
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ArrowLeft, Boxes, Check, Edit2, Globe, Home, Layers, ListFilter, Plus, Server, Settings, Shuffle, Star, Trash2, Zap, X } from 'lucide-vue-next'

import Breadcrumb from '@/components/Breadcrumb.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import GuofengCard from '@/components/common/GuofengCard.vue'
import { addCodexProfile, applyCodexProfile, deleteCodexProfile, getCodexProfile, listCodexProfiles, updateCodexProfile } from '@/api'
import type { CodexProfile, CodexProfileRequest } from '@/types'

const { t } = useI18n()

const loading = ref(false)
const saving = ref(false)

const profiles = ref<CodexProfile[]>([])
const currentProfile = ref<string | null>(null)

const showForm = ref(false)
const editingName = ref<string | null>(null)

const tagsText = ref('')
const extraText = ref('{}')

// 判断是否为官方配置（无 base_url）
const isOfficialConfig = (profile: CodexProfile) => {
  return !profile.base_url || profile.base_url.trim() === ''
}

// 当前配置模式
const currentConfigMode = computed(() => {
  if (!currentProfile.value) return 'official'
  const profile = profiles.value.find(p => p.name === currentProfile.value)
  return profile && isOfficialConfig(profile) ? 'official' : 'custom'
})

const form = reactive<Required<Pick<CodexProfileRequest, 'name' | 'base_url' | 'auth_token' | 'model'>> & Partial<CodexProfileRequest>>({
  name: '',
  description: '',
  base_url: '',
  auth_token: '',
  model: '',
  small_fast_model: '',
  provider: '',
  provider_type: '',
  account: '',
  tags: [],
  enabled: true,
  extra: {},
})

const loadProfiles = async () => {
  try {
    loading.value = true
    const data = await listCodexProfiles()
    profiles.value = data.profiles || []
    currentProfile.value = data.current_profile ?? null
  } catch (error) {
    console.error('Failed to load codex profiles:', error)
    alert(t('codex.states.loadFailed'))
  } finally {
    loading.value = false
  }
}

const resetForm = () => {
  form.name = ''
  form.description = ''
  form.base_url = ''
  form.auth_token = ''
  form.model = ''
  form.small_fast_model = ''
  form.provider = ''
  form.provider_type = ''
  form.account = ''
  form.tags = []
  form.enabled = true
  form.extra = {}
  tagsText.value = ''
  extraText.value = JSON.stringify({}, null, 2)
}

const handleAdd = () => {
  editingName.value = null
  resetForm()
  showForm.value = true
}

const handleEdit = async (name: string) => {
  try {
    editingName.value = name
    resetForm()
    showForm.value = true

    const profile = await getCodexProfile(name)
    form.name = profile.name
    form.description = profile.description || ''
    form.base_url = profile.base_url || ''
    form.auth_token = profile.auth_token || ''
    form.model = profile.model || ''
    form.small_fast_model = profile.small_fast_model || ''
    form.provider = profile.provider || ''
    form.provider_type = profile.provider_type || ''
    form.account = profile.account || ''
    form.tags = profile.tags || []
    form.enabled = profile.enabled !== false
    form.extra = profile.extra || {}

    tagsText.value = (form.tags || []).join(', ')
    extraText.value = JSON.stringify(form.extra || {}, null, 2)
  } catch (error) {
    console.error('Failed to load codex profile:', error)
    alert(t('codex.states.loadFailed'))
    showForm.value = false
  }
}

const handleCloseForm = () => {
  showForm.value = false
  editingName.value = null
}

const parseTags = (raw: string): string[] | undefined => {
  const tags = raw
    .split(',')
    .map(s => s.trim())
    .filter(Boolean)
  return tags.length > 0 ? tags : undefined
}

const parseExtraJson = (raw: string): Record<string, any> | undefined => {
  const trimmed = raw.trim()
  if (!trimmed) return undefined
  const parsed = JSON.parse(trimmed)
  if (parsed === null || typeof parsed !== 'object' || Array.isArray(parsed)) {
    throw new Error('extra must be a JSON object')
  }
  return parsed
}

const handleSave = async () => {
  if (!form.name.trim()) {
    alert(t('codex.profiles.validation.nameRequired'))
    return
  }
  if (!form.base_url.trim()) {
    alert(t('codex.profiles.validation.baseUrlRequired'))
    return
  }
  if (!form.auth_token.trim()) {
    alert(t('codex.profiles.validation.authTokenRequired'))
    return
  }
  if (!form.model.trim()) {
    alert(t('codex.profiles.validation.modelRequired'))
    return
  }

  let extra: Record<string, any> | undefined
  try {
    extra = parseExtraJson(extraText.value) || undefined
  } catch {
    alert(t('codex.profiles.validation.extraJsonInvalid'))
    return
  }

  const request: CodexProfileRequest = {
    name: form.name.trim(),
    description: form.description?.trim() ? form.description.trim() : undefined,
    base_url: form.base_url.trim(),
    auth_token: form.auth_token.trim(),
    model: form.model.trim(),
    small_fast_model: form.small_fast_model?.trim() ? form.small_fast_model.trim() : undefined,
    provider: form.provider?.trim() ? form.provider.trim() : undefined,
    provider_type: form.provider_type?.trim() ? form.provider_type.trim() : undefined,
    account: form.account?.trim() ? form.account.trim() : undefined,
    tags: parseTags(tagsText.value),
    enabled: form.enabled === true,
    extra,
  }

  try {
    saving.value = true
    if (editingName.value) {
      await updateCodexProfile(editingName.value, request)
    } else {
      await addCodexProfile(request)
    }
    handleCloseForm()
    await loadProfiles()
  } catch (error) {
    console.error('Failed to save codex profile:', error)
    alert(t('codex.states.saveFailed'))
  } finally {
    saving.value = false
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('codex.profiles.confirmDelete', { name }))) return
  try {
    await deleteCodexProfile(name)
    await loadProfiles()
  } catch (error) {
    console.error('Failed to delete codex profile:', error)
    alert(t('codex.states.deleteFailed'))
  }
}

const handleApply = async (name: string) => {
  if (!confirm(t('codex.profiles.confirmApply', { name }))) return
  try {
    await applyCodexProfile(name)
    await loadProfiles()
  } catch (error) {
    console.error('Failed to apply codex profile:', error)
    alert(t('codex.states.saveFailed'))
  }
}

onMounted(async () => {
  await loadProfiles()
})
</script>
