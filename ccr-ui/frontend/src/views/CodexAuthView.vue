<template>
  <div class="min-h-screen bg-bg-base p-6">
    <div class="max-w-[1800px] mx-auto">
      <Breadcrumb
        :items="[
          { label: $t('common.home'), path: '/', icon: Home },
          { label: 'Codex', path: '/codex', icon: Boxes },
          { label: $t('codex.auth.breadcrumb'), path: '/codex/auth', icon: KeyRound }
        ]"
        module-color="#ec4899"
      />

      <div class="grid grid-cols-[auto_1fr] gap-6 mt-6">
        <CollapsibleSidebar module="codex" />

        <main class="flex flex-col gap-6 w-full min-w-0">
          <!-- Header Section -->
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <div class="p-2 rounded-xl bg-platform-codex/10">
                <KeyRound class="w-6 h-6 text-platform-codex" />
              </div>
              <div>
                <h1 class="text-2xl font-bold text-text-primary">
                  {{ $t('codex.auth.title') }}
                </h1>
                <p class="text-sm text-text-secondary mt-1">
                  {{ $t('codex.auth.subtitle') }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <RouterLink
                to="/codex"
                class="btn btn-secondary"
              >
                <ArrowLeft class="w-4 h-4" />
                <span>{{ $t('codex.auth.backToCodex') }}</span>
              </RouterLink>

              <button
                class="btn btn-primary"
                :disabled="!canSave"
                @click="handleSave"
              >
                <Save class="w-4 h-4" />
                {{ $t('codex.auth.saveAccount') }}
              </button>
            </div>
          </div>

          <!-- Status Cards -->
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <!-- Login State -->
            <GuofengCard
              :gradient-border="true"
              :glow-color="loginStateColor"
              class="relative overflow-hidden group"
            >
              <div class="flex items-center gap-4">
                <div
                  class="p-3 rounded-xl group-hover:scale-110 transition-transform duration-300"
                  :class="loginStateIconClass"
                >
                  <component
                    :is="loginStateIcon"
                    class="w-6 h-6"
                  />
                </div>
                <div>
                  <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-1">
                    {{ $t('codex.auth.status.loginState') }}
                  </p>
                  <p class="text-xl font-bold text-text-primary truncate">
                    {{ loginStateText }}
                  </p>
                </div>
              </div>
            </GuofengCard>

            <!-- Total Accounts -->
            <GuofengCard
              :interactive="true"
              glow-color="primary"
              class="group"
            >
              <div class="flex items-center gap-4">
                <div class="p-3 rounded-xl bg-indigo-500/10 text-indigo-500 group-hover:scale-110 transition-transform duration-300">
                  <Users class="w-6 h-6" />
                </div>
                <div>
                  <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-1">
                    {{ $t('codex.auth.status.totalAccounts') }}
                  </p>
                  <p class="text-xl font-bold text-text-primary">
                    {{ accounts.length }}
                  </p>
                </div>
              </div>
            </GuofengCard>

            <!-- Current Account -->
            <GuofengCard
              :interactive="true"
              :glow-color="currentAccount ? 'success' : 'secondary'"
              class="group"
            >
              <div class="flex items-center gap-4">
                <div
                  class="p-3 rounded-xl transition-colors duration-300 group-hover:scale-110 transition-transform"
                  :class="currentAccount ? 'bg-emerald-500/10 text-emerald-500' : 'bg-gray-500/10 text-gray-500'"
                >
                  <UserCheck class="w-6 h-6" />
                </div>
                <div>
                  <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-1">
                    {{ $t('codex.auth.status.currentAccount') }}
                  </p>
                  <p class="text-xl font-bold text-text-primary truncate">
                    {{ currentAccount?.name || $t('codex.auth.status.noAccount') }}
                  </p>
                </div>
              </div>
            </GuofengCard>
          </div>

          <!-- Current Session Info -->
          <GuofengCard
            v-if="currentInfo"
            padding="lg"
          >
            <div class="flex items-center gap-2 mb-4">
              <Info class="w-5 h-5 text-platform-codex" />
              <h3 class="text-base font-semibold text-text-primary">
                {{ $t('codex.auth.currentSession') }}
              </h3>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <div class="flex flex-col gap-1">
                <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                  {{ $t('codex.auth.fields.accountId') }}
                </span>
                <code class="font-mono text-text-primary truncate px-2 py-1 rounded bg-bg-surface border border-border-subtle">
                  {{ currentInfo.account_id }}
                </code>
              </div>
              <div class="flex flex-col gap-1">
                <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                  {{ $t('codex.auth.fields.email') }}
                </span>
                <span class="text-text-primary truncate">
                  {{ currentInfo.email || $t('codex.auth.status.notAvailable') }}
                </span>
              </div>
              <div class="flex flex-col gap-1">
                <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                  {{ $t('codex.auth.fields.tokenFreshness') }}
                </span>
                <div class="flex items-center gap-2">
                  <span>{{ currentInfo.freshness_icon }}</span>
                  <span
                    class="text-sm font-medium"
                    :class="freshnessClass(currentInfo.freshness)"
                  >
                    {{ currentInfo.freshness_description }}
                  </span>
                </div>
              </div>
              <div class="flex flex-col gap-1">
                <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                  {{ $t('codex.auth.fields.lastRefresh') }}
                </span>
                <span class="text-text-secondary text-sm">
                  {{ currentInfo.last_refresh || $t('codex.auth.status.notAvailable') }}
                </span>
              </div>
            </div>
          </GuofengCard>

          <!-- Quick Switch -->
          <GuofengCard
            v-if="accounts.length > 0"
            padding="lg"
          >
            <div class="flex items-center gap-2 mb-4">
              <Shuffle class="w-5 h-5 text-platform-codex" />
              <h3 class="text-base font-semibold text-text-primary">
                {{ $t('codex.auth.quickSwitch') }}
              </h3>
            </div>
            <div class="flex flex-wrap gap-3">
              <button
                v-for="account in accounts"
                :key="account.name"
                class="group relative px-4 py-2.5 rounded-xl font-medium text-sm transition-all duration-300 border flex items-center gap-2.5"
                :class="[
                  account.is_current
                    ? 'bg-platform-codex/10 border-platform-codex/50 text-platform-codex shadow-[0_0_15px_rgba(245,158,11,0.2)]'
                    : 'bg-bg-surface border-border-default text-text-secondary hover:border-platform-codex/30 hover:bg-bg-overlay'
                ]"
                @click="handleSwitch(account.name)"
              >
                <span>{{ account.freshness_icon }}</span>
                <span>{{ account.name }}</span>
                <span
                  v-if="account.is_virtual"
                  class="text-xs text-text-muted"
                >
                  ({{ $t('codex.auth.virtual') }})
                </span>
                <div
                  v-if="account.is_current"
                  class="flex items-center justify-center w-4 h-4 rounded-full bg-platform-codex text-white text-[10px]"
                >
                  <Check
                    class="w-2.5 h-2.5"
                    stroke-width="3"
                  />
                </div>
              </button>
            </div>
          </GuofengCard>

          <!-- Account List Title -->
          <div class="flex items-center justify-between">
            <h2 class="text-xl font-bold text-text-primary flex items-center gap-2">
              <ListFilter class="w-5 h-5 text-platform-codex" />
              {{ $t('codex.auth.listTitle') }}
            </h2>
            <button
              class="btn btn-secondary btn-sm"
              @click="handleRefresh"
            >
              <RefreshCw
                class="w-4 h-4"
                :class="{ 'animate-spin': loading }"
              />
              {{ $t('codex.auth.refresh') }}
            </button>
          </div>

          <!-- Loading State -->
          <div
            v-if="loading"
            class="flex justify-center py-20"
          >
            <div class="w-12 h-12 rounded-full border-4 border-transparent border-t-accent-primary border-r-accent-secondary animate-spin" />
          </div>

          <!-- Empty State -->
          <div
            v-else-if="accounts.length === 0"
            class="empty-state bg-bg-elevated rounded-2xl border border-border-subtle"
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

          <!-- Account Grid -->
          <div
            v-else
            class="grid grid-cols-1 xl:grid-cols-2 gap-4"
          >
            <GuofengCard
              v-for="account in accounts"
              :key="account.name"
              class="group relative overflow-hidden transition-all duration-300 hover:-translate-y-1 hover:shadow-xl"
              :class="{ 'ring-1 ring-platform-codex/50': account.is_current }"
              :glow-color="account.is_current ? 'warning' : 'primary'"
              padding="lg"
            >
              <!-- Active Indicator Background -->
              <div
                v-if="account.is_current"
                class="absolute top-0 right-0 w-32 h-32 bg-gradient-to-bl from-platform-codex/10 to-transparent -mr-8 -mt-8 rounded-bl-full pointer-events-none"
              />

              <div class="relative z-10">
                <div class="flex items-start justify-between gap-4 mb-4">
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                      <span class="text-2xl">{{ account.freshness_icon }}</span>
                      <h3 class="text-lg font-bold font-mono text-text-primary truncate">
                        {{ account.name }}
                      </h3>
                      <span
                        v-if="account.is_current"
                        class="badge badge-primary"
                      >
                        {{ $t('codex.auth.currentBadge') }}
                      </span>
                      <span
                        v-if="account.is_virtual"
                        class="badge badge-secondary"
                      >
                        {{ $t('codex.auth.virtualBadge') }}
                      </span>
                    </div>
                    <p
                      v-if="account.description"
                      class="text-sm text-text-secondary line-clamp-1"
                    >
                      {{ account.description }}
                    </p>
                  </div>

                  <!-- Actions -->
                  <div class="flex items-center gap-1 opacity-100 xl:opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                    <button
                      v-if="!account.is_current"
                      class="p-2 rounded-lg hover:bg-bg-overlay text-accent-success transition-colors"
                      :title="$t('codex.auth.switch')"
                      @click.stop="handleSwitch(account.name)"
                    >
                      <Check class="w-4 h-4" />
                    </button>
                    <button
                      v-if="!account.is_virtual"
                      class="p-2 rounded-lg hover:bg-bg-overlay text-accent-danger transition-colors"
                      :title="$t('codex.actions.delete')"
                      @click.stop="handleDelete(account.name)"
                    >
                      <Trash2 class="w-4 h-4" />
                    </button>
                  </div>
                </div>

                <!-- Info Grid -->
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-y-3 gap-x-6 text-sm">
                  <div class="flex flex-col gap-1">
                    <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                      {{ $t('codex.auth.fields.email') }}
                    </span>
                    <span class="text-text-primary truncate">
                      {{ account.email || $t('codex.auth.status.notAvailable') }}
                    </span>
                  </div>

                  <div class="flex flex-col gap-1">
                    <span class="text-xs font-medium text-text-muted uppercase tracking-wider">
                      {{ $t('codex.auth.fields.tokenFreshness') }}
                    </span>
                    <span
                      class="font-medium"
                      :class="freshnessClass(account.freshness)"
                    >
                      {{ account.freshness_description }}
                    </span>
                  </div>
                </div>

                <div
                  v-if="account.last_used || account.last_refresh"
                  class="mt-4 flex items-center justify-between border-t border-border-subtle pt-3 text-xs text-text-muted"
                >
                  <span v-if="account.last_used">
                    {{ $t('codex.auth.fields.lastUsed') }}: {{ account.last_used }}
                  </span>
                  <span v-if="account.last_refresh">
                    {{ $t('codex.auth.fields.lastRefresh') }}: {{ account.last_refresh }}
                  </span>
                </div>
              </div>
            </GuofengCard>
          </div>

          <!-- Save Modal -->
          <div
            v-if="showSaveForm"
            class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50"
          >
            <GuofengCard
              class="w-full max-w-lg max-h-[90vh] overflow-y-auto !p-0 shadow-2xl animate-in zoom-in-95 duration-200"
              :padding="'none'"
            >
              <!-- Modal Header -->
              <div class="px-6 py-4 border-b border-border-subtle flex items-center justify-between sticky top-0 bg-bg-elevated/95 backdrop-blur z-10">
                <h2 class="text-xl font-bold text-text-primary">
                  {{ $t('codex.auth.saveAccount') }}
                </h2>
                <button
                  class="p-1 rounded-lg hover:bg-bg-overlay text-text-muted transition-colors"
                  @click="handleCloseSaveForm"
                >
                  <X class="w-5 h-5" />
                </button>
              </div>

              <!-- Modal Content -->
              <div class="p-6 space-y-6">
                <!-- Process Warning -->
                <div
                  v-if="processWarning"
                  class="p-4 rounded-lg bg-yellow-500/10 border border-yellow-500/30 text-yellow-600 dark:text-yellow-400"
                >
                  <div class="flex items-start gap-3">
                    <AlertTriangle class="w-5 h-5 flex-shrink-0 mt-0.5" />
                    <div>
                      <p class="font-medium">
                        {{ $t('codex.auth.processWarning') }}
                      </p>
                      <p class="text-sm mt-1 opacity-80">
                        {{ processWarning }}
                      </p>
                    </div>
                  </div>
                </div>

                <div class="space-y-4">
                  <div class="space-y-1.5">
                    <label class="text-sm font-semibold text-text-secondary">
                      {{ $t('codex.auth.fields.accountName') }} <span class="text-red-500">*</span>
                    </label>
                    <input
                      v-model="saveForm.name"
                      type="text"
                      class="input"
                      :placeholder="$t('codex.auth.placeholders.accountName')"
                    >
                  </div>
                  <div class="space-y-1.5">
                    <label class="text-sm font-semibold text-text-secondary">
                      {{ $t('codex.auth.fields.description') }}
                    </label>
                    <input
                      v-model="saveForm.description"
                      type="text"
                      class="input"
                      :placeholder="$t('codex.auth.placeholders.description')"
                    >
                  </div>
                  <div class="flex items-center gap-3 p-3 rounded-lg bg-bg-surface border border-border-subtle">
                    <input
                      id="forceOverwrite"
                      v-model="saveForm.force"
                      type="checkbox"
                      class="w-5 h-5 rounded border-border-default text-accent-primary focus:ring-accent-primary/20"
                    >
                    <label
                      for="forceOverwrite"
                      class="text-sm font-medium text-text-primary cursor-pointer select-none"
                    >
                      {{ $t('codex.auth.forceOverwrite') }}
                    </label>
                  </div>
                </div>
              </div>

              <!-- Footer -->
              <div class="px-6 py-4 border-t border-border-subtle flex justify-end gap-3 bg-bg-surface/50">
                <button
                  class="btn btn-secondary"
                  @click="handleCloseSaveForm"
                >
                  {{ $t('codex.actions.cancel') }}
                </button>
                <button
                  class="btn btn-primary"
                  :disabled="saving || !saveForm.name.trim()"
                  @click="handleConfirmSave"
                >
                  <span
                    v-if="saving"
                    class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2"
                  />
                  {{ saving ? $t('codex.states.saving') : $t('codex.actions.save') }}
                </button>
              </div>
            </GuofengCard>
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
import {
  AlertTriangle,
  ArrowLeft,
  Boxes,
  Check,
  Home,
  Info,
  KeyRound,
  ListFilter,
  LogIn,
  LogOut,
  RefreshCw,
  Save,
  Shuffle,
  Trash2,
  UserCheck,
  Users,
  X
} from 'lucide-vue-next'

