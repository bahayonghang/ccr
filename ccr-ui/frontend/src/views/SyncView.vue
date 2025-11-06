<template>
  <div class="min-h-screen relative overflow-y-auto bg-gray-50 dark:bg-gray-900">
    <!-- 动态背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div class="absolute -top-40 -right-40 w-80 h-80 bg-purple-500/20 dark:bg-purple-500/20 rounded-full blur-3xl animate-pulse"></div>
      <div class="absolute top-1/2 -left-40 w-96 h-96 bg-indigo-500/20 dark:bg-indigo-500/20 rounded-full blur-3xl animate-pulse" style="animation-delay: 1s;"></div>
      <div class="absolute bottom-20 right-1/3 w-72 h-72 bg-violet-500/20 dark:bg-violet-500/20 rounded-full blur-3xl animate-pulse" style="animation-delay: 2s;"></div>
    </div>

    <main class="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-8 pb-8">
      <!-- Breadcrumb Navigation -->
      <Breadcrumb
        :items="[
          { label: '首页', path: '/', icon: Home },
          { label: 'Claude Code', path: '/claude-code', icon: Code2 },
          { label: '云同步', path: '/sync', icon: Cloud }
        ]"
        moduleColor="#6366f1"
      />

      <div class="mb-8">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-4">
            <div class="p-3 rounded-2xl backdrop-blur-xl bg-white/20 border border-white/30 shadow-xl">
              <Cloud class="w-8 h-8 text-white drop-shadow-lg" />
            </div>
            <div>
              <h1 class="text-4xl font-bold text-white drop-shadow-lg">WebDAV 云同步</h1>
              <p class="text-white/80 mt-1 drop-shadow-md">预设平台选择 · 一键同步 · 智能管理</p>
            </div>
          </div>
          <RouterLink
            to="/"
            class="group flex items-center gap-2 px-5 py-2.5 rounded-xl backdrop-blur-md bg-white/20 border border-white/30 transition-all duration-300 hover:bg-white/30 hover:scale-105 shadow-lg"
          >
            <Home class="w-4 h-4 text-white" />
            <span class="font-medium text-white">返回首页</span>
          </RouterLink>
        </div>
      </div>

      <!-- 加载状态 -->
      <div v-if="loading" class="flex items-center justify-center py-16">
        <div class="p-6 rounded-2xl backdrop-blur-xl bg-white/20 border border-white/30 shadow-2xl">
          <RefreshCw class="w-10 h-10 animate-spin text-white drop-shadow-lg" />
        </div>
      </div>

      <!-- 错误状态 -->
      <div
        v-else-if="error"
        class="rounded-2xl backdrop-blur-xl bg-red-500/20 border border-red-400/30 p-6 flex items-start gap-4 shadow-xl"
      >
        <XCircle class="w-7 h-7 flex-shrink-0 mt-0.5 text-red-200 drop-shadow-md" />
        <div>
          <h3 class="font-bold text-xl mb-2 text-white drop-shadow-md">加载失败</h3>
          <p class="text-base text-white/90 drop-shadow-md">{{ error }}</p>
        </div>
      </div>

      <!-- 主要内容 -->
      <div v-else class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- 左侧主内容区 (2 columns) -->
        <div class="lg:col-span-2 space-y-6">
          <!-- 预设同步项目选择 -->
          <div
            class="rounded-2xl backdrop-blur-xl bg-white/15 border border-white/30 shadow-2xl overflow-hidden transition-all duration-300 hover:bg-white/20"
          >
            <!-- 头部 -->
            <div class="px-6 py-5 bg-gradient-to-r from-white/25 to-white/15 border-b border-white/30 flex items-center justify-between">
              <h2 class="text-2xl font-bold text-white flex items-center gap-3 drop-shadow-[0_2px_8px_rgba(0,0,0,0.8)]">
                <div class="p-2 rounded-xl bg-white/30">
                  <CheckSquare class="w-6 h-6" />
                </div>
                选择同步平台
              </h2>
              <button
                @click="applySelection"
                :disabled="applying || !hasChanges"
                class="flex items-center gap-2 px-4 py-2 rounded-xl backdrop-blur-md bg-emerald-500/40 border border-emerald-400/30 text-white font-medium transition-all duration-300 hover:bg-emerald-500/50 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Save class="w-4 h-4" />
                <span>{{ applying ? '应用中...' : '应用选择' }}</span>
              </button>
            </div>

            <div class="p-6">
              <!-- Config (必选项) -->
              <div class="mb-6 p-5 rounded-xl bg-amber-500/20 border-2 border-amber-400/50">
                <div class="flex items-center gap-4">
                  <CheckCircle class="w-6 h-6 text-amber-300 flex-shrink-0" />
                  <div class="flex-1">
                    <div class="flex items-center gap-3 mb-2">
                      <h3 class="text-lg font-bold text-white">Platforms 平台配置</h3>
                      <span class="px-2 py-1 rounded-lg text-xs font-bold bg-amber-500/40 border border-amber-400/30 text-amber-100">
                        必选
                      </span>
                    </div>
                    <p class="text-white/70 text-sm mb-3">CCR 供应商配置（API地址、密钥等），强制同步保证配置一致性</p>
                    <div class="flex items-center gap-2">
                      <Folder class="w-4 h-4 text-white/60" />
                      <input
                        v-model="presetItems.config.localPath"
                        type="text"
                        class="flex-1 px-3 py-2 rounded-lg backdrop-blur-md bg-white/20 border border-white/30 text-white text-sm placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-amber-400/50"
                        placeholder="本地路径"
                      />
                    </div>
                  </div>
                </div>
              </div>

              <!-- 可选平台列表 -->
              <div class="space-y-4">
                <div
                  v-for="item in optionalItems"
                  :key="item.key"
                  class="p-5 rounded-xl backdrop-blur-md transition-all duration-300"
                  :class="item.selected ? 'bg-blue-500/20 border border-blue-400/30' : 'bg-white/10 border border-white/20'"
                >
                  <div class="flex items-start gap-4">
                    <button
                      @click="toggleItem(item.key)"
                      class="mt-1 flex-shrink-0"
                    >
                      <div
                        class="w-6 h-6 rounded-lg border-2 flex items-center justify-center transition-all duration-300"
                        :class="item.selected
                          ? 'bg-blue-500/40 border-blue-400/50'
                          : 'bg-white/10 border-white/30 hover:border-white/50'"
                      >
                        <Check v-if="item.selected" class="w-4 h-4 text-white" />
                      </div>
                    </button>
                    <div class="flex-1">
                      <div class="flex items-center gap-3 mb-2">
                        <component :is="item.icon" class="w-5 h-5 text-white/80" />
                        <h3 class="text-lg font-bold text-white">{{ item.name }}</h3>
                      </div>
                      <p class="text-white/70 text-sm mb-3">{{ item.description }}</p>
                      <div v-if="item.selected" class="space-y-2">
                        <div class="flex items-center gap-2">
                          <Folder class="w-4 h-4 text-white/60" />
                          <input
                            v-model="item.localPath"
                            type="text"
                            class="flex-1 px-3 py-2 rounded-lg backdrop-blur-md bg-white/20 border border-white/30 text-white text-sm placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-blue-400/50"
                            placeholder="本地路径"
                          />
                        </div>
                        <div class="flex items-center gap-2">
                          <Cloud class="w-4 h-4 text-white/60" />
                          <input
                            v-model="item.remotePath"
                            type="text"
                            class="flex-1 px-3 py-2 rounded-lg backdrop-blur-md bg-white/20 border border-white/30 text-white text-sm placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-blue-400/50"
                            placeholder="远程路径 (可选)"
                          />
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <!-- 自定义文件夹 -->
              <div class="mt-6 p-5 rounded-xl backdrop-blur-md bg-purple-500/20 border border-purple-400/30">
                <h3 class="text-lg font-bold text-white mb-4 flex items-center gap-2">
                  <Plus class="w-5 h-5" />
                  自定义文件夹
                </h3>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                  <input
                    v-model="customFolder.name"
                    type="text"
                    placeholder="文件夹名称"
                    class="px-4 py-2 rounded-lg backdrop-blur-md bg-white/20 border border-white/30 text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-purple-400/50"
                  />
                  <input
                    v-model="customFolder.localPath"
                    type="text"
                    placeholder="本地路径"
                    class="px-4 py-2 rounded-lg backdrop-blur-md bg-white/20 border border-white/30 text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-purple-400/50"
                  />
                  <input
                    v-model="customFolder.remotePath"
                    type="text"
                    placeholder="远程路径 (可选)"
                    class="px-4 py-2 rounded-lg backdrop-blur-md bg-white/20 border border-white/30 text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-purple-400/50"
                  />
                  <input
                    v-model="customFolder.description"
                    type="text"
                    placeholder="描述 (可选)"
                    class="px-4 py-2 rounded-lg backdrop-blur-md bg-white/20 border border-white/30 text-white placeholder-white/60 focus:outline-none focus:ring-2 focus:ring-purple-400/50"
                  />
                </div>
                <button
                  @click="addCustomFolder"
                  :disabled="!customFolder.name || !customFolder.localPath || addingCustom"
                  class="w-full px-4 py-2 rounded-lg backdrop-blur-md bg-purple-500/40 border border-purple-400/30 text-white font-medium transition-all duration-300 hover:bg-purple-500/50 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                  <Plus class="w-5 h-5" />
                  {{ addingCustom ? '添加中...' : '添加自定义文件夹' }}
                </button>
              </div>
            </div>
          </div>

          <!-- 已启用的文件夹列表 -->
          <div
            class="rounded-2xl backdrop-blur-xl bg-white/15 border border-white/30 shadow-2xl overflow-hidden transition-all duration-300 hover:bg-white/20"
          >
            <div class="px-6 py-5 bg-gradient-to-r from-white/25 to-white/15 border-b border-white/30 flex items-center justify-between">
              <h2 class="text-2xl font-bold text-white flex items-center gap-3 drop-shadow-[0_2px_8px_rgba(0,0,0,0.8)]">
                <div class="p-2 rounded-xl bg-white/30">
                  <Folders class="w-6 h-6" />
                </div>
                已启用的文件夹
              </h2>
              <button
                @click="refreshFolders"
                class="flex items-center gap-2 px-4 py-2 rounded-xl backdrop-blur-md bg-white/20 border border-white/30 transition-all duration-300 hover:bg-white/30 hover:scale-105"
              >
                <RefreshCw class="w-4 h-4 text-white" :class="{ 'animate-spin': refreshingFolders }" />
                <span class="text-white font-medium">刷新</span>
              </button>
            </div>

            <div class="p-6">
              <div v-if="enabledFolders.length === 0" class="text-center py-12">
                <FolderOpen class="w-16 h-16 text-white/40 mx-auto mb-4" />
                <p class="text-white/60 text-lg">暂无启用的同步文件夹</p>
                <p class="text-white/40 text-sm mt-2">请在上方选择要同步的平台</p>
              </div>

              <div v-else class="space-y-4">
                <div
                  v-for="folder in enabledFolders"
                  :key="folder.name"
                  class="p-5 rounded-xl backdrop-blur-md bg-white/15 border border-white/30 transition-all duration-300 hover:bg-white/20"
                >
                  <div class="flex items-start justify-between mb-4">
                    <div class="flex-1">
                      <div class="flex items-center gap-3 mb-2">
                        <h4 class="text-xl font-bold text-white">{{ folder.name }}</h4>
                        <span
                          :class="[
                            'px-3 py-1 rounded-lg text-sm font-medium',
                            folder.enabled
                              ? 'bg-emerald-500/40 border border-emerald-400/30 text-emerald-100'
                              : 'bg-gray-500/40 border border-gray-400/30 text-gray-200'
                          ]"
                        >
                          {{ folder.enabled ? '✓ 已启用' : '✗ 已禁用' }}
                        </span>
                      </div>
                      <p v-if="folder.description" class="text-white/70 text-sm mb-2">{{ folder.description }}</p>
                      <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-sm">
                        <div class="flex items-center gap-2 text-white/80">
                          <Folder class="w-4 h-4" />
                          <span class="font-mono">{{ folder.localPath }}</span>
                        </div>
                        <div class="flex items-center gap-2 text-white/80">
                          <Cloud class="w-4 h-4" />
                          <span class="font-mono">{{ folder.remotePath }}</span>
                        </div>
                      </div>
                    </div>
                  </div>

                  <!-- 操作按钮 -->
                  <div class="flex flex-wrap gap-2">
                    <button
                      @click="toggleFolder(folder.name, folder.enabled)"
                      class="px-4 py-2 rounded-lg backdrop-blur-md bg-blue-500/40 border border-blue-400/30 text-white font-medium transition-all duration-300 hover:bg-blue-500/50 flex items-center gap-2"
                    >
                      <ToggleLeft class="w-4 h-4" />
                      {{ folder.enabled ? '禁用' : '启用' }}
                    </button>
                    <button
                      @click="pushFolder(folder.name)"
                      :disabled="!folder.enabled"
                      class="px-4 py-2 rounded-lg backdrop-blur-md bg-emerald-500/40 border border-emerald-400/30 text-white font-medium transition-all duration-300 hover:bg-emerald-500/50 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                    >
                      <Upload class="w-4 h-4" />
                      上传
                    </button>
                    <button
                      @click="pullFolder(folder.name)"
                      :disabled="!folder.enabled"
                      class="px-4 py-2 rounded-lg backdrop-blur-md bg-purple-500/40 border border-purple-400/30 text-white font-medium transition-all duration-300 hover:bg-purple-500/50 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                    >
                      <Download class="w-4 h-4" />
                      下载
                    </button>
                    <button
                      @click="getFolderStatus(folder.name)"
                      class="px-4 py-2 rounded-lg backdrop-blur-md bg-amber-500/40 border border-amber-400/30 text-white font-medium transition-all duration-300 hover:bg-amber-500/50 flex items-center gap-2"
                    >
                      <Info class="w-4 h-4" />
                      状态
                    </button>
                    <button
                      @click="removeFolder(folder.name)"
                      class="px-4 py-2 rounded-lg backdrop-blur-md bg-red-500/40 border border-red-400/30 text-white font-medium transition-all duration-300 hover:bg-red-500/50 flex items-center gap-2"
                    >
                      <Trash2 class="w-4 h-4" />
                      删除
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 批量操作卡片 -->
          <div
            class="rounded-2xl backdrop-blur-xl bg-white/15 border border-white/30 shadow-2xl overflow-hidden transition-all duration-300 hover:bg-white/20"
          >
            <div class="px-6 py-5 bg-gradient-to-r from-white/25 to-white/15 border-b border-white/30">
              <h2 class="text-2xl font-bold text-white flex items-center gap-3 drop-shadow-[0_2px_8px_rgba(0,0,0,0.8)]">
                <div class="p-2 rounded-xl bg-white/30">
                  <Layers class="w-6 h-6" />
                </div>
                批量操作
              </h2>
            </div>

            <div class="p-6">
              <p class="text-white/80 mb-4">对所有启用的文件夹执行批量同步操作</p>
              <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <button
                  @click="pushAllFolders"
                  :disabled="batchOperating || enabledFolders.length === 0"
                  class="px-6 py-4 rounded-xl backdrop-blur-md bg-emerald-500/40 border border-emerald-400/30 text-white font-bold transition-all duration-300 hover:bg-emerald-500/50 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                >
                  <Upload class="w-5 h-5" />
                  全部上传
                </button>
                <button
                  @click="pullAllFolders"
                  :disabled="batchOperating || enabledFolders.length === 0"
                  class="px-6 py-4 rounded-xl backdrop-blur-md bg-purple-500/40 border border-purple-400/30 text-white font-bold transition-all duration-300 hover:bg-purple-500/50 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                >
                  <Download class="w-5 h-5" />
                  全部下载
                </button>
                <button
                  @click="getAllFoldersStatus"
                  :disabled="batchOperating || enabledFolders.length === 0"
                  class="px-6 py-4 rounded-xl backdrop-blur-md bg-amber-500/40 border border-amber-400/30 text-white font-bold transition-all duration-300 hover:bg-amber-500/50 hover:scale-105 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                >
                  <Info class="w-5 h-5" />
                  查看状态
                </button>
              </div>
            </div>
          </div>

          <!-- 操作输出卡片 -->
          <div
            v-if="operationOutput"
            class="rounded-2xl backdrop-blur-xl bg-white/15 border border-white/30 shadow-2xl overflow-hidden"
          >
            <div class="px-6 py-5 bg-gradient-to-r from-white/25 to-white/15 border-b border-white/30 flex items-center justify-between">
              <h2 class="text-xl font-bold text-white flex items-center gap-3">
                <Terminal class="w-5 h-5" />
                操作输出
              </h2>
              <button
                @click="operationOutput = ''"
                class="p-2 rounded-lg backdrop-blur-md bg-white/20 border border-white/30 transition-all duration-300 hover:bg-white/30"
              >
                <XCircle class="w-4 h-4 text-white" />
              </button>
            </div>
            <div class="p-6">
              <pre class="text-sm text-white/90 font-mono whitespace-pre-wrap overflow-x-auto bg-black/30 p-4 rounded-lg">{{ operationOutput }}</pre>
            </div>
          </div>
        </div>

        <!-- 右侧信息区 (1 column) -->
        <div class="space-y-6">
          <!-- WebDAV 配置状态 -->
          <div
            class="rounded-2xl backdrop-blur-xl bg-white/15 border border-white/30 shadow-2xl overflow-hidden"
          >
            <div class="px-6 py-5 bg-gradient-to-r from-white/25 to-white/15 border-b border-white/30">
              <h2 class="text-xl font-bold text-white flex items-center gap-3 drop-shadow-[0_2px_8px_rgba(0,0,0,0.8)]">
                <Settings class="w-5 h-5" />
                WebDAV 配置
              </h2>
            </div>

            <div class="p-6">
              <div v-if="syncStatus?.configured && syncStatus.config" class="space-y-4">
                <div class="flex items-center gap-2 px-4 py-3 rounded-lg bg-emerald-500/30 border border-emerald-400/30">
                  <CheckCircle class="w-5 h-5 text-emerald-100" />
                  <span class="text-emerald-50 font-medium">已配置</span>
                </div>

                <div class="space-y-3">
                  <div>
                    <div class="text-xs text-white/60 mb-1">服务器</div>
                    <div class="text-sm text-white/90 font-mono break-all">{{ syncStatus.config.webdav_url }}</div>
                  </div>
                  <div>
                    <div class="text-xs text-white/60 mb-1">用户</div>
                    <div class="text-sm text-white/90 font-mono">{{ syncStatus.config.username }}</div>
                  </div>
                  <div>
                    <div class="text-xs text-white/60 mb-1">远程路径</div>
                    <div class="text-sm text-white/90 font-mono break-all">{{ syncStatus.config.remote_path }}</div>
                  </div>
                </div>
              </div>

              <div v-else class="space-y-4">
                <div class="flex items-center gap-2 px-4 py-3 rounded-lg bg-amber-500/30 border border-amber-400/30">
                  <AlertCircle class="w-5 h-5 text-amber-100" />
                  <span class="text-amber-50 font-medium">未配置</span>
                </div>
                <p class="text-sm text-white/70">请使用 CLI 配置 WebDAV:</p>
                <code class="block text-sm text-white/90 font-mono bg-black/30 p-3 rounded-lg">ccr sync config</code>
              </div>
            </div>
          </div>

          <!-- 功能说明 -->
          <div
            class="rounded-2xl backdrop-blur-xl bg-white/15 border border-white/30 shadow-2xl overflow-hidden"
          >
            <div class="px-6 py-5 bg-gradient-to-r from-white/25 to-white/15 border-b border-white/30">
              <h2 class="text-xl font-bold text-white flex items-center gap-3">
                <BookOpen class="w-5 h-5" />
                功能说明
              </h2>
            </div>

            <div class="p-6 space-y-4 text-sm text-white/80">
              <div>
                <h4 class="font-bold text-white mb-2">✅ 预设平台选择</h4>
                <p>Config 必选，Claude/Gemini/Qwen 可选，一键配置常用平台</p>
              </div>
              <div>
                <h4 class="font-bold text-white mb-2">🔄 独立文件夹管理</h4>
                <p>每个文件夹独立同步，可单独启用/禁用和操作</p>
              </div>
              <div>
                <h4 class="font-bold text-white mb-2">💾 智能过滤</h4>
                <p>自动排除 backups/、.locks/、*.tmp、*.bak 等文件</p>
              </div>
              <div>
                <h4 class="font-bold text-white mb-2">⚡ 批量操作</h4>
                <p>一键上传/下载所有启用的文件夹，提高效率</p>
              </div>
            </div>
          </div>

          <!-- 支持的服务 -->
          <div
            class="rounded-2xl backdrop-blur-xl bg-white/15 border border-white/30 shadow-2xl overflow-hidden"
          >
            <div class="px-6 py-5 bg-gradient-to-r from-white/25 to-white/15 border-b border-white/30">
              <h2 class="text-xl font-bold text-white flex items-center gap-3">
                <Server class="w-5 h-5" />
                支持的服务
              </h2>
            </div>

            <div class="p-6 space-y-3 text-sm text-white/80">
              <div class="flex items-center gap-2">
                <CheckCircle class="w-4 h-4 text-emerald-300" />
                <span>坚果云 (Nutstore)</span>
              </div>
              <div class="flex items-center gap-2">
                <CheckCircle class="w-4 h-4 text-emerald-300" />
                <span>Nextcloud</span>
              </div>
              <div class="flex items-center gap-2">
                <CheckCircle class="w-4 h-4 text-emerald-300" />
                <span>ownCloud</span>
              </div>
              <div class="flex items-center gap-2">
                <CheckCircle class="w-4 h-4 text-emerald-300" />
                <span>任何标准 WebDAV 服务器</span>
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
import axios from 'axios'
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
      operationOutput.value = `✓ 成功添加自定义文件夹: ${customFolder.value.name}`
      customFolder.value = { name: '', localPath: '', remotePath: '', description: '' }
      await refreshFolders()
    } else {
      operationOutput.value = `✗ 添加失败: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `✗ 添加失败: ${err.response?.data?.message || err.message}`
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
const parseFoldersList = (output: string) => {
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

// 删除文件夹
const removeFolder = async (name: string) => {
  if (!confirm(`确定要删除文件夹 "${name}" 吗？\n\n注意：这只会移除同步配置，不会删除本地文件。`)) {
    return
  }

  try {
    const response = await axios.delete(`${API_BASE_URL}/api/sync/folders/${name}`)
    if (response.data.success) {
      operationOutput.value = `✓ 成功删除文件夹: ${name}`
      await refreshFolders()
    } else {
      operationOutput.value = `✗ 删除失败: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `✗ 删除失败: ${err.response?.data?.message || err.message}`
  }
}

