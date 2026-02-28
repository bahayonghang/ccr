<template>
  <div class="min-h-screen p-5 transition-colors duration-300">
    <div class="mb-6" />

    <div class="max-w-[1200px] mx-auto">
      <!-- Header -->
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-xl sm:text-2xl font-bold text-guofeng-text-primary flex items-center">
            <Settings2
              class="w-6 h-6 sm:w-7 sm:h-7 mr-2 text-guofeng-red"
              aria-hidden="true"
            />
            {{ $t('claudeSettings.title') }}
          </h2>
        </div>
        <div class="flex gap-3">
          <RouterLink to="/claude-code">
            <button
              class="px-4 py-2 rounded-lg font-medium transition-all bg-guofeng-bg-secondary text-guofeng-text-secondary border border-guofeng-border hover:bg-guofeng-bg-tertiary min-h-[44px] flex items-center"
            >
              <ArrowLeft
                class="w-4 h-4 mr-2"
                aria-hidden="true"
              />
              {{ $t('claudeSettings.back') }}
            </button>
          </RouterLink>
          <button
            class="px-4 py-2 rounded-lg font-medium transition-all hover:scale-105 bg-guofeng-red text-white shadow-md hover:shadow-lg flex items-center min-h-[44px]"
            :disabled="saving"
            @click="handleSave"
          >
            <Save
              class="w-4 h-4 mr-2"
              aria-hidden="true"
            />
            {{ saving ? $t('claudeSettings.saving') : $t('claudeSettings.save') }}
          </button>
        </div>
      </div>

      <!-- Loading -->
      <div
        v-if="loading"
        class="text-center py-20 text-guofeng-text-muted"
      >
        <div class="loading-spinner mx-auto mb-4 w-8 h-8 border-guofeng-red/30 border-t-guofeng-red" />
        <span>Loading...</span>
      </div>

      <template v-else>
        <!-- Tab Navigation -->
        <div
          class="mb-6 flex gap-2 overflow-x-auto pb-2 scrollbar-thin md:flex-wrap md:overflow-x-visible md:pb-0"
          role="tablist"
        >
          <button
            v-for="tab in tabs"
            :key="tab.key"
            role="tab"
            :aria-selected="activeTab === tab.key"
            class="px-4 py-2 rounded-lg font-medium text-sm transition-all min-h-[44px] whitespace-nowrap flex-shrink-0 flex items-center gap-2"
            :class="activeTab === tab.key ? 'bg-guofeng-red text-white shadow-md' : 'bg-guofeng-bg-secondary text-guofeng-text-secondary border border-guofeng-border hover:bg-guofeng-bg-tertiary'"
            @click="activeTab = tab.key"
          >
            <component
              :is="tab.icon"
              class="w-4 h-4"
            />
            {{ tab.label }}
          </button>
        </div>

        <!-- Tab: 模型与推理 -->
        <div
          v-show="activeTab === 'model'"
          class="space-y-6"
        >
          <GuofengCard
            variant="glass"
            pattern
          >
            <div class="p-5 space-y-5">
              <h3 class="text-lg font-bold text-guofeng-text-primary">
                {{ $t('claudeSettings.tabs.model') }}
              </h3>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.model.defaultModel') }}</label>
                <select
                  v-model="form.model"
                  class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                >
                  <option value="">
                    {{ $t('claudeSettings.model.noOverride') }}
                  </option>
                  <option
                    v-for="m in modelOptions"
                    :key="m"
                    :value="m"
                  >
                    {{ m }}
                  </option>
                </select>
              </div>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.model.effortLevel') }}</label>
                <select
                  v-model="form.effortLevel"
                  class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                >
                  <option value="">
                    {{ $t('claudeSettings.model.noOverride') }}
                  </option>
                  <option value="low">
                    low
                  </option>
                  <option value="medium">
                    medium
                  </option>
                  <option value="high">
                    high
                  </option>
                </select>
              </div>

              <!-- Toggle: alwaysThinkingEnabled -->
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="form.alwaysThinkingEnabled"
                  type="checkbox"
                  class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                >
                <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.model.alwaysThinking') }}</span>
              </label>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.model.maxThinkingTokens') }}</label>
                  <input
                    v-model="form.maxThinkingTokens"
                    type="text"
                    placeholder="31999"
                    class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                  >
                </div>
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.model.maxOutputTokens') }}</label>
                  <input
                    v-model="form.maxOutputTokens"
                    type="text"
                    placeholder="64000"
                    class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                  >
                </div>
              </div>

              <!-- TagList: availableModels -->
              <div>
                <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.model.availableModels') }}</label>
                <div
                  v-if="form.availableModels.length > 0"
                  class="flex flex-wrap gap-2 mb-2"
                >
                  <span
                    v-for="(item, i) in form.availableModels"
                    :key="i"
                    class="px-2.5 py-1 rounded-lg bg-guofeng-red/10 text-guofeng-red text-sm flex items-center gap-1.5 border border-guofeng-red/20"
                  >
                    {{ item }}
                    <button
                      class="hover:text-red-400"
                      @click="form.availableModels.splice(i, 1)"
                    ><XIcon class="w-3 h-3" /></button>
                  </span>
                </div>
                <div class="flex gap-2">
                  <input
                    v-model="tagInputs.availableModels"
                    :placeholder="$t('claudeSettings.model.addModel')"
                    class="flex-1 px-3 py-2 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary text-sm"
                    @keydown.enter.prevent="addTag('availableModels', form.availableModels)"
                  >
                  <button
                    class="px-3 py-2 rounded-lg bg-guofeng-red text-white text-sm hover:scale-105 transition-all"
                    @click="addTag('availableModels', form.availableModels)"
                  >
                    <Plus class="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </GuofengCard>
        </div>

        <!-- Tab: 权限管理 -->
        <div
          v-show="activeTab === 'permissions'"
          class="space-y-6"
        >
          <GuofengCard
            variant="glass"
            pattern
          >
            <div class="p-5 space-y-5">
              <h3 class="text-lg font-bold text-guofeng-text-primary">
                {{ $t('claudeSettings.tabs.permissions') }}
              </h3>

              <div>
                <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.permissions.defaultMode') }}</label>
                <select
                  v-model="permDefaultMode"
                  class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                >
                  <option value="">
                    {{ $t('claudeSettings.model.noOverride') }}
                  </option>
                  <option value="default">
                    default
                  </option>
                  <option value="plan">
                    plan
                  </option>
                  <option value="bypassPermissions">
                    bypassPermissions
                  </option>
                </select>
              </div>

              <!-- Toggle: skipDangerousModePermissionPrompt -->
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="form.skipDangerousModePermissionPrompt"
                  type="checkbox"
                  class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                >
                <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.permissions.skipDangerous') }}</span>
              </label>

              <!-- TagList: permAllow -->
              <div>
                <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.permissions.allow') }}</label>
                <div
                  v-if="permAllow.length > 0"
                  class="flex flex-wrap gap-2 mb-2"
                >
                  <span
                    v-for="(item, i) in permAllow"
                    :key="i"
                    class="px-2.5 py-1 rounded-lg bg-guofeng-red/10 text-guofeng-red text-sm flex items-center gap-1.5 border border-guofeng-red/20"
                  >
                    {{ item }}
                    <button
                      class="hover:text-red-400"
                      @click="permAllow.splice(i, 1)"
                    ><XIcon class="w-3 h-3" /></button>
                  </span>
                </div>
                <div class="flex gap-2">
                  <input
                    v-model="tagInputs.permAllow"
                    placeholder="Bash, Read, Write..."
                    class="flex-1 px-3 py-2 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary text-sm"
                    @keydown.enter.prevent="addTag('permAllow', permAllow)"
                  >
                  <button
                    class="px-3 py-2 rounded-lg bg-guofeng-red text-white text-sm hover:scale-105 transition-all"
                    @click="addTag('permAllow', permAllow)"
                  >
                    <Plus class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <!-- TagList: permDeny -->
              <div>
                <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.permissions.deny') }}</label>
                <div
                  v-if="permDeny.length > 0"
                  class="flex flex-wrap gap-2 mb-2"
                >
                  <span
                    v-for="(item, i) in permDeny"
                    :key="i"
                    class="px-2.5 py-1 rounded-lg bg-guofeng-red/10 text-guofeng-red text-sm flex items-center gap-1.5 border border-guofeng-red/20"
                  >
                    {{ item }}
                    <button
                      class="hover:text-red-400"
                      @click="permDeny.splice(i, 1)"
                    ><XIcon class="w-3 h-3" /></button>
                  </span>
                </div>
                <div class="flex gap-2">
                  <input
                    v-model="tagInputs.permDeny"
                    placeholder="mcp__dangerous..."
                    class="flex-1 px-3 py-2 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary text-sm"
                    @keydown.enter.prevent="addTag('permDeny', permDeny)"
                  >
                  <button
                    class="px-3 py-2 rounded-lg bg-guofeng-red text-white text-sm hover:scale-105 transition-all"
                    @click="addTag('permDeny', permDeny)"
                  >
                    <Plus class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <!-- TagList: permAdditionalDirs -->
              <div>
                <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.permissions.additionalDirs') }}</label>
                <div
                  v-if="permAdditionalDirs.length > 0"
                  class="flex flex-wrap gap-2 mb-2"
                >
                  <span
                    v-for="(item, i) in permAdditionalDirs"
                    :key="i"
                    class="px-2.5 py-1 rounded-lg bg-guofeng-red/10 text-guofeng-red text-sm flex items-center gap-1.5 border border-guofeng-red/20"
                  >
                    {{ item }}
                    <button
                      class="hover:text-red-400"
                      @click="permAdditionalDirs.splice(i, 1)"
                    ><XIcon class="w-3 h-3" /></button>
                  </span>
                </div>
                <div class="flex gap-2">
                  <input
                    v-model="tagInputs.permAdditionalDirs"
                    placeholder="/path/to/dir"
                    class="flex-1 px-3 py-2 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary text-sm"
                    @keydown.enter.prevent="addTag('permAdditionalDirs', permAdditionalDirs)"
                  >
                  <button
                    class="px-3 py-2 rounded-lg bg-guofeng-red text-white text-sm hover:scale-105 transition-all"
                    @click="addTag('permAdditionalDirs', permAdditionalDirs)"
                  >
                    <Plus class="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </GuofengCard>
        </div>

        <!-- Tab: 环境变量 -->
        <div
          v-show="activeTab === 'env'"
          class="space-y-6"
        >
          <GuofengCard
            variant="glass"
            pattern
          >
            <div class="p-5 space-y-5">
              <div class="flex items-center justify-between">
                <h3 class="text-lg font-bold text-guofeng-text-primary">
                  {{ $t('claudeSettings.tabs.env') }}
                </h3>
                <button
                  class="px-3 py-1.5 rounded-lg text-sm font-medium bg-guofeng-red text-white hover:scale-105 transition-all flex items-center gap-1"
                  @click="addEnvVar"
                >
                  <Plus class="w-4 h-4" /> {{ $t('claudeSettings.env.add') }}
                </button>
              </div>

              <div
                v-if="envEntries.length === 0"
                class="text-center py-8 text-guofeng-text-muted"
              >
                {{ $t('claudeSettings.env.empty') }}
              </div>

              <div
                v-for="(entry, idx) in envEntries"
                :key="idx"
                class="flex gap-2 items-start"
              >
                <input
                  v-model="entry.key"
                  placeholder="KEY"
                  class="flex-1 px-3 py-2 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary text-sm font-mono"
                >
                <input
                  v-model="entry.value"
                  placeholder="value"
                  :type="entry.key.includes('TOKEN') || entry.key.includes('KEY') || entry.key.includes('SECRET') ? 'password' : 'text'"
                  class="flex-[2] px-3 py-2 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary text-sm font-mono"
                >
                <button
                  class="p-2 rounded-lg text-red-400 hover:bg-red-500/10 transition-all min-w-[36px] min-h-[36px] flex items-center justify-center"
                  @click="envEntries.splice(idx, 1)"
                >
                  <Trash2 class="w-4 h-4" />
                </button>
              </div>
            </div>
          </GuofengCard>
        </div>

        <!-- Tab: UI 体验 -->
        <div
          v-show="activeTab === 'ui'"
          class="space-y-6"
        >
          <GuofengCard
            variant="glass"
            pattern
          >
            <div class="p-5 space-y-5">
              <h3 class="text-lg font-bold text-guofeng-text-primary">
                {{ $t('claudeSettings.tabs.ui') }}
              </h3>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.theme') }}</label>
                  <input
                    v-model="form.theme"
                    type="text"
                    placeholder="dark, light, dark-daltonized..."
                    class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                  >
                </div>
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.language') }}</label>
                  <input
                    v-model="form.language"
                    type="text"
                    placeholder="zh-CN, en, ja..."
                    class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                  >
                </div>
              </div>

              <div class="space-y-3">
                <label class="flex items-center gap-3 cursor-pointer">
                  <input
                    v-model="form.showTurnDuration"
                    type="checkbox"
                    class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                  >
                  <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.showTurnDuration') }}</span>
                </label>
                <label class="flex items-center gap-3 cursor-pointer">
                  <input
                    v-model="form.spinnerTipsEnabled"
                    type="checkbox"
                    class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                  >
                  <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.spinnerTips') }}</span>
                </label>
                <label class="flex items-center gap-3 cursor-pointer">
                  <input
                    v-model="form.terminalProgressBarEnabled"
                    type="checkbox"
                    class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                  >
                  <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.progressBar') }}</span>
                </label>
                <label class="flex items-center gap-3 cursor-pointer">
                  <input
                    v-model="form.showSpinnerTree"
                    type="checkbox"
                    class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                  >
                  <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.spinnerTree') }}</span>
                </label>
                <label class="flex items-center gap-3 cursor-pointer">
                  <input
                    v-model="form.prefersReducedMotion"
                    type="checkbox"
                    class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                  >
                  <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.reducedMotion') }}</span>
                </label>
              </div>
            </div>
          </GuofengCard>

          <GuofengCard
            variant="glass"
            pattern
          >
            <div class="p-5 space-y-5">
              <h3 class="text-lg font-bold text-guofeng-text-primary">
                {{ $t('claudeSettings.ui.misc') }}
              </h3>
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.updateChannel') }}</label>
                  <select
                    v-model="form.autoUpdatesChannel"
                    class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                  >
                    <option value="">
                      {{ $t('claudeSettings.model.noOverride') }}
                    </option>
                    <option value="stable">
                      stable
                    </option>
                    <option value="latest">
                      latest
                    </option>
                  </select>
                </div>
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.cleanupDays') }}</label>
                  <input
                    v-model.number="form.cleanupPeriodDays"
                    type="number"
                    placeholder="30"
                    class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                  >
                </div>
              </div>
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="form.autoUpdates"
                  type="checkbox"
                  class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                >
                <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.autoUpdates') }}</span>
              </label>
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="form.respectGitignore"
                  type="checkbox"
                  class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                >
                <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.ui.respectGitignore') }}</span>
              </label>
            </div>
          </GuofengCard>
        </div>

        <!-- Tab: 沙箱安全 -->
        <div
          v-show="activeTab === 'sandbox'"
          class="space-y-6"
        >
          <GuofengCard
            variant="glass"
            pattern
          >
            <div class="p-5 space-y-5">
              <h3 class="text-lg font-bold text-guofeng-text-primary">
                {{ $t('claudeSettings.tabs.sandbox') }}
              </h3>

              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="sandboxEnabled"
                  type="checkbox"
                  class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                >
                <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.sandbox.enabled') }}</span>
              </label>
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="sandboxAutoAllow"
                  type="checkbox"
                  class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                >
                <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.sandbox.autoAllowBash') }}</span>
              </label>
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="sandboxAllowLocal"
                  type="checkbox"
                  class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                >
                <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.sandbox.allowLocalBinding') }}</span>
              </label>

              <!-- TagList: sandboxAllowedDomains -->
              <div>
                <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.sandbox.allowedDomains') }}</label>
                <div
                  v-if="sandboxAllowedDomains.length > 0"
                  class="flex flex-wrap gap-2 mb-2"
                >
                  <span
                    v-for="(item, i) in sandboxAllowedDomains"
                    :key="i"
                    class="px-2.5 py-1 rounded-lg bg-guofeng-red/10 text-guofeng-red text-sm flex items-center gap-1.5 border border-guofeng-red/20"
                  >
                    {{ item }}
                    <button
                      class="hover:text-red-400"
                      @click="sandboxAllowedDomains.splice(i, 1)"
                    ><XIcon class="w-3 h-3" /></button>
                  </span>
                </div>
                <div class="flex gap-2">
                  <input
                    v-model="tagInputs.sandboxAllowedDomains"
                    placeholder="api.anthropic.com"
                    class="flex-1 px-3 py-2 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary text-sm"
                    @keydown.enter.prevent="addTag('sandboxAllowedDomains', sandboxAllowedDomains)"
                  >
                  <button
                    class="px-3 py-2 rounded-lg bg-guofeng-red text-white text-sm hover:scale-105 transition-all"
                    @click="addTag('sandboxAllowedDomains', sandboxAllowedDomains)"
                  >
                    <Plus class="w-4 h-4" />
                  </button>
                </div>
              </div>

              <!-- TagList: sandboxExcludedCmds -->
              <div>
                <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.sandbox.excludedCommands') }}</label>
                <div
                  v-if="sandboxExcludedCmds.length > 0"
                  class="flex flex-wrap gap-2 mb-2"
                >
                  <span
                    v-for="(item, i) in sandboxExcludedCmds"
                    :key="i"
                    class="px-2.5 py-1 rounded-lg bg-guofeng-red/10 text-guofeng-red text-sm flex items-center gap-1.5 border border-guofeng-red/20"
                  >
                    {{ item }}
                    <button
                      class="hover:text-red-400"
                      @click="sandboxExcludedCmds.splice(i, 1)"
                    ><XIcon class="w-3 h-3" /></button>
                  </span>
                </div>
                <div class="flex gap-2">
                  <input
                    v-model="tagInputs.sandboxExcludedCmds"
                    placeholder="docker, npm..."
                    class="flex-1 px-3 py-2 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary text-sm"
                    @keydown.enter.prevent="addTag('sandboxExcludedCmds', sandboxExcludedCmds)"
                  >
                  <button
                    class="px-3 py-2 rounded-lg bg-guofeng-red text-white text-sm hover:scale-105 transition-all"
                    @click="addTag('sandboxExcludedCmds', sandboxExcludedCmds)"
                  >
                    <Plus class="w-4 h-4" />
                  </button>
                </div>
              </div>
            </div>
          </GuofengCard>
        </div>

        <!-- Tab: Git 归属 -->
        <div
          v-show="activeTab === 'git'"
          class="space-y-6"
        >
          <GuofengCard
            variant="glass"
            pattern
          >
            <div class="p-5 space-y-5">
              <h3 class="text-lg font-bold text-guofeng-text-primary">
                {{ $t('claudeSettings.tabs.git') }}
              </h3>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.git.commitAttribution') }}</label>
                  <select
                    v-model="attrCommit"
                    class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                  >
                    <option value="">
                      {{ $t('claudeSettings.model.noOverride') }}
                    </option>
                    <option value="none">
                      none
                    </option>
                    <option value="co-authored-by">
                      co-authored-by
                    </option>
                    <option value="authored-by">
                      authored-by
                    </option>
                  </select>
                </div>
                <div>
                  <label class="block mb-1.5 text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.git.prAttribution') }}</label>
                  <select
                    v-model="attrPr"
                    class="w-full px-4 py-2.5 rounded-lg bg-guofeng-bg-tertiary border border-guofeng-border focus:border-guofeng-red focus:ring-1 focus:ring-guofeng-red outline-none transition-all text-guofeng-text-primary"
                  >
                    <option value="">
                      {{ $t('claudeSettings.model.noOverride') }}
                    </option>
                    <option value="none">
                      none
                    </option>
                    <option value="co-authored-by">
                      co-authored-by
                    </option>
                    <option value="authored-by">
                      authored-by
                    </option>
                  </select>
                </div>
              </div>

              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  v-model="form.includeCoAuthoredBy"
                  type="checkbox"
                  class="w-4 h-4 rounded border-guofeng-border text-guofeng-red focus:ring-guofeng-red"
                >
                <span class="text-sm font-semibold text-guofeng-text-primary">{{ $t('claudeSettings.git.includeCoAuthored') }}</span>
              </label>
            </div>
          </GuofengCard>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { Settings2, ArrowLeft, Save, Plus, Trash2, X as XIcon, Brain, Shield, Terminal, Palette, Lock, GitBranch } from 'lucide-vue-next'
