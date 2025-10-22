<template>
  <div :style="{ background: 'var(--bg-primary)', minHeight: '100vh', padding: '20px' }">
    <div class="max-w-[1800px] mx-auto">
      <Navbar />
      <StatusHeader :currentConfig="currentConfig" :totalConfigs="totalConfigs" :historyCount="historyCount" />

      <div class="grid grid-cols-[auto_1fr] gap-4">
        <CollapsibleSidebar />

        <main class="rounded-xl p-6 glass-effect" :style="{ border: '1px solid var(--border-color)', boxShadow: 'var(--shadow-small)' }">
          <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
              <Users class="w-6 h-6" :style="{ color: 'var(--accent-primary)' }" />
              <h1 class="text-2xl font-bold" :style="{ color: 'var(--text-primary)' }">Codex Profiles 配置</h1>
            </div>
            <div class="flex items-center gap-3">
              <RouterLink to="/codex" class="inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-colors" :style="{ background: 'var(--bg-secondary)', color: 'var(--text-secondary)', border: '1px solid var(--border-color)' }">
                <ArrowLeft class="w-4 h-4" /><span>返回</span>
              </RouterLink>
              <button class="px-4 py-2 rounded-lg font-semibold text-sm text-white flex items-center gap-2" :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))', boxShadow: '0 0 20px var(--glow-primary)' }" @click="handleAdd">
                <Plus class="w-4 h-4" />添加 Profile
              </button>
            </div>
          </div>

          <div v-if="loading" class="flex justify-center py-20">
            <div class="w-12 h-12 rounded-full border-4 border-transparent animate-spin" :style="{ borderTopColor: 'var(--accent-primary)', borderRightColor: 'var(--accent-secondary)' }" />
          </div>

          <div v-else class="space-y-3">
            <div v-if="!profiles || profiles.length === 0" class="text-center py-10" :style="{ color: 'var(--text-muted)' }">暂无 Codex Profile 配置</div>

            <div v-for="profile in profiles" :key="profile.name" class="group rounded-lg p-4 transition-all duration-300" :style="{ background: 'rgba(255, 255, 255, 0.7)', border: '1px solid rgba(99, 102, 241, 0.12)', outline: 'none', cursor: 'default' }" @mouseenter="(e) => onCardHover(e.currentTarget, true)" @mouseleave="(e) => onCardHover(e.currentTarget, false)">
              <div class="flex items-start justify-between">
                <div class="flex-1">
                  <div class="flex items-center gap-2 mb-2">
                    <h3 class="text-lg font-bold font-mono" :style="{ color: 'var(--text-primary)' }">{{ profile.name }}</h3>
                  </div>
                  <div class="space-y-2 text-sm">
                    <div v-if="profile.model">
                      <span :style="{ color: 'var(--text-muted)' }">Model:</span>
                      <code class="ml-2 px-2 py-1 rounded font-mono" :style="{ background: 'var(--bg-secondary)', color: 'var(--accent-primary)' }">{{ profile.model }}</code>
                    </div>
                    <div v-if="profile.approval_policy">
                      <span :style="{ color: 'var(--text-muted)' }">Approval Policy:</span>
                      <code class="ml-2 px-2 py-1 rounded font-mono" :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }">{{ profile.approval_policy }}</code>
                    </div>
                    <div v-if="profile.sandbox_mode">
                      <span :style="{ color: 'var(--text-muted)' }">Sandbox Mode:</span>
                      <code class="ml-2 px-2 py-1 rounded font-mono" :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }">{{ profile.sandbox_mode }}</code>
                    </div>
                    <div v-if="profile.model_reasoning_effort">
                      <span :style="{ color: 'var(--text-muted)' }">Reasoning Effort:</span>
                      <code class="ml-2 px-2 py-1 rounded font-mono" :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)' }">{{ profile.model_reasoning_effort }}</code>
                    </div>
                  </div>
                </div>
                <div class="flex gap-2">
                  <button class="p-2 rounded-lg transition-all hover:scale-110" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-primary)' }" title="编辑" @click="handleEdit(profile)">
                    <Edit2 class="w-4 h-4" />
                  </button>
                  <button class="p-2 rounded-lg transition-all hover:scale-110" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)', color: 'var(--accent-danger)' }" title="删除" @click="handleDelete(profile.name)">
                    <Trash2 class="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div v-if="showAddForm" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
            <div class="rounded-xl p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border-color)' }">
              <h2 class="text-xl font-bold mb-4" :style="{ color: 'var(--text-primary)' }">{{ editingProfile ? '编辑 Profile' : '添加 Profile' }}</h2>

              <div class="space-y-4">
                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">Profile 名称 *</label>
                  <input v-model="formData.name" type="text" class="w-full px-3 py-2 rounded-lg" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }" placeholder="例如: production" :disabled="!!editingProfile" />
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">Model</label>
                  <select v-model="formData.model" class="w-full px-3 py-2 rounded-lg" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }">
                    <option value="">-- 默认 --</option>
                    <option value="gpt-5">gpt-5</option>
                    <option value="gpt-5-codex">gpt-5-codex</option>
                    <option value="claude-sonnet-4-5-20250929">claude-sonnet-4-5-20250929</option>
                  </select>
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">Approval Policy</label>
                  <select v-model="formData.approval_policy" class="w-full px-3 py-2 rounded-lg" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }">
                    <option value="">-- 默认 --</option>
                    <option value="auto">auto</option>
                    <option value="on-request">on-request</option>
                    <option value="read-only">read-only</option>
                    <option value="full-access">full-access</option>
                  </select>
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">Sandbox Mode</label>
                  <select v-model="formData.sandbox_mode" class="w-full px-3 py-2 rounded-lg" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }">
                    <option value="">-- 默认 --</option>
                    <option value="workspace-write">workspace-write</option>
                    <option value="full-system">full-system</option>
                  </select>
                </div>

                <div>
                  <label class="block text-sm font-semibold mb-1" :style="{ color: 'var(--text-secondary)' }">Reasoning Effort</label>
                  <select v-model="formData.model_reasoning_effort" class="w-full px-3 py-2 rounded-lg" :style="{ background: 'var(--bg-tertiary)', border: '1px solid var(--border-color)', color: 'var(--text-primary)' }">
                    <option value="">-- 默认 --</option>
                    <option value="low">low</option>
                    <option value="medium">medium</option>
                    <option value="high">high</option>
                  </select>
                </div>
              </div>

              <div class="flex gap-3 mt-6">
                <button class="flex-1 px-4 py-2 rounded-lg font-semibold text-white" :style="{ background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))' }" @click="handleSubmit">{{ editingProfile ? '更新' : '添加' }}</button>
                <button class="flex-1 px-4 py-2 rounded-lg font-semibold" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border-color)' }" @click="showAddForm = false">取消</button>
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
import { Users, Plus, Edit2, Trash2, ArrowLeft } from 'lucide-vue-next'
import { listCodexProfiles, addCodexProfile, updateCodexProfile, deleteCodexProfile, listConfigs, getHistory } from '@/api/client'
import type { CodexProfile, CodexProfileRequest } from '@/types'
import Navbar from '@/components/Navbar.vue'
import StatusHeader from '@/components/StatusHeader.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'

