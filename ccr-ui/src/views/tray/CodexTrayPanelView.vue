<template>
  <main class="codex-tray-panel">
    <section class="codex-tray-panel__shell">
      <header class="codex-tray-panel__header">
        <div>
          <p class="codex-tray-panel__eyebrow">
            CCR Desktop
          </p>
          <h1 class="codex-tray-panel__title">
            Codex Tray
          </h1>
        </div>
        <button
          type="button"
          class="codex-tray-panel__icon-button"
          :disabled="loading"
          @click="loadSnapshot(true)"
        >
          <SIcon
            name="RefreshCw"
            size="w-4 h-4"
            :class="{ 'animate-spin': loading }"
          />
        </button>
      </header>

      <div
        v-if="error"
        class="codex-tray-panel__callout codex-tray-panel__callout--danger"
      >
        <SIcon
          name="AlertTriangle"
          size="w-4 h-4"
        />
        <p>{{ error }}</p>
      </div>

      <template v-if="snapshot">
        <TrayOverview
          v-if="screen === 'overview'"
          :snapshot="snapshot"
          :current-account="currentAccount"
          :can-manage-accounts="canManageAccounts"
          @open-main="openMain()"
          @open-switch="goToSwitchScreen()"
          @open-usage="openUsage()"
          @open-auth="openAuth()"
          @quit="quit()"
        />

        <TrayAccountSwitchScreen
          v-else
          :snapshot="snapshot"
          :current-account="currentAccount"
          :accounts="accounts"
          :busy-account="busyAccount"
          :can-manage-accounts="canManageAccounts"
          @back="goToOverview()"
          @switch="switchAccount"
          @open-auth="openAuth()"
        />
      </template>

      <div
        v-else
        class="codex-tray-panel__callout"
      >
        <SIcon
          name="Clock3"
          size="w-4 h-4"
        />
        <p>{{ loading ? 'Loading Codex tray…' : 'No tray snapshot yet.' }}</p>
      </div>
    </section>
  </main>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { useCodexTrayPanel } from '@/composables/useCodexTrayPanel'
import TrayAccountSwitchScreen from '@/views/tray/components/TrayAccountSwitchScreen.vue'
import TrayOverview from '@/views/tray/components/TrayOverview.vue'

const {
  accounts,
  busyAccount,
  canManageAccounts,
  currentAccount,
  error,
  goToOverview,
  goToSwitchScreen,
  loadSnapshot,
  loading,
  openAuth,
  openMain,
  openUsage,
  quit,
  screen,
  snapshot,
  switchAccount,
} = useCodexTrayPanel()
</script>

<style scoped>
.codex-tray-panel {
  min-height: 100vh;
  padding: 12px;
  background:
    radial-gradient(circle at top, rgb(var(--color-accent-primary-rgb) / 10%), transparent 48%),
    rgb(var(--color-bg-base-rgb) / 100%);
}

.codex-tray-panel__shell {
  display: flex;
  min-height: calc(100vh - 24px);
  flex-direction: column;
  gap: 14px;
  overflow: hidden;
  border: 1px solid rgb(var(--color-border-default-rgb) / 44%);
  border-radius: 24px;
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 96%), rgb(var(--color-bg-surface-rgb) / 92%));
  box-shadow:
    0 24px 56px rgb(32 28 24 / 18%),
    inset 0 1px 0 rgb(255 255 255 / 10%);
  padding: 14px;
}

.codex-tray-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.codex-tray-panel__eyebrow {
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

.codex-tray-panel__title {
  color: var(--color-text-primary);
  font-size: 1.1rem;
  font-weight: 700;
  letter-spacing: -0.04em;
}

.codex-tray-panel__icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 52%);
  border-radius: 999px;
  background: rgb(var(--color-bg-base-rgb) / 80%);
  color: var(--color-text-secondary);
  transition: border-color 0.18s ease, background-color 0.18s ease, color 0.18s ease, transform 0.18s ease;
}

.codex-tray-panel__icon-button:hover:not(:disabled) {
  transform: translateY(-1px);
  border-color: rgb(var(--color-accent-primary-rgb) / 24%);
  background: rgb(var(--color-bg-elevated-rgb) / 92%);
  color: var(--color-text-primary);
}

.codex-tray-panel__icon-button:disabled {
  opacity: 0.48;
  cursor: not-allowed;
}

.codex-tray-panel__callout {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 42%);
  border-radius: 18px;
  background: rgb(var(--color-bg-base-rgb) / 48%);
  padding: 12px;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.45;
}

.codex-tray-panel__callout--danger {
  border-color: rgb(var(--color-danger-rgb) / 26%);
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--color-danger);
}
</style>
