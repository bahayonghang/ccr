<template>
  <div class="min-h-screen p-6 transition-colors duration-300 relative overflow-hidden">
    <!-- 🎨 赛博朋克动态背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
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

      <!-- 移动光点 -->
      <div class="absolute top-1/4 left-1/4 w-2 h-2 rounded-full bg-guofeng-jade opacity-40 animate-float-1" />
      <div class="absolute top-1/3 right-1/3 w-1.5 h-1.5 rounded-full bg-guofeng-indigo opacity-30 animate-float-2" />
      <div class="absolute bottom-1/3 left-1/2 w-1 h-1 rounded-full bg-guofeng-gold opacity-20 animate-float-3" />
    </div>

    <div class="max-w-[1920px] mx-auto space-y-6">
      <!-- 🌟 头部区域 -->
      <header class="flex flex-col lg:flex-row lg:items-center justify-between gap-6 animate-fade-in">
        <div class="flex items-center gap-5">
          <div class="relative group">
            <!-- 多层发光效果 -->
            <div class="absolute inset-0 bg-guofeng-jade/30 blur-xl rounded-full group-hover:bg-guofeng-jade/50 transition-all duration-500 animate-pulse-glow" />
            <div class="absolute inset-0 bg-guofeng-jade/20 blur-2xl rounded-full group-hover:bg-guofeng-jade/40 transition-all duration-700" />
            <div class="relative w-16 h-16 rounded-2xl glass-effect flex items-center justify-center border-2 border-guofeng-jade/30 shadow-neon-jade group-hover:scale-110 group-hover:border-guofeng-jade/60 transition-all duration-300">
              <Terminal class="w-8 h-8 text-guofeng-jade drop-shadow-neon" />
            </div>
          </div>
          <div>
            <h1 class="text-3xl font-bold text-guofeng-text-primary tracking-tight mb-1 neon-text-glow">
              {{ $t('ccrControl.title') }}
            </h1>
            <div class="flex items-center gap-3 text-sm text-guofeng-text-secondary">
              <p>{{ $t('ccrControl.description') }}</p>
              <!-- 霓虹版本徽章 -->
              <div
                v-if="versionInfo?.current_version"
                class="flex items-center gap-2 px-3 py-1 rounded-lg bg-guofeng-bg-tertiary/30 border border-guofeng-jade/30 backdrop-blur-sm shadow-neon-jade-sm"
              >
                <span class="text-xs font-mono font-bold text-guofeng-jade neon-text">v{{ versionInfo.current_version }}</span>
                <button
                  v-if="updateInfo?.has_update"
                  class="flex items-center gap-1 text-[10px] font-bold text-guofeng-gold hover:text-white hover:underline transition-colors neon-text"
                  @click="executeUpdateCommand"
                >
                  <Sparkles class="w-3 h-3 animate-pulse" />
                  {{ $t('ccrControl.updateNow') }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <!-- 检查更新按钮 -->
        <button
          class="group flex items-center gap-2 px-5 py-2.5 rounded-xl bg-white/5 hover:bg-guofeng-jade/10 border border-guofeng-jade/20 hover:border-guofeng-jade/50 transition-all backdrop-blur-md shadow-neon-jade-sm hover:shadow-neon-jade"
          :disabled="loadingVersion"
          @click="checkForUpdate"
        >
          <RefreshCw
            class="w-4 h-4 text-guofeng-jade transition-transform duration-700 group-hover:rotate-180"
            :class="{ 'animate-spin': loadingVersion }"
          />
          <span class="text-sm font-bold text-guofeng-text-primary group-hover:text-guofeng-jade transition-colors">
            {{ $t('ccrControl.checkUpdate') }}
          </span>
        </button>
      </header>

      <!-- 🧭 模块选择器 - 重新设计为霓虹标签页 -->
      <section
        class="animate-fade-in"
        style="animation-delay: 0.1s"
      >
        <div class="relative">
          <div class="flex items-center gap-2 overflow-x-auto custom-scrollbar pb-1">
            <button
              v-for="mod in modules"
              :key="mod.id"
              class="relative flex items-center gap-2.5 px-6 py-3 rounded-t-xl transition-all whitespace-nowrap group overflow-hidden"
              :class="selectedModuleId === mod.id
                ? 'bg-guofeng-jade/15 text-guofeng-jade border-b-2 border-guofeng-jade shadow-neon-jade'
                : 'bg-white/5 hover:bg-white/10 text-guofeng-text-secondary hover:text-guofeng-text-primary border-b-2 border-transparent'"
              @click="selectModule(mod.id)"
            >
              <!-- 背景光晕效果 -->
              <div
                v-if="selectedModuleId === mod.id"
                class="absolute inset-0 bg-guofeng-jade/20 blur-xl -z-10 animate-pulse-glow"
              />

              <component
                :is="getIcon(mod.icon)"
                class="w-5 h-5 transition-all duration-300"
                :class="selectedModuleId === mod.id
                  ? 'text-guofeng-jade drop-shadow-neon scale-110'
                  : 'text-guofeng-text-muted group-hover:text-guofeng-text-primary group-hover:scale-110'"
              />
              <span class="text-sm font-bold">{{ mod.name }}</span>
              <span
                v-if="selectedModuleId !== mod.id"
                class="text-[10px] opacity-50 bg-black/20 px-2 py-0.5 rounded-full font-mono"
              >
                {{ mod.commands.length }}
              </span>
            </button>
          </div>
          <!-- 底部霓虹线 -->
          <div class="h-0.5 bg-gradient-to-r from-transparent via-guofeng-jade/30 to-transparent" />
        </div>
      </section>

      <!-- 🏗️ 主体内容区 -->
      <div
        class="grid grid-cols-1 lg:grid-cols-12 gap-6 animate-fade-in"
        style="animation-delay: 0.2s"
      >
        <!-- 👈 左侧：命令列表 - 霓虹卡片设计 -->
        <div class="lg:col-span-3 flex flex-col gap-4">
          <div class="flex items-center justify-between px-1">
            <h2 class="text-lg font-bold text-guofeng-text-primary flex items-center gap-2 neon-text-glow">
              <Layers class="w-5 h-5 text-guofeng-jade drop-shadow-neon" />
              {{ $t('ccrControl.commands') }}
            </h2>
            <span class="text-xs text-guofeng-text-muted bg-guofeng-jade/10 px-2.5 py-1 rounded-lg border border-guofeng-jade/20 font-mono font-bold neon-text">
              {{ selectedModule?.commands.length || 0 }}
            </span>
          </div>

          <div class="grid grid-cols-1 gap-3 overflow-y-auto max-h-[calc(100vh-300px)] custom-scrollbar pr-1">
            <div
              v-for="(cmd, index) in selectedModule?.commands"
              :key="cmd.command"
              class="cursor-pointer group relative text-left animate-slide-in-left"
              :style="{ animationDelay: `${index * 0.05}s` }"
              @click="selectCommand(cmd)"
            >
              <!-- 霓虹边框容器 -->
              <div
                class="relative p-4 rounded-xl border-2 transition-all duration-300 overflow-hidden neon-border"
                :class="selectedCommand?.command === cmd.command
                  ? 'bg-guofeng-jade/10 border-guofeng-jade shadow-neon-jade scale-105'
                  : 'bg-white/5 border-white/10 hover:bg-white/10 hover:border-guofeng-jade/30 hover:shadow-neon-jade-sm hover:scale-[1.02]'"
              >
                <!-- 扫描线动画 -->
                <div
                  v-if="selectedCommand?.command === cmd.command"
                  class="absolute inset-0 bg-gradient-to-b from-transparent via-guofeng-jade/10 to-transparent animate-scan-down pointer-events-none"
                />

                <!-- 内容 -->
                <div class="relative z-10">
                  <div class="flex justify-between items-start mb-2">
                    <div class="flex items-center gap-3 flex-1 min-w-0">
                      <div
                        class="w-9 h-9 rounded-lg flex items-center justify-center transition-all duration-300"
                        :class="selectedCommand?.command === cmd.command
                          ? 'bg-guofeng-jade text-white shadow-neon-jade'
                          : 'bg-white/10 text-guofeng-text-muted group-hover:text-guofeng-jade group-hover:bg-white/20'"
                      >
                        <Terminal class="w-4 h-4" />
                      </div>
                      <div class="flex-1 min-w-0">
                        <div
                          class="font-bold text-sm transition-colors truncate"
                          :class="selectedCommand?.command === cmd.command ? 'text-guofeng-jade neon-text' : 'text-guofeng-text-primary'"
                        >
                          {{ cmd.name }}
                        </div>
                        <div
                          class="text-[10px] font-mono mt-0.5 truncate"
                          :class="selectedCommand?.command === cmd.command ? 'text-guofeng-jade/70' : 'text-guofeng-text-muted'"
                        >
                          ccr {{ cmd.command }}
                        </div>
                      </div>
                    </div>

                    <div class="flex items-center gap-1 shrink-0">
                      <div
                        v-if="cmd.dangerous"
                        class="p-1 rounded bg-guofeng-danger/20 text-guofeng-danger animate-pulse-danger"
                        title="Dangerous"
                      >
                        <AlertTriangle class="w-3.5 h-3.5" />
                      </div>
                      <button
                        class="p-1 rounded hover:bg-white/10 transition-all"
                        @click.stop="toggleFavorite(cmd)"
                      >
                        <Star
                          class="w-3.5 h-3.5 transition-all"
                          :class="isFavorite(cmd.command) ? 'text-guofeng-gold fill-guofeng-gold animate-pulse-star' : 'text-guofeng-text-muted/50 group-hover:text-guofeng-gold'"
                        />
                      </button>
                    </div>
                  </div>

                  <p
                    class="text-xs leading-relaxed line-clamp-2"
                    :class="selectedCommand?.command === cmd.command ? 'text-guofeng-text-secondary' : 'text-guofeng-text-muted'"
                  >
                    {{ cmd.description }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 🖥️ 中间：执行面板 -->
        <main class="lg:col-span-6 flex flex-col gap-6">
          <!-- 参数配置面板 -->
          <GuofengCard
            variant="glass"
            class="relative overflow-hidden flex flex-col !p-0 neon-card"
          >
            <!-- 顶部装饰线 -->
            <div class="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-guofeng-jade to-transparent" />

            <div class="p-4 border-b border-guofeng-border/30 flex items-center justify-between bg-gradient-to-r from-guofeng-jade/5 to-transparent backdrop-blur-sm">
              <div class="flex items-center gap-2">
                <Settings class="w-5 h-5 text-guofeng-jade drop-shadow-neon" />
                <h3 class="font-bold text-guofeng-text-primary neon-text-glow">
                  {{ selectedCommand ? $t('ccrControl.commandParams') : $t('ccrControl.selectCommandFirst') }}
                </h3>
              </div>
            </div>

            <div
              v-if="selectedCommand"
              class="p-6 space-y-6"
            >
              <!-- 命令横幅 - 终端风格 -->
              <div class="relative flex items-center gap-4 p-4 rounded-xl bg-gradient-to-r from-guofeng-jade/10 to-transparent border-2 border-guofeng-jade/20 shadow-neon-jade-sm overflow-hidden group hover:border-guofeng-jade/40 transition-all">
                <!-- 背景扫描效果 -->
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/5 to-transparent translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000" />

                <div class="relative w-12 h-12 rounded-xl bg-guofeng-jade/20 flex items-center justify-center border border-guofeng-jade/30 shadow-neon-jade">
                  <Terminal class="w-6 h-6 text-guofeng-jade drop-shadow-neon" />
                </div>
                <div class="flex-1 min-w-0 relative">
                  <div class="text-xs text-guofeng-text-muted mb-1 font-mono">
                    Selected Command
                  </div>
                  <div class="text-lg font-mono font-bold text-guofeng-jade truncate neon-text">
                    ccr {{ selectedCommand.command }}
                  </div>
                </div>
                <div
                  v-if="selectedCommand.dangerous"
                  class="flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-guofeng-danger/20 border-2 border-guofeng-danger/30 text-xs font-bold text-guofeng-danger shadow-neon-danger animate-pulse-danger"
                >
                  <AlertTriangle class="w-4 h-4" />
                  Dangerous
                </div>
              </div>

              <!-- 必需参数 -->
              <div v-if="selectedCommand.args && selectedCommand.args.length > 0">
                <h4 class="text-xs font-bold uppercase text-guofeng-jade mb-3 flex items-center gap-2 tracking-wider neon-text">
                  <div class="w-1.5 h-1.5 rounded-full bg-guofeng-jade shadow-neon-jade animate-pulse" />
                  {{ $t('ccrControl.requiredArgs') }}
                </h4>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div
                    v-for="arg in selectedCommand.args"
                    :key="arg.name"
                    class="group"
                  >
                    <label class="block text-xs font-medium text-guofeng-text-secondary mb-2 ml-1">
                      {{ arg.name }}
                      <span
                        v-if="arg.required"
                        class="text-guofeng-danger"
                      >*</span>
                    </label>
                    <div class="relative">
                      <select
                        v-if="arg.type === 'select' && arg.options"
                        v-model="commandArgs[arg.name]"
                        class="w-full px-4 py-3 rounded-xl bg-white/5 border-2 border-white/10 text-sm text-guofeng-text-primary focus:outline-none focus:border-guofeng-jade focus:bg-white/10 focus:shadow-neon-jade transition-all appearance-none font-mono"
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
                      <input
                        v-else
                        v-model="commandArgs[arg.name]"
                        type="text"
                        :placeholder="arg.placeholder"
                        class="w-full px-4 py-3 rounded-xl bg-white/5 border-2 border-white/10 text-sm text-guofeng-text-primary placeholder:text-guofeng-text-muted/50 focus:outline-none focus:border-guofeng-jade focus:bg-white/10 focus:shadow-neon-jade transition-all font-mono"
                      >
                    </div>
                    <p class="mt-1.5 ml-1 text-[10px] text-guofeng-text-muted">
                      {{ arg.description }}
                    </p>
                  </div>
                </div>
              </div>

              <!-- 可选标志 -->
              <div v-if="selectedCommand.flags && selectedCommand.flags.length > 0">
                <h4 class="text-xs font-bold uppercase text-guofeng-indigo mb-3 flex items-center gap-2 tracking-wider neon-text">
                  <div class="w-1.5 h-1.5 rounded-full bg-guofeng-indigo shadow-neon-indigo animate-pulse" />
                  {{ $t('ccrControl.optionalFlags') }}
                </h4>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                  <div
                    v-for="flag in selectedCommand.flags"
                    :key="flag.name"
                    class="flex items-center gap-3 p-3 rounded-xl bg-white/5 border-2 border-white/10 hover:border-guofeng-indigo/30 hover:bg-white/10 transition-all"
                  >
                    <template v-if="flag.type === 'boolean'">
                      <input
                        :id="`flag-${flag.name}`"
                        v-model="commandFlags[flag.name]"
                        type="checkbox"
                        class="w-5 h-5 rounded border-2 border-guofeng-border text-guofeng-indigo focus:ring-2 focus:ring-guofeng-indigo/20 bg-transparent cursor-pointer transition-all"
                      >
                      <label
                        :for="`flag-${flag.name}`"
                        class="cursor-pointer flex-1"
                      >
                        <div class="text-sm font-medium text-guofeng-text-primary">{{ flag.name }}</div>
                        <div class="text-[10px] font-mono text-guofeng-text-muted">{{ flag.flag }}</div>
                      </label>
                    </template>
                    <template v-else>
                      <div class="flex-1">
                        <label class="text-[10px] text-guofeng-text-secondary block mb-1">
                          {{ flag.name }} <code class="text-[10px] text-guofeng-text-muted bg-white/10 px-1.5 py-0.5 rounded font-mono">{{ flag.flag }}</code>
                        </label>
                        <input
                          v-model="commandFlags[flag.name]"
                          :type="flag.type === 'number' ? 'number' : 'text'"
                          :placeholder="String(flag.default ?? '')"
                          class="w-full px-3 py-2 text-sm rounded-lg bg-black/20 border-2 border-white/10 text-guofeng-text-primary focus:outline-none focus:border-guofeng-indigo transition-all font-mono"
                        >
                      </div>
                    </template>
                  </div>
                </div>
              </div>

              <!-- 执行按钮区域 -->
              <div class="pt-6 border-t border-dashed border-guofeng-border/30 flex items-center justify-end gap-4">
                <button
                  class="relative group flex items-center gap-2 px-10 py-3 rounded-xl font-bold text-white shadow-2xl transition-all disabled:opacity-50 disabled:cursor-not-allowed overflow-hidden"
                  :class="selectedCommand.dangerous
                    ? 'bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 shadow-neon-danger'
                    : 'bg-gradient-to-r from-guofeng-jade to-emerald-500 hover:from-emerald-500 hover:to-guofeng-jade shadow-neon-jade'"
                  :disabled="isExecuting"
                  @click="executeCommand(selectedCommand)"
                >
                  <!-- 按钮背景动画 -->
                  <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-700" />

                  <!-- 脉冲圆环 -->
                  <div
                    v-if="!isExecuting"
                    class="absolute inset-0 rounded-xl border-2 border-white/50 animate-ping opacity-20"
                  />

                  <Loader2
                    v-if="isExecuting"
                    class="w-5 h-5 animate-spin relative z-10"
                  />
                  <Play
                    v-else
                    class="w-5 h-5 fill-current relative z-10"
                  />
                  <span class="relative z-10">{{ isExecuting ? $t('ccrControl.executing') : $t('ccrControl.execute') }}</span>
                </button>
              </div>
            </div>

            <!-- 空状态 -->
            <div
              v-else
              class="flex flex-col items-center justify-center py-20 text-guofeng-text-muted"
            >
              <div class="relative w-20 h-20 rounded-2xl bg-white/5 flex items-center justify-center mb-4 border border-white/10">
                <Terminal class="w-10 h-10 opacity-30" />
                <div class="absolute inset-0 rounded-2xl border border-guofeng-jade/20 animate-ping" />
              </div>
              <p class="text-sm">
                {{ $t('ccrControl.selectCommandHint') }}
              </p>
            </div>
          </GuofengCard>

          <!-- 输出终端 - 赛博朋克终端美学 -->
          <GuofengCard
            variant="glass"
            class="flex-1 flex flex-col overflow-hidden min-h-[300px] !p-0 neon-card terminal-card"
          >
            <div class="p-3 border-b border-guofeng-border/30 flex items-center justify-between bg-gradient-to-r from-guofeng-jade/5 to-transparent backdrop-blur-md">
              <h3 class="text-sm font-bold text-guofeng-text-primary flex items-center gap-2 neon-text-glow">
                <Monitor class="w-4 h-4 text-guofeng-jade drop-shadow-neon" />
                {{ $t('ccrControl.output') }}
              </h3>
              <div class="flex items-center gap-2">
                <div
                  v-if="lastExitCode !== null"
                  class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-mono font-bold border-2"
                  :class="lastExitCode === 0
                    ? 'bg-guofeng-jade/10 text-guofeng-jade border-guofeng-jade/30 shadow-neon-jade-sm'
                    : 'bg-guofeng-danger/10 text-guofeng-danger border-guofeng-danger/30 shadow-neon-danger-sm'"
                >
                  <component
                    :is="lastExitCode === 0 ? CheckCircle : XCircle"
                    class="w-3.5 h-3.5"
                  />
                  Code: {{ lastExitCode }}
                </div>
              </div>
            </div>

            <div class="flex-1 overflow-hidden relative terminal-screen">
              <!-- CRT 扫描线效果 -->
              <div
                class="absolute inset-0 pointer-events-none opacity-10 animate-crt-scan"
                style="background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0, 255, 100, 0.03) 2px, rgba(0, 255, 100, 0.03) 4px);"
              />

              <!-- 终端内容 -->
              <div
                ref="outputContainer"
                class="h-full overflow-y-auto font-mono text-sm bg-[#0a0e27]/80 custom-scrollbar p-4 terminal-content"
              >
                <div
                  v-if="outputLines.length === 0"
                  class="h-full flex flex-col items-center justify-center text-guofeng-text-muted/30"
                >
                  <Terminal class="w-12 h-12 mb-2 opacity-20" />
                  <span class="text-xs">{{ $t('ccrControl.noOutput') }}</span>
                </div>
                <div
                  v-else
                  class="space-y-1"
                >
                  <div
                    v-for="(line, idx) in outputLines"
                    :key="idx"
                    class="break-all whitespace-pre-wrap terminal-line animate-fade-in"
                  >
                    <span class="text-guofeng-text-muted/30 select-none w-10 inline-block text-right mr-3 text-xs">{{ idx + 1 }}</span>
                    <span
                      :class="{
                        'text-guofeng-jade font-bold neon-text': line.startsWith('$') || line.startsWith('✅'),
                        'text-red-400 neon-text-danger': line.startsWith('❌') || line.startsWith('[error]') || line.startsWith('[stderr]'),
                        'text-amber-400': line.startsWith('⚠'),
                        'text-gray-100': !line.startsWith('$') && !line.startsWith('✅') && !line.startsWith('❌') && !line.startsWith('⚠')
                      }"
                      v-html="renderAnsi(line)"
                    />
                  </div>
                </div>
              </div>

              <!-- 浮动操作按钮 -->
              <div class="absolute top-2 right-2 flex gap-1">
                <button
                  v-if="outputLines.length > 0"
                  class="p-2 rounded-lg hover:bg-white/10 text-guofeng-text-muted hover:text-white transition-all backdrop-blur-sm border border-white/10 hover:border-guofeng-danger/50 hover:shadow-neon-danger-sm"
                  :title="$t('ccrControl.clearOutput')"
                  @click="clearOutput"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </GuofengCard>
        </main>

        <!-- 👉 右侧：历史与收藏 -->
        <aside class="lg:col-span-3 flex flex-col gap-6">
          <!-- 收藏 -->
          <GuofengCard
            variant="glass"
            class="flex flex-col overflow-hidden max-h-[400px] !p-0 neon-card"
          >
            <div class="p-3 border-b border-guofeng-border/30 flex items-center justify-between bg-gradient-to-r from-guofeng-gold/5 to-transparent">
              <h2 class="text-xs font-bold uppercase tracking-wider text-guofeng-gold flex items-center gap-2 neon-text">
                <Star class="w-4 h-4 drop-shadow-neon" />
                {{ $t('ccrControl.favorites') }}
              </h2>
              <span class="text-[10px] bg-guofeng-gold/10 px-2 py-1 rounded-lg border border-guofeng-gold/20 text-guofeng-gold font-mono font-bold">
                {{ favorites.length }}
              </span>
            </div>

            <div class="p-2 space-y-1 overflow-y-auto custom-scrollbar flex-1">
              <div
                v-if="favorites.length === 0"
                class="p-4 text-center text-xs text-guofeng-text-muted"
              >
                {{ $t('ccrControl.noFavorites') }}
              </div>
              <div
                v-for="(fav, index) in favorites"
                :key="fav.id"
                class="cursor-pointer group w-full flex items-center gap-2 px-3 py-2.5 rounded-lg hover:bg-guofeng-gold/10 transition-all border border-transparent hover:border-guofeng-gold/20 hover:shadow-neon-gold-sm animate-slide-in-right"
                :style="{ animationDelay: `${index * 0.05}s` }"
                @click="executeFromFavorite(fav)"
              >
                <div class="w-7 h-7 rounded-lg bg-guofeng-gold/10 flex items-center justify-center border border-guofeng-gold/20 group-hover:bg-guofeng-gold group-hover:text-white group-hover:shadow-neon-gold transition-all text-guofeng-gold">
                  <Play class="w-3.5 h-3.5 fill-current" />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-guofeng-text-primary truncate group-hover:text-guofeng-gold transition-colors">
                    {{ fav.display_name || fav.command }}
                  </div>
                  <div class="text-[10px] text-guofeng-text-muted truncate font-mono">
                    ccr {{ fav.command }}
                  </div>
                </div>
                <button
                  class="p-1.5 rounded-lg hover:bg-red-500/20 hover:text-red-400 text-guofeng-text-muted opacity-0 group-hover:opacity-100 transition-all border border-transparent hover:border-red-500/30"
                  @click.stop="removeFromFavorites(fav.id)"
                >
                  <X class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          </GuofengCard>

          <!-- 历史记录 -->
          <GuofengCard
            variant="glass"
            class="flex flex-col overflow-hidden flex-1 min-h-[300px] !p-0 neon-card"
          >
            <div class="p-3 border-b border-guofeng-border/30 flex items-center justify-between bg-gradient-to-r from-guofeng-indigo/5 to-transparent">
              <h2 class="text-xs font-bold uppercase tracking-wider text-guofeng-indigo flex items-center gap-2 neon-text">
                <History class="w-4 h-4 drop-shadow-neon" />
                {{ $t('ccrControl.history') }}
              </h2>
              <button
                v-if="history.length > 0"
                class="text-[10px] text-guofeng-text-muted hover:text-guofeng-danger transition-colors px-2 py-1 rounded hover:bg-guofeng-danger/10 border border-transparent hover:border-guofeng-danger/20"
                @click="clearHistoryData"
              >
                {{ $t('ccrControl.clearHistory') }}
              </button>
            </div>

            <div class="p-2 space-y-1 overflow-y-auto custom-scrollbar flex-1">
              <div
                v-if="history.length === 0"
                class="p-4 text-center text-xs text-guofeng-text-muted"
              >
                {{ $t('ccrControl.noHistory') }}
              </div>
              <button
                v-for="(item, index) in history"
                :key="item.id"
                class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg hover:bg-guofeng-indigo/10 transition-all group text-left border border-transparent hover:border-guofeng-indigo/20 hover:shadow-neon-indigo-sm animate-slide-in-right"
                :style="{ animationDelay: `${index * 0.05}s` }"
                @click="executeFromHistory(item)"
              >
                <div class="relative">
                  <div
                    class="w-2.5 h-2.5 rounded-full animate-pulse"
                    :class="item.success
                      ? 'bg-guofeng-jade shadow-[0_0_10px_rgba(16,185,129,0.8)]'
                      : 'bg-guofeng-danger shadow-[0_0_10px_rgba(239,68,68,0.8)]'"
                  />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="text-xs font-mono text-guofeng-text-primary truncate group-hover:text-guofeng-indigo transition-colors">
                    {{ item.command }}
                  </div>
                  <div class="flex items-center gap-2 text-[10px] text-guofeng-text-muted mt-0.5 font-mono">
                    <span>{{ formatTime(item.executed_at) }}</span>
                    <span class="w-1 h-1 rounded-full bg-guofeng-text-muted" />
                    <span>{{ item.duration_ms }}ms</span>
                  </div>
                </div>
                <Play class="w-3.5 h-3.5 text-guofeng-indigo opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0" />
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
import { AnsiUp } from 'ansi_up'
import {
  Terminal,
  RefreshCw,
  Layers,
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

// Render ANSI to HTML
const renderAnsi = (text: string) => {
  const ansiUp = new AnsiUp()
  ansiUp.use_classes = true
  return ansiUp.ansi_to_html(text)
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
/* ===== 自定义滚动条 ===== */
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
  height: 4px;
}

.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}

.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(16, 185, 129, 0.3);
  border-radius: 2px;
}

