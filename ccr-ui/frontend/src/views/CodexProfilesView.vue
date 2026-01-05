<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="max-w-[1800px] mx-auto">
      <Breadcrumb
        :items="[
          { label: $t('common.home'), path: '/', icon: Home },
          { label: 'Codex', path: '/codex', icon: Boxes },
          { label: $t('codex.profiles.breadcrumb'), path: '/codex/profiles', icon: Settings }
        ]"
        module-color="#ec4899"
      />

      <div class="grid grid-cols-[auto_1fr] gap-4">
        <CollapsibleSidebar module="codex" />

        <main
          class="rounded-xl p-6 glass-effect"
          :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }"
        >
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <Settings
                class="w-6 h-6"
                :style="{ color: 'var(--accent-primary)' }"
              />
              <div>
                <h1
                  class="text-2xl font-bold"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('codex.profiles.title') }}
                </h1>
                <p
                  class="text-sm mt-1"
                  :style="{ color: 'var(--text-secondary)' }"
                >
                  {{ $t('codex.profiles.subtitle') }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <RouterLink
                to="/codex"
                class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors"
                :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }"
              >
                <ArrowLeft class="w-4 h-4" />
                <span>{{ $t('codex.profiles.backToCodex') }}</span>
              </RouterLink>

              <button
                class="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2"
                :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }"
                @click="handleAdd"
              >
                <Plus class="w-4 h-4" />
                {{ $t('codex.profiles.addProfile') }}
              </button>
            </div>

            <!-- 状态卡片区 -->
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
              <!-- 当前配置 -->
              <div
                class="rounded-xl p-4 glass-effect"
                :style="{ border: '2px solid rgba(245, 158, 11, 0.3)', background: 'rgba(245, 158, 11, 0.05)' }"
              >
                <div class="flex items-center gap-3">
                  <div
                    class="p-2 rounded-lg"
                    :style="{ background: 'rgba(245, 158, 11, 0.15)' }"
                  >
                    <Zap
                      class="w-5 h-5"
                      :style="{ color: 'var(--platform-codex)' }"
                    />
                  </div>
                  <div class="flex-1 min-w-0">
                    <p
                      class="text-xs font-medium"
                      :style="{ color: 'var(--text-muted)' }"
                    >
                      {{ $t('codex.status.currentConfig') }}
                    </p>
                    <p
                      class="text-lg font-bold truncate"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      {{ currentProfile || $t('codex.status.notSet') }}
                    </p>
                  </div>
                </div>
              </div>

              <!-- 配置总数 -->
              <div
                class="rounded-xl p-4 glass-effect"
                :style="{ border: '1px solid var(--border-color)' }"
              >
                <div class="flex items-center gap-3">
                  <div
                    class="p-2 rounded-lg"
                    :style="{ background: 'rgba(139, 92, 246, 0.15)' }"
                  >
                    <Layers
                      class="w-5 h-5"
                      :style="{ color: 'var(--platform-claude)' }"
                    />
                  </div>
                  <div class="flex-1">
                    <p
                      class="text-xs font-medium"
                      :style="{ color: 'var(--text-muted)' }"
                    >
                      {{ $t('codex.status.totalProfiles') }}
                    </p>
                    <p
                      class="text-lg font-bold"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      {{ profiles.length }}
                    </p>
                  </div>
                </div>
              </div>

              <!-- 配置模式 -->
              <div
                class="rounded-xl p-4 glass-effect"
                :style="{ border: '1px solid var(--border-color)' }"
              >
                <div class="flex items-center gap-3">
                  <div
                    class="p-2 rounded-lg"
                    :style="{ background: currentConfigMode === 'official' ? 'rgba(16, 185, 129, 0.15)' : 'rgba(236, 72, 153, 0.15)' }"
                  >
                    <component
                      :is="currentConfigMode === 'official' ? Globe : Server"
                      class="w-5 h-5"
                      :style="{ color: currentConfigMode === 'official' ? 'var(--accent-success)' : 'var(--accent-tertiary)' }"
                    />
                  </div>
                  <div class="flex-1">
                    <p
                      class="text-xs font-medium"
                      :style="{ color: 'var(--text-muted)' }"
                    >
                      {{ $t('codex.status.configMode') }}
                    </p>
                    <p
                      class="text-lg font-bold"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      {{ currentConfigMode === 'official' ? $t('codex.profiles.officialConfig') : $t('codex.profiles.customRelay') }}
                    </p>
                  </div>
                </div>
              </div>
            </div>

            <!-- 快速切换区 -->
            <div
              v-if="profiles.length > 0"
              class="rounded-xl p-4 mb-6 glass-effect"
              :style="{ border: '1px solid var(--border-color)' }"
            >
              <div class="flex items-center gap-2 mb-3">
                <Shuffle
                  class="w-4 h-4"
                  :style="{ color: 'var(--platform-codex)' }"
                />
                <span
                  class="text-sm font-semibold"
                  :style="{ color: 'var(--text-secondary)' }"
                >{{ $t('codex.profiles.quickSwitch') }}</span>
              </div>
              <div class="flex flex-wrap gap-2">
                <button
                  v-for="profile in profiles"
                  :key="profile.name"
                  class="px-4 py-2 rounded-lg font-medium text-sm transition-all duration-200 flex items-center gap-2 cursor-pointer"
                  :style="{
                    background: profile.name === currentProfile ? 'rgba(245, 158, 11, 0.15)' : 'var(--bg-secondary)',
                    border: profile.name === currentProfile ? '2px solid rgba(245, 158, 11, 0.5)' : '1px solid var(--border-color)',
                    color: profile.name === currentProfile ? 'var(--platform-codex)' : 'var(--text-secondary)'
                  }"
                  @click="handleApply(profile.name)"
                >
                  <Star
                    v-if="isOfficialConfig(profile)"
                    class="w-3.5 h-3.5"
                    :style="{ color: 'var(--accent-success)' }"
                  />
                  <span>{{ profile.name }}</span>
                  <Check
                    v-if="profile.name === currentProfile"
                    class="w-4 h-4"
                  />
                </button>
              </div>
            </div>

            <!-- 配置列表标题 -->
            <div class="flex items-center justify-between mb-4">
              <h2
                class="text-lg font-bold flex items-center gap-2"
                :style="{ color: 'var(--text-primary)' }"
              >
                <ListFilter
                  class="w-5 h-5"
                  :style="{ color: 'var(--platform-codex)' }"
                />
                {{ $t('codex.profiles.listTitle') }}
              </h2>
            </div>

            <div
              v-if="loading"
              class="flex justify-center py-20"
            >
              <div
                class="w-12 h-12 rounded-full border-4 border-transparent animate-spin"
                :style="{ borderTopColor: 'var(--accent-primary)', borderRightColor: 'var(--accent-secondary)' }"
              />
            </div>

            <div
              v-else
              class="space-y-3"
            >
              <div
                v-if="profiles.length === 0"
                class="text-center py-10"
                :style="{ color: 'var(--text-muted)' }"
              >
                {{ $t('codex.profiles.emptyState') }}
              </div>

              <div
                v-for="profile in profiles"
                :key="profile.name"
                class="group rounded-lg p-4 transition-all duration-300"
                :style="{ background: 'rgba(255, 255, 255, 0.7)', border: '1px solid rgba(99, 102, 241, 0.12)' }"
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                      <h3
                        class="text-lg font-bold font-mono truncate"
                        :style="{ color: 'var(--text-primary)' }"
                      >
                        {{ profile.name }}
                      </h3>
                      <span
                        v-if="currentProfile && profile.name === currentProfile"
                        class="px-2 py-0.5 rounded text-xs font-semibold"
                        :style="{ background: 'var(--accent-success)', color: 'white' }"
                      >
                        {{ $t('codex.profiles.currentBadge') }}
                      </span>
                      <span
                        v-else-if="profile.enabled === false"
                        class="px-2 py-0.5 rounded text-xs font-semibold"
                        :style="{ background: 'var(--accent-danger)', color: 'white' }"
                      >
                        {{ $t('codex.states.disabled') }}
                      </span>
                      <span
                        v-else
                        class="px-2 py-0.5 rounded text-xs font-semibold"
                        :style="{ background: 'var(--accent-primary)', color: 'white' }"
                      >
                        {{ $t('codex.states.enabled') }}
                      </span>
                    </div>

                    <p
                      v-if="profile.description"
                      class="text-sm mb-2 line-clamp-2"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ profile.description }}
                    </p>

                    <div class="space-y-1 text-sm">
                      <div class="flex items-center gap-2">
                        <span :style="{ color: 'var(--text-muted)' }">{{ $t('codex.profiles.fields.baseUrl') }}</span>
                        <code
                          class="px-2 py-1 rounded font-mono truncate"
                          :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }"
                        >
                          {{ profile.base_url }}
                        </code>
                      </div>

                      <div class="flex items-center gap-2">
                        <span :style="{ color: 'var(--text-muted)' }">{{ $t('codex.profiles.fields.model') }}</span>
                        <code
                          class="px-2 py-1 rounded font-mono"
                          :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }"
                        >
                          {{ profile.model }}
                        </code>
                      </div>

                      <div
                        v-if="profile.provider || profile.provider_type"
                        class="flex items-center gap-2"
                      >
                        <span :style="{ color: 'var(--text-muted)' }">{{ $t('codex.profiles.fields.provider') }}</span>
                        <span :style="{ color: 'var(--text-secondary)' }">
                          {{ profile.provider || '-' }}
                        </span>
                        <span
                          v-if="profile.provider_type"
                          class="px-2 py-0.5 rounded text-xs font-semibold"
                          :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }"
                        >
                          {{ profile.provider_type }}
                        </span>
                      </div>

                      <div
                        v-if="profile.tags && profile.tags.length > 0"
                        class="flex flex-wrap gap-1 mt-2"
                      >
                        <span
                          v-for="tag in profile.tags"
                          :key="tag"
                          class="px-2 py-0.5 rounded text-xs font-semibold"
                          :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }"
                        >
                          {{ tag }}
                        </span>
                      </div>

                      <div
                        v-if="profile.extra && Object.keys(profile.extra).length > 0"
                        class="flex items-center gap-2 mt-2"
                      >
                        <span :style="{ color: 'var(--text-muted)' }">{{ $t('codex.profiles.fields.extra') }}</span>
                        <span :style="{ color: 'var(--text-secondary)' }">
                          {{ Object.keys(profile.extra).length }}
                        </span>
                      </div>
                    </div>
                  </div>

                  <div class="flex gap-2 shrink-0">
                    <button
                      class="p-2 rounded-lg transition-all hover:scale-110"
                      :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-success)' }"
                      :title="$t('codex.profiles.apply')"
                      @click="handleApply(profile.name)"
                    >
                      <Check class="w-4 h-4" />
                    </button>
                    <button
                      class="p-2 rounded-lg transition-all hover:scale-110"
                      :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }"
                      :title="$t('codex.actions.edit')"
                      @click="handleEdit(profile.name)"
                    >
                      <Edit2 class="w-4 h-4" />
                    </button>
                    <button
                      class="p-2 rounded-lg transition-all hover:scale-110"
                      :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }"
                      :title="$t('codex.actions.delete')"
                      @click="handleDelete(profile.name)"
                    >
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <!-- Add/Edit Modal -->
            <div
              v-if="showForm"
              class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
            >
              <div
                class="rounded-xl p-6 max-w-3xl w-full max-h-[90vh] overflow-y-auto"
                :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)' }"
              >
                <h2
                  class="text-xl font-bold mb-4"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ editingName ? $t('codex.profiles.editProfile') : $t('codex.profiles.addProfile') }}
                </h2>

                <div class="space-y-4">
                  <div>
                    <label
                      class="block text-sm font-semibold mb-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >{{ $t('codex.profiles.fields.name') }}</label>
                    <input
                      v-model="form.name"
                      :disabled="!!editingName"
                      type="text"
                      class="w-full px-3 py-2 rounded-lg font-mono disabled:opacity-60"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      :placeholder="$t('codex.profiles.placeholders.name')"
                    >
                  </div>

                  <div>
                    <label
                      class="block text-sm font-semibold mb-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >{{ $t('codex.profiles.fields.description') }}</label>
                    <input
                      v-model="form.description"
                      type="text"
                      class="w-full px-3 py-2 rounded-lg"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      :placeholder="$t('codex.profiles.placeholders.description')"
                    >
                  </div>

                  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <label
                        class="block text-sm font-semibold mb-1"
                        :style="{ color: 'var(--text-secondary)' }"
                      >{{ $t('codex.profiles.fields.baseUrl') }}</label>
                      <input
                        v-model="form.base_url"
                        type="text"
                        class="w-full px-3 py-2 rounded-lg font-mono"
                        :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                        :placeholder="$t('codex.profiles.placeholders.baseUrl')"
                      >
                    </div>

                    <div>
                      <label
                        class="block text-sm font-semibold mb-1"
                        :style="{ color: 'var(--text-secondary)' }"
                      >{{ $t('codex.profiles.fields.model') }}</label>
                      <input
                        v-model="form.model"
                        type="text"
                        class="w-full px-3 py-2 rounded-lg font-mono"
                        :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                        :placeholder="$t('codex.profiles.placeholders.model')"
                      >
                    </div>
                  </div>

                  <div>
                    <label
                      class="block text-sm font-semibold mb-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >{{ $t('codex.profiles.fields.authToken') }}</label>
                    <input
                      v-model="form.auth_token"
                      type="password"
                      class="w-full px-3 py-2 rounded-lg font-mono"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      :placeholder="$t('codex.profiles.placeholders.authToken')"
                    >
                  </div>

                  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                      <label
                        class="block text-sm font-semibold mb-1"
                        :style="{ color: 'var(--text-secondary)' }"
                      >{{ $t('codex.profiles.fields.smallFastModel') }}</label>
                      <input
                        v-model="form.small_fast_model"
                        type="text"
                        class="w-full px-3 py-2 rounded-lg font-mono"
                        :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                        :placeholder="$t('codex.profiles.placeholders.smallFastModel')"
                      >
                    </div>

                    <div>
                      <label
                        class="block text-sm font-semibold mb-1"
                        :style="{ color: 'var(--text-secondary)' }"
                      >{{ $t('codex.profiles.fields.tags') }}</label>
                      <input
                        v-model="tagsText"
                        type="text"
                        class="w-full px-3 py-2 rounded-lg"
                        :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                        :placeholder="$t('codex.profiles.placeholders.tags')"
                      >
                    </div>
                  </div>

                  <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                      <label
                        class="block text-sm font-semibold mb-1"
                        :style="{ color: 'var(--text-secondary)' }"
                      >{{ $t('codex.profiles.fields.provider') }}</label>
                      <input
                        v-model="form.provider"
                        type="text"
                        class="w-full px-3 py-2 rounded-lg"
                        :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                        :placeholder="$t('codex.profiles.placeholders.provider')"
                      >
                    </div>

                    <div>
                      <label
                        class="block text-sm font-semibold mb-1"
                        :style="{ color: 'var(--text-secondary)' }"
                      >{{ $t('codex.profiles.fields.providerType') }}</label>
                      <input
                        v-model="form.provider_type"
                        type="text"
                        class="w-full px-3 py-2 rounded-lg"
                        :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                        :placeholder="$t('codex.profiles.placeholders.providerType')"
                      >
                    </div>

                    <div>
                      <label
                        class="block text-sm font-semibold mb-1"
                        :style="{ color: 'var(--text-secondary)' }"
                      >{{ $t('codex.profiles.fields.account') }}</label>
                      <input
                        v-model="form.account"
                        type="text"
                        class="w-full px-3 py-2 rounded-lg"
                        :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                        :placeholder="$t('codex.profiles.placeholders.account')"
                      >
                    </div>
                  </div>

                  <div class="flex items-center gap-2">
                    <input
                      id="profileEnabled"
                      v-model="form.enabled"
                      type="checkbox"
                      class="w-4 h-4"
                    >
                    <label
                      for="profileEnabled"
                      class="text-sm font-semibold"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ $t('codex.profiles.fields.enabled') }}
                    </label>
                  </div>

                  <div>
                    <label
                      class="block text-sm font-semibold mb-1"
                      :style="{ color: 'var(--text-secondary)' }"
                    >{{ $t('codex.profiles.fields.extraJson') }}</label>
                    <textarea
                      v-model="extraText"
                      rows="10"
                      class="w-full px-3 py-2 rounded-lg font-mono text-sm"
                      :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }"
                      :placeholder="$t('codex.profiles.placeholders.extraJson')"
                    />
                    <p
                      class="text-xs mt-2"
                      :style="{ color: 'var(--text-muted)' }"
                    >
                      {{ $t('codex.profiles.extraHint') }}
                    </p>
                  </div>
                </div>

                <div class="flex justify-end gap-3 mt-6">
                  <button
                    class="px-4 py-2 rounded-lg font-semibold text-sm"
                    :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-secondary)' }"
                    @click="handleCloseForm"
                  >
                    {{ $t('codex.actions.cancel') }}
                  </button>
                  <button
                    class="px-4 py-2 rounded-lg font-semibold text-sm text-white"
                    :disabled="saving"
                    :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', opacity: saving ? 0.6 : 1 }"
                    @click="handleSave"
                  >
                    {{ saving ? $t('codex.states.saving') : $t('codex.actions.save') }}
                  </button>
                </div>
              </div>
            </div>
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
import { ArrowLeft, Boxes, Check, Edit2, Globe, Home, Layers, ListFilter, Plus, Server, Settings, Shuffle, Star, Trash2, Zap } from 'lucide-vue-next'

import Breadcrumb from '@/components/Breadcrumb.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
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