import GuofengCard from '@/components/common/GuofengCard.vue'
import { getClaudeSettings, updateClaudeSettings } from '@/api/modules/claudeSettings'
import type { ClaudeSettingsData } from '@/api/modules/claudeSettings'
import { useUIStore } from '@/store'

const { t } = useI18n()
const uiStore = useUIStore()

const loading = ref(true)
const saving = ref(false)
const activeTab = ref('model')

const tabs = computed(() => [
  { key: 'model', label: t('claudeSettings.tabs.model'), icon: Brain },
  { key: 'permissions', label: t('claudeSettings.tabs.permissions'), icon: Shield },
  { key: 'env', label: t('claudeSettings.tabs.env'), icon: Terminal },
  { key: 'ui', label: t('claudeSettings.tabs.ui'), icon: Palette },
  { key: 'sandbox', label: t('claudeSettings.tabs.sandbox'), icon: Lock },
  { key: 'git', label: t('claudeSettings.tabs.git'), icon: GitBranch },
])

const modelOptions = ['opus', 'sonnet', 'haiku', 'claude-opus-4-6', 'claude-sonnet-4-5-20250929', 'claude-haiku-4-5-20251001']

// --- Form state ---
const form = reactive<{
  model?: string
  availableModels: string[]
  alwaysThinkingEnabled?: boolean
  maxThinkingTokens?: string
  maxOutputTokens?: string
  effortLevel?: string
  skipDangerousModePermissionPrompt?: boolean
  theme?: string
  language?: string
  showTurnDuration?: boolean
  prefersReducedMotion?: boolean
  spinnerTipsEnabled?: boolean
  terminalProgressBarEnabled?: boolean
  showSpinnerTree?: boolean
  includeCoAuthoredBy?: boolean
  autoUpdates?: boolean
  autoUpdatesChannel?: string
  cleanupPeriodDays?: number
  respectGitignore?: boolean
}>({
  availableModels: [],
})

