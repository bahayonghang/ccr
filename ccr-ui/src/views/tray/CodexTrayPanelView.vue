<template>
  <main class="codex-tray-panel">
    <section class="codex-tray-panel__shell">
      <header class="codex-tray-panel__header">
        <div
          class="codex-tray-panel__drag-surface"
          :class="{ 'codex-tray-panel__drag-surface--dragging': isDragging }"
          :title="$t('codex.auth.tray.dragWindow')"
          @mousedown.left.prevent="startPanelDrag"
        >
          <span
            class="codex-tray-panel__drag-grip"
            aria-hidden="true"
          >
            <span />
            <span />
            <span />
          </span>
          <div class="codex-tray-panel__header-copy">
            <p class="codex-tray-panel__eyebrow">
              CCR Desktop
            </p>
            <h1 class="codex-tray-panel__title">
              Codex Tray
            </h1>
          </div>
        </div>
        <button
          type="button"
          class="codex-tray-panel__icon-button"
          title="Refresh"
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
        aria-live="polite"
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
  isDragging,
  loadSnapshot,
  loading,
  openAuth,
  openMain,
  openUsage,
  quit,
  screen,
  snapshot,
  startPanelDrag,
  switchAccount,
} = useCodexTrayPanel()
</script>

<style scoped>
.codex-tray-panel {
  min-height: 100vh;
  padding: 14px;
  background:
    radial-gradient(circle at 14% 0%, rgb(var(--color-accent-primary-rgb) / 10%), transparent 28%),
    radial-gradient(circle at 100% 100%, rgb(var(--color-accent-primary-rgb) / 6%), transparent 22%),
    rgb(var(--color-bg-base-rgb) / 100%);
}

.codex-tray-panel__shell {
  display: flex;
  min-height: calc(100vh - 28px);
  flex-direction: column;
  gap: 16px;
  overflow: hidden;
  border: 1px solid rgb(var(--color-border-default-rgb) / 44%);
  border-radius: 28px;
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 97%), rgb(var(--color-bg-surface-rgb) / 92%));
  box-shadow:
    0 30px 68px rgb(32 28 24 / 18%),
    inset 0 1px 0 rgb(255 255 255 / 10%);
  padding: 16px;
}

.codex-tray-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.codex-tray-panel__drag-surface {
  display: flex;
  min-width: 0;
  flex: 1;
  align-items: center;
  gap: 12px;
  cursor: grab;
  user-select: none;
}

.codex-tray-panel__drag-surface--dragging {
  cursor: grabbing;
}

.codex-tray-panel__drag-grip {
  display: inline-grid;
  grid-template-columns: repeat(2, 4px);
  gap: 4px;
  padding: 8px 6px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 32%);
  border-radius: 14px;
  background: rgb(var(--color-bg-base-rgb) / 60%);
  flex-shrink: 0;
}

.codex-tray-panel__drag-grip span {
  width: 4px;
  height: 4px;
  border-radius: 999px;
  background: rgb(var(--color-text-muted-rgb, 148 163 184) / 88%);
}

.codex-tray-panel__header-copy {
  min-width: 0;
}

.codex-tray-panel__eyebrow {
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.22em;
  text-transform: uppercase;
}

.codex-tray-panel__title {
  color: var(--color-text-primary);
  margin-top: 2px;
  font-size: 1.42rem;
  font-weight: 700;
  letter-spacing: -0.05em;
  line-height: 1.05;
}

.codex-tray-panel__icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 52%);
  border-radius: 999px;
  background: rgb(var(--color-bg-base-rgb) / 72%);
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
  gap: 10px;
  border: 1px solid rgb(var(--color-border-default-rgb) / 42%);
  border-radius: 20px;
  background: rgb(var(--color-bg-base-rgb) / 52%);
  padding: 13px 14px;
  color: var(--color-text-secondary);
  font-size: 12px;
  line-height: 1.45;
}

.codex-tray-panel__callout--danger {
  border-color: rgb(var(--color-danger-rgb) / 26%);
  background: rgb(var(--color-danger-rgb) / 10%);
  color: var(--color-danger);
}

@media (prefers-reduced-motion: reduce) {
  .codex-tray-panel__icon-button {
    transition: none;
  }
}
</style>
