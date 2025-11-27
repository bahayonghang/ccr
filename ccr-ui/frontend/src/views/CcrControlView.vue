<template>
  <div class="min-h-screen p-6 transition-colors duration-300">
    <!-- 🎨 动态背景装饰 (Dynamic Background) -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-0 right-0 w-[600px] h-[600px] rounded-full opacity-10 blur-3xl"
        :style="{ background: 'radial-gradient(circle, var(--accent-primary) 0%, transparent 70%)' }"
      />
      <div
        class="absolute bottom-0 left-0 w-[500px] h-[500px] rounded-full opacity-10 blur-3xl"
        :style="{ background: 'radial-gradient(circle, var(--accent-secondary) 0%, transparent 70%)' }"
      />
    </div>

    <div class="max-w-[1920px] mx-auto space-y-6">
      <!-- 🌟 头部区域 (Header) -->
      <header class="flex flex-col lg:flex-row lg:items-center justify-between gap-6 animate-fade-in">
        <div class="flex items-center gap-5">
          <div class="relative group">
            <div class="absolute inset-0 bg-guofeng-jade/20 blur-lg rounded-full group-hover:bg-guofeng-jade/30 transition-all duration-500"></div>
            <div class="relative w-16 h-16 rounded-2xl glass-effect flex items-center justify-center border border-white/20 shadow-lg group-hover:scale-105 transition-transform duration-300">
              <Terminal class="w-8 h-8 text-guofeng-jade" />
            </div>
          </div>
          <div>
            <h1 class="text-3xl font-bold brand-gradient-text tracking-tight mb-1">
              {{ $t('ccrControl.title') }}
            </h1>
            <div class="flex items-center gap-3 text-sm text-guofeng-text-secondary">
              <p>{{ $t('ccrControl.description') }}</p>
              <!-- Version Badge -->
              <div v-if="versionInfo?.current_version" class="flex items-center gap-2 px-2 py-0.5 rounded-md bg-guofeng-bg-tertiary/50 border border-guofeng-border/50 backdrop-blur-sm">
                <span class="text-xs font-mono text-guofeng-text-muted">v{{ versionInfo.current_version }}</span>
                <button 
                  v-if="updateInfo?.has_update"
                  class="flex items-center gap-1 text-[10px] font-bold text-guofeng-jade hover:underline"
                  @click="executeUpdateCommand"
                >
                  <Sparkles class="w-3 h-3" />
                  {{ $t('ccrControl.updateNow') }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- Check Update Button -->
        <button
          class="group flex items-center gap-2 px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 transition-all backdrop-blur-md"
          :disabled="loadingVersion"
          @click="checkForUpdate"
        >
          <RefreshCw class="w-4 h-4 text-guofeng-jade transition-transform duration-700" :class="{ 'animate-spin': loadingVersion }" />
          <span class="text-sm font-medium text-guofeng-text-primary group-hover:text-guofeng-jade transition-colors">
            {{ $t('ccrControl.checkUpdate') }}
          </span>
        </button>
      </header>

      <!-- 🧭 模块选择器 (Module Selector - Top Bar) -->
      <section class="animate-fade-in" style="animation-delay: 0.1s">
        <GuofengCard variant="glass" class="!p-0">
          <div class="flex items-center gap-2 overflow-x-auto custom-scrollbar p-2">
            <button
              v-for="mod in modules"
              :key="mod.id"
              class="flex items-center gap-2 px-4 py-2 rounded-lg transition-all whitespace-nowrap relative group overflow-hidden shrink-0"
              :class="selectedModuleId === mod.id 
                ? 'bg-guofeng-jade text-white shadow-lg shadow-guofeng-jade/20' 
                : 'hover:bg-white/10 text-guofeng-text-secondary hover:text-guofeng-text-primary'"
              @click="selectModule(mod.id)"
            >
              <component
                :is="getIcon(mod.icon)"
                class="w-4 h-4 transition-transform group-hover:scale-110"
                :class="{ 'text-white': selectedModuleId === mod.id, 'text-guofeng-text-muted group-hover:text-guofeng-jade': selectedModuleId !== mod.id }"
              />
              <span class="text-sm font-medium">{{ mod.name }}</span>
              <span 
                v-if="selectedModuleId !== mod.id"
                class="ml-1 text-[10px] opacity-50 bg-black/10 px-1.5 py-0.5 rounded-full"
              >
                {{ mod.commands.length }}
              </span>
            </button>
          </div>
        </GuofengCard>
      </section>

      <!-- 🏗️ 主体内容区 (Main Content Grid) -->
      <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 animate-fade-in" style="animation-delay: 0.2s">
        
        <!-- 👈 左侧：命令列表 (Command List) - 3 cols -->
        <div class="lg:col-span-3 flex flex-col gap-4 min-h-[500px]">
          <div class="flex items-center justify-between px-1">
            <h2 class="text-lg font-bold text-guofeng-text-primary flex items-center gap-2">
              <Layers class="w-5 h-5 text-guofeng-jade" />
              {{ $t('ccrControl.commands') }}
            </h2>
            <span class="text-xs text-guofeng-text-muted bg-white/5 px-2 py-1 rounded-md">
              {{ selectedModule?.commands.length || 0 }}
            </span>
          </div>

          <div class="grid grid-cols-1 gap-4 overflow-y-auto max-h-[calc(100vh-300px)] custom-scrollbar pr-1">
            <button
              v-for="cmd in selectedModule?.commands"
              :key="cmd.command"
              class="group relative"
              @click="selectCommand(cmd)"
            >
              <!-- Corner Decorations -->
              <div class="absolute top-0 left-0 w-4 h-4 border-t-2 border-l-2 transition-all duration-300 z-10 pointer-events-none"
                :class="selectedCommand?.command === cmd.command ? 'border-guofeng-jade' : 'border-transparent group-hover:border-guofeng-jade/40'"
              ></div>
              <div class="absolute top-0 right-0 w-4 h-4 border-t-2 border-r-2 transition-all duration-300 z-10 pointer-events-none"
                :class="selectedCommand?.command === cmd.command ? 'border-guofeng-jade' : 'border-transparent group-hover:border-guofeng-jade/40'"
              ></div>
              <div class="absolute bottom-0 left-0 w-4 h-4 border-b-2 border-l-2 transition-all duration-300 z-10 pointer-events-none"
                :class="selectedCommand?.command === cmd.command ? 'border-guofeng-jade' : 'border-transparent group-hover:border-guofeng-jade/40'"
              ></div>
              <div class="absolute bottom-0 right-0 w-4 h-4 border-b-2 border-r-2 transition-all duration-300 z-10 pointer-events-none"
                :class="selectedCommand?.command === cmd.command ? 'border-guofeng-jade' : 'border-transparent group-hover:border-guofeng-jade/40'"
              ></div>

              <!-- Card Content -->
              <div 
                class="relative p-4 rounded-xl border-l-4 transition-all duration-300 backdrop-blur-md text-left"
                :class="selectedCommand?.command === cmd.command 
                  ? 'bg-gradient-to-r from-guofeng-jade/15 via-guofeng-jade/8 to-transparent border-l-guofeng-jade shadow-lg shadow-guofeng-jade/20 ring-1 ring-guofeng-jade/30' 
                  : 'bg-white/5 border-l-guofeng-indigo/30 hover:bg-white/10 hover:border-l-guofeng-jade/60 hover:shadow-md'"
              >
                <!-- Pulse Dot -->
                <div v-if="selectedCommand?.command === cmd.command" 
                  class="absolute -left-1.5 top-1/2 -translate-y-1/2 w-2.5 h-2.5 rounded-full bg-guofeng-jade shadow-[0_0_12px_rgba(16,185,129,0.7)] animate-pulse z-10"
                ></div>

                <!-- Header -->
                <div class="flex justify-between items-center mb-3">
                  <div class="flex items-center gap-3">
                    <div class="w-9 h-9 rounded-lg flex items-center justify-center transition-all duration-300"
                      :class="selectedCommand?.command === cmd.command 
                        ? 'bg-guofeng-jade/25 text-guofeng-jade shadow-inner' 
                        : 'bg-gradient-to-br from-guofeng-indigo/15 to-guofeng-indigo/5 text-guofeng-indigo group-hover:from-guofeng-jade/15 group-hover:to-guofeng-jade/5 group-hover:text-guofeng-jade'"
                    >
                      <Terminal class="w-5 h-5" />
                    </div>
                    <div class="font-bold text-base transition-colors"
                      :class="selectedCommand?.command === cmd.command ? 'text-guofeng-jade' : 'text-guofeng-text-primary group-hover:text-guofeng-jade'"
                    >
                      {{ cmd.name }}
                    </div>
                  </div>
                  <div class="flex items-center gap-2">
                    <div v-if="cmd.dangerous" 
                      class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-gradient-to-r from-guofeng-danger/15 to-guofeng-danger/10 border border-guofeng-danger/40"
                    >
                      <AlertTriangle class="w-3.5 h-3.5 text-guofeng-danger" />
                      <span class="text-xs font-bold text-guofeng-danger">危险</span>
                    </div>
                    <div
                      class="p-2 rounded-lg hover:bg-white/15 transition-all"
                      @click.stop="toggleFavorite(cmd)"
                    >
                      <Star 
                        class="w-4 h-4 transition-all" 
                        :class="isFavorite(cmd.command) ? 'text-guofeng-gold fill-guofeng-gold scale-110 drop-shadow-[0_0_4px_rgba(251,191,36,0.5)]' : 'text-guofeng-text-muted group-hover:text-guofeng-gold/70'" 
                      />
                    </div>
                  </div>
                </div>

                <!-- Command Badge -->
                <div class="mb-3">
                  <div class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg font-mono text-xs font-bold transition-all"
                    :class="selectedCommand?.command === cmd.command 
                      ? 'bg-guofeng-jade/20 text-guofeng-jade border border-guofeng-jade/40 shadow-sm' 
                      : 'bg-black/15 text-guofeng-text-muted border border-white/10 group-hover:border-guofeng-jade/30 group-hover:text-guofeng-jade/90'"
                  >
                    <span class="text-guofeng-jade/70">$</span> ccr {{ cmd.command }}
                  </div>
                </div>

                <!-- Description -->
                <p class="text-sm leading-relaxed line-clamp-2"
                  :class="selectedCommand?.command === cmd.command ? 'text-guofeng-text-primary/90' : 'text-guofeng-text-secondary group-hover:text-guofeng-text-primary/80'"
                >
                  {{ cmd.description }}
                </p>
              </div>
            </button>
          </div>

        </div>

        <!-- 🖕 中间：执行面板 (Execution Panel) - 6 cols -->
        <main class="lg:col-span-6 flex flex-col gap-6">
          <!-- 参数配置 (Params) -->
          <GuofengCard variant="glass" class="relative overflow-hidden flex flex-col">
            <div class="p-4 border-b border-white/10 flex items-center justify-between bg-white/5 backdrop-blur-sm">
              <div class="flex items-center gap-2">
                <Settings class="w-5 h-5 text-guofeng-jade" />
                <h3 class="font-bold text-guofeng-text-primary">
                  {{ selectedCommand ? $t('ccrControl.commandParams') : $t('ccrControl.selectCommandFirst') }}
                </h3>
              </div>
            </div>
            
            <!-- Command Display Banner -->
            <div v-if="selectedCommand" class="relative mx-4 mt-4 mb-6 p-4 rounded-xl bg-gradient-to-r from-guofeng-jade/20 via-guofeng-jade/10 to-transparent border-2 border-guofeng-jade/30 overflow-hidden">
              <!-- Decorative Elements -->
              <div class="absolute top-0 left-0 w-full h-full opacity-10">
                <div class="absolute top-2 right-2 w-12 h-12 rounded-full bg-guofeng-jade/30 blur-xl"></div>
                <div class="absolute bottom-2 left-2 w-8 h-8 rounded-full bg-guofeng-indigo/30 blur-lg"></div>
              </div>
              
              <!-- Content -->
              <div class="relative flex items-center gap-3">
                <div class="flex-shrink-0 w-12 h-12 rounded-lg bg-guofeng-jade/20 border border-guofeng-jade/40 flex items-center justify-center">
                  <Terminal class="w-6 h-6 text-guofeng-jade" />
                </div>
                <div class="flex-1">
                  <div class="text-xs text-guofeng-text-muted mb-1 font-medium">执行命令</div>
                  <code class="text-2xl font-mono font-bold text-guofeng-jade tracking-wide">
                    ccr {{ selectedCommand.command }}
                  </code>
                </div>
                <div v-if="selectedCommand.dangerous" class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-guofeng-danger/10 border border-guofeng-danger/30">
                  <AlertTriangle class="w-5 h-5 text-guofeng-danger" />
                  <span class="text-sm font-bold text-guofeng-danger">危险操作</span>
                </div>
              </div>
            </div>

            <div v-if="selectedCommand" class="p-6 space-y-6">
              <!-- Required Args -->
              <div v-if="selectedCommand.args && selectedCommand.args.length > 0">
                <h4 class="text-xs font-bold uppercase text-guofeng-text-muted mb-3 flex items-center gap-2">
                  <div class="w-1 h-1 rounded-full bg-guofeng-jade"></div>
                  {{ $t('ccrControl.requiredArgs') }}
                </h4>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div v-for="arg in selectedCommand.args" :key="arg.name" class="group">
                    <label class="block text-sm font-medium text-guofeng-text-secondary mb-1.5 group-focus-within:text-guofeng-jade transition-colors">
                      {{ arg.name }} <span v-if="arg.required" class="text-guofeng-danger">*</span>
                    </label>
                    <div class="relative">
                      <select
                        v-if="arg.type === 'select' && arg.options"
                        v-model="commandArgs[arg.name]"
                        class="w-full px-4 py-2.5 rounded-xl bg-black/5 border border-white/10 text-sm text-guofeng-text-primary focus:outline-none focus:border-guofeng-jade/50 focus:ring-2 focus:ring-guofeng-jade/20 transition-all appearance-none"
                      >
                        <option value="" disabled>{{ $t('ccrControl.selectOption') }}</option>
                        <option v-for="opt in arg.options" :key="opt" :value="opt">{{ opt }}</option>
                      </select>
                      <input
                        v-else
                        v-model="commandArgs[arg.name]"
                        type="text"
                        :placeholder="arg.placeholder"
                        class="w-full px-4 py-2.5 rounded-xl bg-black/5 border border-white/10 text-sm text-guofeng-text-primary placeholder:text-guofeng-text-muted/50 focus:outline-none focus:border-guofeng-jade/50 focus:ring-2 focus:ring-guofeng-jade/20 transition-all"
                      />
                    </div>
                    <p class="mt-1 text-xs text-guofeng-text-muted">{{ arg.description }}</p>
                  </div>
                </div>
              </div>

              <!-- Optional Flags -->
              <div v-if="selectedCommand.flags && selectedCommand.flags.length > 0">
                <h4 class="text-xs font-bold uppercase text-guofeng-text-muted mb-3 flex items-center gap-2">
                  <div class="w-1 h-1 rounded-full bg-guofeng-indigo"></div>
                  {{ $t('ccrControl.optionalFlags') }}
                </h4>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div v-for="flag in selectedCommand.flags" :key="flag.name" class="flex items-center gap-3 p-3 rounded-xl bg-white/5 border border-white/5 hover:border-white/10 transition-colors">
                    <template v-if="flag.type === 'boolean'">
                      <div class="relative flex items-center">
                        <input
                          :id="`flag-${flag.name}`"
                          v-model="commandFlags[flag.name]"
                          type="checkbox"
                          class="peer w-5 h-5 rounded border-guofeng-border text-guofeng-jade focus:ring-guofeng-jade/20 bg-transparent cursor-pointer"
                        />
                      </div>
                      <label :for="`flag-${flag.name}`" class="cursor-pointer flex-1">
                        <div class="text-sm font-medium text-guofeng-text-primary">{{ flag.name }}</div>
                        <div class="text-xs font-mono text-guofeng-text-muted">{{ flag.flag }}</div>
                      </label>
                    </template>
                    <template v-else>
                      <div class="flex-1">
                        <label class="text-xs text-guofeng-text-secondary block mb-1">
                          {{ flag.name }} <code class="text-[10px] text-guofeng-text-muted bg-white/10 px-1 rounded">{{ flag.flag }}</code>
                        </label>
                        <input
                          v-model="commandFlags[flag.name]"
                          :type="flag.type === 'number' ? 'number' : 'text'"
                          :placeholder="String(flag.default ?? '')"
                          class="w-full px-2 py-1.5 text-sm rounded-lg bg-black/10 border border-white/5 text-guofeng-text-primary focus:outline-none focus:border-guofeng-jade/50 transition-colors"
                        />
                      </div>
                    </template>
                  </div>
                </div>
              </div>

              <!-- Action Bar -->
              <div class="pt-4 border-t border-dashed border-white/10 flex items-center justify-between gap-4">
                <div v-if="selectedCommand.dangerous" class="flex items-center gap-2 text-guofeng-danger text-sm bg-guofeng-danger/10 px-3 py-1.5 rounded-lg border border-guofeng-danger/20">
                  <AlertTriangle class="w-4 h-4" />
                  <span>{{ $t('ccrControl.dangerousCommand') }}</span>
                </div>
                <div v-else></div> <!-- Spacer -->

                <button
                  class="flex items-center gap-2 px-8 py-3 rounded-xl font-bold text-white shadow-lg shadow-guofeng-jade/20 hover:shadow-guofeng-jade/40 hover:-translate-y-0.5 active:translate-y-0 transition-all disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none"
                  :class="selectedCommand.dangerous ? 'bg-gradient-to-r from-red-500 to-red-600' : 'bg-gradient-to-r from-guofeng-jade to-emerald-500'"
                  :disabled="isExecuting"
                  @click="executeCommand(selectedCommand)"
                >
                  <Loader2 v-if="isExecuting" class="w-5 h-5 animate-spin" />
                  <Play v-else class="w-5 h-5 fill-current" />
                  {{ isExecuting ? $t('ccrControl.executing') : $t('ccrControl.execute') }}
                </button>
              </div>
            </div>
            
            <!-- Empty State -->
            <div v-else class="flex flex-col items-center justify-center py-20 text-guofeng-text-muted">
              <div class="w-20 h-20 rounded-full bg-white/5 flex items-center justify-center mb-4">
                <Terminal class="w-10 h-10 opacity-50" />
              </div>
              <p>{{ $t('ccrControl.selectCommandHint') }}</p>
            </div>
          </GuofengCard>

          <!-- 输出面板 (Output) -->
          <GuofengCard variant="glass" class="flex-1 flex flex-col overflow-hidden min-h-[300px]">
            <div class="p-3 border-b border-white/10 flex items-center justify-between bg-black/20 backdrop-blur-md">
              <h3 class="text-sm font-bold text-guofeng-text-primary flex items-center gap-2">
                <Monitor class="w-4 h-4 text-guofeng-jade" />
                {{ $t('ccrControl.output') }}
              </h3>
              <div class="flex items-center gap-2">
                <div v-if="lastExitCode !== null" class="flex items-center gap-1.5 px-2 py-0.5 rounded text-xs font-mono" :class="lastExitCode === 0 ? 'bg-guofeng-jade/10 text-guofeng-jade' : 'bg-guofeng-danger/10 text-guofeng-danger'">
                  <component :is="lastExitCode === 0 ? CheckCircle : XCircle" class="w-3 h-3" />
                  Code: {{ lastExitCode }}
                </div>
                <button
                  v-if="outputLines.length > 0"
                  class="p-1.5 rounded hover:bg-white/10 text-guofeng-text-muted hover:text-white transition-colors"
                  :title="$t('ccrControl.clearOutput')"
                  @click="clearOutput"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
            
            <div
              ref="outputContainer"
              class="flex-1 p-4 overflow-y-auto font-mono text-sm bg-[#0d1117]/95 custom-scrollbar"
            >
              <div v-if="outputLines.length === 0" class="h-full flex flex-col items-center justify-center text-guofeng-text-muted/30">
                <Terminal class="w-12 h-12 mb-2 opacity-20" />
                <span class="text-xs">{{ $t('ccrControl.noOutput') }}</span>
              </div>
              <div v-else class="space-y-1">
                <div
                  v-for="(line, idx) in outputLines"
                  :key="idx"
                  class="break-all whitespace-pre-wrap"
                  :class="{
                    'text-guofeng-jade font-bold': line.startsWith('$') || line.startsWith('✅'),
                    'text-red-400': line.startsWith('❌') || line.startsWith('[error]') || line.startsWith('[stderr]'),
                    'text-amber-400': line.startsWith('⚠'),
                    'text-gray-100': !line.startsWith('$') && !line.startsWith('✅') && !line.startsWith('❌') && !line.startsWith('⚠')
                  }"
                >
                  {{ line || '\u00A0' }}
                </div>
              </div>
            </div>
          </GuofengCard>
        </main>

        <!-- 👉 右侧：历史与收藏 (History & Favorites) - 3 cols -->
        <aside class="lg:col-span-3 flex flex-col gap-6">
          <!-- Favorites -->
          <GuofengCard variant="glass" class="flex flex-col overflow-hidden max-h-[400px]">
            <div class="p-3 border-b border-white/10 flex items-center justify-between">
              <h2 class="text-xs font-bold uppercase tracking-wider text-guofeng-text-secondary flex items-center gap-2">
                <Star class="w-4 h-4 text-guofeng-gold" />
                {{ $t('ccrControl.favorites') }}
              </h2>
              <span class="text-[10px] bg-white/10 px-1.5 py-0.5 rounded text-guofeng-text-muted">{{ favorites.length }}</span>
            </div>
            
            <div class="p-2 space-y-1 overflow-y-auto custom-scrollbar flex-1">
              <div v-if="favorites.length === 0" class="p-4 text-center text-xs text-guofeng-text-muted">
                {{ $t('ccrControl.noFavorites') }}
              </div>
              <div
                v-for="fav in favorites"
                :key="fav.id"
                class="group flex items-center gap-2 px-2 py-2 rounded-lg hover:bg-white/5 transition-all border border-transparent hover:border-white/5"
              >
                <button
                  class="flex-1 flex items-center gap-2 text-left min-w-0"
                  @click="executeFromFavorite(fav)"
                >
                  <div class="w-6 h-6 rounded bg-guofeng-jade/10 flex items-center justify-center shrink-0 group-hover:bg-guofeng-jade group-hover:text-white transition-colors text-guofeng-jade">
                    <Play class="w-3 h-3 fill-current" />
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="text-sm font-medium text-guofeng-text-primary truncate">{{ fav.display_name || fav.command }}</div>
                    <div class="text-[10px] text-guofeng-text-muted truncate font-mono">ccr {{ fav.command }}</div>
                  </div>
                </button>
                <button
                  class="p-1.5 rounded hover:bg-red-500/20 hover:text-red-400 text-guofeng-text-muted opacity-0 group-hover:opacity-100 transition-all"
                  @click="removeFromFavorites(fav.id)"
                >
                  <X class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </GuofengCard>

          <!-- History -->
          <GuofengCard variant="glass" class="flex flex-col overflow-hidden flex-1 min-h-[300px]">
            <div class="p-3 border-b border-white/10 flex items-center justify-between">
              <h2 class="text-xs font-bold uppercase tracking-wider text-guofeng-text-secondary flex items-center gap-2">
                <History class="w-4 h-4 text-guofeng-indigo" />
                {{ $t('ccrControl.history') }}
              </h2>
              <button
                v-if="history.length > 0"
                class="text-[10px] text-guofeng-text-muted hover:text-guofeng-danger transition-colors"
                @click="clearHistoryData"
              >
                {{ $t('ccrControl.clearHistory') }}
              </button>
            </div>
            
            <div class="p-2 space-y-1 overflow-y-auto custom-scrollbar flex-1">
              <div v-if="history.length === 0" class="p-4 text-center text-xs text-guofeng-text-muted">
                {{ $t('ccrControl.noHistory') }}
              </div>
              <button
                v-for="item in history"
                :key="item.id"
                class="w-full flex items-center gap-3 px-2 py-2 rounded-lg hover:bg-white/5 transition-all group text-left border border-transparent hover:border-white/5"
                @click="executeFromHistory(item)"
              >
                <div class="relative">
                  <div class="w-2 h-2 rounded-full" :class="item.success ? 'bg-guofeng-jade shadow-[0_0_8px_rgba(16,185,129,0.5)]' : 'bg-guofeng-danger shadow-[0_0_8px_rgba(239,68,68,0.5)]'"></div>
                </div>
                <div class="flex-1 min-w-0">
                  <div class="text-xs font-mono text-guofeng-text-primary truncate">
                    {{ item.command }}
                  </div>
                  <div class="flex items-center gap-2 text-[10px] text-guofeng-text-muted mt-0.5">
                    <span>{{ formatTime(item.executed_at) }}</span>
                    <span class="w-0.5 h-0.5 rounded-full bg-guofeng-text-muted"></span>
                    <span>{{ item.duration_ms }}ms</span>
                  </div>
                </div>
                <Play class="w-3 h-3 text-guofeng-jade opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0" />
              </button>
            </div>
          </GuofengCard>
        </aside>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Terminal,
  Info,
  RefreshCw,
  Layers,
  ChevronRight,
  Star,
  History,
  Play,
  X,
  CheckCircle,
  XCircle,
  Settings,
  AlertTriangle,
  Monitor,
  Trash2,
  Loader2,
  Key,
  Puzzle,
  FileText,
  FileUp,
  ArrowRightLeft,
  BarChart,
  Sparkles,
} from 'lucide-vue-next'

