<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="max-w-[1200px] mx-auto">
      <!-- Breadcrumb -->
      <Breadcrumb :items="breadcrumbs" />

      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-2xl font-bold text-guofeng-text-primary flex items-center">
            <Monitor
              class="w-7 h-7 mr-2 text-guofeng-orange"
              aria-hidden="true"
            />
            {{ $t('statusline.pageTitle') }}
          </h2>
        </div>
      </div>

      <!-- Loading State -->
      <div
        v-if="loading"
        class="text-center py-20 text-guofeng-text-muted"
        role="status"
        aria-live="polite"
      >
        <div
          class="loading-spinner mx-auto mb-4 w-8 h-8 border-guofeng-orange/30 border-t-guofeng-orange"
          aria-hidden="true"
        />
        <span>{{ $t('common.loading') }}</span>
      </div>

      <!-- Configuration Card -->
      <div
        v-else
        class="space-y-6"
      >
        <!-- Status Card -->
        <div class="glass-effect rounded-2xl p-6 border border-white/20 shadow-sm">
          <h3 class="text-lg font-bold text-guofeng-text-primary mb-4 flex items-center">
            <Settings
              class="w-5 h-5 mr-2 text-guofeng-orange"
              aria-hidden="true"
            />
            {{ $t('statusline.configuration') }}
          </h3>

          <div class="space-y-6">
            <!-- Enable Toggle -->
            <div class="flex items-center justify-between p-4 bg-guofeng-bg-tertiary/50 rounded-xl border border-guofeng-border/30">
              <div>
                <p
                  id="enabled-label"
                  class="font-semibold text-guofeng-text-primary"
                >
                  {{ $t('statusline.enabled') }}
                </p>
                <p
                  id="enabled-description"
                  class="text-sm text-guofeng-text-muted mt-1"
                >
                  {{ $t('statusline.enabledDescription') }}
                </p>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input
                  id="statusline-enabled"
                  v-model="config.enabled"
                  type="checkbox"
                  class="sr-only peer"
                  aria-labelledby="enabled-label"
                  aria-describedby="enabled-description"
                >
                <div class="w-11 h-6 bg-guofeng-bg-tertiary rounded-full peer peer-checked:after:translate-x-full peer-checked:bg-guofeng-orange after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all border border-guofeng-border peer-checked:border-guofeng-orange/50" />
                <span class="sr-only">{{ config.enabled ? $t('statusline.statusEnabled') : $t('statusline.statusDisabled') }}</span>
              </label>
            </div>

            <!-- Command Input -->
            <div class="p-4 bg-guofeng-bg-tertiary/50 rounded-xl border border-guofeng-border/30">
              <label
                for="statusline-command"
                class="block mb-2 font-semibold text-guofeng-text-primary"
              >
                {{ $t('statusline.command') }}
              </label>
              <p
                id="command-description"
                class="text-sm text-guofeng-text-muted mb-3"
              >
                {{ $t('statusline.commandDescription') }}
              </p>
              <input
                id="statusline-command"
                v-model="config.command"
                type="text"
                class="w-full px-4 py-3 rounded-lg bg-white/50 border border-guofeng-border focus:border-guofeng-orange focus:ring-2 focus:ring-guofeng-orange/20 outline-none transition-all font-mono text-sm"
                :placeholder="$t('statusline.commandPlaceholder')"
                aria-describedby="command-description command-help"
              >
              <p
                id="command-help"
                class="text-xs text-guofeng-text-muted mt-2"
              >
                {{ $t('statusline.commandHelp') }}
              </p>
            </div>
          </div>

          <!-- Save Button -->
          <div class="flex justify-end mt-6 pt-4 border-t border-guofeng-border/30">
            <button
              class="px-6 py-2.5 rounded-lg font-medium transition-all bg-guofeng-orange text-white shadow-md hover:shadow-lg hover:-translate-y-0.5 flex items-center min-h-[44px]"
              :disabled="saving"
              :aria-busy="saving"
              @click="handleSave"
            >
              <span
                v-if="saving"
                class="loading-spinner w-4 h-4 mr-2 border-white/30 border-t-white"
                aria-hidden="true"
              />
              <Save
                v-else
                class="w-4 h-4 mr-2"
                aria-hidden="true"
              />
              {{ saving ? $t('common.saving') : $t('common.save') }}
            </button>
          </div>
        </div>

        <!-- Info Card -->
        <div
          class="glass-effect rounded-2xl p-6 border border-white/20 shadow-sm"
          role="region"
          aria-labelledby="about-title"
        >
          <h3
            id="about-title"
            class="text-lg font-bold text-guofeng-text-primary mb-4 flex items-center"
          >
            <Info
              class="w-5 h-5 mr-2 text-guofeng-blue"
              aria-hidden="true"
            />
            {{ $t('statusline.about') }}
          </h3>
          <div class="prose prose-sm max-w-none text-guofeng-text-secondary">
            <p>{{ $t('statusline.aboutDescription') }}</p>
            <ul
              class="mt-3 space-y-2"
              role="list"
            >
              <li class="flex items-start gap-2">
                <span
                  class="text-guofeng-orange"
                  aria-hidden="true"
                >•</span>
                {{ $t('statusline.feature1') }}
              </li>
              <li class="flex items-start gap-2">
                <span
                  class="text-guofeng-orange"
                  aria-hidden="true"
                >•</span>
                {{ $t('statusline.feature2') }}
              </li>
              <li class="flex items-start gap-2">
                <span
                  class="text-guofeng-orange"
                  aria-hidden="true"
                >•</span>
                {{ $t('statusline.feature3') }}
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Monitor, Settings, Save, Info, Home, Code2 } from 'lucide-vue-next'
import Breadcrumb from '@/components/common/Breadcrumb.vue'
import { getStatusline, updateStatusline } from '@/api/client'
import { useUIStore } from '@/store'
import type { StatuslineConfig } from '@/types'

const { t } = useI18n()
const uiStore = useUIStore()

const loading = ref(true)
const saving = ref(false)
const config = ref<StatuslineConfig>({
  command: '',
  enabled: false
})

const breadcrumbs = [
  { label: t('common.home'), to: '/', icon: Home },
  { label: t('claudeCode.title'), to: '/claude-code', icon: Code2 },
  { label: t('statusline.pageTitle') }
]

onMounted(async () => {
  await loadConfig()
})

const loadConfig = async () => {
  loading.value = true
  try {
    config.value = await getStatusline()
  } catch (err) {
    console.error('Failed to load statusline config:', err)
    uiStore.showError(t('common.loadFailed'))
    // Use defaults
    config.value = { command: '', enabled: false }
  } finally {
    loading.value = false
  }
}

const handleSave = async () => {
  saving.value = true
  try {
    await updateStatusline(config.value)
    uiStore.showSuccess(t('common.saveSuccess'))
  } catch (err) {
    console.error('Failed to save statusline config:', err)
    uiStore.showError(t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}
</script>
