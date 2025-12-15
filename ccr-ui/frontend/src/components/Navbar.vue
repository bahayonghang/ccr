<template>
  <nav
    class="animate-gpu-accelerated relative overflow-hidden glass-effect"
    :style="{
      borderRadius: 'var(--radius-xl)',
      margin: '0 0 var(--space-xl) 0',
      padding: 'var(--space-lg)',
      border: '1px solid var(--border-color)',
      boxShadow: 'var(--shadow-lg)'
    }"
  >
    <!-- 底部渐变线 -->
    <div
      class="absolute bottom-0 left-0 w-full h-0.5 opacity-50"
      :style="{
        background: 'linear-gradient(90deg, transparent, var(--accent-primary), var(--accent-secondary), transparent)'
      }"
      aria-hidden="true"
    />

    <div class="flex items-center justify-between">
      <!-- 品牌区域 -->
      <div
        class="flex items-center"
        :style="{ gap: 'var(--space-lg)' }"
      >
        <div class="flex flex-col">
          <div
            class="flex items-center"
            :style="{ gap: 'var(--space-sm)' }"
          >
            <Zap
              class="w-7 h-7"
              :style="{ color: 'var(--accent-primary)' }"
            />
            <h1
              class="text-display animate-gpu-accelerated"
              :style="{
                background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
                WebkitBackgroundClip: 'text',
                WebkitTextFillColor: 'transparent',
                backgroundClip: 'text',
                fontSize: 'var(--font-size-3xl)',
                fontWeight: 'var(--font-weight-bold)',
                letterSpacing: '-0.02em'
              }"
            >
              CCR
            </h1>
          </div>
          <div
            class="text-label animate-fade-in"
            :style="{
              color: 'var(--text-muted)',
              marginTop: 'var(--space-xs)'
            }"
          >
            Web Console
          </div>
        </div>

        <div
          class="hidden md:block w-px h-12"
          :style="{
            background: 'linear-gradient(180deg, transparent, var(--border-color), transparent)'
          }"
          aria-hidden="true"
        />

        <div class="hidden md:block">
          <div
            class="text-sm font-semibold"
            :style="{ color: 'var(--text-primary)' }"
          >
            Claude Code Configuration Switcher
          </div>
          <div class="flex items-center space-x-3 mt-1">
            <span
              class="flex items-center space-x-1 text-xs"
              :style="{ color: 'var(--text-secondary)' }"
            >
              <span
                class="w-1 h-1 rounded-full"
                :style="{ background: 'var(--text-muted)' }"
              />
              <span>MIT License</span>
            </span>
          </div>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div
        class="flex items-center"
        :style="{ gap: 'var(--space-sm)' }"
      >
        <ThemeToggle />

        <!-- 分隔线 -->
        <div
          class="hidden sm:block w-px h-8"
          :style="{
            background: 'linear-gradient(180deg, transparent, var(--border-color), transparent)',
            margin: '0 var(--space-xs)'
          }"
          aria-hidden="true"
        />

        <button
          v-if="onRefresh"
          class="btn-enhanced touch-optimized animate-gpu-accelerated"
          :style="{
            padding: 'var(--space-sm) var(--space-md)',
            borderRadius: 'var(--radius-lg)',
            background: 'var(--bg-tertiary)',
            color: 'var(--text-primary)',
            border: '1px solid var(--border-color)'
          }"
          :aria-label="$t('nav.refresh')"
          :title="$t('nav.refresh')"
          @click="onRefresh"
        >
          <RefreshCw class="w-4 h-4" />
          <span class="hidden md:inline">刷新</span>
        </button>

        <button
          v-if="onValidate"
          class="px-3 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-1.5 hover:scale-105"
          :style="{
            background: 'var(--bg-tertiary)',
            color: 'var(--text-primary)',
            border: '1px solid var(--border-color)'
          }"
          aria-label="验证配置"
          title="验证配置"
          @click="onValidate"
        >
          <CheckCircle class="w-4 h-4" />
          <span class="hidden md:inline">验证</span>
        </button>

        <button
          v-if="onClean"
          class="px-3 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-1.5 hover:scale-105"
          :style="{
            background: 'var(--bg-tertiary)',
            color: 'var(--accent-warning)',
            border: '1px solid var(--border-color)'
          }"
          aria-label="清理备份"
          title="清理备份"
          @click="onClean"
        >
          <Trash2 class="w-4 h-4" />
          <span class="hidden md:inline">清理</span>
        </button>

        <!-- 分隔线 -->
        <div
          v-if="onImport || onExport || onAdd"
          class="hidden sm:block w-px h-8 mx-1"
          :style="{
            background: 'linear-gradient(180deg, transparent, var(--border-color), transparent)'
          }"
          aria-hidden="true"
        />

        <button
          v-if="onImport"
          class="px-3 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-1.5 hover:scale-105"
          :style="{
            background: 'var(--bg-tertiary)',
            color: 'var(--text-primary)',
            border: '1px solid var(--border-color)'
          }"
          aria-label="导入配置"
          title="导入配置"
          @click="onImport"
        >
          <Upload class="w-4 h-4" />
          <span class="hidden md:inline">导入</span>
        </button>

        <button
          v-if="onExport"
          class="px-3 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-1.5 hover:scale-105"
          :style="{
            background: 'var(--bg-tertiary)',
            color: 'var(--text-primary)',
            border: '1px solid var(--border-color)'
          }"
          aria-label="导出配置"
          title="导出配置"
          @click="onExport"
        >
          <Download class="w-4 h-4" />
          <span class="hidden md:inline">导出</span>
        </button>

        <button
          v-if="onAdd"
          class="px-3 py-2 rounded-lg font-semibold text-sm transition-all flex items-center space-x-1.5 text-white shadow-lg hover:scale-105"
          :style="{
            background: 'linear-gradient(135deg, var(--accent-primary), var(--accent-secondary))',
            boxShadow: '0 0 20px var(--glow-primary)'
          }"
          aria-label="添加新配置"
          title="添加新配置"
          @click="onAdd"
        >
          <PlusCircle class="w-4 h-4" />
          <span class="hidden md:inline">添加</span>
        </button>
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import {
  Zap,
  RefreshCw,
  Upload,
  Download,
  PlusCircle,
  CheckCircle,
  Trash2
} from 'lucide-vue-next'
import ThemeToggle from './ThemeToggle.vue'

interface Props {
  onRefresh?: () => void
  onImport?: () => void
  onExport?: () => void
  onAdd?: () => void
  onValidate?: () => void
  onClean?: () => void
}

defineProps<Props>()
</script>
