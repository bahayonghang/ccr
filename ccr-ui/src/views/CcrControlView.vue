<!-- -->
<template>
  <div class="h-screen w-full bg-bg-primary text-white overflow-hidden flex flex-col relative transition-colors duration-300">
    <!-- 🎨 赛博朋克动态背景装饰 -->
    <div class="absolute inset-0 overflow-hidden pointer-events-none -z-10">
      <!-- 径向渐变光晕 -->
      <div
        class="absolute top-0 right-0 w-[600px] h-[600px] rounded-full opacity-10 blur-3xl animate-pulse-slow"
        :style="{ background: 'radial-gradient(circle, var(--accent-primary) 0%, transparent 70%)' }"
      />
      <div
        class="absolute bottom-0 left-0 w-[500px] h-[500px] rounded-full opacity-10 blur-3xl animate-pulse-slow"
        :style="{ background: 'radial-gradient(circle, var(--accent-secondary) 0%, transparent 70%)', animationDelay: '1s' }"
      />

      <!-- 网格背景 -->
      <div
        class="absolute inset-0 opacity-[0.03]"
        style="background-image: linear-gradient(var(--accent-primary) 1px, transparent 1px), linear-gradient(90deg, var(--accent-primary) 1px, transparent 1px); background-size: 50px 50px;"
      />

      <!-- 扫描线效果 -->
      <div
        class="absolute inset-0 opacity-[0.02] pointer-events-none animate-scan-lines"
        style="background: repeating-linear-gradient(0deg, transparent, transparent 2px, var(--accent-primary) 2px, var(--accent-primary) 4px);"
      />
    </div>

    <!-- 🌟 头部区域 -->
    <header class="flex-none px-6 py-4 flex items-center justify-between border-b border-border-color bg-bg-primary/80 backdrop-blur-md z-10 animate-fade-in-down">
      <div class="flex items-center gap-4">
        <div class="relative group">
          <div class="absolute inset-0 bg-accent-primary/30 blur-xl rounded-full group-hover:bg-accent-primary/50 transition-colors duration-500 animate-pulse-glow" />
          <div class="relative w-10 h-10 rounded-xl glass-effect flex items-center justify-center border border-accent-primary/30 shadow-neon-jade group-hover:scale-110 group-hover:border-accent-primary/60 transition-[color,background-color,border-color,transform] duration-300">
            <SIcon
              name="Terminal"
              size="w-5 h-5"
              class="text-accent-primary drop-shadow-neon"
            />
          </div>
        </div>
        <div>
          <h1 class="text-xl font-bold text-white tracking-tight neon-text-glow flex items-center gap-3">
            {{ $t('ccrControl.title') }}
            <span
              v-if="versionInfo?.current_version"
              class="text-xs px-2 py-0.5 rounded-full bg-accent-primary/10 border border-accent-primary/20 text-accent-primary font-mono"
            >
              v{{ versionInfo.current_version }}
            </span>
          </h1>
          <p class="text-xs text-white/80">
            {{ $t('ccrControl.description') }}
          </p>
        </div>
      </div>

      <!-- 右侧装饰或状态 -->
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2 text-xs font-mono text-white/50">
          <span class="w-2 h-2 rounded-full bg-accent-primary animate-pulse" />
          System Online
        </div>
        <ThemeToggle />
      </div>
    </header>

    <!-- 🏗️ 主体内容区 -->
    <div class="flex-1 flex overflow-hidden p-4 gap-4 animate-fade-in">
      <!-- 👈 左侧侧边栏：命令/收藏/历史 -->
      <aside class="w-80 flex-none flex flex-col gap-4 animate-slide-in-left">
        <Card 
          variant="glass" 
          class="flex-1 flex flex-col !p-0 overflow-hidden neon-card"
          padding="none"
          body-class="h-full flex flex-col"
        >
          <!-- 侧边栏 Tabs -->
          <div class="flex p-2 gap-1 border-b border-border-color bg-bg-secondary/50">
            <button
              v-for="tab in sidebarTabs"
              :key="tab.id"
              class="flex-1 flex items-center justify-center gap-2 py-2 rounded-lg text-xs font-bold transition-colors duration-300 relative overflow-hidden group"
              :class="activeTab === tab.id 
                ? 'bg-accent-primary/10 text-accent-primary shadow-neon-jade-sm' 
                : 'text-white/50 hover:bg-bg-hover hover:text-white'"
              @click="activeTab = tab.id"
            >
              <SIcon
                :name="tab.icon || ''"
                size="w-4 h-4"
              />
              <span>{{ tab.label }}</span>
              <!-- Tab 激活光效 -->
              <div
                v-if="activeTab === tab.id"
                class="absolute inset-0 bg-gradient-to-t from-accent-primary/10 to-transparent opacity-50"
              />
            </button>
          </div>

          <!-- 内容区域 -->
          <div class="flex-1 overflow-hidden relative">
            <Transition
              name="fade-slide"
              mode="out-in"
            >
              <!-- 1. 命令列表 -->
              <div
                v-if="activeTab === 'commands'"
                key="commands"
                class="h-full flex flex-col"
              >
                <!-- 模块选择 (Mini) -->
                <div class="px-3 py-3 border-b border-border-color">
                  <div class="flex gap-2 overflow-x-auto custom-scrollbar pb-1">
                    <button
                      v-for="mod in modules"
                      :key="mod.id"
                      class="flex-shrink-0 px-3 py-1.5 rounded-lg text-xs font-bold transition-colors border border-transparent"
                      :class="selectedModuleId === mod.id
                        ? 'bg-accent-primary/20 text-accent-primary border-accent-primary/30'
                        : 'bg-bg-secondary text-white/50 hover:bg-bg-hover hover:text-white'"
                      @click="selectModule(mod.id)"
                    >
                      {{ mod.name }}
                    </button>
                  </div>
                </div>

                <!-- 命令列表 -->
                <div class="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-2">
                  <div
                    v-for="cmd in selectedModule?.commands"
                    :key="cmd.command"
                    class="cursor-pointer group relative p-3 rounded-xl border border-transparent hover:bg-bg-hover hover:border-accent-primary/20 transition-colors duration-300"
                    :class="selectedCommand?.command === cmd.command ? 'bg-accent-primary/10 border-accent-primary/40 shadow-neon-jade-sm' : ''"
                    @click="selectCommand(cmd)"
                  >
                    <div class="flex items-start gap-3">
                      <div
                        class="mt-0.5 w-7 h-7 rounded-lg bg-bg-secondary flex items-center justify-center group-hover:scale-110 transition-transform"
                        :class="selectedCommand?.command === cmd.command ? 'bg-accent-primary text-white' : 'text-white/50 group-hover:text-accent-primary'"
                      >
                        <SIcon
                          name="Terminal"
                          size="w-4 h-4"
                        />
                      </div>
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center justify-between mb-0.5">
                          <span
                            class="text-sm font-bold truncate"
                            :class="selectedCommand?.command === cmd.command ? 'text-accent-primary' : 'text-white'"
                          >{{ cmd.name }}</span>
                          <div class="flex gap-1">
                            <SIcon
                              v-if="cmd.dangerous"
                              name="AlertTriangle"
                              size="w-3 h-3"
                              class="text-accent-danger animate-pulse"
                            />
                            <SIcon
                              name="Star"
                              size="w-3 h-3"
                              class="transition-transform cursor-pointer hover:scale-125"
                              :class="isFavorite(cmd.command) ? 'text-accent-warning fill-accent-warning' : 'text-transparent stroke-text-muted hover:stroke-accent-warning'"
                              @click.stop="toggleFavorite(cmd)"
                            />
                          </div>
                        </div>
                        <div class="text-[10px] font-mono opacity-60 mb-1 text-white/80">
                          ccr {{ cmd.command }}
                        </div>
                        <p class="text-[10px] text-white/50 line-clamp-2 leading-relaxed">
                          {{ cmd.description }}
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 2. 收藏列表 -->
              <div
                v-else-if="activeTab === 'favorites'"
                key="favorites"
                class="h-full overflow-y-auto custom-scrollbar p-2 space-y-2"
              >
                <div
                  v-if="favorites.length === 0"
                  class="h-full flex flex-col items-center justify-center text-white/50"
                >
                  <SIcon
                    name="Star"
                    size="w-8 h-8"
                    class="opacity-20 mb-2"
                  />
                  <span class="text-xs">{{ $t('ccrControl.noFavorites') }}</span>
                </div>
                <div
                  v-for="fav in favorites"
                  :key="fav.id"
                  class="p-3 rounded-xl bg-bg-secondary border border-border-color hover:border-accent-warning/30 hover:shadow-neon-gold-sm transition-[border-color,box-shadow] cursor-pointer group"
                  @click="executeFromFavorite(fav)"
                >
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-xs font-bold text-accent-warning">{{ fav.display_name || fav.command }}</span>
                    <button
                      class="text-white/50 hover:text-accent-danger transition-colors"
                      @click.stop="removeFromFavorites(fav.id)"
                    >
                      <SIcon
                        name="X"
                        size="w-3 h-3"
                      />
                    </button>
                  </div>
                  <div class="text-[10px] font-mono text-white/80 mb-2">
                    ccr {{ fav.command }}
                  </div>
                  <div class="flex justify-end">
                    <button class="p-1.5 rounded-lg bg-accent-warning/10 text-accent-warning hover:bg-accent-warning hover:text-white transition-colors">
                      <SIcon
                        name="Play"
                        size="w-3 h-3"
                        class="fill-current"
                      />
                    </button>
                  </div>
                </div>
              </div>

              <!-- 3. 历史记录 -->
              <div
                v-else-if="activeTab === 'history'"
                key="history"
                class="h-full flex flex-col"
              >
                <div class="p-2 border-b border-border-color flex justify-end">
                  <button 
                    v-if="history.length > 0"
                    class="text-[10px] flex items-center gap-1 text-white/50 hover:text-accent-danger px-2 py-1 hover:bg-bg-hover rounded transition-colors"
                    @click="clearHistoryData"
                  >
                    <SIcon
                      name="Trash2"
                      size="w-3 h-3"
                    />
                    {{ $t('ccrControl.clearHistory') }}
                  </button>
                </div>
                <div class="flex-1 overflow-y-auto custom-scrollbar p-2 space-y-2">
                  <div
                    v-if="history.length === 0"
                    class="h-full flex flex-col items-center justify-center text-white/50"
                  >
                    <SIcon
                      name="History"
                      size="w-8 h-8"
                      class="opacity-20 mb-2"
                    />
                    <span class="text-xs">{{ $t('ccrControl.noHistory') }}</span>
                  </div>
                  <div
                    v-for="item in history"
                    :key="item.id"
                    class="p-2.5 rounded-lg bg-bg-secondary border border-border-color hover:bg-bg-hover transition-colors cursor-pointer flex items-center gap-3 group"
                    @click="executeFromHistory(item)"
                  >
                    <div
                      class="w-2 h-2 rounded-full flex-shrink-0"
                      :class="item.success ? 'bg-accent-success shadow-neon-jade-sm' : 'bg-accent-danger shadow-neon-danger-sm'"
                    />
                    <div class="flex-1 min-w-0">
                      <div class="text-xs font-mono font-bold truncate text-white">
                        {{ item.command }}
                      </div>
                      <div class="text-[10px] text-white/50 flex items-center gap-2">
                        <span>{{ formatTime(item.executed_at) }}</span>
                        <span>{{ item.duration_ms }}ms</span>
                      </div>
                    </div>
                    <SIcon
                      name="Play"
                      size="w-3 h-3"
                      class="text-white/50 opacity-0 group-hover:opacity-100 transition-opacity"
                    />
                  </div>
                </div>
              </div>
            </Transition>
          </div>
        </Card>
      </aside>

      <!-- 👉 右侧主区域：参数配置 + 终端输出 -->
      <main class="flex-1 flex flex-col gap-4 overflow-hidden animate-slide-in-right">
        <!-- 1. 参数配置区 (高度自适应) -->
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
            <span class="text-xs font-bold text-white">{{ selectedCommand ? $t('ccrControl.commandParams') : $t('ccrControl.selectCommandFirst') }}</span>
          </div>
           
          <div class="p-4">
            <div v-if="selectedCommand">
              <!-- 命令预览 & 执行按钮行 -->
              <div class="flex items-center gap-4 mb-4">
                <div class="flex-1 px-4 py-2.5 rounded-lg bg-bg-secondary border border-accent-primary/20 font-mono text-sm text-accent-primary flex items-center gap-2 shadow-inner">
                  <span class="text-white/50 select-none">$</span>
                  ccr {{ selectedCommand.command }}
                </div>
                <button
                  class="flex items-center gap-2 px-6 py-2.5 rounded-lg font-bold text-sm text-white shadow-lg transition-[color,background-color,border-color,transform] active:scale-95"
                  :class="selectedCommand.dangerous
                    ? 'bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 shadow-neon-danger'
                    : 'bg-gradient-to-r from-accent-primary to-accent-secondary hover:from-accent-secondary hover:to-accent-primary shadow-neon-jade'"
                  :disabled="isExecuting"
                  @click="executeCommand(selectedCommand)"
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
                  {{ isExecuting ? $t('ccrControl.executing') : $t('ccrControl.execute') }}
                </button>
              </div>

              <!-- 参数表单 -->
              <div
                v-if="(selectedCommand.args && selectedCommand.args.length > 0) || (selectedCommand.flags && selectedCommand.flags.length > 0)"
                class="grid grid-cols-2 gap-4 animate-fade-in"
              >
                <!-- Required Args -->
                <div
                  v-for="arg in selectedCommand.args"
                  :key="arg.name"
                >
                  <label class="block text-[10px] font-bold text-white/80 mb-1 ml-1 uppercase">{{ arg.name }} <span
                    v-if="arg.required"
                    class="text-accent-danger"
                  >*</span></label>
                  <input
                    v-if="arg.type !== 'select'"
                    v-model="commandArgs[arg.name]"
                    type="text"
                    :placeholder="arg.placeholder"
                    class="w-full px-3 py-2 rounded-lg bg-bg-secondary border border-border-color text-sm text-white focus:border-accent-primary focus:bg-bg-hover transition-colors font-mono"
                  >
                  <select
                    v-else
                    v-model="commandArgs[arg.name]"
                    class="w-full px-3 py-2 rounded-lg bg-bg-secondary border border-border-color text-sm text-white focus:border-accent-primary transition-colors font-mono"
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

                <!-- Flags -->
                <div
                  v-for="flag in selectedCommand.flags"
                  :key="flag.name"
                  class="flex items-center gap-3 p-2 rounded-lg border border-border-color bg-bg-secondary/50"
                >
                  <template v-if="flag.type === 'boolean'">
                    <input
                      :id="`flag-${flag.name}`"
                      v-model="commandFlags[flag.name]"
                      type="checkbox"
                      class="accent-accent-primary w-4 h-4 cursor-pointer"
                    >
                    <label
                      :for="`flag-${flag.name}`"
                      class="cursor-pointer flex-1"
                    >
                      <div class="text-xs font-medium text-white">{{ flag.name }}</div>
                      <div class="text-[10px] font-mono text-white/50">{{ flag.flag }}</div>
                    </label>
                  </template>
                  <template v-else>
                    <div class="flex-1">
                      <div class="text-[10px] text-white/50 mb-1">
                        {{ flag.name }} <code class="bg-bg-tertiary px-1 rounded">{{ flag.flag }}</code>
                      </div>
                      <input 
                        v-model="commandFlags[flag.name]" 
                        :type="flag.type === 'number' ? 'number' : 'text'"
                        class="w-full px-2 py-1 rounded bg-bg-tertiary border border-border-color text-xs font-mono text-white focus:border-accent-secondary transition-colors"
                      >
                    </div>
                  </template>
                </div>
              </div>
            </div>
              
            <div
              v-else
              class="py-8 flex flex-col items-center justify-center text-white/50 opacity-50"
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

        <!-- 2. 终端输出区 (剩余空间全部占满) -->
        <div class="flex-1 flex flex-col overflow-hidden min-h-0 rounded-xl border border-border-color bg-bg-primary/50 backdrop-blur-md shadow-2xl relative transition-[box-shadow] duration-300 hover:shadow-neon-jade-sm group">
          <!-- Terminal Header -->
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
              <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-bg-tertiary text-white/50 font-mono">{{ outputLines.length }} lines</span>
            </div>
            <div class="flex items-center gap-3">
              <!-- Exit Code Badge -->
              <div
                v-if="lastExitCode !== null"
                class="flex items-center gap-1.5 px-2 py-1 rounded-md text-[10px] font-mono font-bold border transition-colors animate-fade-in"
                :class="lastExitCode === 0 ? 'bg-accent-success/10 text-accent-success border-accent-success/30' : 'bg-accent-danger/10 text-accent-danger border-accent-danger/30'"
              >
                <SIcon
                  :name="lastExitCode === 0 ? 'CheckCircle' : 'XCircle'"
                  size="w-3.5 h-3.5"
                />
                <span>Exited: {{ lastExitCode }}</span>
              </div>
              <!-- Clear Button -->
              <button 
                class="p-1.5 rounded-lg hover:bg-bg-hover text-white/50 hover:text-accent-danger transition-[color,background-color,transform] active:scale-95"
                :title="$t('ccrControl.clearOutput')"
                @click="clearOutput"
              >
                <SIcon
                  name="Trash2"
                  size="w-4 h-4"
                />
              </button>
            </div>
          </div>

          <!-- Terminal Body -->
          <div class="flex-1 relative overflow-hidden bg-[#09090b]">
            <!-- CRT Scanline Overlay -->
            <div
              class="absolute inset-0 pointer-events-none opacity-[0.03] animate-crt-scan z-10" 
              style="background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgb(255 255 255 / 10%) 2px, rgb(255 255 255 / 10%) 4px);"
            />
              
            <!-- Terminal Content -->
            <div
              ref="outputContainer"
              class="absolute inset-0 overflow-y-auto p-4 custom-scrollbar font-mono text-sm leading-relaxed z-20 scroll-smooth"
            >
              <!-- Empty State -->
              <div
                v-if="outputLines.length === 0"
                class="h-full flex flex-col items-center justify-center text-gray-600 opacity-50"
              >
                <SIcon
                  name="Terminal"
                  size="w-16 h-16"
                  class="mb-4"
                />
                <span class="text-xs tracking-[0.2em] uppercase font-bold">Ready for Input</span>
              </div>

              <!-- Lines -->
              <div
                v-else
                class="flex flex-col pb-4"
              >
                <div
                  v-for="(line, idx) in outputLines"
                  :key="idx" 
                  class="break-all whitespace-pre-wrap py-[1px] font-mono text-gray-300 hover:bg-white/5 transition-colors border-l-2 border-transparent hover:border-accent-primary pl-2 -ml-2"
                >
                  <span class="inline-block w-8 text-right mr-4 text-[10px] text-gray-700 select-none opacity-50">{{ idx + 1 }}</span>
                  <span v-html="renderAnsi(line)" />
                </div>
                    
                <!-- Typing Cursor (Visual Only) -->
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
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, watch, nextTick } from 'vue'
import { AnsiUp } from 'ansi_up'
import Card from '@/components/ui/Card.vue'
import ThemeToggle from '@/components/ThemeToggle.vue'
import { useCcrControl } from '@/composables/useCcrControl'
import type { CcrCommand } from '@/api/ccr-control'
import { sanitizeTerminal } from '@/utils/sanitize'