.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: rgba(16, 185, 129, 0.5);
}

/* ===== 霓虹发光效果 ===== */
.shadow-neon-jade {
  box-shadow: 0 0 20px rgba(16, 185, 129, 0.4), 0 0 40px rgba(16, 185, 129, 0.2);
}

.shadow-neon-jade-sm {
  box-shadow: 0 0 10px rgba(16, 185, 129, 0.3), 0 0 20px rgba(16, 185, 129, 0.1);
}

.shadow-neon-indigo {
  box-shadow: 0 0 20px rgba(99, 102, 241, 0.4), 0 0 40px rgba(99, 102, 241, 0.2);
}

.shadow-neon-indigo-sm {
  box-shadow: 0 0 10px rgba(99, 102, 241, 0.3), 0 0 20px rgba(99, 102, 241, 0.1);
}

.shadow-neon-gold {
  box-shadow: 0 0 20px rgba(251, 191, 36, 0.4), 0 0 40px rgba(251, 191, 36, 0.2);
}

.shadow-neon-gold-sm {
  box-shadow: 0 0 10px rgba(251, 191, 36, 0.3), 0 0 20px rgba(251, 191, 36, 0.1);
}

.shadow-neon-danger {
  box-shadow: 0 0 20px rgba(239, 68, 68, 0.4), 0 0 40px rgba(239, 68, 68, 0.2);
}

