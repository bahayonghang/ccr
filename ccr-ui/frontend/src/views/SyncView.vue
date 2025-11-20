<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色渐变背景装饰 - 像首页一样 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #06b6d4 0%, #3b82f6 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-20 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #8b5cf6 0%, #ec4899 100%)',
          animationDelay: '1s'
        }"
      />
      <div
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[500px] h-[500px] rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #10b981 0%, #f59e0b 100%)',
          animationDelay: '2s'
        }"
      />
    </div>

    <main class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-8 pb-8">
      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          { label: $t('sync.breadcrumb.home'), path: '/', icon: Home },
          { label: $t('sync.breadcrumb.claudeCode'), path: '/claude-code', icon: Code2 },
          { label: $t('sync.breadcrumb.sync'), path: '/sync', icon: Cloud }
        ]"
        module-color="#6366f1"
      />

      <div class="mb-12">
        <div class="flex items-center justify-between mb-6 animate-fade-in">
          <div class="flex items-center gap-4">
            <div
              class="p-4 rounded-3xl glass-card"
              :style="{ background: 'rgba(6, 182, 212, 0.1)' }"
            >
              <Cloud
                class="w-10 h-10"
                :style="{ color: '#06b6d4' }"
              />
            </div>
            <div>
              <h1 class="text-4xl md:text-5xl font-bold mb-2 bg-gradient-to-r from-[#06b6d4] via-[#3b82f6] to-[#8b5cf6] bg-clip-text text-transparent">
                {{ $t('sync.title') }}
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t('sync.subtitle') }}
              </p>
            </div>
          </div>
          <RouterLink
            to="/"
            class="group glass-card flex items-center gap-2 px-5 py-3 hover:scale-105 transition-all duration-300"
          >
            <Home
              class="w-5 h-5"
              :style="{ color: '#64748b' }"
            />
            <span
              class="font-medium"
              :style="{ color: 'var(--text-secondary)' }"
            >{{ $t('sync.backHome') }}</span>
          </RouterLink>
        </div>
      </div>

      <!-- Loading state -->
      <div
        v-if="loading"
        class="flex items-center justify-center py-16"
      >
        <div class="p-8 glass-card">
          <RefreshCw
            class="w-12 h-12 animate-spin"
            :style="{ color: 'var(--accent-info)' }"
          />
        </div>
      </div>

      <!-- Error state -->
      <div
        v-else-if="error"
        class="glass-card p-6 flex items-start gap-4"
      >
        <div
          class="p-3 rounded-2xl"
          :style="{ background: 'rgba(239, 68, 68, 0.1)' }"
        >
          <XCircle
            class="w-7 h-7"
            :style="{ color: 'var(--accent-danger)' }"
          />
        </div>
        <div class="flex-1">
          <h3
            class="font-bold text-xl mb-2"
            :style="{ color: 'var(--text-primary)' }"
          >
            {{ $t('sync.loadFailed') }}
          </h3>
          <p
            class="text-base"
            :style="{ color: 'var(--text-secondary)' }"
          >
            {{ error }}
          </p>
        </div>
      </div>

      <!-- 主要内容 -->
      <div
        v-else
        class="grid grid-cols-1 lg:grid-cols-3 gap-6"
      >
        <!-- 左侧主内容区 (2 columns) -->
        <div class="lg:col-span-2 space-y-6">
          <!-- Platform selection -->
          <div class="glass-card p-6 hover:scale-[1.01] transition-all duration-300">
            <!-- Header -->
            <div class="flex items-center justify-between mb-6">
              <div class="flex items-center gap-3">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(16, 185, 129, 0.1)' }"
                >
                  <CheckSquare
                    class="w-6 h-6"
                    :style="{ color: 'var(--accent-success)' }"
                  />
                </div>
                <h2
                  class="text-2xl font-bold"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('sync.platformSelection.title') }}
                </h2>
              </div>
              <button
                :disabled="applying || !hasChanges"
                class="flex items-center gap-2 px-4 py-2.5 rounded-xl glass-card font-medium transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
                :style="{ background: applying || !hasChanges ? 'rgba(156, 163, 175, 0.1)' : 'rgba(16, 185, 129, 0.1)', color: applying || !hasChanges ? '#9ca3af' : 'var(--accent-success)' }"
                @click="applySelection"
              >
                <Save class="w-4 h-4" />
                <span>{{ applying ? $t('sync.platformSelection.applying') : $t('sync.platformSelection.applyButton') }}</span>
              </button>
            </div>

            <!-- Config (required) -->
            <div
              class="mb-6 p-5 rounded-xl glass-card"
              :style="{ background: 'rgba(245, 158, 11, 0.05)' }"
            >
              <div class="flex items-center gap-4">
                <div
                  class="p-2 rounded-xl"
                  :style="{ background: 'rgba(245, 158, 11, 0.15)' }"
                >
                  <CheckCircle
                    class="w-6 h-6"
                    :style="{ color: 'var(--accent-warning)' }"
                  />
                </div>
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-2">
                    <h3
                      class="text-lg font-bold"
                      :style="{ color: 'var(--text-primary)' }"
                    >
                      {{ $t('sync.platformSelection.configRequired') }}
                    </h3>
                    <span
                      class="px-2.5 py-1 rounded-full text-xs font-bold"
                      :style="{ background: 'rgba(245, 158, 11, 0.2)', color: 'var(--accent-warning)' }"
                    >
                      {{ $t('sync.platformSelection.configRequiredBadge') }}
                    </span>
                  </div>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    {{ $t('sync.platformSelection.configDescription') }}
                  </p>
                  <div class="flex items-center gap-2">
                    <Folder
                      class="w-4 h-4"
                      :style="{ color: '#94a3b8' }"
                    />
                    <input
                      v-model="presetItems.config.localPath"
                      type="text"
                      class="flex-1 px-3 py-2 rounded-lg glass-card text-sm focus:outline-none focus:ring-2"
                      :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
                      :placeholder="$t('sync.customFolder.localPathPlaceholder')"
                    >
                  </div>
                </div>
              </div>
            </div>

            <!-- 可选平台列表 -->
            <div class="space-y-4">
              <div
                v-for="(item, index) in optionalItems"
                :key="item.key"
                class="p-5 rounded-xl glass-card cursor-pointer hover:scale-[1.02] transition-all duration-300"
                :style="{ 
                  background: item.selected ? 'rgba(99, 102, 241, 0.05)' : 'transparent',
                  animationDelay: `${index * 0.05}s`
                }"
                @click="toggleItem(item.key)"
              >
                <div class="flex items-start gap-4">
                  <div class="flex-shrink-0">
                    <div
                      class="w-7 h-7 rounded-lg flex items-center justify-center transition-all duration-300"
                      :style="{ 
                        background: item.selected ? 'rgba(99, 102, 241, 0.15)' : 'rgba(156, 163, 175, 0.1)',
                        border: item.selected ? '2px solid #6366f1' : '2px solid #e5e7eb'
                      }"
                    >
                      <Check
                        v-if="item.selected"
                        class="w-4 h-4"
                        :style="{ color: '#6366f1' }"
                      />
                    </div>
                  </div>
                  <div class="flex-1">
                    <div class="flex items-center gap-3 mb-2">
                      <div
                        class="p-2 rounded-lg"
                        :style="{ background: 'rgba(99, 102, 241, 0.1)' }"
                      >
                        <component
                          :is="item.icon"
                          class="w-5 h-5"
                          :style="{ color: '#6366f1' }"
                        />
                      </div>
                      <h3
                        class="text-lg font-bold"
                        :style="{ color: 'var(--text-primary)' }"
                      >
                        {{ item.name }}
                      </h3>
                    </div>
                    <p
                      class="text-sm mb-3"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ item.description }}
                    </p>
                    <div
                      v-if="item.selected"
                      class="space-y-2"
                      @click.stop
                    >
                      <div class="flex items-center gap-2">
                        <Folder
                          class="w-4 h-4"
                          :style="{ color: '#94a3b8' }"
                        />
                        <input
                          v-model="item.localPath"
                          type="text"
                          class="flex-1 px-3 py-2 rounded-lg glass-card text-sm focus:outline-none focus:ring-2"
                          :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
                          :placeholder="$t('sync.customFolder.localPathPlaceholder')"
                        >
                      </div>
                      <div class="flex items-center gap-2">
                        <Cloud
                          class="w-4 h-4"
                          :style="{ color: '#94a3b8' }"
                        />
                        <input
                          v-model="item.remotePath"
                          type="text"
                          class="flex-1 px-3 py-2 rounded-lg glass-card text-sm focus:outline-none focus:ring-2"
                          :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
                          :placeholder="$t('sync.customFolder.remotePathPlaceholder')"
                        >
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- Custom folder -->
            <div
              class="mt-6 p-5 rounded-xl glass-card"
              :style="{ background: 'rgba(139, 92, 246, 0.05)' }"
            >
              <div class="flex items-center gap-3 mb-4">
                <div
                  class="p-2 rounded-xl"
                  :style="{ background: 'rgba(139, 92, 246, 0.15)' }"
                >
                  <Plus
                    class="w-5 h-5"
                    :style="{ color: 'var(--accent-secondary)' }"
                  />
                </div>
                <h3
                  class="text-lg font-bold"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('sync.customFolder.title') }}
                </h3>
              </div>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                <input
                  v-model="customFolder.name"
                  type="text"
                  :placeholder="$t('sync.customFolder.namePlaceholder')"
                  class="px-4 py-2 rounded-lg glass-card text-sm focus:outline-none focus:ring-2"
                  :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
                >
                <input
                  v-model="customFolder.localPath"
                  type="text"
                  :placeholder="$t('sync.customFolder.localPathPlaceholder')"
                  class="px-4 py-2 rounded-lg glass-card text-sm focus:outline-none focus:ring-2"
                  :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
                >
                <input
                  v-model="customFolder.remotePath"
                  type="text"
                  :placeholder="$t('sync.customFolder.remotePathPlaceholder')"
                  class="px-4 py-2 rounded-lg glass-card text-sm focus:outline-none focus:ring-2"
                  :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
                >
                <input
                  v-model="customFolder.description"
                  type="text"
                  :placeholder="$t('sync.customFolder.descriptionPlaceholder')"
                  class="px-4 py-2 rounded-lg glass-card text-sm focus:outline-none focus:ring-2"
                  :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
                >
              </div>
              <button
                :disabled="!customFolder.name || !customFolder.localPath || addingCustom"
                class="w-full px-4 py-2.5 rounded-lg glass-card font-medium transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                :style="{ background: 'rgba(139, 92, 246, 0.1)', color: 'var(--accent-secondary)' }"
                @click="addCustomFolder"
              >
                <Plus class="w-5 h-5" />
                {{ addingCustom ? $t('sync.customFolder.adding') : $t('sync.customFolder.addButton') }}
              </button>
            </div>
          </div>

          <!-- Enabled folders list -->
          <div class="glass-card p-6 hover:scale-[1.01] transition-all duration-300">
            <div class="flex items-center justify-between mb-6">
              <div class="flex items-center gap-3">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(59, 130, 246, 0.1)' }"
                >
                  <Folders
                    class="w-6 h-6"
                    :style="{ color: 'var(--accent-info)' }"
                  />
                </div>
                <h2
                  class="text-2xl font-bold"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('sync.enabledFolders.title') }}
                </h2>
              </div>
              <button
                class="flex items-center gap-2 px-4 py-2.5 rounded-xl glass-card transition-all duration-300 hover:scale-105"
                :style="{ background: 'rgba(59, 130, 246, 0.1)', color: 'var(--accent-info)' }"
                @click="refreshFolders"
              >
                <RefreshCw
                  class="w-4 h-4"
                  :class="{ 'animate-spin': refreshingFolders }"
                />
                <span class="font-medium">{{ $t('sync.enabledFolders.refresh') }}</span>
              </button>
            </div>
            <div
              v-if="enabledFolders.length === 0"
              class="text-center py-12"
            >
              <div
                class="p-4 rounded-2xl inline-block"
                :style="{ background: 'rgba(156, 163, 175, 0.1)' }"
              >
                <FolderOpen
                  class="w-16 h-16"
                  :style="{ color: '#9ca3af' }"
                />
              </div>
              <p
                class="text-lg mt-4"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t('sync.enabledFolders.noFolders') }}
              </p>
              <p
                class="text-sm mt-2"
                :style="{ color: 'var(--text-muted)' }"
              >
                {{ $t('sync.enabledFolders.noFoldersHint') }}
              </p>
            </div>

            <div
              v-else
              class="space-y-4"
            >
              <div
                v-for="(folder, index) in enabledFolders"
                :key="folder.name"
                class="p-5 rounded-xl glass-card hover:scale-[1.01] transition-all duration-300"
                :style="{ animationDelay: `${index * 0.05}s` }"
              >
                <div class="flex items-start justify-between mb-4">
                  <div class="flex-1">
                    <div class="flex items-center gap-3 mb-2">
                      <h4
                        class="text-xl font-bold"
                        :style="{ color: 'var(--text-primary)' }"
                      >
                        {{ folder.name }}
                      </h4>
                      <span
                        class="px-3 py-1 rounded-full text-sm font-medium"
                        :style="{
                          background: folder.enabled ? 'rgba(16, 185, 129, 0.15)' : 'rgba(156, 163, 175, 0.15)',
                          color: folder.enabled ? 'var(--accent-success)' : '#9ca3af'
                        }"
                      >
                        {{ folder.enabled ? $t('sync.enabledFolders.enabled') : $t('sync.enabledFolders.disabled') }}
                      </span>
                    </div>
                    <p
                      v-if="folder.description"
                      class="text-sm mb-2"
                      :style="{ color: 'var(--text-secondary)' }"
                    >
                      {{ folder.description }}
                    </p>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-sm">
                      <div
                        class="flex items-center gap-2"
                        :style="{ color: 'var(--text-secondary)' }"
                      >
                        <Folder class="w-4 h-4" />
                        <span class="font-mono">{{ folder.localPath }}</span>
                      </div>
                      <div
                        class="flex items-center gap-2"
                        :style="{ color: 'var(--text-secondary)' }"
                      >
                        <Cloud class="w-4 h-4" />
                        <span class="font-mono">{{ folder.remotePath }}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Action buttons -->
                <div class="flex flex-wrap gap-2">
                  <button
                    class="px-4 py-2 rounded-lg glass-card font-medium transition-all duration-300 hover:scale-105 flex items-center gap-2"
                    :style="{ background: 'rgba(99, 102, 241, 0.1)', color: 'var(--accent-primary)' }"
                    @click="toggleFolder(folder.name, folder.enabled)"
                  >
                    <ToggleLeft class="w-4 h-4" />
                    {{ folder.enabled ? $t('sync.operations.disable') : $t('sync.operations.enable') }}
                  </button>
                  <button
                    :disabled="!folder.enabled"
                    class="px-4 py-2 rounded-lg glass-card font-medium transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                    :style="{ background: 'rgba(16, 185, 129, 0.1)', color: 'var(--accent-success)' }"
                    @click="pushFolder(folder.name)"
                  >
                    <Upload class="w-4 h-4" />
                    {{ $t('sync.operations.upload') }}
                  </button>
                  <button
                    :disabled="!folder.enabled"
                    class="px-4 py-2 rounded-lg glass-card font-medium transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                    :style="{ background: 'rgba(139, 92, 246, 0.1)', color: 'var(--accent-secondary)' }"
                    @click="pullFolder(folder.name)"
                  >
                    <Download class="w-4 h-4" />
                    {{ $t('sync.operations.download') }}
                  </button>
                  <button
                    class="px-4 py-2 rounded-lg glass-card font-medium transition-all duration-300 hover:scale-105 flex items-center gap-2"
                    :style="{ background: 'rgba(59, 130, 246, 0.1)', color: 'var(--accent-info)' }"
                    @click="getFolderStatus(folder.name)"
                  >
                    <Info class="w-4 h-4" />
                    {{ $t('sync.operations.status') }}
                  </button>
                  <button
                    class="px-4 py-2 rounded-lg glass-card font-medium transition-all duration-300 hover:scale-105 flex items-center gap-2"
                    :style="{ background: 'rgba(239, 68, 68, 0.1)', color: 'var(--accent-danger)' }"
                    @click="removeFolder(folder.name)"
                  >
                    <Trash2 class="w-4 h-4" />
                    {{ $t('sync.operations.delete') }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- Batch operations card -->
          <div class="glass-card p-6 hover:scale-[1.01] transition-all duration-300">
            <div class="flex items-center gap-3 mb-4">
              <div
                class="p-3 rounded-2xl"
                :style="{ background: 'rgba(245, 158, 11, 0.1)' }"
              >
                <Layers
                  class="w-6 h-6"
                  :style="{ color: 'var(--accent-warning)' }"
                />
              </div>
              <h2
                class="text-2xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('sync.batchOperations.title') }}
              </h2>
            </div>

            <p
              class="text-sm mb-4"
              :style="{ color: 'var(--text-secondary)' }"
            >
              {{ $t('sync.batchOperations.description') }}
            </p>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
              <button
                :disabled="batchOperating || enabledFolders.length === 0"
                class="px-6 py-4 rounded-xl glass-card font-bold transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                :style="{ background: 'rgba(16, 185, 129, 0.1)', color: 'var(--accent-success)' }"
                @click="pushAllFolders"
              >
                <Upload class="w-5 h-5" />
                {{ $t('sync.batchOperations.uploadAll') }}
              </button>
              <button
                :disabled="batchOperating || enabledFolders.length === 0"
                class="px-6 py-4 rounded-xl glass-card font-bold transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                :style="{ background: 'rgba(139, 92, 246, 0.1)', color: 'var(--accent-secondary)' }"
                @click="pullAllFolders"
              >
                <Download class="w-5 h-5" />
                {{ $t('sync.batchOperations.downloadAll') }}
              </button>
              <button
                :disabled="batchOperating || enabledFolders.length === 0"
                class="px-6 py-4 rounded-xl glass-card font-bold transition-all duration-300 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                :style="{ background: 'rgba(59, 130, 246, 0.1)', color: 'var(--accent-info)' }"
                @click="getAllFoldersStatus"
              >
                <Info class="w-5 h-5" />
                {{ $t('sync.batchOperations.viewStatus') }}
              </button>
            </div>
          </div>

          <!-- Operation output card -->
          <div
            v-if="operationOutput"
            class="glass-card p-6"
          >
            <div class="flex items-center justify-between mb-4">
              <div class="flex items-center gap-3">
                <div
                  class="p-2 rounded-xl"
                  :style="{ background: 'rgba(99, 102, 241, 0.1)' }"
                >
                  <Terminal
                    class="w-5 h-5"
                    :style="{ color: 'var(--accent-primary)' }"
                  />
                </div>
                <h2
                  class="text-xl font-bold"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('sync.output.title') }}
                </h2>
              </div>
              <button
                class="p-2 rounded-lg glass-card transition-all duration-300 hover:scale-110"
                :style="{ background: 'rgba(156, 163, 175, 0.1)' }"
                @click="operationOutput = ''"
              >
                <XCircle
                  class="w-4 h-4"
                  :style="{ color: '#9ca3af' }"
                />
              </button>
            </div>
            <pre
              class="text-sm font-mono whitespace-pre-wrap overflow-x-auto glass-card p-4 rounded-lg"
              :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
            >{{ operationOutput }}</pre>
          </div>
        </div>

        <!-- 右侧信息区 (1 column) -->
        <div class="space-y-6">
          <!-- WebDAV configuration status -->
          <div class="glass-card p-6 hover:scale-[1.01] transition-all duration-300">
            <div class="flex items-center gap-3 mb-6">
              <div
                class="p-3 rounded-2xl"
                :style="{ background: 'rgba(99, 102, 241, 0.1)' }"
              >
                <Settings
                  class="w-6 h-6"
                  :style="{ color: 'var(--accent-primary)' }"
                />
              </div>
              <h2
                class="text-xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('sync.webdav.title') }}
              </h2>
            </div>

            <div
              v-if="syncStatus?.configured && syncStatus.config"
              class="space-y-4"
            >
              <div
                class="flex items-center gap-3 px-4 py-3 rounded-xl"
                :style="{ background: 'rgba(16, 185, 129, 0.1)' }"
              >
                <CheckCircle
                  class="w-5 h-5"
                  :style="{ color: 'var(--accent-success)' }"
                />
                <span
                  class="font-medium"
                  :style="{ color: 'var(--text-primary)' }"
                >{{ $t('sync.webdav.configured') }}</span>
              </div>

              <div class="space-y-3">
                <div>
                  <div
                    class="text-xs mb-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('sync.webdav.server') }}
                  </div>
                  <div
                    class="text-sm font-mono break-all"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ syncStatus.config.webdav_url }}
                  </div>
                </div>
                <div>
                  <div
                    class="text-xs mb-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('sync.webdav.username') }}
                  </div>
                  <div
                    class="text-sm font-mono"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ syncStatus.config.username }}
                  </div>
                </div>
                <div>
                  <div
                    class="text-xs mb-1"
                    :style="{ color: 'var(--text-muted)' }"
                  >
                    {{ $t('sync.webdav.remotePath') }}
                  </div>
                  <div
                    class="text-sm font-mono break-all"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    {{ syncStatus.config.remote_path }}
                  </div>
                </div>
              </div>
            </div>

            <div
              v-else
              class="space-y-4"
            >
              <div
                class="flex items-center gap-3 px-4 py-3 rounded-xl"
                :style="{ background: 'rgba(245, 158, 11, 0.1)' }"
              >
                <AlertCircle
                  class="w-5 h-5"
                  :style="{ color: 'var(--accent-warning)' }"
                />
                <span
                  class="font-medium"
                  :style="{ color: 'var(--text-primary)' }"
                >{{ $t('sync.webdav.notConfigured') }}</span>
              </div>
              <p
                class="text-sm"
                :style="{ color: 'var(--text-secondary)' }"
              >
                {{ $t('sync.webdav.configureHint') }}
              </p>
              <code
                class="block text-sm font-mono glass-card p-3 rounded-lg"
                :style="{ color: 'var(--text-primary)', background: 'rgba(255, 255, 255, 0.5)' }"
              >{{ $t('sync.webdav.configureCommand') }}</code>
            </div>
          </div>

          <!-- Features description -->
          <div class="glass-card p-6 hover:scale-[1.01] transition-all duration-300">
            <div class="flex items-center gap-3 mb-6">
              <div
                class="p-3 rounded-2xl"
                :style="{ background: 'rgba(236, 72, 153, 0.1)' }"
              >
                <BookOpen
                  class="w-6 h-6"
                  :style="{ color: 'var(--accent-tertiary)' }"
                />
              </div>
              <h2
                class="text-xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('sync.features.title') }}
              </h2>
            </div>

            <div
              class="space-y-4 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <div>
                <h4
                  class="font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('sync.features.presetPlatform') }}
                </h4>
                <p>{{ $t('sync.features.presetPlatformDesc') }}</p>
              </div>
              <div>
                <h4
                  class="font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('sync.features.independentManagement') }}
                </h4>
                <p>{{ $t('sync.features.independentManagementDesc') }}</p>
              </div>
              <div>
                <h4
                  class="font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('sync.features.smartFiltering') }}
                </h4>
                <p>{{ $t('sync.features.smartFilteringDesc') }}</p>
              </div>
              <div>
                <h4
                  class="font-bold mb-2"
                  :style="{ color: 'var(--text-primary)' }"
                >
                  {{ $t('sync.features.batchOperations') }}
                </h4>
                <p>{{ $t('sync.features.batchOperationsDesc') }}</p>
              </div>
            </div>
          </div>

          <!-- Supported services -->
          <div class="glass-card p-6 hover:scale-[1.01] transition-all duration-300">
            <div class="flex items-center gap-3 mb-6">
              <div
                class="p-3 rounded-2xl"
                :style="{ background: 'rgba(16, 185, 129, 0.1)' }"
              >
                <Server
                  class="w-6 h-6"
                  :style="{ color: 'var(--accent-success)' }"
                />
              </div>
              <h2
                class="text-xl font-bold"
                :style="{ color: 'var(--text-primary)' }"
              >
                {{ $t('sync.supportedServices.title') }}
              </h2>
            </div>

            <div
              class="space-y-3 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <div class="flex items-center gap-2">
                <CheckCircle
                  class="w-4 h-4"
                  :style="{ color: 'var(--accent-success)' }"
                />
                <span>{{ $t('sync.supportedServices.nutstore') }}</span>
              </div>
              <div class="flex items-center gap-2">
                <CheckCircle
                  class="w-4 h-4"
                  :style="{ color: 'var(--accent-success)' }"
                />
                <span>{{ $t('sync.supportedServices.nextcloud') }}</span>
              </div>
              <div class="flex items-center gap-2">
                <CheckCircle
                  class="w-4 h-4"
                  :style="{ color: 'var(--accent-success)' }"
                />
                <span>{{ $t('sync.supportedServices.owncloud') }}</span>
              </div>
              <div class="flex items-center gap-2">
                <CheckCircle
                  class="w-4 h-4"
                  :style="{ color: 'var(--accent-success)' }"
                />
                <span>{{ $t('sync.supportedServices.any') }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useI18n } from 'vue-i18n'