// Use Composables
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
  clearOutput
} = useCcrControl()

// UI State
const activeTab = ref<'commands' | 'favorites' | 'history'>('commands')
const outputContainer = ref<HTMLElement | null>(null)

// Sidebar Tabs Configuration
const sidebarTabs: { id: 'commands' | 'favorites' | 'history'; label: string; icon: string }[] = [
  { id: 'commands', label: 'Commands', icon: 'List' },
  { id: 'favorites', label: 'Favorites', icon: 'Star' },
  { id: 'history', label: 'History', icon: 'History' }
]


// Render ANSI
const ansiUp = new AnsiUp()
const renderAnsi = (text: string) => {
  const html = ansiUp.ansi_to_html(text || '')
  return sanitizeTerminal(html)
}

// Toggle Favorite
const toggleFavorite = async (cmd: CcrCommand) => {
  if (isFavorite(cmd.command)) {
    const fav = favorites.value.find(f => f.command === cmd.command)
    if (fav) await removeFromFavorites(fav.id)
  } else {
    await addToFavorites(cmd)
  }
}

// Format Time
const formatTime = (dateStr: string) => {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  if (diff < 60000) return 'Just now'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
  return date.toLocaleDateString()
}

// Auto Scroll
watch(outputLines, async () => {
  await nextTick()
  if (outputContainer.value) {
    outputContainer.value.scrollTop = outputContainer.value.scrollHeight
  }
}, { deep: true })

