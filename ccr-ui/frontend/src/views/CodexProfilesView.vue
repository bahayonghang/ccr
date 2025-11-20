<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="max-w-[1800px] mx-auto">
      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          { label: $t('common.home'), path: '/', icon: Home },
          { label: 'Codex', path: '/codex', icon: Boxes },
          { label: $t('codex.profiles.breadcrumb'), path: '/codex/profiles', icon: Users }
        ]"
        module-color="#ec4899"
      />
      
      <div class="grid grid-cols-[auto_1fr] gap-4">
        <CollapsibleSidebar module="codex" />

        <main
          class="rounded-xl p-6 glass-effect"
          :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
        >
          <!-- Header Section -->
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <div
                class="p-2 rounded-lg"
                :style="{ background: 'linear-gradient(135deg, rgba(99, 102, 241, 0.1), rgba(236, 72, 153, 0.1))' }"
              >
                <Users
                  class="w-6 h-6"
                  :style="{ color: 'var(--accent-primary)' }"
                />
              </div>
              <div>
                <h1
                  class="text-2xl font-bold"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  Codex Profiles
                </h1>
                <p
                  class="text-sm"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  {{ $t('codex.profiles.subtitle') }}
                </p>
              </div>
              <span
                class="px-3 py-1 rounded-full text-sm font-medium"
                :style="{ background: 'var(--accent-primary)', color: '#fff' }"
              >{{ profiles.length }}</span>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink
                to="/codex"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-all hover:scale-105"
                :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }"
              >
                <ArrowLeft class="w-4 h-4" />
                <span>{{ $t('codex.profiles.backToCodex') }}</span>
              </RouterLink>
              <button
                class="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2 transition-all hover:scale-105"
                :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }"
                @click="handleAdd"
              >
                <Plus class="w-4 h-4" />
                {{ $t('codex.profiles.addProfile') }}
              </button>
            </div>
          </div>

          <!-- Loading State -->
          <div
            v-if="loading"
            class="flex justify-center py-20"
          >
            <div
              class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
              :style="{ borderTopColor: 'var(--accent-primary)', borderRightColor: 'var(--accent-secondary)' }"
            />
          </div>

          <!-- Profiles List -->
          <div
            v-else
            class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
          >
            <div
              v-if="!profiles || profiles.length === 0"
              class="col-span-full text-center py-20"
            >
              <Users class="w-16 h-16 mx-auto mb-4 opacity-20" />
              <p
                class="text-lg font-medium"
                :style="{ color: 'var(--text-muted)' }"
              >
                {{ $t('codex.profiles.emptyState') }}
              </p>
              <p
                class="text-sm mt-2"
                :style="{ color: 'var(--text-muted)' }"
              >
                {{ $t('codex.profiles.emptyHint') }}
              </p>
            </div>

            <div
              v-for="profile in profiles"
              :key="profile.name"
              class="group rounded-xl p-5 transition-all duration-300 relative overflow-hidden"
              :style="{ background: 'rgba(255, 255, 255, 0.7)', border: '1px solid rgba(99, 102, 241, 0.12)' }"
              @mouseenter="(e) => onCardHover(e.currentTarget, true)"
              @mouseleave="(e) => onCardHover(e.currentTarget, false)"
            >
              <!-- Provider Badge -->
              <div class="absolute top-3 right-3">
                <span
                  v-if="profile.provider"
                  class="px-2 py-1 rounded text-xs font-semibold uppercase"
                  :style="{ background: getProviderColor(profile.provider), color: 'white' }"
                >
                  {{ profile.provider }}
                </span>
              </div>

              <!-- Profile Name & Description -->
              <div class="mb-4">
                <h3
                  class="text-lg font-bold font-mono mb-1"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ profile.name }}
                </h3>
                <p
                  v-if="profile.description"
                  class="text-sm italic"
                  :style="{ color: 'var(--text-muted)' }"
                >
                  {{ profile.description }}
                </p>
              </div>

              <!-- Profile Details -->
              <div class="space-y-2 text-sm mb-4">
                <div class="flex items-start gap-2">
                  <span
                    class="text-xs font-medium mt-0.5"
                    :style="{ color: 'var(--text-muted)', minWidth: '50px' }"
                  >URL:</span>
                  <code
                    class="flex-1 px-2 py-1 rounded font-mono text-xs break-all"
                    :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                  >
                    {{ profile.base_url }}
                  </code>
                </div>

                <div class="flex items-start gap-2">
                  <span
                    class="text-xs font-medium mt-0.5"
                    :style="{ color: 'var(--text-muted)', minWidth: '50px' }"
                  >Token:</span>
                  <code
                    class="flex-1 px-2 py-1 rounded font-mono text-xs"
                    :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }"
                  >
                    {{ maskToken(profile.auth_token) }}
                  </code>
                </div>

                <div class="flex items-start gap-2">
                  <span
                    class="text-xs font-medium mt-0.5"
                    :style="{ color: 'var(--text-muted)', minWidth: '50px' }"
                  >Model:</span>
                  <code
                    class="flex-1 px-2 py-1 rounded font-mono text-xs"
                    :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                  >
                    {{ profile.model }}
                  </code>
                </div>

                <div
                  v-if="profile.small_fast_model"
                  class="flex items-start gap-2"
                >
                  <span
                    class="text-xs font-medium mt-0.5"
                    :style="{ color: 'var(--text-muted)', minWidth: '50px' }"
                  >Fast:</span>
                  <code
                    class="flex-1 px-2 py-1 rounded font-mono text-xs"
                    :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }"
                  >
                    {{ profile.small_fast_model }}
                  </code>
                </div>
              </div>

              <!-- Action Buttons -->
              <div
                class="flex gap-2 mt-4 pt-4 border-t"
                :style="{ borderColor: 'var(--border-color)' }"
              >
                <button
                  class="flex-1 p-2 rounded-lg transition-all hover:scale-105 flex items-center justify-center gap-1"
                  :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }"
                  :title="$t('codex.actions.edit')"
                  @click="handleEdit(profile)"
                >
                  <Edit2 class="w-4 h-4" />
                  <span class="text-xs font-medium">{{ $t('codex.actions.edit') }}</span>
                </button>
                <button
                  class="flex-1 p-2 rounded-lg transition-all hover:scale-105 flex items-center justify-center gap-1"
                  :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }"
                  :title="$t('codex.actions.delete')"
                  @click="handleDelete(profile.name)"
                >
                  <Trash2 class="w-4 h-4" />
                  <span class="text-xs font-medium">{{ $t('codex.actions.delete') }}</span>
                </button>
              </div>
            </div>
          </div>

          <!-- Add/Edit Form Modal -->
          <div
            v-if="showAddForm"
            class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
            @click="showAddForm = false"
          >
            <div
              class="rounded-xl p-6 max-w-3xl w-full max-h-[90vh] overflow-y-auto"
              :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)' }"
              @click.stop
            >
              <h2
                class="text-2xl font-bold mb-6"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ editingProfile ? $t('codex.profiles.editProfile') : $t('codex.profiles.addProfile') }}
              </h2>

              <div class="space-y-4">
                <!-- Profile Name -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-2"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('codex.profiles.profileName') }} *
                  </label>
                  <input
                    v-model="formData.name"
                    type="text"
                    class="w-full px-4 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('codex.profiles.placeholders.name')"
                    :disabled="!!editingProfile"
                  >
                </div>

                <!-- Description -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-2"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('codex.profiles.description') }}
                  </label>
                  <input
                    v-model="formData.description"
                    type="text"
                    class="w-full px-4 py-2 rounded-lg"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('codex.profiles.placeholders.description')"
                  >
                </div>

                <!-- Base URL -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-2"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('codex.profiles.baseUrl') }} *
                  </label>
                  <input
                    v-model="formData.base_url"
                    type="text"
                    class="w-full px-4 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('codex.profiles.placeholders.baseUrl')"
                  >
                </div>

                <!-- Auth Token -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-2"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('codex.profiles.authToken') }} *
                  </label>
                  <input
                    v-model="formData.auth_token"
                    type="password"
                    class="w-full px-4 py-2 rounded-lg font-mono"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    :placeholder="$t('codex.profiles.placeholders.authToken')"
                  >
                </div>

                <!-- Model -->
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <label
                      class="block text-sm font-semibold mb-2"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ $t('codex.profiles.model') }} *
                    </label>
                    <select
                      v-model="formData.model"
                      class="w-full px-4 py-2 rounded-lg"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    >
                      <option value="gpt-4">
                        GPT-4
                      </option>
                      <option value="gpt-4-turbo">
                        GPT-4 Turbo
                      </option>
                      <option value="gpt-3.5-turbo">
                        GPT-3.5 Turbo
                      </option>
                      <option value="claude-sonnet-4-5-20250929">
                        Claude Sonnet 4.5
                      </option>
                    </select>
                  </div>

                  <div>
                    <label
                      class="block text-sm font-semibold mb-2"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ $t('codex.profiles.fastModel') }}
                    </label>
                    <select
                      v-model="formData.small_fast_model"
                      class="w-full px-4 py-2 rounded-lg"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                    >
                      <option value="">
                        {{ $t('codex.profiles.placeholders.selectFastModel') }}
                      </option>
                      <option value="gpt-3.5-turbo">
                        GPT-3.5 Turbo
                      </option>
                      <option value="gpt-4">
                        GPT-4
                      </option>
                    </select>
                  </div>
                </div>

                <!-- Provider -->
                <div>
                  <label
                    class="block text-sm font-semibold mb-2"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('codex.profiles.provider') }}
                  </label>
                  <select
                    v-model="formData.provider"
                    class="w-full px-4 py-2 rounded-lg"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                  >
                    <option value="">
                      {{ $t('codex.profiles.placeholders.selectProvider') }}
                    </option>
                    <option value="GitHub">
                      {{ $t('codex.profiles.providers.github') }}
                    </option>
                    <option value="Azure">
                      {{ $t('codex.profiles.providers.azure') }}
                    </option>
                    <option value="OpenAI">
                      {{ $t('codex.profiles.providers.openai') }}
                    </option>
                    <option value="Custom">
                      {{ $t('codex.profiles.providers.custom') }}
                    </option>
                  </select>
                </div>
              </div>

              <!-- Action Buttons -->
              <div class="flex gap-3 mt-6">
                <button
                  class="flex-1 px-6 py-3 rounded-lg font-semibold text-white transition-all hover:scale-105"
                  :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }"
                  @click="handleSubmit"
                >
                  {{ editingProfile ? $t('codex.profiles.updateProfile') : $t('codex.profiles.addProfile') }}
                </button>
                <button
                  class="flex-1 px-6 py-3 rounded-lg font-semibold transition-all hover:scale-105"
                  :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }"
                  @click="showAddForm = false"
                >
                  {{ $t('codex.actions.cancel') }}
                </button>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { RouterLink } from 'vue-router'