.shadow-neon-danger-sm {
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.3), 0 0 20px rgba(239, 68, 68, 0.1);
}

.drop-shadow-neon {
  filter: drop-shadow(0 0 8px rgba(16, 185, 129, 0.6));
}

.neon-text {
  text-shadow: 0 0 10px rgba(16, 185, 129, 0.5);
}

.neon-text-glow {
  text-shadow: 0 0 10px rgba(16, 185, 129, 0.3), 0 0 20px rgba(16, 185, 129, 0.2);
}

.neon-text-danger {
  text-shadow: 0 0 10px rgba(239, 68, 68, 0.6);
}

/* ===== 玻璃拟态增强 ===== */
.glass-effect {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.neon-card {
  border: 1px solid rgba(16, 185, 129, 0.1);
}

.neon-border {
  position: relative;
}

.neon-border::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(16, 185, 129, 0.5), transparent);
}

/* ===== 终端样式 ===== */
.terminal-card {
  border: 2px solid rgba(16, 185, 129, 0.2);
}

.terminal-screen {
  background: linear-gradient(180deg, #0a0e27 0%, #0f1419 100%);
}

.terminal-content {
  font-family: 'Courier New', monospace;
  letter-spacing: 0.05em;
}

.terminal-line {
  line-height: 1.6;
  padding: 2px 0;
}

/* ===== 动画定义 ===== */
@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-fade-in {
  animation: fade-in 0.6s ease-out forwards;
}

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

.animate-slide-in-left {
  animation: slide-in-left 0.4s ease-out forwards;
}

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

.animate-slide-in-right {
  animation: slide-in-right 0.4s ease-out forwards;
}

@keyframes pulse-slow {
  0%, 100% {
    opacity: 0.1;
  }
  50% {
    opacity: 0.15;
  }
}

.animate-pulse-slow {
  animation: pulse-slow 4s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% {
    opacity: 0.3;
  }
  50% {
    opacity: 0.6;
  }
}

.animate-pulse-glow {
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-danger {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.animate-pulse-danger {
  animation: pulse-danger 1.5s ease-in-out infinite;
}

@keyframes pulse-star {
  0%, 100% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    transform: scale(1.2);
    opacity: 0.8;
  }
}

.animate-pulse-star {
  animation: pulse-star 1s ease-in-out infinite;
}

@keyframes scan-lines {
  0% {
    transform: translateY(-100%);
  }
  100% {
    transform: translateY(100%);
  }
}

.animate-scan-lines {
  animation: scan-lines 8s linear infinite;
}

@keyframes scan-down {
  0% {
    transform: translateY(-100%);
  }
  100% {
    transform: translateY(100%);
  }
}

.animate-scan-down {
  animation: scan-down 2s ease-in-out infinite;
}

@keyframes crt-scan {
  0% {
    transform: translateY(0);
  }
  100% {
    transform: translateY(100vh);
  }
}

.animate-crt-scan {
  animation: crt-scan 10s linear infinite;
}

@keyframes float-1 {
  0%, 100% {
    transform: translate(0, 0);
  }
  33% {
    transform: translate(30px, -30px);
  }
  66% {
    transform: translate(-20px, 20px);
  }
}

.animate-float-1 {
  animation: float-1 8s ease-in-out infinite;
}

@keyframes float-2 {
  0%, 100% {
    transform: translate(0, 0);
  }
  33% {
    transform: translate(-25px, 25px);
  }
  66% {
    transform: translate(20px, -20px);
  }
}

.animate-float-2 {
  animation: float-2 10s ease-in-out infinite;
}

@keyframes float-3 {
  0%, 100% {
    transform: translate(0, 0);
  }
  33% {
    transform: translate(20px, 30px);
  }
  66% {
    transform: translate(-30px, -15px);
  }
}

.animate-float-3 {
  animation: float-3 12s ease-in-out infinite;
}

/* ===== ANSI 颜色 ===== */
:deep(.ansi-black-fg) { color: #3e4451; }
:deep(.ansi-red-fg) { color: #e06c75; }
:deep(.ansi-green-fg) { color: #98c379; }
:deep(.ansi-yellow-fg) { color: #e5c07b; }
:deep(.ansi-blue-fg) { color: #61afef; }
:deep(.ansi-magenta-fg) { color: #c678dd; }
:deep(.ansi-cyan-fg) { color: #56b6c2; }
:deep(.ansi-white-fg) { color: #abb2bf; }

:deep(.ansi-bright-black-fg) { color: #5c6370; }
:deep(.ansi-bright-red-fg) { color: #e06c75; }
:deep(.ansi-bright-green-fg) { color: #98c379; }
:deep(.ansi-bright-yellow-fg) { color: #e5c07b; }
:deep(.ansi-bright-blue-fg) { color: #61afef; }
:deep(.ansi-bright-magenta-fg) { color: #c678dd; }
:deep(.ansi-bright-cyan-fg) { color: #56b6c2; }
:deep(.ansi-bright-white-fg) { color: #ffffff; }

/* ===== 响应式优化 ===== */
@media (max-width: 1024px) {
  .neon-text-glow {
    text-shadow: none;
  }

  .shadow-neon-jade,
  .shadow-neon-jade-sm,
  .shadow-neon-indigo,
  .shadow-neon-indigo-sm,
  .shadow-neon-gold,
  .shadow-neon-gold-sm,
  .shadow-neon-danger,
  .shadow-neon-danger-sm {
    box-shadow: none;
  }
}

/* ===== 性能优化 ===== */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
</style>
