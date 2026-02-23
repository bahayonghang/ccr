<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <AnimatedBackground variant="minimal" />

    <div class="max-w-3xl mx-auto space-y-5">
      <!-- 页面标题 -->
      <div class="flex items-center justify-between animate-slide-up">
        <div class="flex items-center gap-3">
          <RouterLink
            to="/opencode"
            class="p-2 rounded-lg text-text-muted hover:text-text-primary transition-colors"
          >
            <ChevronLeft class="w-5 h-5" />
          </RouterLink>
          <div>
            <h1 class="text-2xl font-bold text-text-primary">
              插件管理
            </h1>
            <p class="text-text-muted text-sm">
              管理 OpenCode npm 插件包
            </p>
          </div>
        </div>
        <button
          class="flex items-center gap-2 px-4 py-2 rounded-lg font-medium text-sm transition-all hover:scale-105"
          style="background: var(--accent-primary); color: white;"
          @click="showAddDialog = true"
        >
          <Plus class="w-4 h-4" />
          添加插件
        </button>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading"
        class="flex items-center justify-center py-16"
      >
        <div class="w-8 h-8 border-2 border-emerald-500 border-t-transparent rounded-full animate-spin" />
      </div>

      <!-- 错误状态 -->
      <Card
        v-else-if="error"
        variant="elevated"
        class="p-6 text-center"
      >
        <p class="text-red-400 mb-3">
          {{ error }}
        </p>
        <button
          class="text-sm text-accent-primary hover:underline"
          @click="loadPlugins"
        >
          重新加载
        </button>
      </Card>

      <!-- 空状态 -->
      <Card
        v-else-if="plugins.length === 0"
        variant="glass"
        class="p-10 text-center"
      >
        <Package class="w-12 h-12 text-text-muted mx-auto mb-4" />
        <h3 class="text-lg font-bold text-text-primary mb-2">
          暂无插件
        </h3>
        <p class="text-text-muted text-sm mb-4">
          添加 npm 插件包来扩展 OpenCode 功能
        </p>
        <button
          class="px-4 py-2 rounded-lg font-medium text-sm transition-all hover:scale-105"
          style="background: var(--accent-primary); color: white;"
          @click="showAddDialog = true"
        >
          添加第一个插件
        </button>
      </Card>

      <!-- 插件列表 -->
      <div
        v-else
        class="space-y-2"
      >
        <Card
          v-for="plugin in plugins"
          :key="plugin.npm"
          variant="elevated"
          class="p-4 animate-slide-up"
        >
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-3 min-w-0">
              <div class="w-9 h-9 rounded-lg bg-emerald-500/10 flex items-center justify-center shrink-0">
                <Package class="w-4 h-4 text-emerald-500" />
              </div>
              <div class="min-w-0">
                <p class="font-mono font-medium text-text-primary truncate text-sm">
                  {{ plugin.npm }}
                </p>
                <p class="text-xs text-text-muted">
                  npm 包
                </p>
              </div>
            </div>

            <button
              class="p-2 rounded-lg text-text-muted hover:text-red-400 hover:bg-red-500/10 transition-colors shrink-0"
              title="删除"
              @click="confirmDelete(plugin.npm)"
            >
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        </Card>
      </div>
    </div>

    <!-- 添加插件弹窗 -->
    <div
      v-if="showAddDialog"
      class="fixed inset-0 flex items-center justify-center z-50 p-4"
      style="background: rgba(0,0,0,0.5); backdrop-filter: blur(4px);"
      @click.self="showAddDialog = false"
    >
      <Card
        variant="glass"
        class="w-full max-w-md p-6 space-y-4"
      >
        <div class="flex items-center justify-between">
          <h2 class="text-lg font-bold text-text-primary">
            添加插件
          </h2>
          <button
            class="p-1 rounded text-text-muted hover:text-text-primary"
            @click="showAddDialog = false"
          >
            <X class="w-5 h-5" />
          </button>
        </div>

        <div>
          <label class="block text-xs font-bold text-text-muted uppercase tracking-wider mb-1">npm 包名 *</label>
          <input
            v-model="newNpm"
            type="text"
            placeholder="例：@opencode-ai/omo"
            class="w-full px-3 py-2 rounded-lg text-sm bg-bg-elevated border border-border-default text-text-primary placeholder:text-text-muted focus:outline-none focus:border-emerald-500"
            @keyup.enter="doAdd"
          />
        </div>

        <div class="flex justify-end gap-3">
          <button
            class="px-4 py-2 rounded-lg text-sm text-text-muted hover:text-text-primary"
            @click="showAddDialog = false"
          >
            取消
          </button>
          <button
            :disabled="!newNpm.trim() || saving"
            class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all hover:scale-105 disabled:opacity-50 disabled:hover:scale-100"
            style="background: var(--accent-primary); color: white;"
            @click="doAdd"
          >
            <Loader2
              v-if="saving"
              class="w-4 h-4 animate-spin"
            />
            添加
          </button>
        </div>
      </Card>
    </div>

    <!-- 删除确认弹窗 -->
    <div
      v-if="deletingNpm"
      class="fixed inset-0 flex items-center justify-center z-50 p-4"
      style="background: rgba(0,0,0,0.5); backdrop-filter: blur(4px);"
      @click.self="deletingNpm = ''"
    >
      <Card
        variant="glass"
        class="w-full max-w-sm p-6 space-y-4"
      >
        <h2 class="text-lg font-bold text-text-primary">
          确认删除
        </h2>
        <p class="text-text-secondary text-sm">
          确定要删除插件 <strong class="font-mono">{{ deletingNpm }}</strong> 吗？
        </p>
        <div class="flex justify-end gap-3">
          <button
            class="px-4 py-2 rounded-lg text-sm text-text-muted hover:text-text-primary"
            @click="deletingNpm = ''"
          >
            取消
          </button>
          <button
            :disabled="saving"
            class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-red-500 text-white hover:bg-red-600 disabled:opacity-50"
            @click="doDelete"
          >
            <Loader2
              v-if="saving"
              class="w-4 h-4 animate-spin"
            />
            删除
          </button>
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ChevronLeft, Plus, Package, Trash2, X, Loader2 } from 'lucide-vue-next'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import {
  listOpenCodePlugins,
  addOpenCodePlugin,
  deleteOpenCodePlugin,
} from '@/api/modules/opencode'
import type { OpenCodePlugin } from '@/types/opencode'

const plugins = ref<OpenCodePlugin[]>([])
const loading = ref(true)
const error = ref('')
const saving = ref(false)
const showAddDialog = ref(false)
const newNpm = ref('')
const deletingNpm = ref('')

const loadPlugins = async () => {
  loading.value = true
  error.value = ''
  try {
    plugins.value = await listOpenCodePlugins()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '加载失败'
  } finally {
    loading.value = false
  }
}

const confirmDelete = (npm: string) => {
  deletingNpm.value = npm
}

const doAdd = async () => {
  if (!newNpm.value.trim()) return
  saving.value = true
  try {
    await addOpenCodePlugin({ npm: newNpm.value.trim() })
    newNpm.value = ''
    showAddDialog.value = false
    await loadPlugins()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '添加失败'
  } finally {
    saving.value = false
  }
}

const doDelete = async () => {
  if (!deletingNpm.value) return
  saving.value = true
  try {
    await deleteOpenCodePlugin(deletingNpm.value)
    deletingNpm.value = ''
    await loadPlugins()
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '删除失败'
  } finally {
    saving.value = false
  }
}

onMounted(loadPlugins)
</script>
