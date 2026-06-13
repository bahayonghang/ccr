<template>
  <div class="codex-auth-accounts-tab">
    <Card
      v-if="currentInfo"
      surface="workspace"
      :elevation="2"
      motion="subtle"
      padding="lg"
    >
      <div class="codex-auth-view__section-header">
        <SIcon
          name="Info"
          size="w-5 h-5"
          class="codex-auth-view__section-icon"
        />
        <h3 class="codex-auth-view__section-title">
          {{ $t('codex.auth.currentSession') }}
        </h3>
      </div>
      <div class="codex-auth-view__session-grid">
        <div class="codex-auth-view__session-field">
          <span class="codex-auth-view__field-label">
            {{ $t('codex.auth.fields.accountId') }}
          </span>
          <code class="codex-auth-view__field-code">
            {{ currentInfo.account_id }}
          </code>
        </div>
        <div class="codex-auth-view__session-field">
          <span class="codex-auth-view__field-label">
            {{ $t('codex.auth.fields.email') }}
          </span>
          <span class="codex-auth-view__field-value codex-auth-view__field-value--truncate">
            {{ currentInfo.email || $t('codex.auth.status.notAvailable') }}
          </span>
        </div>
        <div class="codex-auth-view__session-field">
          <span class="codex-auth-view__field-label">
            {{ tf('codex.auth.fields.authMethod', 'Auth method') }}
          </span>
          <span class="codex-auth-view__field-value codex-auth-view__field-value--muted">
            {{ formatAuthMethod(currentInfo.auth_method || '') }}
          </span>
        </div>
        <div class="codex-auth-view__session-field">
          <span class="codex-auth-view__field-label">
            {{ tf('codex.auth.fields.planType', 'Plan') }}
          </span>
          <span class="codex-auth-view__field-value codex-auth-view__field-value--muted">
            {{ currentInfo.plan_type || $t('codex.auth.status.notAvailable') }}
          </span>
        </div>
        <div class="codex-auth-view__session-field">
          <span class="codex-auth-view__field-label">
            {{ $t('codex.auth.fields.lastRefresh') }}
          </span>
          <span class="codex-auth-view__field-value codex-auth-view__field-value--muted">
            {{ currentInfo.last_refresh || $t('codex.auth.status.notAvailable') }}
          </span>
        </div>
      </div>
    </Card>

    <Card
      surface="workspace"
      :elevation="2"
      motion="subtle"
      padding="lg"
      :glow-color="canManageAuthAccounts ? 'success' : 'warning'"
    >
      <div class="codex-auth-view__guard">
        <div
          class="codex-auth-view__guard-icon-shell"
          :class="
            canManageAuthAccounts
              ? 'bg-emerald-500/10 text-emerald-400'
              : 'bg-yellow-500/10 text-yellow-400'
          "
        >
          <SIcon
            name="AlertTriangle"
            size="w-5 h-5"
          />
        </div>
        <div class="codex-auth-view__guard-body">
          <p class="codex-auth-view__guard-title">
            {{ $t('codex.auth.profileGuard.title') }}
          </p>
          <p class="codex-auth-view__guard-message">
            {{ profileGuardMessage }}
          </p>
          <p
            v-if="authActionError"
            class="codex-auth-view__guard-error"
          >
            {{ authActionError }}
          </p>
        </div>
      </div>
    </Card>

    <Card
      v-if="!loading && accounts.length > 0"
      surface="workspace"
      :elevation="2"
      motion="subtle"
      padding="lg"
      class="codex-auth-view__filters-card"
    >
      <div class="codex-auth-view__filters-grid">
        <label class="codex-auth-view__search-box">
          <SIcon
            name="Search"
            size="w-4 h-4"
          />
          <input
            :value="searchQuery"
            type="text"
            :placeholder="$t('codex.auth.filters.searchPlaceholder')"
            @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
          >
        </label>

        <div class="codex-auth-view__filter-group">
          <p class="codex-auth-view__filter-label">
            {{ $t('codex.auth.filters.statusLabel') }}
          </p>
          <div class="codex-auth-view__filter-row">
            <button
              v-for="option in statusOptions"
              :key="option.value"
              type="button"
              class="codex-auth-view__filter-pill"
              :class="{
                'codex-auth-view__filter-pill--active': statusFilter === option.value,
              }"
              @click="$emit('update:statusFilter', option.value)"
            >
              {{ option.label }}
            </button>
          </div>
        </div>

        <div class="codex-auth-view__filter-group">
          <label
            class="codex-auth-view__filter-label"
            for="codex-auth-plan-filter"
          >
            {{ $t('codex.auth.filters.planLabel') }}
          </label>
          <select
            id="codex-auth-plan-filter"
            :value="planFilter"
            class="codex-auth-view__filter-select"
            @change="$emit('update:planFilter', ($event.target as HTMLSelectElement).value as AccountPlanFilter)"
          >
            <option
              v-for="option in planOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </select>
        </div>

        <div class="codex-auth-view__filter-group">
          <label
            class="codex-auth-view__filter-label"
            for="codex-auth-sort"
          >
            {{ $t('codex.auth.filters.sortLabel') }}
          </label>
          <select
            id="codex-auth-sort"
            :value="sortBy"
            class="codex-auth-view__filter-select"
            @change="$emit('update:sortBy', ($event.target as HTMLSelectElement).value as AccountSort)"
          >
            <option
              v-for="option in sortOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </select>
        </div>
      </div>

      <div class="codex-auth-view__filters-footer">
        <p class="codex-auth-view__filters-summary">
          {{ filtersResultsCount }}
        </p>
        <Button
          v-if="hasActiveFilters"
          variant="secondary"
          surface="status"
          density="compact"
          motion="subtle"
          @click="$emit('clearFilters')"
        >
          {{ $t('common.clearFilters') }}
        </Button>
      </div>
    </Card>

    <div
      v-if="loading"
      class="flex justify-center py-20"
    >
      <div
        class="w-12 h-12 rounded-full border-4 border-transparent border-t-accent-primary border-r-accent-secondary animate-spin"
      />
    </div>

    <div
      v-else-if="accounts.length === 0"
      class="empty-state glass-effect rounded-2xl border border-border-default/10"
    >
      <div class="p-4 rounded-full glass-surface mb-4">
        <SIcon
          name="KeyRound"
          size="w-8 h-8"
          class="text-text-muted"
        />
      </div>
      <p class="text-text-primary">
        {{ $t('codex.auth.emptyState') }}
      </p>
      <p class="text-sm text-text-muted mt-2">
        {{
          tf(
            'codex.auth.emptyStateHintV2',
            'Add a new account through OAuth, API key, token JSON, or import the local runtime snapshot.'
          )
        }}
      </p>
      <Button
        variant="primary"
        surface="card"
        density="compact"
        motion="standard"
        class="mt-4"
        @click="$emit('openAddAccount')"
      >
        <template #leading>
          <SIcon
            name="Plus"
            size="w-4 h-4"
          />
        </template>
        {{ tf('codex.auth.actions.addAccount', 'Add account') }}
      </Button>
    </div>

    <div
      v-else-if="filteredAccounts.length === 0"
      class="empty-state glass-effect rounded-2xl border border-border-default/10"
    >
      <div class="p-4 rounded-full glass-surface mb-4">
        <SIcon
          name="Search"
          size="w-8 h-8"
          class="text-text-muted"
        />
      </div>
      <p class="text-text-primary">
        {{ $t('codex.auth.filters.noResultsTitle') }}
      </p>
      <p class="text-sm text-text-muted mt-2">
        {{ $t('codex.auth.filters.noResultsHint') }}
      </p>
      <Button
        variant="secondary"
        surface="status"
        density="compact"
        motion="subtle"
        class="mt-4"
        @click="$emit('clearFilters')"
      >
        {{ $t('common.clearFilters') }}
      </Button>
    </div>

    <div
      v-else
      class="grid grid-cols-1 xl:grid-cols-2 gap-4"
    >
      <CodexAccountCard
        v-for="account in filteredAccounts"
        :key="account.name"
        :account="account"
        :quota="quotaMap.get(account.name) ?? null"
        :quota-loading="quotaLoading"
        :is-current="account.is_current"
        :busy-action="busyName === account.name ? busyAction : null"
        :disabled="actionLoading"
        @switch="$emit('switch', $event)"
        @delete="$emit('delete', $event)"
        @refresh="$emit('refresh', $event)"
        @tag="$emit('tag', $event)"
        @export="$emit('export', $event)"
        @rename="$emit('rename', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { CodexAuthAccountItem, CodexAuthCurrentInfo, CodexAccountQuota } from '@/types'
