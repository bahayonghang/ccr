<template>
  <div class="flex h-screen bg-bg-primary">
    <!-- Sidebar -->
    <div class="w-64 bg-bg-secondary border-r border-border-color flex flex-col">
      <!-- Logo and Title -->
      <div class="p-4 border-b border-border-color">
        <div class="flex items-center space-x-3">
          <div class="p-2 rounded-lg bg-bg-tertiary animate-pulse-subtle">
            <Zap class="w-6 h-6 text-accent-primary" />
          </div>
          <div>
            <h1 class="text-xl font-bold text-text-primary">
              CCR UI
            </h1>
            <p class="text-xs text-text-secondary">
              AI CLI 配置管理
            </p>
          </div>
        </div>
      </div>

      <!-- Navigation Menu -->
      <nav class="flex-1 overflow-y-auto py-4">
        <!-- 首页链接 -->
        <div class="px-3 space-y-1">
          <RouterLink 
            to="/" 
            class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
            active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
          >
            <Home class="w-5 h-5 mr-3" />
            <span class="font-medium">首页</span>
          </RouterLink>
        </div>

        <!-- 分隔线 -->
        <div class="px-3 mt-4">
          <div class="border-t border-border-color/50" />
        </div>

        <!-- 主要模块 -->
        <div class="px-3 mt-4">
          <h2 class="px-3 text-xs font-semibold text-text-muted uppercase tracking-wider mb-3 flex items-center">
            <span class="flex-1">主要模块</span>
            <span class="w-2 h-2 rounded-full bg-accent-primary/30 animate-pulse" />
          </h2>
          <div class="space-y-1">
            <RouterLink 
              to="/claude-code" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Code2 class="w-5 h-5 mr-3" />
              <span class="font-medium">Claude Code</span>
            </RouterLink>
            <RouterLink 
              to="/codex" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Settings class="w-5 h-5 mr-3" />
              <span class="font-medium">Codex</span>
            </RouterLink>
            <RouterLink 
              to="/gemini-cli" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Sparkles class="w-5 h-5 mr-3" />
              <span class="font-medium">Gemini CLI</span>
            </RouterLink>
            <RouterLink 
              to="/qwen" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Zap class="w-5 h-5 mr-3" />
              <span class="font-medium">Qwen</span>
            </RouterLink>
            <RouterLink 
              to="/iflow" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Activity class="w-5 h-5 mr-3" />
              <span class="font-medium">IFLOW</span>
            </RouterLink>
          </div>
        </div>

        <!-- 分隔线 -->
        <div class="px-3 mt-4">
          <div class="border-t border-border-color/50" />
        </div>

        <!-- 工具中心 -->
        <div class="px-3 mt-4">
          <h2 class="px-3 text-xs font-semibold text-text-muted uppercase tracking-wider mb-3 flex items-center">
            <span class="flex-1">工具中心</span>
            <span class="w-2 h-2 rounded-full bg-accent-warning/30 animate-pulse" />
          </h2>
          <div class="space-y-1">
            <RouterLink 
              to="/commands" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Terminal class="w-5 h-5 mr-3" />
              <span class="font-medium">命令执行</span>
            </RouterLink>
            <RouterLink 
              to="/converter" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <TrendingUp class="w-5 h-5 mr-3" />
              <span class="font-medium">配置转换</span>
            </RouterLink>
            <RouterLink 
              to="/sync" 
              class="flex items-center px-3 py-3 rounded-xl text-text-secondary hover:bg-bg-tertiary transition-all duration-300 transform hover:translate-x-1"
              active-class="bg-gradient-to-r from-accent-primary/10 to-transparent text-accent-primary border-l-4 border-accent-primary font-bold shadow-sm"
            >
              <Cloud class="w-5 h-5 mr-3" />
              <span class="font-medium">云同步</span>
            </RouterLink>
          </div>
        </div>
      </nav>

      <!-- Version Info -->
      <div class="p-4 border-t border-border-color">
        <div class="text-xs text-text-muted flex items-center justify-between">
          <span>CCR UI v3.10.2</span>
          <span class="animate-pulse-subtle">●</span>
        </div>
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex flex-col overflow-auto">
      <RouterView />
    </div>
  </div>
</template>

<script setup lang="ts">
import { 
  Home, 
  Code2, 
  Settings, 
  Sparkles, 
  Zap, 
  Activity, 
  Terminal, 
  TrendingUp, 
  Cloud,
} from 'lucide-vue-next'
</script>