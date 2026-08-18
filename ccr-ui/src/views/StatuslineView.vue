<template>
  <PageShell>
    <template #header>
      <PageHeader :title="$t('statusline.pageTitle')" />
    </template>
    <template #subnav>
      <ModuleSubnav module="claude-code" />
    </template>

      <!-- Loading State -->
      <div
        v-if="loading"
        class="text-center py-20 text-text-muted"
        role="status"
        aria-live="polite"
      >
        <div
          class="loading-spinner mx-auto mb-4 w-8 h-8 border-accent-secondary/30 border-t-accent-secondary"
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
        <div class="glass-effect rounded-2xl p-6 border border-border-default/25 shadow-sm">
          <h3 class="text-lg font-bold text-text-primary mb-4 flex items-center">
            <SIcon
              name="Settings"
              size="w-5 h-5"
              class="mr-2 text-accent-secondary"
            />
            {{ $t('statusline.configuration') }}
          </h3>

          <div class="space-y-6">
            <!-- Enable Toggle -->
            <div class="flex items-center justify-between p-4 bg-bg-elevated rounded-xl border border-border-default/30">
              <div>
                <p
                  id="enabled-label"
                  class="font-semibold text-text-primary"
                >
                  {{ $t('statusline.enabled') }}
                </p>
                <p
                  id="enabled-description"
                  class="text-sm text-text-muted mt-1"
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
                <div class="w-11 h-6 bg-bg-surface rounded-full peer peer-checked:after:translate-x-full peer-checked:bg-accent-secondary after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all border border-border-default peer-checked:border-accent-secondary/50" />
                <span class="sr-only">{{ config.enabled ? $t('statusline.statusEnabled') : $t('statusline.statusDisabled') }}</span>
              </label>
            </div>

            <!-- Command Input -->
            <div class="p-4 bg-bg-elevated rounded-xl border border-border-default/30">
              <label
                for="statusline-command"
                class="block mb-2 font-semibold text-text-primary"
              >
                {{ $t('statusline.command') }}
              </label>
              <p
                id="command-description"
                class="text-sm text-text-muted mb-3"
              >
                {{ $t('statusline.commandDescription') }}
              </p>
              <input
                id="statusline-command"
                v-model="config.command"
                type="text"
                class="w-full px-4 py-3 rounded-lg bg-bg-surface border border-border-default focus:border-accent-secondary focus:ring-2 focus:ring-accent-secondary/20 outline-none transition-[border-color,box-shadow] font-mono text-sm"
                :placeholder="$t('statusline.commandPlaceholder')"
                aria-describedby="command-description command-help"
              >
              <p
                id="command-help"
                class="text-xs text-text-muted mt-2"
              >
                {{ $t('statusline.commandHelp') }}
              </p>
            </div>
          </div>

          <!-- Save Button -->
          <div class="flex justify-end mt-6 pt-4 border-t border-border-default/30">
            <button
              class="px-6 py-2.5 rounded-lg font-medium transition-[box-shadow,transform] bg-accent-secondary text-[color:var(--color-accent-primary-contrast)] shadow-md hover:shadow-lg hover:-translate-y-0.5 flex items-center min-h-[44px]"
              :disabled="saving"
              :aria-busy="saving"
              @click="handleSave"
            >
              <span
                v-if="saving"
                class="loading-spinner w-4 h-4 mr-2 border-border-default/30 border-t-white"
                aria-hidden="true"
              />
              <SIcon
                v-else
                name="Save"
                size="w-4 h-4"
                class="mr-2"
              />
              {{ saving ? $t('common.saving') : $t('common.save') }}
            </button>
          </div>
        </div>

        <!-- Info Card -->
        <div
          class="glass-effect rounded-2xl p-6 border border-border-default/25 shadow-sm"
          role="region"
          aria-labelledby="about-title"
        >
          <h3
            id="about-title"
            class="text-lg font-bold text-text-primary mb-4 flex items-center"
          >
            <SIcon
              name="Info"
              size="w-5 h-5"
              class="mr-2 text-accent-secondary"
            />
            {{ $t('statusline.about') }}
          </h3>
          <div class="prose prose-sm max-w-none text-text-secondary">
            <p>{{ $t('statusline.aboutDescription') }}</p>
            <ul
              class="mt-3 space-y-2"
              role="list"
            >
              <li class="flex items-start gap-2">
                <span
                  class="text-accent-secondary"
                  aria-hidden="true"
                >•</span>
                {{ $t('statusline.feature1') }}
              </li>
              <li class="flex items-start gap-2">
                <span
                  class="text-accent-secondary"
                  aria-hidden="true"
                >•</span>
                {{ $t('statusline.feature2') }}
              </li>
              <li class="flex items-start gap-2">
                <span
                  class="text-accent-secondary"
                  aria-hidden="true"
                >•</span>
                {{ $t('statusline.feature3') }}
              </li>
            </ul>
          </div>
        </div>
      </div>
  </PageShell>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import ModuleSubnav from '@/components/ModuleSubnav.vue'
import PageHeader from '@/components/ui/PageHeader.vue'
import PageShell from '@/components/ui/PageShell.vue'
import { getStatusline, updateStatusline } from '@/api'
import { useUIStore } from '@/stores/ui'
import type { StatuslineConfig } from '@/types'
import { logger } from '@/utils/logger'

const { t } = useI18n()
const uiStore = useUIStore()

const loading = ref(true)
const saving = ref(false)
const config = ref<StatuslineConfig>({
  command: '',
  enabled: false
})

onMounted(async () => {
  await loadConfig()
})

const loadConfig = async () => {
  loading.value = true
  try {
    config.value = await getStatusline()
  } catch (err) {
    logger.error('Failed to load statusline config:', err)
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
    logger.error('Failed to save statusline config:', err)
    uiStore.showError(t('common.operationFailed'))
  } finally {
    saving.value = false
  }
}
</script>