import GuofengCard from '@/components/common/GuofengCard.vue'
import { useCcrControl } from '@/composables/useCcrControl'
import type { CcrCommand } from '@/api/ccr-control'

const { t } = useI18n()

// 使用组合式函数
const {
  // 版本
  versionInfo,
  updateInfo,
  loadingVersion,
  loadVersionInfo,
  checkForUpdate,
  // 模块
  modules,
  selectedModuleId,
  selectedModule,
  selectModule,
  // 命令
  selectedCommand,
  selectCommand,
  commandArgs,
  commandFlags,
  // 收藏
  favorites,
  addToFavorites,
  removeFromFavorites,
  isFavorite,
  // 历史
  history,
  clearHistory: clearHistoryData,
  // 执行
  isExecuting,
  outputLines,
  lastExitCode,
  executeCommand,
  executeFromFavorite,
  executeFromHistory,
  clearOutput
} = useCcrControl()

// 输出容器引用
const outputContainer = ref<HTMLElement | null>(null)

// 图标映射
const iconMap: Record<string, any> = {
  'Settings': Settings,
  'Layers': Layers,
  'FileUp': FileUp,
  'ArrowRightLeft': ArrowRightLeft,
  'Key': Key,
  'Puzzle': Puzzle,
  'FileText': FileText,
  'BarChart': BarChart,
  'Terminal': Terminal,
}