import axios from 'axios'

const { t } = useI18n()
import {
  Cloud,
  Home,
  RefreshCw,
  XCircle,
  CheckCircle,
  AlertCircle,
  Settings,
  Server,
  FolderOpen,
  Folder,
  Code2,
  BookOpen,
  Upload,
  Download,
  Info,
  Plus,
  Trash2,
  ToggleLeft,
  Folders,
  Layers,
  Terminal,
  CheckSquare,
  Check,
  Save
} from 'lucide-vue-next'
import Breadcrumb from '@/components/Breadcrumb.vue'

// API 基础 URL
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8081'

// 状态
const loading = ref(true)
const error = ref('')
const syncStatus = ref<any>(null)
const enabledFolders = ref<any[]>([])
const operationOutput = ref('')

// 操作状态
const refreshingFolders = ref(false)
const applying = ref(false)
const addingCustom = ref(false)
const batchOperating = ref(false)

// 预设项目配置
const presetItems = ref({
  config: {
    key: 'config',
    name: 'Platforms 平台配置',
    description: 'CCR 供应商配置（API地址、密钥等）',
    localPath: '~/.ccr/platforms/',
    remotePath: '',
    selected: true, // 必选
    required: true
  }
})

// 可选平台列表
const optionalItems = ref([
  {
    key: 'claude',
    name: 'Claude Code',
    description: 'Anthropic Claude Code CLI 配置和数据',
    icon: Code2,
    localPath: '~/.claude/',
    remotePath: '',
    selected: false
  },
  {
    key: 'gemini',
    name: 'Gemini CLI',
    description: 'Google Gemini CLI 配置和数据',
    icon: Cloud,
    localPath: '~/.gemini/',
    remotePath: '',
    selected: false
  },
  {
    key: 'qwen',
    name: 'Qwen',
    description: '通义千问 CLI 配置和数据',
    icon: Cloud,
    localPath: '~/.qwen/',
    remotePath: '',
    selected: false
  },
  {
    key: 'iflow',
    name: 'iFlow',
    description: 'iFlow CLI 配置和数据',
    icon: Cloud,
    localPath: '~/.iflow/',
    remotePath: '',
    selected: false
  }
])

