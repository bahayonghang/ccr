<template>
  <OpenCodePageShell
    title="Plugins"
    description="将 npm 插件配置与本地插件文件分开展示，并补上官方 load order 语义。"
    icon="Puzzle"
    tone="emerald"
    badge="plugin"
  >
    <template #actions>
      <Button
        variant="success"
        surface="card"
        density="compact"
        motion="standard"
        @click="showModal = true"
      >
        <template #leading>
          <SIcon
            name="Plus"
            size="w-4 h-4"
          />
        </template>
        添加 npm 插件
      </Button>
    </template>

    <div class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_360px]">
      <div class="space-y-5">
        <Card
          variant="glass"
          class="p-5"
        >
          <div class="mb-4 flex items-start justify-between gap-3">
            <div>
              <h2 class="text-lg font-semibold text-text-primary">
                npm plugin packages
              </h2>
              <p class="mt-1 text-sm text-text-secondary">
                这些条目来自 `opencode.json` 的 `plugin` 数组，会在启动时通过 Bun 自动安装。
              </p>
            </div>
            <span class="rounded-full border border-emerald-300/20 bg-emerald-300/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-emerald-200">
              {{ packages.length }}
            </span>
          </div>

          <div
            v-if="loading"
            class="flex justify-center py-8"
          >
            <div class="h-8 w-8 rounded-full border-2 border-emerald-300/25 border-t-emerald-300 animate-spin" />
          </div>

          <div
            v-else-if="packages.length === 0"
            class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4 text-sm text-text-secondary"
          >
            暂无 npm 插件配置。
          </div>

          <div
            v-else
            class="space-y-3"
          >
            <div
              v-for="item in packages"
              :key="item.name"
              class="flex items-center justify-between gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 p-4"
            >
              <div>
                <strong class="block font-mono text-sm text-text-primary">{{ item.name }}</strong>
                <span class="mt-1 block text-xs text-text-muted">cached in ~/.cache/opencode/node_modules</span>
              </div>
              <Button
                variant="danger"
                surface="status"
                density="compact"
                motion="subtle"
                @click="removePackage(item.name)"
              >
                删除
              </Button>
            </div>
          </div>
        </Card>

        <Card
          variant="glass"
          class="p-5"
        >
          <div class="mb-4 flex items-start justify-between gap-3">
            <div>
              <h2 class="text-lg font-semibold text-text-primary">
                Local plugin files
              </h2>
              <p class="mt-1 text-sm text-text-secondary">
                来自 `.opencode/plugins/` 和 `~/.config/opencode/plugins/` 的本地脚本文件。
              </p>
            </div>
            <span class="rounded-full border border-border-default/55 bg-bg-base/35 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-text-secondary">
              {{ localPlugins.length }}
            </span>
          </div>

          <div
            v-if="localPlugins.length === 0"
            class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4 text-sm text-text-secondary"
          >
            未发现本地插件文件。
          </div>

          <div
            v-else
            class="space-y-3"
          >
            <div
              v-for="plugin in localPlugins"
              :key="plugin.path"
              class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4"
            >
              <div class="flex items-center justify-between gap-3">
                <div>
                  <strong class="block text-sm text-text-primary">{{ plugin.name }}</strong>
                  <span class="mt-1 block break-all font-mono text-xs text-text-muted">{{ plugin.path }}</span>
                </div>
                <span class="rounded-full bg-bg-base/45 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-text-secondary">
                  {{ plugin.scope }}
                </span>
              </div>
            </div>
          </div>
        </Card>
      </div>

      <Card
        variant="glass"
        class="p-5"
      >
        <h2 class="text-lg font-semibold text-text-primary">
          Load order
        </h2>
        <p class="mt-2 text-sm text-text-secondary">
          插件会按以下顺序加载，适合在排查覆盖关系时直接对照。
        </p>

        <ol class="mt-4 space-y-3">
          <li class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4">
            <strong class="block text-sm text-text-primary">1. Global config</strong>
            <span class="mt-1 block text-xs text-text-muted">~/.config/opencode/opencode.json</span>
          </li>
          <li class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4">
            <strong class="block text-sm text-text-primary">2. Project config</strong>
            <span class="mt-1 block text-xs text-text-muted">opencode.json</span>
          </li>
          <li class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4">
            <strong class="block text-sm text-text-primary">3. Global plugin directory</strong>
            <span class="mt-1 block text-xs text-text-muted">~/.config/opencode/plugins</span>
          </li>
          <li class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4">
            <strong class="block text-sm text-text-primary">4. Project plugin directory</strong>
            <span class="mt-1 block text-xs text-text-muted">.opencode/plugins</span>
          </li>
        </ol>
      </Card>
    </div>

    <BaseModal
      v-model="showModal"
      title="添加 npm 插件"
      description="向 `plugin` 数组追加一个 npm package。"
      size="md"
      content-class="max-w-md"
    >
      <div class="space-y-4">
        <div>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">package name</label>
          <input
            v-model="newPackage"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
            placeholder="@my-org/custom-plugin"
          >
        </div>

        <div class="flex justify-end gap-3 border-t border-border-default/55 pt-4">
          <Button
            variant="secondary"
            surface="status"
            density="compact"
            motion="subtle"
            @click="showModal = false"
          >
            取消
          </Button>
          <Button
            variant="success"
            surface="card"
            density="compact"
            motion="standard"
            :disabled="saving"
            @click="savePackage"
          >
            <template #leading>
              <SIcon
                v-if="saving"
                name="Loader2"
                size="w-4 h-4"
                class="animate-spin"
              />
            </template>
            保存
          </Button>
        </div>
      </div>
    </BaseModal>
  </OpenCodePageShell>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import { useUIStore } from '@/stores/ui'
import { addOpenCodePlugin, deleteOpenCodePlugin, listOpenCodeLocalPlugins, listOpenCodePlugins } from '@/api'
import type { OpenCodeLocalPluginFile, OpenCodePluginPackage } from '@/types'

const uiStore = useUIStore()
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const newPackage = ref('')
const packages = ref<OpenCodePluginPackage[]>([])
const localPlugins = ref<OpenCodeLocalPluginFile[]>([])

async function loadPlugins() {
  loading.value = true
  try {
    const [packageNames, localPluginList] = await Promise.all([
      listOpenCodePlugins<string[]>(),
      listOpenCodeLocalPlugins<OpenCodeLocalPluginFile[]>(),
    ])
    packages.value = packageNames.map((name) => ({ name }))
    localPlugins.value = localPluginList
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    loading.value = false
  }
}

async function savePackage() {
  if (!newPackage.value.trim()) {
    uiStore.showError('package name 不能为空')
    return
  }

  saving.value = true
  try {
    await addOpenCodePlugin({ name: newPackage.value.trim() })
    uiStore.showSuccess('npm 插件已添加')
    newPackage.value = ''
    showModal.value = false
    await loadPlugins()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    saving.value = false
  }
}

async function removePackage(name: string) {
  try {
    await deleteOpenCodePlugin(name)
    uiStore.showSuccess('npm 插件已删除')
    await loadPlugins()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

onMounted(() => {
  void loadPlugins()
})
</script>