const getIcon = (name: string) => {
  return iconMap[name] || Terminal
}

// 切换收藏
const toggleFavorite = async (cmd: CcrCommand) => {
  if (isFavorite(cmd.command)) {
    const fav = favorites.value.find(f => f.command === cmd.command)
    if (fav) {
      await removeFromFavorites(fav.id)
    }
  } else {
    await addToFavorites(cmd)
  }
}

// 执行更新命令
const executeUpdateCommand = () => {
  const updateCmd = modules.value
    .find(m => m.id === 'system')
    ?.commands.find(c => c.command === 'update')
  if (updateCmd) {
    selectModule('system')
    selectCommand(updateCmd)
    executeCommand(updateCmd)
  }
}

// 格式化时间
const formatTime = (dateStr: string) => {
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  
  if (diff < 60000) return 'Just now'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
  return date.toLocaleDateString()
}

// 自动滚动输出到底部
watch(outputLines, async () => {
  await nextTick()
  if (outputContainer.value) {
    outputContainer.value.scrollTop = outputContainer.value.scrollHeight
  }
}, { deep: true })

// 初始化
loadVersionInfo()
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
  height: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

/* 玻璃拟态增强 */
.glass-effect {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

/* 品牌渐变文字 */
.brand-gradient-text {
  background: linear-gradient(135deg, var(--text-primary) 0%, var(--accent-primary) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}
</style>