// Permissions (flat refs for reactivity)
const permAllow = ref<string[]>([])
const permDeny = ref<string[]>([])
const permDefaultMode = ref('')
const permAdditionalDirs = ref<string[]>([])

// Env entries (key-value pairs)
const envEntries = ref<{ key: string; value: string }[]>([])

// Sandbox
const sandboxEnabled = ref<boolean | undefined>()
const sandboxAutoAllow = ref<boolean | undefined>()
const sandboxAllowLocal = ref<boolean | undefined>()
const sandboxAllowedDomains = ref<string[]>([])
const sandboxExcludedCmds = ref<string[]>([])

// Attribution
const attrCommit = ref('')
const attrPr = ref('')

// Tag list input state
const tagInputs = reactive<Record<string, string>>({
  availableModels: '',
  permAllow: '',
  permDeny: '',
  permAdditionalDirs: '',
  sandboxAllowedDomains: '',
  sandboxExcludedCmds: '',
})

// --- Tag helpers ---
function addTag(field: string, targetArray: string[]) {
  const val = tagInputs[field]?.trim()
  if (val && !targetArray.includes(val)) {
    targetArray.push(val)
    tagInputs[field] = ''
  }
}

// --- Load ---
async function loadSettings() {
  loading.value = true
  try {
    const data = await getClaudeSettings()

    form.model = data.model || ''
    form.availableModels = data.availableModels || []
    form.alwaysThinkingEnabled = data.alwaysThinkingEnabled
    form.maxThinkingTokens = data.maxThinkingTokens
    form.maxOutputTokens = data.maxOutputTokens
    form.effortLevel = data.effortLevel || ''
    form.skipDangerousModePermissionPrompt = data.skipDangerousModePermissionPrompt
    form.theme = data.theme
    form.language = data.language
    form.showTurnDuration = data.showTurnDuration
    form.prefersReducedMotion = data.prefersReducedMotion
    form.spinnerTipsEnabled = data.spinnerTipsEnabled
    form.terminalProgressBarEnabled = data.terminalProgressBarEnabled
    form.showSpinnerTree = data.showSpinnerTree
    form.includeCoAuthoredBy = data.includeCoAuthoredBy
    form.autoUpdates = data.autoUpdates
    form.autoUpdatesChannel = data.autoUpdatesChannel || ''
    form.cleanupPeriodDays = data.cleanupPeriodDays
    form.respectGitignore = data.respectGitignore

    permAllow.value = data.permissions?.allow || []
    permDeny.value = data.permissions?.deny || []
    permDefaultMode.value = data.permissions?.defaultMode || ''
    permAdditionalDirs.value = data.permissions?.additionalDirectories || []

    envEntries.value = Object.entries(data.env || {}).map(([key, value]) => ({ key, value }))

    sandboxEnabled.value = data.sandbox?.enabled
    sandboxAutoAllow.value = data.sandbox?.autoAllowBashIfSandboxed
    sandboxAllowLocal.value = data.sandbox?.network?.allowLocalBinding
    sandboxAllowedDomains.value = data.sandbox?.network?.allowedDomains || []
    sandboxExcludedCmds.value = data.sandbox?.excludedCommands || []

    attrCommit.value = data.attribution?.commit || ''
    attrPr.value = data.attribution?.pr || ''
  } catch (e: unknown) {
    uiStore.showError(`Failed to load settings: ${e instanceof Error ? e.message : e}`)
  } finally {
    loading.value = false
  }
}