import { Users, Plus, Edit2, Trash2, ArrowLeft, Home, Boxes } from 'lucide-vue-next'
import { listCodexProfiles, addCodexProfile, updateCodexProfile, deleteCodexProfile } from '@/api/client'
import type { CodexProfile, CodexProfileRequest } from '@/types'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import Breadcrumb from '@/components/Breadcrumb.vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const profiles = ref<CodexProfile[]>([])
const loading = ref(true)
const showAddForm = ref(false)
const editingProfile = ref<CodexProfile | null>(null)
const formData = ref<CodexProfileRequest>({
  name: '',
  description: '',
  base_url: '',
  auth_token: '',
  model: 'gpt-4',
  small_fast_model: '',
  provider: ''
})

const loadProfiles = async () => {
  try {
    loading.value = true
    const data = await listCodexProfiles()
    profiles.value = data || []
  } catch (err) {
    console.error('Failed to load Codex profiles:', err)
    profiles.value = []
    alert(t('codex.profiles.messages.loadFailed'))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadProfiles()
})

const handleAdd = () => {
  showAddForm.value = true
  editingProfile.value = null
  formData.value = {
    name: '',
    description: '',
    base_url: 'https://api.github.com/copilot',
    auth_token: '',
    model: 'gpt-4',
    small_fast_model: 'gpt-3.5-turbo',
    provider: 'GitHub'
  }
}

