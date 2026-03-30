<template>
  <div class="min-h-full p-6 lg:p-10 relative overflow-hidden">
    <AnimatedBackground
      contained
      variant="minimal"
    />

    <div class="relative z-10 mx-auto max-w-3xl space-y-5">
      <!-- 页面标题 -->
      <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between animate-slide-up">
        <div class="flex items-center gap-3">
          <RouterLink
            to="/opencode"
            class="inline-flex"
          >
            <Button
              variant="ghost"
              surface="status"
              density="compact"
              motion="subtle"
            >
              <template #leading>
                <SIcon
                  name="ChevronLeft"
                  size="w-5 h-5"
                />
              </template>
            </Button>
          </RouterLink>
          <div>
            <h1 class="text-2xl font-bold text-text-primary">
              插件管理
            </h1>
            <p class="text-sm text-text-secondary">
              管理 OpenCode npm 插件包
            </p>
          </div>
        </div>
        <Button
          variant="success"
          surface="card"
          density="compact"
          motion="standard"
          @click="showAddDialog = true"
        >
          <template #leading>
            <SIcon
              name="Plus"
              size="w-4 h-4"
            />
          </template>
          添加插件
        </Button>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading"
        class="flex items-center justify-center py-16"
      >
        <div class="w-8 h-8 rounded-full border-2 border-accent-success/30 border-t-accent-success animate-spin" />
      </div>

      <!-- 错误状态 -->
      <Card
        v-else-if="error"
        surface="card"
        :elevation="2"
        motion="subtle"
        class="p-6 text-center"
      >
        <p class="mb-3 text-accent-danger">
          {{ error }}
        </p>
        <button
          type="button"
          class="min-h-[44px] rounded-lg px-3 text-sm text-accent-success transition-colors hover:bg-accent-success/10 hover:underline"
          @click="loadPlugins"
        >
          重新加载
        </button>
      </Card>

      <!-- 空状态 -->
      <Card
        v-else-if="plugins.length === 0"
        surface="workspace"
        :elevation="2"
        motion="subtle"
        class="p-10 text-center"
      >
        <div class="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-2xl bg-accent-success/10 text-accent-success">
          <SIcon
            name="Package"
            size="w-7 h-7"
          />
        </div>
        <h3 class="mb-2 text-lg font-bold text-text-primary">
          暂无插件
        </h3>
        <p class="mb-4 text-sm text-text-secondary">
          添加 npm 插件包来扩展 OpenCode 功能
        </p>
        <Button
          variant="success"
          surface="card"
          density="compact"
          motion="standard"
          @click="showAddDialog = true"
        >
          添加第一个插件
        </Button>
      </Card>

      <!-- 插件列表 -->
      <div
        v-else
        class="space-y-2"
      >
        <Card
          v-for="plugin in plugins"
          :key="plugin.npm"
          surface="card"
          :elevation="2"
          motion="subtle"
          class="p-4 animate-slide-up"
        >
          <div class="flex items-center justify-between gap-4">
            <div class="flex items-center gap-3 min-w-0">
              <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-accent-success/10 text-accent-success">
                <SIcon
                  name="Package"
                  size="w-4 h-4"
                />
              </div>
              <div class="min-w-0">
                <p class="truncate font-mono text-sm font-medium text-text-primary">
                  {{ plugin.npm }}
                </p>
                <p class="text-xs text-text-secondary">
                  npm 包
                </p>
              </div>
            </div>

            <button
              type="button"
              class="flex min-h-[44px] min-w-[44px] shrink-0 items-center justify-center rounded-xl text-text-secondary transition-colors hover:bg-accent-danger/10 hover:text-accent-danger focus:outline-none focus:ring-2 focus:ring-accent-danger/20"
              @click="confirmDelete(plugin.npm)"
            >
              <SIcon
                name="Trash2"
                size="w-4 h-4"
              />
            </button>
          </div>
        </Card>
      </div>
    </div>

    <!-- 添加插件弹窗 -->
    <BaseModal
      v-model="showAddDialog"
      title="添加插件"
      description="输入要安装到 OpenCode 的 npm 插件包名。"
      size="md"
      content-class="max-w-md"
    >
      <div class="space-y-4">
        <div>
          <label
            for="opencode-plugin-npm"
            class="mb-2 block text-xs font-bold uppercase tracking-wider text-text-secondary"
          >npm 包名 *</label>
          <input
            id="opencode-plugin-npm"
            v-model="newNpm"
            type="text"
            placeholder="例：@opencode-ai/omo"
            class="w-full rounded-xl border border-border-default bg-bg-surface px-4 py-3 text-sm text-text-primary placeholder:text-text-muted focus:border-accent-success focus:outline-none focus:ring-2 focus:ring-accent-success/20"
            @keyup.enter="doAdd"
          >
        </div>

        <div class="flex justify-end gap-3 border-t border-border-default/50 pt-4">
          <Button
            variant="secondary"
            surface="status"
            density="compact"
            motion="subtle"
            @click="showAddDialog = false"
          >
            取消
          </Button>
          <Button
            variant="success"
            surface="card"
            density="compact"
            motion="standard"
            :disabled="!newNpm.trim() || saving"
            @click="doAdd"
          >
            <template #leading>
              <SIcon
                v-if="saving"
                name="Loader2"
                size="w-4 h-4"
                class="animate-spin"
              />
            </template>
            添加
          </Button>
        </div>
      </div>
    </BaseModal>

    <!-- 删除确认弹窗 -->
    <BaseModal
      :model-value="Boolean(deletingNpm)"
      title="确认删除"
      description="删除后插件会从 OpenCode 配置中移除。"
      size="sm"
      content-class="max-w-sm"
      @update:model-value="(value) => !value && (deletingNpm = '')"
    >
      <div class="space-y-4">
        <p class="text-sm text-text-secondary">
          确定要删除插件 <strong class="font-mono">{{ deletingNpm }}</strong> 吗？
        </p>
        <div class="flex justify-end gap-3 border-t border-border-default/50 pt-4">
          <Button
            variant="secondary"
            surface="status"
            density="compact"
            motion="subtle"
            @click="deletingNpm = ''"
          >
            取消
          </Button>
          <Button
            variant="danger"
            surface="status"
            density="compact"
            motion="standard"
            :disabled="saving"
            @click="doDelete"
          >
            <template #leading>
              <SIcon
                v-if="saving"
                name="Loader2"
                size="w-4 h-4"
                class="animate-spin"
              />
            </template>
            删除
          </Button>
        </div>
      </div>
    </BaseModal>
  </div>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'
import { ref, onMounted } from 'vue'
import AnimatedBackground from '@/components/common/AnimatedBackground.vue'
import Card from '@/components/ui/Card.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import Button from '@/components/ui/Button.vue'
import {
  listOpenCodePlugins,
  addOpenCodePlugin,
  deleteOpenCodePlugin,
} from '@/api'
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