import Breadcrumb from '@/components/Breadcrumb.vue'
import CollapsibleSidebar from '@/components/CollapsibleSidebar.vue'
import GuofengCard from '@/components/common/GuofengCard.vue'
import {
  listCodexAuthAccounts,
  getCodexAuthCurrent,
  saveCodexAuth,
  switchCodexAuth,
  deleteCodexAuth,
  detectCodexProcess
} from '@/api'
import type {
  CodexAuthAccountItem,
  CodexAuthCurrentInfo,
  LoginState,
  TokenFreshness
} from '@/types'

const { t } = useI18n()

const loading = ref(false)
const saving = ref(false)

const accounts = ref<CodexAuthAccountItem[]>([])
const loginState = ref<LoginState>({ type: 'NotLoggedIn' })
const currentInfo = ref<CodexAuthCurrentInfo | null>(null)

const showSaveForm = ref(false)
const processWarning = ref<string | null>(null)

const saveForm = reactive({
  name: '',
  description: '',
  force: false,
})

// Computed properties
const currentAccount = computed(() => accounts.value.find(a => a.is_current))

const canSave = computed(() => {
  return loginState.value.type === 'LoggedInUnsaved' || loginState.value.type === 'LoggedInSaved'
})

const loginStateColor = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved': return 'success'
    case 'LoggedInUnsaved': return 'warning'
    default: return 'danger'
  }
})

