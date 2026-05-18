<template>
  <CcrControlShell :version-info="versionInfo">
    <template #status>
      <CcrStatusSummary
        :version-info="versionInfo"
        :executable-command-count="executableCommandCount"
        :total-command-count="totalCommandCount"
      />
    </template>

    <CcrCommandRail
      :modules="modules"
      :selected-module-id="selectedModuleId"
      :selected-module="selectedModule"
      :selected-command="selectedCommand"
      :favorites="favorites"
      :history="history"
      :is-favorite="isFavorite"
      @select-module="selectModule"
      @select-command="selectCommand"
      @toggle-favorite="toggleFavorite"
      @execute-favorite="runFavoriteCommand"
      @remove-favorite="removeFromFavorites"
      @execute-history="runHistoryCommand"
      @clear-history="clearHistoryData"
    />

    <main class="flex min-h-0 flex-1 flex-col gap-4 overflow-visible animate-slide-in-right xl:overflow-hidden">
      <CcrCommandDetails
        v-model:danger-accepted="dangerAccepted"
        :command="selectedCommand"
        :command-args="commandArgs"
        :command-flags="commandFlags"
        :is-executing="isExecuting"
        :can-execute="canExecuteSelectedCommand"
        :execute-button-label="executeButtonLabel"
        @execute="runSelectedCommand"
        @update-arg="updateCommandArg"
        @update-flag="updateCommandFlag"
      />

      <CcrOutputConsole
        :output-lines="outputLines"
        :is-executing="isExecuting"
        :last-exit-code="lastExitCode"
        @clear-output="clearOutput"
      />
    </main>
  </CcrControlShell>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { CcrCommand, CommandHistory, FavoriteCommand } from '@/api/ccr-control'
import { useCcrControl } from '@/composables/useCcrControl'
import CcrCommandDetails from './ccr-control/CcrCommandDetails.vue'
import CcrCommandRail from './ccr-control/CcrCommandRail.vue'
import CcrControlShell from './ccr-control/CcrControlShell.vue'
import CcrOutputConsole from './ccr-control/CcrOutputConsole.vue'
import CcrStatusSummary from './ccr-control/CcrStatusSummary.vue'

const { t } = useI18n()

const {
  versionInfo,
  loadVersionInfo,
  modules,
  selectedModuleId,
  selectedModule,
  selectModule,
  selectedCommand,
  selectCommand,
  commandArgs,
  commandFlags,
  favorites,
  addToFavorites,
  removeFromFavorites,
  isFavorite,
  history,
  clearHistory: clearHistoryData,
  isExecuting,
  outputLines,
  lastExitCode,
  executeCommand,
  executeFromFavorite,
  executeFromHistory,
  clearOutput,
} = useCcrControl()

const dangerAccepted = ref(false)
const totalCommandCount = computed(() => modules.value.reduce((total, module) => total + module.commands.length, 0))
const executableCommandCount = computed(() => modules.value.reduce(
  (total, module) => total + module.commands.filter((command) => command.executable).length,
  0,
))
const canExecuteSelectedCommand = computed(() => {
  if (!selectedCommand.value?.executable) return false
  if (selectedCommand.value.dangerous && !dangerAccepted.value) return false
  return !isExecuting.value
})
const executeButtonLabel = computed(() => {
  if (isExecuting.value) return t('ccrControl.executing')
  if (!selectedCommand.value?.executable) return t('ccrControl.unsupported')
  if (selectedCommand.value.dangerous && !dangerAccepted.value) return t('ccrControl.confirmToExecute')
  return t('ccrControl.execute')
})

const runSelectedCommand = async () => {
  if (!selectedCommand.value || !canExecuteSelectedCommand.value) return
  await executeCommand(selectedCommand.value, { confirmedDanger: dangerAccepted.value })
}

const findCatalogCommand = (commandText: string) => {
  for (const module of modules.value) {
    const command = module.commands.find((item) => item.command === commandText)
    if (command) return { moduleId: module.id, command }
  }
  return null
}

const focusCatalogCommand = (command: CcrCommand, moduleId: string) => {
  if (selectedModuleId.value !== moduleId) selectModule(moduleId)
  selectCommand(command)
}

