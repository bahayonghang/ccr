<template>
  <div class="min-h-screen relative">
    <!-- 🎨 彩色背景装饰 -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none -z-10">
      <div
        class="absolute top-20 right-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{ background: 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%)' }"
      />
      <div
        class="absolute bottom-20 left-20 w-96 h-96 rounded-full opacity-15 blur-3xl animate-pulse"
        :style="{
          background: 'linear-gradient(135deg, #ec4899 0%, #f59e0b 100%)',
          animationDelay: '1s'
        }"
      />
    </div>

    <div class="relative z-10 p-6 max-w-7xl mx-auto">
      <!-- Hero 区域 -->
      <div class="mb-8">
        <div class="flex items-center justify-between mb-6">
          <div class="flex items-center gap-4">
            <div
              class="p-4 rounded-3xl glass-card"
              :style="{ background: 'rgba(99, 102, 241, 0.1)' }"
            >
              <Code2
                class="w-10 h-10"
                :style="{ color: '#6366f1' }"
              />
            </div>
            <div>
              <h1 class="text-4xl md:text-5xl font-bold mb-2 bg-gradient-to-r from-[#6366f1] via-[#8b5cf6] to-[#ec4899] bg-clip-text text-transparent">
                Claude Code
              </h1>
              <p
                class="text-lg"
                :style="{ color: 'var(--text-secondary)' }"
              >
                Anthropic 官方 AI 编码助手配置中心
              </p>
            </div>
          </div>
          <RouterLink 
            to="/" 
            class="glass-card flex items-center gap-2 px-5 py-3 hover:scale-105 transition-all duration-300"
          >
            <Home
              class="w-5 h-5"
              :style="{ color: '#64748b' }"
            />
            <span
              class="font-medium"
              :style="{ color: 'var(--text-secondary)' }"
            >返回首页</span>
          </RouterLink>
        </div>

        <!-- 特性标签 -->
        <div class="flex flex-wrap gap-3 mb-6">
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(99, 102, 241, 0.1)', color: '#6366f1' }"
          >
            ⚡ MCP 协议支持
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(139, 92, 246, 0.1)', color: '#8b5cf6' }"
          >
            🤖 AI Agents
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(236, 72, 153, 0.1)', color: '#ec4899' }"
          >
            ☁️ 云同步
          </span>
          <span
            class="px-4 py-2 rounded-full text-sm font-medium glass-card"
            :style="{ background: 'rgba(16, 185, 129, 0.1)', color: '#10b981' }"
          >
            🔧 配置管理
          </span>
        </div>
      </div>

      <!-- 功能模块网格 -->
      <div class="mb-8">
        <h2
          class="text-2xl font-bold mb-6 flex items-center gap-3"
          :style="{ color: 'var(--text-primary)' }"
        >
          <Boxes
            class="w-7 h-7"
            :style="{ color: '#6366f1' }"
          />
          功能模块
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          <RouterLink
            to="/configs"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(99, 102, 241, 0.15)' }"
                >
                  <Settings
                    class="w-7 h-7"
                    :style="{ color: '#6366f1' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    配置管理
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    Claude Code 配置切换、验证、历史记录管理
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(99, 102, 241, 0.15)',
                      color: '#6366f1'
                    }"
                  >
                    🔥 核心功能
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/sync"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(139, 92, 246, 0.15)' }"
                >
                  <Cloud
                    class="w-7 h-7"
                    :style="{ color: '#8b5cf6' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    云同步
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    WebDAV 云端配置同步和自动备份
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(139, 92, 246, 0.15)',
                      color: '#8b5cf6'
                    }"
                  >
                    ✨ 新功能
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/mcp"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(236, 72, 153, 0.15)' }"
                >
                  <Server
                    class="w-7 h-7"
                    :style="{ color: '#ec4899' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    MCP 服务器
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    Model Context Protocol 服务器配置和管理
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(236, 72, 153, 0.15)',
                      color: '#ec4899'
                    }"
                  >
                    🔌 MCP
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/slash-commands"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(245, 158, 11, 0.15)' }"
                >
                  <Terminal
                    class="w-7 h-7"
                    :style="{ color: '#f59e0b' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    Slash Commands
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    自定义命令管理和文件夹组织
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(245, 158, 11, 0.15)',
                      color: '#f59e0b'
                    }"
                  >
                    ⌨️ 命令
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/agents"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(16, 185, 129, 0.15)' }"
                >
                  <Users
                    class="w-7 h-7"
                    :style="{ color: '#10b981' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    Agents
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    AI Agent 配置、工具绑定和模型管理
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(16, 185, 129, 0.15)',
                      color: '#10b981'
                    }"
                  >
                    🤖 AI
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>

          <RouterLink
            to="/plugins"
            class="block group"
          >
            <div class="glass-card p-6 h-full hover:scale-105 transition-all duration-300">
              <div class="flex items-start gap-4">
                <div
                  class="p-3 rounded-2xl"
                  :style="{ background: 'rgba(59, 130, 246, 0.15)' }"
                >
                  <Puzzle
                    class="w-7 h-7"
                    :style="{ color: '#3b82f6' }"
                  />
                </div>
                <div class="flex-1">
                  <h3
                    class="text-xl font-bold mb-2"
                    :style="{ color: 'var(--text-primary)' }"
                  >
                    插件管理
                  </h3>
                  <p
                    class="text-sm mb-3"
                    :style="{ color: 'var(--text-secondary)' }"
                  >
                    插件启用/禁用和配置管理
                  </p>
                  <span
                    class="inline-flex items-center px-3 py-1.5 rounded-full text-xs font-bold"
                    :style="{
                      background: 'rgba(59, 130, 246, 0.15)',
                      color: '#3b82f6'
                    }"
                  >
                    🧩 插件
                  </span>
                </div>
              </div>
            </div>
          </RouterLink>
        </div>
      </div>

      <!-- 快速操作区 -->
      <div class="mb-8">
        <h2
          class="text-2xl font-bold mb-6 flex items-center gap-3"
          :style="{ color: 'var(--text-primary)' }"
        >
          <Zap
            class="w-7 h-7"
            :style="{ color: '#f59e0b' }"
          />
          快速操作
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="glass-card p-6">
            <h3
              class="text-lg font-bold mb-4 flex items-center gap-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              <Terminal
                class="w-5 h-5"
                :style="{ color: '#6366f1' }"
              />
              常用命令
            </h3>
            <div class="space-y-3">
              <div
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all cursor-pointer"
                :style="{ background: 'rgba(99, 102, 241, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >查看当前配置</span>
                <code
                  class="text-xs font-mono px-2 py-1 rounded"
                  :style="{ background: 'rgba(99, 102, 241, 0.1)', color: '#6366f1' }"
                >ccr current</code>
              </div>
              <div
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all cursor-pointer"
                :style="{ background: 'rgba(139, 92, 246, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >切换配置</span>
                <code
                  class="text-xs font-mono px-2 py-1 rounded"
                  :style="{ background: 'rgba(139, 92, 246, 0.1)', color: '#8b5cf6' }"
                >ccr switch</code>
              </div>
              <div
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all cursor-pointer"
                :style="{ background: 'rgba(236, 72, 153, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >列出所有配置</span>
                <code
                  class="text-xs font-mono px-2 py-1 rounded"
                  :style="{ background: 'rgba(236, 72, 153, 0.1)', color: '#ec4899' }"
                >ccr list</code>
              </div>
            </div>
          </div>

          <div class="glass-card p-6">
            <h3
              class="text-lg font-bold mb-4 flex items-center gap-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              <BookOpen
                class="w-5 h-5"
                :style="{ color: '#10b981' }"
              />
              文档资源
            </h3>
            <div class="space-y-3">
              <a
                href="https://docs.anthropic.com/claude/docs"
                target="_blank"
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all"
                :style="{ background: 'rgba(16, 185, 129, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >官方文档</span>
                <ExternalLink
                  class="w-4 h-4"
                  :style="{ color: '#10b981' }"
                />
              </a>
              <a
                href="https://modelcontextprotocol.io"
                target="_blank"
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all"
                :style="{ background: 'rgba(59, 130, 246, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >MCP 协议</span>
                <ExternalLink
                  class="w-4 h-4"
                  :style="{ color: '#3b82f6' }"
                />
              </a>
              <a
                href="https://github.com/anthropics/anthropic-sdk-typescript"
                target="_blank"
                class="flex items-center justify-between p-3 rounded-xl hover:scale-105 transition-all"
                :style="{ background: 'rgba(245, 158, 11, 0.05)' }"
              >
                <span
                  class="text-sm font-medium"
                  :style="{ color: 'var(--text-secondary)' }"
                >SDK 仓库</span>
                <ExternalLink
                  class="w-4 h-4"
                  :style="{ color: '#f59e0b' }"
                />
              </a>
            </div>
          </div>
        </div>
      </div>

      <!-- 提示卡片 -->
      <div class="glass-card p-6">
        <div class="flex items-start gap-4">
          <div
            class="p-3 rounded-2xl"
            :style="{ background: 'rgba(59, 130, 246, 0.15)' }"
          >
            <Info
              class="w-6 h-6"
              :style="{ color: '#3b82f6' }"
            />
          </div>
          <div class="flex-1">
            <h3
              class="text-lg font-bold mb-2"
              :style="{ color: 'var(--text-primary)' }"
            >
              💡 使用提示
            </h3>
            <ul
              class="space-y-2 text-sm"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>Claude Code 支持多配置管理，可以为不同项目创建专属配置</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>MCP 服务器可以扩展 Claude 的能力，连接外部工具和数据源</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>使用云同步功能可以在多台设备间同步配置，保持工作环境一致</span>
              </li>
              <li class="flex items-start gap-2">
                <span class="text-lg">•</span>
                <span>Agents 功能允许你创建专门的 AI 助手，绑定特定工具和上下文</span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RouterLink } from 'vue-router'
import { 
  Settings, 
  Cloud, 
  Server, 
  Terminal, 
  Users, 
  Puzzle,
  Home,
  Code2,
  Boxes,
  Zap,
  BookOpen,
  ExternalLink,
  Info
} from 'lucide-vue-next'
</script>