// --- Save ---
async function handleSave() {
  saving.value = true
  try {
    const env: Record<string, string> = {}
    for (const entry of envEntries.value) {
      if (entry.key.trim()) {
        env[entry.key.trim()] = entry.value
      }
    }

    const data: ClaudeSettingsData = {
      model: form.model || undefined,
      availableModels: form.availableModels.length > 0 ? form.availableModels : undefined,
      alwaysThinkingEnabled: form.alwaysThinkingEnabled,
      maxThinkingTokens: form.maxThinkingTokens || undefined,
      maxOutputTokens: form.maxOutputTokens || undefined,
      effortLevel: form.effortLevel || undefined,
      permissions: {
        allow: permAllow.value,
        deny: permDeny.value,
        defaultMode: permDefaultMode.value || undefined,
        additionalDirectories: permAdditionalDirs.value.length > 0 ? permAdditionalDirs.value : undefined,
      },
      skipDangerousModePermissionPrompt: form.skipDangerousModePermissionPrompt,
      env,
      theme: form.theme || undefined,
      language: form.language || undefined,
      showTurnDuration: form.showTurnDuration,
      prefersReducedMotion: form.prefersReducedMotion,
      spinnerTipsEnabled: form.spinnerTipsEnabled,
      terminalProgressBarEnabled: form.terminalProgressBarEnabled,
      showSpinnerTree: form.showSpinnerTree,
      sandbox: (sandboxEnabled.value != null || sandboxAutoAllow.value != null || sandboxAllowedDomains.value.length > 0 || sandboxExcludedCmds.value.length > 0) ? {
        enabled: sandboxEnabled.value,
        autoAllowBashIfSandboxed: sandboxAutoAllow.value,
        excludedCommands: sandboxExcludedCmds.value.length > 0 ? sandboxExcludedCmds.value : undefined,
        network: (sandboxAllowLocal.value != null || sandboxAllowedDomains.value.length > 0) ? {
          allowLocalBinding: sandboxAllowLocal.value,
          allowedDomains: sandboxAllowedDomains.value.length > 0 ? sandboxAllowedDomains.value : undefined,
        } : undefined,
      } : undefined,
      attribution: (attrCommit.value || attrPr.value) ? {
        commit: attrCommit.value || undefined,
        pr: attrPr.value || undefined,
      } : undefined,
      includeCoAuthoredBy: form.includeCoAuthoredBy,
      autoUpdates: form.autoUpdates,
      autoUpdatesChannel: form.autoUpdatesChannel || undefined,
      cleanupPeriodDays: form.cleanupPeriodDays,
      respectGitignore: form.respectGitignore,
    }

    await updateClaudeSettings(data)
    uiStore.showSuccess(t('claudeSettings.saveSuccess'))
  } catch (e: unknown) {
    uiStore.showError(`Failed to save: ${e instanceof Error ? e.message : e}`)
  } finally {
    saving.value = false
  }
}

function addEnvVar() {
  envEntries.value.push({ key: '', value: '' })
}

onMounted(loadSettings)
</script>