const loginStateIcon = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved': return UserCheck
    case 'LoggedInUnsaved': return LogIn
    default: return LogOut
  }
})

const loginStateIconClass = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved': return 'bg-emerald-500/10 text-emerald-500'
    case 'LoggedInUnsaved': return 'bg-yellow-500/10 text-yellow-500'
    default: return 'bg-red-500/10 text-red-500'
  }
})

const loginStateText = computed(() => {
  switch (loginState.value.type) {
    case 'LoggedInSaved':
      return t('codex.auth.loginState.loggedInSaved', { name: loginState.value.account_name })
    case 'LoggedInUnsaved':
      return t('codex.auth.loginState.loggedInUnsaved')
    default:
      return t('codex.auth.loginState.notLoggedIn')
  }
})

// Helper functions
const freshnessClass = (freshness: TokenFreshness) => {
  switch (freshness) {
    case 'Fresh': return 'text-emerald-500'
    case 'Stale': return 'text-yellow-500'
    case 'Old': return 'text-orange-500'
    default: return 'text-gray-500'
  }
}

// Data loading
const loadAccounts = async () => {
  try {
    loading.value = true
    const data = await listCodexAuthAccounts()
    accounts.value = data.accounts || []
    loginState.value = data.login_state
  } catch (error) {
    console.error('Failed to load codex auth accounts:', error)
    alert(t('codex.states.loadFailed'))
  } finally {
    loading.value = false
  }
}