import type { AccountStatusFilter, AccountPlanFilter, AccountSort } from '@/views/codex/codexAuthAccounts'
import SIcon from '@/components/ui/SIcon.vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import CodexAccountCard from '@/components/codex/CodexAccountCard.vue'
import { useTf } from '@/composables/useTf'

defineOptions({ name: 'CodexAuthAccountsTab' })

interface Props {
  loading: boolean
  accounts: CodexAuthAccountItem[]
  currentInfo: CodexAuthCurrentInfo | null
  canManageAuthAccounts: boolean
  profileGuardMessage: string
  authActionError: string | null
  searchQuery: string
  statusFilter: AccountStatusFilter
  planFilter: AccountPlanFilter
  sortBy: AccountSort
  statusOptions: Array<{ value: AccountStatusFilter; label: string }>
  planOptions: Array<{ value: AccountPlanFilter; label: string }>
  sortOptions: Array<{ value: AccountSort; label: string }>
  filteredAccounts: CodexAuthAccountItem[]
  filtersResultsCount: string
  hasActiveFilters: boolean
  quotaMap: Map<string, CodexAccountQuota>
  quotaLoading: boolean
  busyName: string | null
  busyAction: 'switch' | 'delete' | null
  actionLoading: boolean
  formatAuthMethod: (method: string) => string
}

defineProps<Props>()

defineEmits<{
  'update:searchQuery': [value: string]
  'update:statusFilter': [value: AccountStatusFilter]
  'update:planFilter': [value: AccountPlanFilter]
  'update:sortBy': [value: AccountSort]
  clearFilters: []
  openAddAccount: []
  switch: [name: string]
  delete: [name: string]
  refresh: [name: string]
  tag: [name: string]
  export: [name: string]
  rename: [name: string]
}>()

const tf = useTf()
</script>

<style scoped>
/* 样式继承自 CodexAuthView.vue，无需重复定义 */
</style>