// 自定义文件夹表单
const customFolder = ref({
  name: '',
  localPath: '',
  remotePath: '',
  description: ''
})

// 计算是否有变更
const hasChanges = computed(() => {
  // 检查预设项目是否有选择
  if (optionalItems.value.some(item => item.selected)) {
    return true
  }
  return false
})

// 切换选项
const toggleItem = (key: string) => {
  const item = optionalItems.value.find(i => i.key === key)
  if (item) {
    item.selected = !item.selected
  }
}

// 应用选择 - 将选中的项目注册为同步文件夹
const applySelection = async () => {
  applying.value = true
  try {
    const selectedItems = [
      presetItems.value.config,
      ...optionalItems.value.filter(item => item.selected)
    ]

    for (const item of selectedItems) {
      // 检查文件夹是否已存在
      const existingFolder = enabledFolders.value.find(f => f.name === item.key)
      if (existingFolder) {
        continue // 跳过已存在的文件夹
      }

      // 添加文件夹
      const payload: any = {
        name: item.key,
        local_path: item.localPath
      }
      if (item.remotePath) {
        payload.remote_path = item.remotePath
      }
      if (item.description) {
        payload.description = item.description
      } else {
        payload.description = item.name
      }

      try {
        await axios.post(`${API_BASE_URL}/api/sync/folders`, payload)
      } catch (err: any) {
        console.error(`添加文件夹 ${item.name} 失败:`, err)
        // 继续添加其他文件夹
      }
    }

    operationOutput.value = '✓ 同步配置已应用'
    await refreshFolders()
  } catch (err: any) {
    operationOutput.value = `✗ 应用失败: ${err.response?.data?.message || err.message}`
  } finally {
    applying.value = false
  }
}