const profiles = ref<CodexProfile[]>([])
const loading = ref(true)
const currentConfig = ref<string>('')
const totalConfigs = ref(0)
const historyCount = ref(0)
const showAddForm = ref(false)
const editingProfile = ref<CodexProfile | null>(null)
const formData = ref<CodexProfileRequest>({ name: '', model: undefined, approval_policy: undefined, sandbox_mode: undefined, model_reasoning_effort: undefined })

const loadProfiles = async () => {
  try {
    loading.value = true
    const data = await listCodexProfiles()
    profiles.value = data || []

    try {
      const configData = await listConfigs()
      currentConfig.value = configData.current_config
      totalConfigs.value = configData.configs.length
      const historyData = await getHistory()
      historyCount.value = historyData.total
    } catch (err) { console.error('Failed to load system info:', err) }
  } catch (err) {
    console.error('Failed to load Codex profiles:', err)
    profiles.value = []
    alert('加载 Codex Profiles 失败')
  } finally { loading.value = false }
}

onMounted(() => { loadProfiles() })

const handleAdd = () => {
  showAddForm.value = true
  editingProfile.value = null
  formData.value = { name: '', model: undefined, approval_policy: undefined, sandbox_mode: undefined, model_reasoning_effort: undefined }
}

const handleEdit = (profile: CodexProfile) => {
  editingProfile.value = profile
  showAddForm.value = true
  formData.value = { name: profile.name, model: profile.model, approval_policy: profile.approval_policy, sandbox_mode: profile.sandbox_mode, model_reasoning_effort: profile.model_reasoning_effort }
}

const handleSubmit = async () => {
  if (!formData.value.name) { alert('请填写 Profile 名称'); return }

  const request: CodexProfileRequest = { ...formData.value }

  try {
    if (editingProfile.value) {
      await updateCodexProfile(editingProfile.value.name, request)
      alert('✓ Profile 更新成功')
    } else {
      await addCodexProfile(request)
      alert('✓ Profile 添加成功')
    }
    showAddForm.value = false
    await loadProfiles()
  } catch (err) { alert(`操作失败: ${err instanceof Error ? err.message : 'Unknown error'}`) }
}

const handleDelete = async (name: string) => {
  if (!confirm(`确定删除 Profile "${name}" 吗？`)) return
  try {
    await deleteCodexProfile(name)
    alert('✓ Profile 删除成功')
    await loadProfiles()
  } catch (err) { alert(`删除失败: ${err instanceof Error ? err.message : 'Unknown error'}`) }
}

const onCardHover = (el: HTMLElement, hover: boolean) => {
  if (hover) {
    el.style.background = 'rgba(255, 255, 255, 0.9)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.24)'
    el.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -2px rgba(0, 0, 0, 0.08)'
    el.style.transform = 'translateY(-2px)'
  } else {
    el.style.background = 'rgba(255, 255, 255, 0.7)'
    el.style.borderColor = 'rgba(99, 102, 241, 0.12)'
    el.style.boxShadow = 'none'
    el.style.transform = 'translateY(0)'
  }
}
</script>
