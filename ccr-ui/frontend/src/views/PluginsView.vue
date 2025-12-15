<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <div class="max-w-[1800px] mx-auto">
      <Navbar />
      
      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          { label: $t('plugins.breadcrumb.home'), path: '/', icon: Home },
          { label: $t('plugins.breadcrumb.claudeCode'), path: '/claude-code', icon: Code2 },
          { label: $t('plugins.breadcrumb.plugins'), path: '/plugins', icon: Puzzle }
        ]"
        module-color="#6366f1"
        class="mb-6"
      />

      <div class="grid grid-cols-[auto_1fr] gap-6">
        <CollapsibleSidebar module="claude-code" />

        <main class="min-w-0">
          <!-- Header -->
          <div class="glass-effect rounded-2xl p-6 mb-6 border border-white/20 flex flex-col md:flex-row items-center justify-between gap-4 sticky top-6 z-20 backdrop-blur-xl shadow-sm">
            <div class="flex items-center gap-4">
              <div class="p-3 rounded-xl bg-guofeng-indigo/10 text-guofeng-indigo">
                <Puzzle class="w-6 h-6" />
              </div>
              <div>
                <div class="flex items-center gap-3">
                  <h1 class="text-2xl font-bold text-guofeng-text-primary">
                    {{ $t('plugins.title') }}
                  </h1>
                  <span class="px-2.5 py-0.5 rounded-full text-xs font-bold bg-guofeng-indigo/10 text-guofeng-indigo border border-guofeng-indigo/20">
                    Claude Code
                  </span>
                  <span class="px-2.5 py-0.5 rounded-full text-xs font-bold bg-guofeng-indigo text-white shadow-sm">
                    {{ plugins.length }}
                  </span>
                </div>
                <p class="text-sm mt-1 text-guofeng-text-secondary">
                  {{ $t('plugins.subtitle') }}
                </p>
              </div>
            </div>
            
            <div class="flex items-center gap-3">
              <button
                class="px-5 py-2.5 rounded-xl font-bold text-sm text-white flex items-center gap-2 transition-all hover:scale-105 bg-guofeng-indigo shadow-lg shadow-guofeng-indigo/20 hover:shadow-guofeng-indigo/30"
                @click="handleAdd"
              >
                <Plus class="w-5 h-5" />
                {{ $t('plugins.addPlugin') }}
              </button>
            </div>
          </div>

          <!-- Content -->
          <div
            v-if="loading"
            class="flex justify-center py-20"
          >
            <div class="w-10 h-10 rounded-full border-4 border-guofeng-indigo/30 border-t-guofeng-indigo animate-spin" />
          </div>

          <div
            v-else-if="!plugins || plugins.length === 0"
            class="text-center py-16 glass-effect rounded-3xl border border-white/20 border-dashed"
          >
            <div class="bg-guofeng-bg-secondary w-20 h-20 rounded-full flex items-center justify-center mx-auto mb-4">
              <Puzzle class="w-10 h-10 opacity-30 text-guofeng-text-muted" />
            </div>
            <p class="text-lg font-bold text-guofeng-text-primary">
              {{ $t('plugins.noPlugins') }}
            </p>
          </div>

          <div
            v-else
            class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5"
          >
            <div
              v-for="plugin in plugins"
              :key="plugin.id"
              class="group glass-effect rounded-2xl p-5 border border-white/20 transition-all duration-300 hover:shadow-md hover:border-guofeng-indigo/30 flex flex-col"
            >
              <div class="flex items-start justify-between mb-3">
                <div class="flex-1 min-w-0">
                  <h3 class="text-lg font-bold text-guofeng-text-primary group-hover:text-guofeng-indigo transition-colors truncate">
                    {{ plugin.name }}
                  </h3>
                  <p class="text-xs font-mono mt-1 text-guofeng-text-muted truncate">
                    ID: {{ plugin.id }}
                  </p>
                </div>
                <span
                  v-if="!plugin.enabled"
                  class="ml-2 px-2 py-0.5 rounded text-xs font-semibold uppercase bg-guofeng-red/10 text-guofeng-red border border-guofeng-red/20 flex-shrink-0"
                >
                  {{ $t('plugins.disabled') }}
                </span>
              </div>

              <div class="flex-1 mb-4 space-y-2">
                <p class="text-sm text-guofeng-text-secondary">
                  <strong class="text-guofeng-text-primary">{{ $t('plugins.version') }}</strong> {{ plugin.version }}
                </p>
                <div
                  v-if="plugin.config"
                  class="text-xs font-mono p-3 rounded-xl bg-guofeng-bg-tertiary border border-guofeng-border/50 overflow-auto max-h-32 text-guofeng-text-secondary"
                >
                  {{ JSON.stringify(plugin.config, null, 2) }}
                </div>
              </div>

              <div class="flex gap-2 pt-3 border-t border-guofeng-border/30">
                <button
                  class="flex-1 px-3 py-2 rounded-lg transition-all hover:scale-105 flex items-center justify-center gap-2 text-sm font-medium border"
                  :class="plugin.enabled ? 'bg-guofeng-red/10 text-guofeng-red border-guofeng-red/20 hover:bg-guofeng-red/20' : 'bg-guofeng-green/10 text-guofeng-green border-guofeng-green/20 hover:bg-guofeng-green/20'"
                  :title="plugin.enabled ? $t('plugins.disable') : $t('plugins.enable')"
                  @click="handleToggle(plugin.id)"
                >
                  <PowerOff
                    v-if="!plugin.enabled"
                    class="w-4 h-4"
                  />
                  <Power
                    v-else
                    class="w-4 h-4"
                  />
                  <span>{{ plugin.enabled ? $t('plugins.disable') : $t('plugins.enable') }}</span>
                </button>
                <button
                  class="p-2 rounded-lg transition-all hover:scale-110 text-guofeng-text-secondary hover:text-guofeng-indigo hover:bg-guofeng-indigo/10"
                  :title="$t('plugins.edit')"
                  @click="handleEdit(plugin)"
                >
                  <Edit2 class="w-4 h-4" />
                </button>
                <button
                  class="p-2 rounded-lg transition-all hover:scale-110 text-guofeng-text-secondary hover:text-guofeng-red hover:bg-guofeng-red/10"
                  :title="$t('plugins.delete')"
                  @click="handleDelete(plugin.id)"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>

          <!-- Add/Edit Form Modal -->
          <div
            v-if="showAddForm"
            class="fixed inset-0 bg-guofeng-ink/20 backdrop-blur-sm flex items-center justify-center p-4 z-50 transition-all"
            @click="showAddForm = false"
          >
            <div
              class="glass-effect rounded-3xl p-8 max-w-2xl w-full max-h-[90vh] overflow-y-auto shadow-2xl border border-white/30 relative"
              @click.stop
            >
              <button 
                class="absolute top-4 right-4 p-2 rounded-full hover:bg-guofeng-bg-tertiary text-guofeng-text-muted transition-colors"
                @click="showAddForm = false"
              >
                <X class="w-5 h-5" />
              </button>

              <h2 class="text-2xl font-bold mb-6 text-guofeng-text-primary flex items-center gap-3">
                <div class="w-10 h-10 rounded-xl bg-guofeng-indigo/10 flex items-center justify-center text-guofeng-indigo">
                  <component
                    :is="editingPlugin ? Edit2 : Plus"
                    class="w-5 h-5"
                  />
                </div>
                {{ editingPlugin ? $t('plugins.editPlugin') : $t('plugins.addPlugin') }}
              </h2>

              <div class="space-y-5">
                <div>
                  <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-2">
                    {{ $t('plugins.form.id') }} <span class="text-guofeng-red">*</span>
                  </label>
                  <input
                    v-model="formData.id"
                    type="text"
                    class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-indigo focus:ring-4 focus:ring-guofeng-indigo/10 outline-none transition-all"
                    :placeholder="$t('plugins.form.idPlaceholder')"
                  >
                </div>

                <div>
                  <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-2">
                    {{ $t('plugins.form.name') }} <span class="text-guofeng-red">*</span>
                  </label>
                  <input
                    v-model="formData.name"
                    type="text"
                    class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-indigo focus:ring-4 focus:ring-guofeng-indigo/10 outline-none transition-all"
                    :placeholder="$t('plugins.form.namePlaceholder')"
                  >
                </div>

                <div>
                  <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-2">
                    {{ $t('plugins.form.version') }} <span class="text-guofeng-red">*</span>
                  </label>
                  <input
                    v-model="formData.version"
                    type="text"
                    class="w-full px-4 py-3 rounded-xl bg-white/50 border border-guofeng-border focus:border-guofeng-indigo focus:ring-4 focus:ring-guofeng-indigo/10 outline-none transition-all"
                    :placeholder="$t('plugins.form.versionPlaceholder')"
                  >
                </div>

                <div>
                  <label class="block text-xs font-bold text-guofeng-text-secondary uppercase tracking-wider mb-2">
                    {{ $t('plugins.form.config') }}
                  </label>
                  <textarea
                    v-model="configJson"
                    rows="8"
                    class="w-full px-4 py-3 rounded-xl font-mono text-sm bg-white/50 border border-guofeng-border focus:border-guofeng-indigo focus:ring-4 focus:ring-guofeng-indigo/10 outline-none transition-all"
                    :placeholder="$t('plugins.form.configPlaceholder')"
                  />
                  <div class="text-xs mt-1.5 text-guofeng-text-muted">
                    {{ $t('plugins.form.configHint') }}
                  </div>
                </div>

                <div class="flex items-center gap-3 p-4 rounded-xl bg-guofeng-bg-tertiary/50 border border-guofeng-border/50">
                  <input
                    id="enabled"
                    v-model="formData.enabled"
                    type="checkbox"
                    class="w-5 h-5 rounded text-guofeng-indigo focus:ring-guofeng-indigo/20 border-guofeng-border"
                  >
                  <label
                    for="enabled"
                    class="text-sm font-medium text-guofeng-text-secondary cursor-pointer"
                  >
                    {{ $t('plugins.form.enablePlugin') }}
                  </label>
                </div>
              </div>

              <div class="flex gap-4 mt-8 pt-6 border-t border-guofeng-border/50">
                <button
                  class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-white text-guofeng-text-secondary hover:bg-guofeng-bg-tertiary border border-guofeng-border"
                  @click="showAddForm = false"
                >
                  {{ $t('plugins.form.cancel') }}
                </button>
                <button
                  class="flex-1 px-6 py-3.5 rounded-xl font-bold transition-all bg-guofeng-indigo text-white shadow-lg shadow-guofeng-indigo/20 hover:shadow-xl hover:shadow-guofeng-indigo/30 hover:-translate-y-0.5"
                  @click="handleSubmit"
                >
                  {{ editingPlugin ? $t('plugins.form.update') : $t('plugins.form.add') }}
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
import { useI18n } from 'vue-i18n'
import { Puzzle, Plus, Edit2, Trash2, Power, PowerOff, Home, Code2, X } from 'lucide-vue-next'
import Breadcrumb from '@/components/Breadcrumb.vue'
import { listPlugins, addPlugin, updatePlugin, deletePlugin, togglePlugin } from '@/api/client'
import type { Plugin as PluginType, PluginRequest } from '@/types'
import Navbar from '@/components/Navbar.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'