const loadCurrentInfo = async () => {
  try {
    const data = await getCodexAuthCurrent()
    if (data.logged_in && data.info) {
      currentInfo.value = data.info
    } else {
      currentInfo.value = null
    }
  } catch (error) {
    console.error('Failed to load current auth info:', error)
  }
}

const handleRefresh = async () => {
  await Promise.all([loadAccounts(), loadCurrentInfo()])
}

// Actions
const handleSave = async () => {
  // Check for running Codex processes
  try {
    const processInfo = await detectCodexProcess()
    if (processInfo.has_running_process) {
      processWarning.value = processInfo.warning || t('codex.auth.processDetected', { pids: processInfo.pids.join(', ') })
    } else {
      processWarning.value = null
    }
  } catch {
    processWarning.value = null
  }

  // Reset form
  saveForm.name = ''
  saveForm.description = ''
  saveForm.force = false
  showSaveForm.value = true
}

const handleCloseSaveForm = () => {
  showSaveForm.value = false
  processWarning.value = null
}

const handleConfirmSave = async () => {
  if (!saveForm.name.trim()) {
    alert(t('codex.auth.validation.nameRequired'))
    return
  }

  try {
    saving.value = true
    await saveCodexAuth({
      name: saveForm.name.trim(),
      description: saveForm.description.trim() || undefined,
      force: saveForm.force,
    })
    handleCloseSaveForm()
    await handleRefresh()
  } catch (error) {
    console.error('Failed to save auth:', error)
    alert(t('codex.states.saveFailed'))
  } finally {
    saving.value = false
  }
}

const handleSwitch = async (name: string) => {
  if (!confirm(t('codex.auth.confirmSwitch', { name }))) return
  try {
    await switchCodexAuth(name)
    await handleRefresh()
  } catch (error) {
    console.error('Failed to switch auth:', error)
    alert(t('codex.states.saveFailed'))
  }
}

const handleDelete = async (name: string) => {
  if (!confirm(t('codex.auth.confirmDelete', { name }))) return
  try {
    await deleteCodexAuth(name)
    await handleRefresh()
  } catch (error) {
    console.error('Failed to delete auth:', error)
    alert(t('codex.states.deleteFailed'))
  }
}

onMounted(async () => {
  await handleRefresh()
})
</script>
