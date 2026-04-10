<template>
  <div class="space-y-6">
    <div class="glass-card p-6 transition-[transform,box-shadow] duration-300 hover:scale-[1.01]">
      <div class="mb-6 flex items-center gap-3">
        <div
          class="rounded-2xl p-3"
          :style="{ background: 'rgb(var(--color-accent-primary-rgb) / 10%)' }"
        >
          <SIcon
            name="Settings"
            size="w-6 h-6"
            :style="{ color: 'var(--accent-primary)' }"
          />
        </div>
        <h2
          class="text-xl font-bold"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ $t('sync.webdav.title') }}
        </h2>
      </div>

      <div
        v-if="syncStatus?.configured && syncStatus.config"
        class="space-y-4"
      >
        <div
          class="flex items-center gap-3 rounded-xl px-4 py-3"
          :style="{ background: 'rgb(var(--color-success-rgb) / 10%)' }"
        >
          <SIcon
            name="CheckCircle"
            size="w-5 h-5"
            :style="{ color: 'var(--accent-success)' }"
          />
          <span
            class="font-medium"
            :style="{ color: 'var(--text-primary)' }"
          >{{ $t('sync.webdav.configured') }}</span>
        </div>

        <div class="space-y-3">
          <div>
            <div
              class="mb-1 text-xs"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('sync.webdav.server') }}
            </div>
            <div
              class="break-all font-mono text-sm"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ syncStatus.config.webdav_url }}
            </div>
          </div>
          <div>
            <div
              class="mb-1 text-xs"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('sync.webdav.username') }}
            </div>
            <div
              class="font-mono text-sm"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ syncStatus.config.username }}
            </div>
          </div>
          <div>
            <div
              class="mb-1 text-xs"
              :style="{ color: 'var(--text-muted)' }"
            >
              {{ $t('sync.webdav.remotePath') }}
            </div>
            <div
              class="break-all font-mono text-sm"
              :style="{ color: 'var(--text-primary)' }"
            >
              {{ syncStatus.config.remote_path }}
            </div>
          </div>
        </div>
      </div>

      <div
        v-else
        class="space-y-4"
      >
        <div
          class="flex items-center gap-3 rounded-xl px-4 py-3"
          :style="{ background: 'rgb(var(--color-warning-rgb) / 10%)' }"
        >
          <SIcon
            name="AlertCircle"
            size="w-5 h-5"
            :style="{ color: 'var(--accent-warning)' }"
          />
          <span
            class="font-medium"
            :style="{ color: 'var(--text-primary)' }"
          >{{ $t('sync.webdav.notConfigured') }}</span>
        </div>
        <p
          class="text-sm"
          :style="{ color: 'var(--text-secondary)' }"
        >
          {{ $t('sync.webdav.configureHint') }}
        </p>
        <code
          class="glass-card block rounded-lg p-3 font-mono text-sm"
          :style="{ color: 'var(--text-primary)', background: 'rgb(var(--color-bg-elevated-rgb) / 62%)' }"
        >{{ $t('sync.webdav.configureCommand') }}</code>
      </div>
    </div>

    <div class="glass-card p-6 transition-[transform,box-shadow] duration-300 hover:scale-[1.01]">
      <div class="mb-6 flex items-center gap-3">
        <div
          class="rounded-2xl p-3"
          :style="{ background: 'rgb(var(--color-accent-tertiary-rgb) / 10%)' }"
        >
          <SIcon
            name="BookOpen"
            size="w-6 h-6"
            :style="{ color: 'var(--accent-tertiary)' }"
          />
        </div>
        <h2
          class="text-xl font-bold"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ $t('sync.features.title') }}
        </h2>
      </div>

      <div
        class="space-y-4 text-sm"
        :style="{ color: 'var(--text-secondary)' }"
      >
        <div>
          <h4
            class="mb-2 font-bold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.features.presetPlatform') }}
          </h4>
          <p>{{ $t('sync.features.presetPlatformDesc') }}</p>
        </div>
        <div>
          <h4
            class="mb-2 font-bold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.features.independentManagement') }}
          </h4>
          <p>{{ $t('sync.features.independentManagementDesc') }}</p>
        </div>
        <div>
          <h4
            class="mb-2 font-bold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.features.smartFiltering') }}
          </h4>
          <p>{{ $t('sync.features.smartFilteringDesc') }}</p>
        </div>
        <div>
          <h4
            class="mb-2 font-bold"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.features.batchOperations') }}
          </h4>
          <p>{{ $t('sync.features.batchOperationsDesc') }}</p>
        </div>
      </div>
    </div>

    <div class="glass-card p-6 transition-[transform,box-shadow] duration-300 hover:scale-[1.01]">
      <div class="mb-6 flex items-center gap-3">
        <div
          class="rounded-2xl p-3"
          :style="{ background: 'rgb(var(--color-success-rgb) / 10%)' }"
        >
          <SIcon
            name="Server"
            size="w-6 h-6"
            :style="{ color: 'var(--accent-success)' }"
          />
        </div>
        <h2
          class="text-xl font-bold"
          :style="{ color: 'var(--text-primary)' }"
        >
          {{ $t('sync.supportedServices.title') }}
        </h2>
      </div>

      <div
        class="space-y-3 text-sm"
        :style="{ color: 'var(--text-secondary)' }"
      >
        <div
          v-for="service in serviceItems"
          :key="service"
          class="flex items-center gap-2"
        >
          <SIcon
            name="CheckCircle"
            size="w-4 h-4"
            :style="{ color: 'var(--accent-success)' }"
          />
          <span>{{ service }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import SIcon from '@/components/ui/SIcon.vue'
import type { SyncStatusView } from '@/types/syncSelection'

interface Props {
  syncStatus: SyncStatusView | null
}

defineProps<Props>()

const { t } = useI18n()

const serviceItems = computed(() => [
  t('sync.supportedServices.nutstore'),
  t('sync.supportedServices.nextcloud'),
  t('sync.supportedServices.owncloud'),
  t('sync.supportedServices.any'),
])
</script>
