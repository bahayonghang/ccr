<template>
  <section
    class="grid flex-none grid-cols-1 gap-3 px-3 pt-3 sm:grid-cols-3 sm:px-4 animate-fade-in"
    aria-label="CCR status summary"
  >
    <div class="rounded-xl border border-border-color bg-bg-secondary/60 p-3">
      <div class="text-[10px] font-bold uppercase tracking-wide text-text-muted">
        {{ $t('ccrControl.status.version') }}
      </div>
      <div class="mt-1 text-sm font-semibold text-white">
        {{ versionInfo?.current_version ? `v${versionInfo.current_version}` : $t('ccrControl.status.versionUnknown') }}
      </div>
    </div>
    <div class="rounded-xl border border-accent-success/25 bg-accent-success/10 p-3">
      <div class="text-[10px] font-bold uppercase tracking-wide text-accent-success">
        {{ $t('ccrControl.status.executable') }}
      </div>
      <div class="mt-1 text-sm font-semibold text-white">
        {{ executableCommandCount }} / {{ totalCommandCount }}
      </div>
    </div>
    <div class="rounded-xl border border-accent-info/25 bg-accent-info/10 p-3">
      <div class="text-[10px] font-bold uppercase tracking-wide text-accent-info">
        {{ $t('ccrControl.status.executionModel') }}
      </div>
      <div class="mt-1 text-sm font-semibold text-white">
        {{ $t('ccrControl.status.nonStreaming') }}
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { VersionInfo } from '@/types'

defineProps<{
  versionInfo: VersionInfo | null
  executableCommandCount: number
  totalCommandCount: number
}>()
</script>
