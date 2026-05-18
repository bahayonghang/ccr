<template>
  <div class="flex min-h-[24rem] flex-1 flex-col overflow-hidden rounded-xl border border-border-color bg-bg-primary/50 backdrop-blur-md shadow-2xl relative transition-[box-shadow] duration-300 hover:shadow-neon-jade-sm group xl:min-h-0">
    <div class="flex-none px-4 py-3 border-b border-border-color bg-bg-secondary/50 flex items-center justify-between backdrop-blur-md">
      <div class="flex items-center gap-2">
        <div class="p-1 rounded bg-accent-primary/10">
          <SIcon
            name="Monitor"
            size="w-4 h-4"
            class="text-accent-primary"
          />
        </div>
        <span class="text-xs font-bold text-white">{{ $t('ccrControl.output') }}</span>
        <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-bg-tertiary text-text-muted font-mono">{{ $t('ccrControl.lineCount', { count: outputLines.length }) }}</span>
      </div>
      <div class="flex items-center gap-3">
        <div
          v-if="lastExitCode !== null"
          class="flex items-center gap-1.5 px-2 py-1 rounded-md text-[10px] font-mono font-bold border transition-colors animate-fade-in"
          :class="lastExitCode === 0 ? 'bg-accent-success/10 text-accent-success border-accent-success/30' : 'bg-accent-danger/10 text-accent-danger border-accent-danger/30'"
        >
          <SIcon
            :name="lastExitCode === 0 ? 'CheckCircle' : 'XCircle'"
            size="w-3.5 h-3.5"
          />
          <span>{{ $t('ccrControl.exited', { code: lastExitCode }) }}</span>
        </div>
        <button
          type="button"
          class="p-1.5 rounded-lg hover:bg-bg-hover text-text-muted hover:text-accent-danger transition-[color,background-color,transform] active:scale-95"
          :title="$t('ccrControl.clearOutput')"
          :aria-label="$t('ccrControl.clearOutput')"
          @click="$emit('clearOutput')"
        >
          <SIcon
            name="Trash2"
            size="w-4 h-4"
          />
        </button>
      </div>
    </div>

    <div class="flex-1 relative overflow-hidden bg-[#09090b]">
      <div
        class="absolute inset-0 pointer-events-none opacity-[0.03] animate-crt-scan z-10"
        style="background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgb(255 255 255 / 10%) 2px, rgb(255 255 255 / 10%) 4px);"
      />

      <div
        ref="outputContainer"
        class="absolute inset-0 overflow-y-auto p-4 custom-scrollbar font-mono text-sm leading-relaxed z-20 scroll-smooth"
      >
        <div
          v-if="outputLines.length === 0"
          class="h-full flex flex-col items-center justify-center text-text-muted opacity-50"
        >
          <SIcon
            name="Terminal"
            size="w-16 h-16"
            class="mb-4"
          />
          <span class="text-xs tracking-[0.2em] uppercase font-bold">{{ $t('ccrControl.readyForResult') }}</span>
          <span class="mt-2 max-w-sm text-center text-[11px] normal-case tracking-normal">{{ $t('ccrControl.nonStreamingHint') }}</span>
        </div>

        <div
          v-else
          class="flex flex-col pb-4"
        >
          <div
            v-for="(_, idx) in outputLines"
            :key="idx"
            class="break-all whitespace-pre-wrap py-[1px] font-mono text-text-secondary hover:bg-bg-surface/70 transition-colors border-l-2 border-transparent hover:border-accent-primary pl-2 -ml-2"
          >
            <span class="inline-block w-8 text-right mr-4 text-[10px] text-text-muted select-none opacity-50">{{ idx + 1 }}</span>
            <span v-html="renderedOutputLines[idx] ?? ''" />
          </div>

          <div
            v-if="isExecuting"
            class="pl-14 mt-1"
          >
            <span class="inline-block w-2 h-4 bg-accent-primary animate-pulse" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import { createAnsiRenderer } from '@/utils/ansiRenderer'

const props = defineProps<{
  outputLines: string[]
  isExecuting: boolean
  lastExitCode: number | null
}>()

const ansiRenderer = createAnsiRenderer()
const renderedOutputLines = ref<string[]>([])
let previousOutputLines: string[] = []

const syncRenderedOutputLines = (nextLines: string[]) => {
  const shouldRebuild = previousOutputLines.length === 0
    || nextLines.length < previousOutputLines.length
    || nextLines[0] !== previousOutputLines[0]

  if (shouldRebuild) {
    renderedOutputLines.value = nextLines.map((line) => ansiRenderer.renderLine(line))
    previousOutputLines = [...nextLines]
    return
  }

  const appended = nextLines.slice(previousOutputLines.length)
  if (appended.length > 0) {
    renderedOutputLines.value = [
      ...renderedOutputLines.value,
      ...appended.map((line) => ansiRenderer.renderLine(line)),
    ]
  }

  previousOutputLines = [...nextLines]
}

defineEmits<{
  clearOutput: []
}>()

const outputContainer = ref<HTMLElement | null>(null)

watch(() => props.outputLines, async (nextLines) => {
  if (nextLines.length === 0) {
    ansiRenderer.clear()
    renderedOutputLines.value = []
    previousOutputLines = []
  } else {
    syncRenderedOutputLines(nextLines)
  }

  await nextTick()
  if (outputContainer.value) {
    outputContainer.value.scrollTop = outputContainer.value.scrollHeight
  }
}, { deep: true, immediate: true })
</script>
