<template>
  <Card
    variant="glass"
    class="flex-none overflow-hidden neon-card"
    padding="none"
  >
    <div class="p-3 border-b border-border-color bg-gradient-to-r from-accent-primary/5 to-transparent flex items-center gap-2">
      <SIcon
        name="Settings"
        size="w-4 h-4"
        class="text-accent-primary"
      />
      <span class="text-xs font-bold text-white">{{ command ? $t('ccrControl.commandParams') : $t('ccrControl.selectCommandFirst') }}</span>
    </div>

    <div class="p-4">
      <div v-if="command">
        <div class="mb-4 flex flex-col gap-4 sm:flex-row sm:items-center">
          <div class="flex min-w-0 flex-1 items-center gap-2 rounded-lg border border-accent-primary/20 bg-bg-secondary px-4 py-2.5 font-mono text-sm text-accent-primary shadow-inner">
            <span class="text-text-muted select-none">$</span>
            <span class="min-w-0 truncate">ccr {{ command.command }}</span>
          </div>
          <span
            class="inline-flex items-center rounded-full border px-2 py-1 text-[10px] font-bold uppercase tracking-wide"
            :class="command.executable ? 'border-accent-success/30 bg-accent-success/10 text-accent-success' : 'border-accent-warning/30 bg-accent-warning/10 text-accent-warning'"
          >
            {{ command.executable ? $t('ccrControl.executable') : $t('ccrControl.unsupported') }}
          </span>
          <button
            type="button"
            :aria-label="`${$t('ccrControl.execute')} ccr ${command.command}`"
            class="flex items-center gap-2 px-6 py-2.5 rounded-lg font-bold text-sm text-white shadow-lg transition-[color,background-color,border-color,transform] active:scale-95 disabled:cursor-not-allowed disabled:opacity-60"
            :class="command.dangerous
              ? 'bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 shadow-neon-danger'
              : 'bg-gradient-to-r from-accent-primary to-accent-secondary hover:from-accent-secondary hover:to-accent-primary shadow-neon-jade'"
            :disabled="isExecuting || !canExecute"
            @click="$emit('execute')"
          >
            <SIcon
              v-if="isExecuting"
              name="Loader2"
              size="w-4 h-4"
              class="animate-spin"
            />
            <SIcon
              v-else
              name="Play"
              size="w-4 h-4"
              class="fill-current"
            />
            {{ executeButtonLabel }}
          </button>
        </div>

        <div
          v-if="!command.executable"
          class="mb-4 rounded-lg border border-accent-warning/30 bg-accent-warning/10 p-3 text-xs text-accent-warning"
          role="status"
        >
          {{ $t('ccrControl.unsupportedHint') }}
        </div>

        <label
          v-if="command.dangerous && command.executable"
          class="mb-4 flex items-start gap-3 rounded-lg border border-accent-danger/30 bg-accent-danger/10 p-3 text-xs text-text-primary"
        >
          <input
            :checked="dangerAccepted"
            type="checkbox"
            class="mt-0.5 h-4 w-4 accent-accent-danger"
            @change="$emit('update:dangerAccepted', ($event.target as HTMLInputElement).checked)"
          >
          <span>
            <strong class="text-accent-danger">{{ $t('ccrControl.dangerousCommand') }}</strong>
            {{ $t('ccrControl.dangerConfirmHint') }}
          </span>
        </label>

        <div
          v-if="(command.args && command.args.length > 0) || (command.flags && command.flags.length > 0)"
          class="grid grid-cols-1 gap-4 animate-fade-in md:grid-cols-2"
        >
          <div
            v-for="arg in command.args"
            :key="arg.name"
          >
            <label
              class="block text-[10px] font-bold text-text-primary mb-1 ml-1 uppercase"
              :for="argDomId(arg.name)"
            >{{ arg.name }} <span
              v-if="arg.required"
              class="text-accent-danger"
            >*</span></label>
            <input
              v-if="arg.type !== 'select'"
              :id="argDomId(arg.name)"
              :value="commandArgs[arg.name] ?? ''"
              type="text"
              :placeholder="arg.placeholder"
              class="w-full px-3 py-2 rounded-lg bg-bg-secondary border border-border-color text-sm text-white focus:border-accent-primary focus:bg-bg-hover transition-colors font-mono"
              @input="$emit('updateArg', arg.name, getInputValue($event))"
            >
            <select
              v-else
              :id="argDomId(arg.name)"
              :value="commandArgs[arg.name] ?? ''"
              class="w-full px-3 py-2 rounded-lg bg-bg-secondary border border-border-color text-sm text-white focus:border-accent-primary transition-colors font-mono"
              @change="$emit('updateArg', arg.name, getSelectValue($event))"
            >
              <option
                value=""
                disabled
              >
                {{ $t('ccrControl.selectOption') }}
              </option>
              <option
                v-for="opt in arg.options"
                :key="opt"
                :value="opt"
              >
                {{ opt }}
              </option>
            </select>
          </div>

          <div
            v-for="flag in command.flags"
            :key="flag.name"
            class="flex items-center gap-3 p-2 rounded-lg border border-border-color bg-bg-secondary/50"
          >
            <template v-if="flag.type === 'boolean'">
              <input
                :id="flagDomId(flag.name)"
                :checked="Boolean(commandFlags[flag.name])"
                type="checkbox"
                class="accent-accent-primary w-4 h-4 cursor-pointer"
                @change="$emit('updateFlag', flag.name, getChecked($event))"
              >
              <label
                :for="flagDomId(flag.name)"
                class="cursor-pointer flex-1"
              >
                <div class="text-xs font-medium text-white">{{ flag.name }}</div>
                <div class="text-[10px] font-mono text-text-muted">{{ flag.flag }}</div>
              </label>
            </template>
            <template v-else>
              <div class="flex-1">
                <label
                  class="mb-1 block text-[10px] text-text-muted"
                  :for="flagDomId(flag.name)"
                >
                  {{ flag.name }} <code class="bg-bg-tertiary px-1 rounded">{{ flag.flag }}</code>
                </label>
                <input
                  :id="flagDomId(flag.name)"
                  :value="commandFlags[flag.name] ?? ''"
                  :type="flag.type === 'number' ? 'number' : 'text'"
                  class="w-full px-2 py-1 rounded bg-bg-tertiary border border-border-color text-xs font-mono text-white focus:border-accent-secondary transition-colors"
                  @input="$emit('updateFlag', flag.name, getInputValue($event))"
                >
              </div>
            </template>
          </div>
        </div>
      </div>

      <div
        v-else
        class="py-8 flex flex-col items-center justify-center text-text-muted opacity-50"
      >
        <SIcon
          name="Terminal"
          size="w-12 h-12"
          class="mb-2"
        />
        <p class="text-xs">
          {{ $t('ccrControl.selectCommandHint') }}
        </p>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import Card from '@/components/ui/Card.vue'
import SIcon from '@/components/ui/SIcon.vue'
import type { CcrCommand } from '@/api/ccr-control'

defineProps<{
  command: CcrCommand | null
  commandArgs: Record<string, string>
  commandFlags: Record<string, unknown>
  dangerAccepted: boolean
  isExecuting: boolean
  canExecute: boolean
  executeButtonLabel: string
}>()

defineEmits<{
  execute: []
  'update:dangerAccepted': [value: boolean]
  updateArg: [name: string, value: string]
  updateFlag: [name: string, value: unknown]
}>()

const getInputValue = (event: Event) => (event.target as HTMLInputElement).value
const getSelectValue = (event: Event) => (event.target as HTMLSelectElement).value
const getChecked = (event: Event) => (event.target as HTMLInputElement).checked

const controlDomId = (prefix: string, name: string) =>
  `${prefix}-${name.replace(/[^A-Za-z0-9_-]/g, '-')}`

const argDomId = (name: string) => controlDomId('arg', name)
const flagDomId = (name: string) => controlDomId('flag', name)
</script>
