<template>
  <div class="p-6">
    <!-- Header with Back Button -->
    <div class="mb-8">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--text-primary)' }">Qwen</h1>
          <p class="text-base" :style="{ color: 'var(--text-secondary)' }">阿里通义千问配置管理和服务集成</p>
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
      <RouterLink to="/qwen/mcp" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(245, 158, 11, 0.1)' }">
              <Server class="w-6 h-6" :style="{ color: '#f59e0b' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                MCP 服务器
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                Qwen MCP 服务器配置管理
              </p>
            </div>
          </div>
        </Card>
      </RouterLink>

      <RouterLink to="/qwen/agents" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(16, 185, 129, 0.1)' }">
              <Bot class="w-6 h-6" :style="{ color: '#10b981' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                Agents
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                Qwen Agent 配置和管理
              </p>
            </div>
          </div>
        </Card>
      </RouterLink>

      <RouterLink to="/qwen/plugins" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(139, 92, 246, 0.1)' }">
              <Puzzle class="w-6 h-6" :style="{ color: '#8b5cf6' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                插件管理
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                Qwen 插件配置和管理
              </p>
            </div>
          </div>
        </Card>
      </RouterLink>

      <RouterLink to="/qwen/slash-commands" class="block group">
        <Card class="h-full hover:transform hover:scale-105 transition-all">
          <div class="flex items-start gap-4">
            <div class="p-3 rounded-xl" :style="{ background: 'rgba(99, 102, 241, 0.1)' }">
              <Command class="w-6 h-6" :style="{ color: '#6366f1' }" />
            </div>
            <div class="flex-1">
              <h3 class="text-lg font-bold mb-2 group-hover:text-accent-primary transition-colors" :style="{ color: 'var(--text-primary)' }">
                Slash Commands
              </h3>
              <p class="text-sm mb-3" :style="{ color: 'var(--text-secondary)' }">
                Qwen 自定义命令管理
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
import { Server, Bot, Puzzle, Command, Home } from 'lucide-vue-next'
</script>