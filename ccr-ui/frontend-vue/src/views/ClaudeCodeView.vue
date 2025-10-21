<template>
  <div class="p-6">
    <!-- Header with Back Button -->
    <div class="mb-8">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--text-primary)' }">Claude Code</h1>
          <p class="text-base" :style="{ color: 'var(--text-secondary)' }">配置管理、云同步、MCP 服务器、Agents、插件</p>
        </div>
        <RouterLink 
          to="/" 
          class="flex items-center px-4 py-2 rounded-lg text-sm font-medium transition-all hover:scale-105"
          :style="{
            background: 'var(--bg-tertiary)',
            color: 'var(--text-primary)',
            border: '1px solid var(--border-color)'
          }"
        >
          <Home class="w-4 h-4 mr-2" />
          返回首页
        </RouterLink>
      </div>
    </div>

    <!-- Module Cards Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <RouterLink to="/configs" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(99, 102, 241, 0.1)' }">
              <Settings class="w-6 h-6" :style="{ color: '#6366f1' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                配置管理
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                Claude Code 配置切换、验证、历史记录管理
              </p>
              <span class="inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold" :style="{
                background: 'var(--bg-secondary)',
                color: 'var(--text-muted)',
                border: '1px solid var(--border-color)'
              }">
                核心功能
              </span>
            </div>
          </div>
        </Card>
      </RouterLink>

      <RouterLink to="/sync" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(139, 92, 246, 0.1)' }">
              <Cloud class="w-6 h-6" :style="{ color: '#8b5cf6' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                云同步
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                WebDAV 云端配置同步和自动备份
              </p>
              <span class="inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold" :style="{
                background: 'var(--bg-secondary)',
                color: 'var(--text-muted)',
                border: '1px solid var(--border-color)'
              }">
                新功能
              </span>
            </div>
          </div>
        </Card>
      </RouterLink>

      <RouterLink to="/mcp" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(236, 72, 153, 0.1)' }">
              <Server class="w-6 h-6" :style="{ color: '#ec4899' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                MCP 服务器
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                Model Context Protocol 服务器配置和管理
              </p>
            </div>
          </div>
        </Card>
      </RouterLink>

      <RouterLink to="/slash-commands" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(245, 158, 11, 0.1)' }">
              <Terminal class="w-6 h-6" :style="{ color: '#f59e0b' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                Slash Commands
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                自定义命令管理和文件夹组织
              </p>
            </div>
          </div>
        </Card>
      </RouterLink>

      <RouterLink to="/agents" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(16, 185, 129, 0.1)' }">
              <Users class="w-6 h-6" :style="{ color: '#10b981' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                Agents
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                AI Agent 配置、工具绑定和模型管理
              </p>
            </div>
          </div>
        </Card>
      </RouterLink>

      <RouterLink to="/plugins" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(99, 102, 241, 0.1)' }">
              <Puzzle class="w-6 h-6" :style="{ color: '#6366f1' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                插件管理
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                插件启用/禁用和配置管理
              </p>
            </div>
          </div>
        </Card>
      </RouterLink>
    </div>
  </div>
</template>

<script setup lang="ts">
import Card from '@/components/Card.vue'
import { RouterLink } from 'vue-router'
import { 
  Settings, 
  Cloud, 
  Server, 
  Terminal, 
  Users, 
  Puzzle,
  Home
} from 'lucide-vue-next'
</script>