const shouldReviewBeforeShortcutExecution = (command: CcrCommand) =>
  !command.executable || command.dangerous

const runFavoriteCommand = async (favorite: FavoriteCommand) => {
  const entry = findCatalogCommand(favorite.command)
  if (entry && shouldReviewBeforeShortcutExecution(entry.command)) {
    focusCatalogCommand(entry.command, entry.moduleId)
    return
  }
  await executeFromFavorite(favorite)
}

const runHistoryCommand = async (historyItem: CommandHistory) => {
  const entry = findCatalogCommand(historyItem.command)
  if (entry && shouldReviewBeforeShortcutExecution(entry.command)) {
    focusCatalogCommand(entry.command, entry.moduleId)
    return
  }
  await executeFromHistory(historyItem)
}

const updateCommandArg = (name: string, value: string) => {
  commandArgs.value = {
    ...commandArgs.value,
    [name]: value,
  }
}

const updateCommandFlag = (name: string, value: unknown) => {
  commandFlags.value = {
    ...commandFlags.value,
    [name]: value,
  }
}

const toggleFavorite = async (command: CcrCommand) => {
  if (isFavorite(command.command)) {
    const favorite = favorites.value.find((item) => item.command === command.command)
    if (favorite) await removeFromFavorites(favorite.id)
  } else {
    await addToFavorites(command)
  }
}

watch(selectedCommand, () => {
  dangerAccepted.value = false
})

loadVersionInfo()
</script>

<style>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
  height: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--accent-primary);
  border-radius: 2px;
  opacity: 0.3;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: var(--accent-secondary);
}

.shadow-neon-jade { box-shadow: 0 0 15px rgb(var(--accent-primary-rgb), 0.25); }
.shadow-neon-jade-sm { box-shadow: 0 0 8px rgb(var(--accent-primary-rgb), 0.2); }
.shadow-neon-danger { box-shadow: 0 0 15px rgb(var(--accent-danger-rgb), 0.25); }
.shadow-neon-gold-sm { box-shadow: 0 0 8px rgb(var(--accent-warning-rgb), 0.2); }
.drop-shadow-neon { filter: drop-shadow(0 0 5px rgb(var(--accent-primary-rgb), 0.5)); }
.neon-text-glow { text-shadow: 0 0 10px rgb(var(--accent-primary-rgb), 0.3); }

.glass-effect {
  background: var(--bg-card);
  backdrop-filter: blur(12px);
}

.neon-card {
  border: 1px solid var(--border-color);
}

@keyframes fade-in-down {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}
.animate-fade-in-down { animation: fade-in-down 0.5s ease-out forwards; }

@keyframes slide-in-left {
  from {
    opacity: 0;
    transform: translateX(-20px);
  }

  to {
    opacity: 1;
    transform: translateX(0);
  }
}
.animate-slide-in-left { animation: slide-in-left 0.5s ease-out forwards; }

@keyframes slide-in-right {
  from {
    opacity: 0;
    transform: translateX(20px);
  }

  to {
    opacity: 1;
    transform: translateX(0);
  }
}
.animate-slide-in-right { animation: slide-in-right 0.5s ease-out forwards; }

@keyframes pulse-slow {
  0%, 100% { opacity: 0.1; }
  50% { opacity: 0.15; }
}
.animate-pulse-slow { animation: pulse-slow 4s ease-in-out infinite; }

@keyframes crt-scan {
  0% { transform: translateY(0); }
  100% { transform: translateY(100vh); }
}
.animate-crt-scan { animation: crt-scan 8s linear infinite; }

.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(-10px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(10px);
}

.ansi-black-fg { color: var(--text-primary); }
.ansi-red-fg { color: var(--accent-danger); }
.ansi-green-fg { color: var(--accent-success); }
.ansi-yellow-fg { color: var(--accent-warning); }
.ansi-blue-fg { color: var(--accent-info); }
.ansi-magenta-fg { color: var(--accent-secondary); }
.ansi-cyan-fg { color: var(--accent-tertiary); }
.ansi-white-fg { color: var(--text-muted); }
</style>