// Init
loadVersionInfo()
</script>

<style scoped>
/* Scrollbar */
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

/* Neon Effects */
.shadow-neon-jade { box-shadow: 0 0 15px rgb(var(--accent-primary-rgb), 0.25); }
.shadow-neon-jade-sm { box-shadow: 0 0 8px rgb(var(--accent-primary-rgb), 0.2); }
.shadow-neon-danger { box-shadow: 0 0 15px rgb(var(--accent-danger-rgb), 0.25); }
.shadow-neon-gold-sm { box-shadow: 0 0 8px rgb(var(--accent-warning-rgb), 0.2); }
.drop-shadow-neon { filter: drop-shadow(0 0 5px rgb(var(--accent-primary-rgb), 0.5)); }
.neon-text-glow { text-shadow: 0 0 10px rgb(var(--accent-primary-rgb), 0.3); }

/* Glass Effect */
.glass-effect {
  background: var(--bg-card);
  backdrop-filter: blur(12px);
}

.neon-card {
  border: 1px solid var(--border-color);
}

.terminal-card {
  border: 1px solid var(--border-color);
}

/* Animations */
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

/* Transitions */
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

/* ANSI Colors - Themed */
:deep(.ansi-black-fg) { color: var(--text-primary); }
:deep(.ansi-red-fg) { color: var(--accent-danger); }
:deep(.ansi-green-fg) { color: var(--accent-success); }
:deep(.ansi-yellow-fg) { color: var(--accent-warning); }
:deep(.ansi-blue-fg) { color: var(--accent-info); }
:deep(.ansi-magenta-fg) { color: var(--accent-secondary); }
:deep(.ansi-cyan-fg) { color: var(--accent-tertiary); }
:deep(.ansi-white-fg) { color: var(--text-muted); }
</style>
