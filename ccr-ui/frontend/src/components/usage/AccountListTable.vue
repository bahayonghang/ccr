<template>
  <div class="account-list-table">
    <!-- Table Header -->
    <div
      class="grid items-center gap-4 px-4 py-3 text-xs font-semibold uppercase tracking-wider text-text-muted border-b border-border-subtle bg-bg-surface/50 rounded-t-xl"
      :style="gridTemplateStyle"
    >
      <div class="flex items-center gap-2">
        <span>{{ $t('codex.auth.table.name') }}</span>
      </div>
      <div class="hidden sm:block">
        {{ $t('codex.auth.table.email') }}
      </div>
      <div class="hidden md:block">
        {{ $t('codex.auth.table.tokenStatus') }}
      </div>
      <div class="hidden lg:block">
        {{ $t('codex.auth.table.lastUsed') }}
      </div>
      <div class="text-right">
        {{ $t('codex.auth.table.actions') }}
      </div>
    </div>

    <!-- Table Body -->
    <div class="divide-y divide-border-subtle">
      <div
        v-for="(account, index) in accounts"
        :key="account.name"
        class="group relative grid items-center gap-4 px-4 py-3 transition-all duration-200 hover:bg-bg-overlay/50"
        :class="[
          account.is_current ? 'bg-platform-codex/5 hover:bg-platform-codex/10' : '',
          index === accounts.length - 1 ? 'rounded-b-xl' : ''
        ]"
        :style="gridTemplateStyle"
      >
        <!-- Active Indicator Line -->
        <div
          v-if="account.is_current"
          class="absolute left-0 top-0 bottom-0 w-1 bg-gradient-to-b from-platform-codex via-platform-codex/80 to-platform-codex rounded-l-xl"
        />

        <!-- Name Column -->
        <div class="flex items-center gap-3 min-w-0">
          <span class="text-lg flex-shrink-0">{{ account.freshness_icon }}</span>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="font-mono font-semibold text-text-primary truncate">
                {{ account.name }}
              </span>
              <span
                v-if="account.is_current"
                class="flex-shrink-0 inline-flex items-center gap-1 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider rounded-full bg-platform-codex/20 text-platform-codex der border-platform-codex/30"
              >
                <Check
                  class="w-2.5 h-2.5"
                  stroke-width="3"
                />
                {{ $t('codex.auth.currentBadge') }}
              </span>
              <span
                v-if="account.is_virtual"
                class="flex-shrink-0 px-2 py-0.5 text-[10px] font-medium uppercase tracking-wider rounded-full bg-gray-500/10 text-gray-500 border border-gray-500/20"
              >
                {{ $t('codex.auth.virtualBadge') }}
              </span>
            </div>
            <p
              v-if="account.description"
              class="text-xs text-text-muted truncate mt-0.5"
            >
              {{ account.description }}
            </p>
          </div>
        </div>

        <!-- Email Column -->
        <div class="hidden sm:block text-sm text-text-secondary truncate">
          {{ account.email || '—' }}
        </div>

        <!-- Token Status Column -->
        <div class="hidden md:flex items-center gap-2">
          <span
            class="inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg"
            :class="freshnessClasses(account.freshness)"
          >
            <span
              class="w-1.5 h-1.5 rounded-full"
              :class="freshnessDotClass(account.freshness)"
            />
            {{ account.freshness_description }}
          </span>
        </div>

        <!-- Last Used Column -->
        <div class="hidden lg:block text-sm text-text-muted">
          {{ account.last_used || '—' }}
        </div>

        <!-- Actions Column -->
        <div class="flex items-center justify-end gap-1">
          <!-- Quick Switch Button (non-current accounts) -->
          <button
            v-if="!account.is_current"
            class="p-2 rounded-lg text-text-muted hover:text-accent-success hover:bg-accent-success/10 transition-all duration-200 opacity-0 group-hover:opacity-100 focus:opacity-100"
            :title="$t('codex.auth.switch')"
            @click="$emit('switch', account.name)"
          >
            <ArrowRightCircle class="w-4 h-4" />
          </button>

          <!-- Current Account Indicator -->
          <div
            v-else
            class="p-2 text-platform-codex"
            :title="$t('codex.auth.currentAccount')"
          >
            <CheckCircle2 class="w-4 h-4" />
          </div>

          <!-- Delete Button (non-virtual accounts) -->
          <button
            v-if="!account.is_virtual"
            class="p-2 rounded-lg text-text-muted hover:text-accent-danger hover:bg-accent-danger/10 transition-all duration-200 opacity-0 group-hover:opacity-100 focus:opacity-100"
            :title="$t('codex.actions.delete')"
            @click="$emit('delete', account.name)"
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div
      v-if="accounts.length === 0"
      class="flex flex-col items-center justify-center py-12 text-center"
    >
      <div class="p-4 rounded-full bg-bg-surface mb-4">
        <KeyRound class="w-8 h-8 text-text-muted" />
      </div>
      <p class="text-text-secondary">
        {{ $t('codex.auth.emptyState') }}
      </p>
      <p class="text-sm text-text-muted mt-2">
        {{ $t('codex.auth.emptyStateHint') }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  ArrowRightCircle,
  Check,
  CheckCircle2,
  KeyRound,
  Trash2
} from 'lucide-vue-next'
import type { CodexAuthAccountItem, TokenFreshness } from '@/types'

interface Props {
  accounts: CodexAuthAccountItem[]
}

defineProps<Props>()

defineEmits<{
  switch: [name: string]
  delete: [name: string]
}>()

// Grid template for responsive columns
const gridTemplateStyle = computed(() => ({
  gridTemplateColumns: 'minmax(180px, 2fr) minmax(120px, 1fr) minmax(100px, auto) minmax(100px, auto) auto'
}))

// Token freshness styling
const freshnessClasses = (freshness: TokenFreshness) => {
  switch (freshness) {
    case 'Fresh':
      return 'bg-emerald-500/10 text-emerald-500 border border-emerald-500/20'
    case 'Stale':
      return 'bg-yellow-500/10 text-yellow-500 border border-yellow-500/20'
    case 'Old':
      return 'bg-orange-500/10 text-orange-500 border border-orange-500/20'
    default:
      return 'bg-gray-500/10 text-gray-500 border border-gray-500/20'
  }
}

const freshnessDotClass = (freshness: TokenFreshness) => {
  switch (freshness) {
    case 'Fresh':
      return 'bg-emerald-500'
    case 'Stale':
      return 'bg-yellow-500'
    case 'Old':
      return 'bg-orange-500'
    default:
      return 'bg-gray-500'
  }
}
</script>

<style scoped>
.account-list-table {
  @apply bg-bg-elevated rounded-xl border border-border-subtle overflow-hidden;
}

/* Responsive grid adjustments */
@media (max-width: 1024px) {
  .account-list-table > div {
    grid-template-columns: minmax(180px, 2fr) minmax(120px, 1fr) minmax(100px, auto) auto !important;
  }
}

@media (max-width: 768px) {
  .account-list-table > div {
    grid-template-columns: minmax(180px, 2fr) minmax(120px, 1fr) auto !important;
  }
}

@media (max-width: 640px) {
  .account-list-table > div {
    grid-template-columns: 1fr auto !important;
  }
}
</style>