// 添加自定义文件夹
const addCustomFolder = async () => {
  if (!customFolder.value.name || !customFolder.value.localPath) return

  addingCustom.value = true
  try {
    const payload: any = {
      name: customFolder.value.name,
      local_path: customFolder.value.localPath
    }
    if (customFolder.value.remotePath) {
      payload.remote_path = customFolder.value.remotePath
    }
    if (customFolder.value.description) {
      payload.description = customFolder.value.description
    }

    const response = await axios.post(`${API_BASE_URL}/api/sync/folders`, payload)
    if (response.data.success) {
      operationOutput.value = `${t('sync.messages.addSuccess')}: ${customFolder.value.name}`
      customFolder.value = { name: '', localPath: '', remotePath: '', description: '' }
      await refreshFolders()
    } else {
      operationOutput.value = `${t('sync.messages.addFailed')}: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `${t('sync.messages.addFailed')}: ${err.response?.data?.message || err.message}`
  } finally {
    addingCustom.value = false
  }
}

// 获取同步状态
const fetchSyncStatus = async () => {
  try {
    const response = await axios.get(`${API_BASE_URL}/api/sync/status`)
    if (response.data.success) {
      syncStatus.value = response.data.data
    }
  } catch (err: any) {
    console.error('Failed to fetch sync status:', err)
  }
}

// 获取文件夹列表
const fetchFolders = async () => {
  try {
    const response = await axios.get(`${API_BASE_URL}/api/sync/folders`)
    if (response.data.success) {
      // 解析 CLI 输出获取文件夹列表
      parseFoldersList(response.data.data.output)
    }
  } catch (err: any) {
    console.error('Failed to fetch folders:', err)
  }
}

// 解析文件夹列表输出
const parseFoldersList = (_output: string) => {
  // TODO: 实现解析逻辑
  // 暂时设置为空数组
  enabledFolders.value = []
}

// 刷新文件夹列表
const refreshFolders = async () => {
  refreshingFolders.value = true
  try {
    await fetchFolders()
  } finally {
    refreshingFolders.value = false
  }
}

// Delete folder
const removeFolder = async (name: string) => {
  if (!confirm(t('sync.messages.deleteConfirm', { name }))) {
    return
  }

  try {
    const response = await axios.delete(`${API_BASE_URL}/api/sync/folders/${name}`)
    if (response.data.success) {
      operationOutput.value = `${t('sync.messages.deleteSuccess')}: ${name}`
      await refreshFolders()
    } else {
      operationOutput.value = `${t('sync.messages.deleteFailed')}: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `${t('sync.messages.deleteFailed')}: ${err.response?.data?.message || err.message}`
  }
}

// Toggle folder status
const toggleFolder = async (name: string, currentEnabled: boolean) => {
  const action = currentEnabled ? 'disable' : 'enable'
  const actionText = currentEnabled ? t('sync.messages.disabled') : t('sync.messages.enabled')
  try {
    const response = await axios.put(`${API_BASE_URL}/api/sync/folders/${name}/${action}`)
    if (response.data.success) {
      operationOutput.value = t('sync.messages.toggleSuccess', { action: actionText }) + `: ${name}`
      await refreshFolders()
    } else {
      operationOutput.value = `${t('sync.messages.toggleFailed')}: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `${t('sync.messages.toggleFailed')}: ${err.response?.data?.message || err.message}`
  }
}

// Upload folder
const pushFolder = async (name: string) => {
  try {
    const response = await axios.post(`${API_BASE_URL}/api/sync/folders/${name}/push`, { force: false })
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `${t('sync.messages.uploadFailed')}: ${response.data.data.error}`
    }
  } catch (err: any) {
    operationOutput.value = `${t('sync.messages.uploadFailed')}: ${err.response?.data?.message || err.message}`
  }
}

// Download folder
const pullFolder = async (name: string) => {
  try {
    const response = await axios.post(`${API_BASE_URL}/api/sync/folders/${name}/pull`, { force: false })
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `${t('sync.messages.downloadFailed')}: ${response.data.data.error}`
    }
  } catch (err: any) {
    operationOutput.value = `${t('sync.messages.downloadFailed')}: ${err.response?.data?.message || err.message}`
  }
}

// Get folder status
const getFolderStatus = async (name: string) => {
  try {
    const response = await axios.get(`${API_BASE_URL}/api/sync/folders/${name}/status`)
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `${t('sync.messages.statusFailed')}: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `${t('sync.messages.statusFailed')}: ${err.response?.data?.message || err.message}`
  }
}

// Batch upload
const pushAllFolders = async () => {
  batchOperating.value = true
  try {
    const response = await axios.post(`${API_BASE_URL}/api/sync/all/push`, { force: false })
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `${t('sync.messages.batchUploadFailed')}: ${response.data.data.error}`
    }
  } catch (err: any) {
    operationOutput.value = `${t('sync.messages.batchUploadFailed')}: ${err.response?.data?.message || err.message}`
  } finally {
    batchOperating.value = false
  }
}

// Batch download
const pullAllFolders = async () => {
  batchOperating.value = true
  try {
    const response = await axios.post(`${API_BASE_URL}/api/sync/all/pull`, { force: false })
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `${t('sync.messages.batchDownloadFailed')}: ${response.data.data.error}`
    }
  } catch (err: any) {
    operationOutput.value = `${t('sync.messages.batchDownloadFailed')}: ${err.response?.data?.message || err.message}`
  } finally {
    batchOperating.value = false
  }
}

// Batch view status
const getAllFoldersStatus = async () => {
  batchOperating.value = true
  try {
    const response = await axios.get(`${API_BASE_URL}/api/sync/all/status`)
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `${t('sync.messages.statusFailed')}: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `${t('sync.messages.statusFailed')}: ${err.response?.data?.message || err.message}`
  } finally {
    batchOperating.value = false
  }
}

// 初始化
onMounted(async () => {
  loading.value = true
  try {
    await Promise.all([
      fetchSyncStatus(),
      fetchFolders()
    ])
  } catch (err: any) {
    error.value = err.response?.data?.message || err.message || t('sync.loadFailed')
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
/* 自定义样式 */
</style>