const { t } = useI18n({ useScope: 'global' })

const plugins = ref<PluginType[]>([])
const loading = ref(true)
const showAddForm = ref(false)
const editingPlugin = ref<PluginType | null>(null)
const formData = ref<PluginRequest>({ id: '', name: '', version: '', enabled: true, config: undefined })
const configJson = ref('')

const loadPlugins = async () => {
  try {
    loading.value = true
    const data = await listPlugins()
    plugins.value = data || []
  } catch (err) {
    console.error('Failed to load plugins:', err)
    plugins.value = []
    alert(t('plugins.loadFailed'))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadPlugins()
})

const handleAdd = () => {
  showAddForm.value = true
  editingPlugin.value = null
  formData.value = { id: '', name: '', version: '1.0.0', enabled: true, config: undefined }
  configJson.value = ''
}

const handleEdit = (plugin: PluginType) => {
  editingPlugin.value = plugin
  showAddForm.value = true
  formData.value = { id: plugin.id, name: plugin.name, version: plugin.version, enabled: plugin.enabled, config: plugin.config }
  configJson.value = plugin.config ? JSON.stringify(plugin.config, null, 2) : ''
}

const handleSubmit = async () => {
  if (!formData.value.id || !formData.value.name || !formData.value.version) {
    alert(t('plugins.fillRequired'))
    return
  }

  let config = undefined
  if (configJson.value.trim()) {
    try {
      config = JSON.parse(configJson.value)
    } catch (err) {
      alert(t('plugins.configJsonError'))
      return
    }
  }

  const request: PluginRequest = { ...formData.value, config }

  try {
    if (editingPlugin.value) {
      await updatePlugin(editingPlugin.value.id, request)
      alert(t('plugins.updateSuccess'))
    } else {
      await addPlugin(request)
      alert(t('plugins.addSuccess'))
    }
    showAddForm.value = false
    await loadPlugins()
  } catch (err) {
    alert(`${t('plugins.operationFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  }
}

const handleDelete = async (id: string) => {
  if (!confirm(t('plugins.deleteConfirm', { id }))) return

  try {
    await deletePlugin(id)
    alert(t('plugins.deleteSuccess'))
    await loadPlugins()
  } catch (err) {
    alert(`${t('plugins.deleteFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  }
}

const handleToggle = async (id: string) => {
  try {
    await togglePlugin(id)
    await loadPlugins()
  } catch (err) {
    alert(`${t('plugins.toggleFailed')}: ${err instanceof Error ? err.message : t('commands.unknownError')}`)
  }
}
</script>