// 切换文件夹状态
const toggleFolder = async (name: string, currentEnabled: boolean) => {
  const action = currentEnabled ? 'disable' : 'enable'
  try {
    const response = await axios.put(`${API_BASE_URL}/api/sync/folders/${name}/${action}`)
    if (response.data.success) {
      operationOutput.value = `✓ 成功${currentEnabled ? '禁用' : '启用'}文件夹: ${name}`
      await refreshFolders()
    } else {
      operationOutput.value = `✗ 操作失败: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `✗ 操作失败: ${err.response?.data?.message || err.message}`
  }
}

// 上传文件夹
const pushFolder = async (name: string) => {
  try {
    const response = await axios.post(`${API_BASE_URL}/api/sync/folders/${name}/push`, { force: false })
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `✗ 上传失败: ${response.data.data.error}`
    }
  } catch (err: any) {
    operationOutput.value = `✗ 上传失败: ${err.response?.data?.message || err.message}`
  }
}

// 下载文件夹
const pullFolder = async (name: string) => {
  try {
    const response = await axios.post(`${API_BASE_URL}/api/sync/folders/${name}/pull`, { force: false })
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `✗ 下载失败: ${response.data.data.error}`
    }
  } catch (err: any) {
    operationOutput.value = `✗ 下载失败: ${err.response?.data?.message || err.message}`
  }
}

// 获取文件夹状态
const getFolderStatus = async (name: string) => {
  try {
    const response = await axios.get(`${API_BASE_URL}/api/sync/folders/${name}/status`)
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `✗ 获取状态失败: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `✗ 获取状态失败: ${err.response?.data?.message || err.message}`
  }
}

// 批量上传
const pushAllFolders = async () => {
  batchOperating.value = true
  try {
    const response = await axios.post(`${API_BASE_URL}/api/sync/all/push`, { force: false })
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `✗ 批量上传失败: ${response.data.data.error}`
    }
  } catch (err: any) {
    operationOutput.value = `✗ 批量上传失败: ${err.response?.data?.message || err.message}`
  } finally {
    batchOperating.value = false
  }
}

// 批量下载
const pullAllFolders = async () => {
  batchOperating.value = true
  try {
    const response = await axios.post(`${API_BASE_URL}/api/sync/all/pull`, { force: false })
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `✗ 批量下载失败: ${response.data.data.error}`
    }
  } catch (err: any) {
    operationOutput.value = `✗ 批量下载失败: ${err.response?.data?.message || err.message}`
  } finally {
    batchOperating.value = false
  }
}

// 批量查看状态
const getAllFoldersStatus = async () => {
  batchOperating.value = true
  try {
    const response = await axios.get(`${API_BASE_URL}/api/sync/all/status`)
    if (response.data.success) {
      operationOutput.value = response.data.data.output
    } else {
      operationOutput.value = `✗ 获取状态失败: ${response.data.message}`
    }
  } catch (err: any) {
    operationOutput.value = `✗ 获取状态失败: ${err.response?.data?.message || err.message}`
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
    error.value = err.response?.data?.message || err.message || '加载失败'
  } finally {
    loading.value = false
  }
})
</script>

<style scoped>
/* 自定义样式 */
</style>