const handleEdit = (profile: CodexProfile) => {
  editingProfile.value = profile
  showAddForm.value = true
  formData.value = {
    name: profile.name,
    description: profile.description || '',
    base_url: profile.base_url,
    auth_token: profile.auth_token,
    model: profile.model,
    small_fast_model: profile.small_fast_model || '',
    provider: profile.provider || ''
  }
}

const handleSubmit = async () => {
  if (!formData.value.name || !formData.value.base_url || !formData.value.auth_token || !formData.value.model) {
    alert(t('codex.profiles.validation.required'))
    return
  }

  try {
    if (editingProfile.value) {
      await updateCodexProfile(editingProfile.value.name, formData.value)
      alert(t('codex.profiles.messages.updateSuccess'))
    } else {
      await addCodexProfile(formData.value)
      alert(t('codex.profiles.messages.addSuccess'))
    }
    showAddForm.value = false
    await loadProfiles()
  } catch (err) {
    alert(t('codex.profiles.messages.operationFailed', { error: err instanceof Error ? err.message : 'Unknown error' }))
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('codex.profiles.deleteConfirm', { name }))) return

  try {
    await deleteCodexProfile(name)
    alert(t('codex.profiles.messages.deleteSuccess'))
    await loadProfiles()
  } catch (err) {
    alert(t('codex.profiles.messages.deleteFailed', { error: err instanceof Error ? err.message : 'Unknown error' }))
  }
}

const maskToken = (token: string): string => {
  if (!token) return ''
  if (token.length <= 8) return '****'
  return token.slice(0, 4) + '****' + token.slice(-4)
}

const getProviderColor = (provider: string): string => {
  const colors: Record<string, string> = {
    'GitHub': '#6366f1',
    'Azure': '#0078d4',
    'OpenAI': '#10a37f',
    'Custom': '#ec4899'
  }
  return colors[provider] || '#8b5cf6'
}

const onCardHover = (el: HTMLElement, hover: boolean) => {
  if (hover) {
    el.style.background = 'rgba(255, 255, 255, 0.95)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.3)'
    el.style.boxShadow = '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -4px rgba(0, 0, 0, 0.1)'
    el.style.transform = 'translateY(-4px)'
  } else {
    el.style.background = 'rgba(255, 255, 255, 0.7)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.12)'
    el.style.boxShadow = 'none'
    el.style.transform = 'translateY(0)'
  }
}
</script>
