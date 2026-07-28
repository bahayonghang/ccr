<template>
  <OpenCodePageShell
    :title="tt('插件', 'Plugins')"
    :description="tt('将 npm 插件配置与本地插件文件分开展示，并补上官方 load order 语义。', 'Separate npm plugin packages from local plugin files and show the official load-order semantics.')"
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
        {{ tt('添加 npm 插件', 'Add npm plugin') }}
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
                {{ tt('npm plugin packages', 'npm plugin packages') }}
              </h2>
              <p class="mt-1 text-sm text-text-secondary">
                {{ tt('这些条目来自 `opencode.json` 的 `plugin` 数组，会在启动时通过 Bun 自动安装。', 'These entries come from the `plugin` array in `opencode.json` and are installed by Bun on startup.') }}
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
            class="rounded-2xl border border-border-default/55 bg-bg-base p-4 text-sm text-text-secondary"
          >
            {{ tt('暂无 npm 插件配置。', 'No npm plugin packages configured.') }}
          </div>

          <div
            v-else
            class="space-y-3"
          >
            <div
              v-for="item in packages"
              :key="item.name"
              class="flex items-center justify-between gap-3 rounded-2xl border border-border-default/55 bg-bg-base p-4"
            >
              <div>
                <strong class="block font-mono text-sm text-text-primary">{{ item.name }}</strong>
                <span class="mt-1 block text-xs text-text-muted">{{ isZh ? `缓存目录 ${pluginCachePath}` : `cached in ${pluginCachePath}` }}</span>
              </div>
              <Button
                variant="danger"
                surface="status"
                density="compact"
                motion="subtle"
                @click="removePackage(item.name)"
              >
                {{ tt('删除', 'Delete') }}
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
                {{ tt('Local plugin files', 'Local plugin files') }}
              </h2>
              <p class="mt-1 text-sm text-text-secondary">
                {{ tt('来自 `.opencode/plugins/` 和 `~/.config/opencode/plugins/` 的本地脚本文件。', 'Local script files discovered in `.opencode/plugins/` and `~/.config/opencode/plugins/`.') }}
              </p>
            </div>
            <span class="rounded-full border border-border-default/55 bg-bg-base px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-text-secondary">
              {{ localPlugins.length }}
            </span>
          </div>

          <div
            v-if="localPlugins.length === 0"
            class="rounded-2xl border border-border-default/55 bg-bg-base p-4 text-sm text-text-secondary"
          >
            {{ tt('未发现本地插件文件。', 'No local plugin files found.') }}
          </div>

          <div
            v-else
            class="space-y-3"
          >
            <div
              v-for="plugin in localPlugins"
              :key="plugin.path"
              class="rounded-2xl border border-border-default/55 bg-bg-base p-4"
            >
              <div class="flex items-center justify-between gap-3">
                <div>
                  <strong class="block text-sm text-text-primary">{{ plugin.name }}</strong>
                  <span class="mt-1 block break-all font-mono text-xs text-text-muted">{{ plugin.path }}</span>
                </div>
                <span class="rounded-full bg-bg-base px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-text-secondary">
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
          {{ tt('Load order', 'Load order') }}
        </h2>
        <p class="mt-2 text-sm text-text-secondary">
          {{ tt('插件会按以下顺序加载，适合在排查覆盖关系时直接对照。', 'Plugins load in this order, which is useful when tracing overrides.') }}
        </p>

        <ol class="mt-4 space-y-3">
          <li class="rounded-2xl border border-border-default/55 bg-bg-base p-4">
            <strong class="block text-sm text-text-primary">{{ tt('1. 全局配置', '1. Global config') }}</strong>
            <span class="mt-1 block text-xs text-text-muted">{{ globalConfigPath }}</span>
          </li>
          <li class="rounded-2xl border border-border-default/55 bg-bg-base p-4">
            <strong class="block text-sm text-text-primary">{{ tt('2. 项目配置', '2. Project config') }}</strong>
            <span class="mt-1 block text-xs text-text-muted">{{ projectConfigPath }}</span>
          </li>
          <li class="rounded-2xl border border-border-default/55 bg-bg-base p-4">
            <strong class="block text-sm text-text-primary">{{ tt('3. 全局插件目录', '3. Global plugin directory') }}</strong>
            <span class="mt-1 block text-xs text-text-muted">{{ globalPluginDir }}</span>
          </li>
          <li class="rounded-2xl border border-border-default/55 bg-bg-base p-4">
            <strong class="block text-sm text-text-primary">{{ tt('4. 项目插件目录', '4. Project plugin directory') }}</strong>
            <span class="mt-1 block text-xs text-text-muted">{{ projectPluginDir }}</span>
          </li>
        </ol>
      </Card>
    </div>

    <BaseModal
      v-model="showModal"
      :title="tt('添加 npm 插件', 'Add npm plugin')"
      :description="tt('向 `plugin` 数组追加一个 npm package。', 'Append an npm package to the `plugin` array.')"
      size="md"
      content-class="max-w-md"
    >
      <div class="space-y-4">
        <div>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">{{ tt('package name', 'package name') }}</label>
          <input
            v-model="newPackage"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
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
            {{ tt('取消', 'Cancel') }}
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
            {{ tt('保存', 'Save') }}
          </Button>
        </div>
      </div>
    </BaseModal>
  </OpenCodePageShell>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getErrorMessage } from '@/utils/errorHandler'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import { useUIStore } from '@/stores/ui'
import { addOpenCodePlugin, deleteOpenCodePlugin, listOpenCodeLocalPlugins, listOpenCodePlugins } from '@/api'
import type { OpenCodeLocalPluginFile, OpenCodePluginPackage } from '@/types'

const { locale } = useI18n()
const uiStore = useUIStore()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const newPackage = ref('')
const packages = ref<OpenCodePluginPackage[]>([])
const localPlugins = ref<OpenCodeLocalPluginFile[]>([])
const pluginCachePath = '~/.cache/opencode/node_modules'
const globalConfigPath = '~/.config/opencode/opencode.json'
const projectConfigPath = 'opencode.json'
const globalPluginDir = '~/.config/opencode/plugins'
const projectPluginDir = '.opencode/plugins'

async function loadPlugins() {
  loading.value = true
  try {
    const [packageNames, localPluginList] = await Promise.all([
      listOpenCodePlugins(),
      listOpenCodeLocalPlugins(),
    ])
    packages.value = packageNames.map((name) => ({ name }))
    localPlugins.value = localPluginList
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  } finally {
    loading.value = false
  }
}

async function savePackage() {
  if (!newPackage.value.trim()) {
    uiStore.showError(tt('package name 不能为空', 'Package name is required'))
    return
  }

  saving.value = true
  try {
    await addOpenCodePlugin({ name: newPackage.value.trim() })
    uiStore.showSuccess(tt('npm 插件已添加', 'npm plugin added'))
    newPackage.value = ''
    showModal.value = false
    await loadPlugins()
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  } finally {
    saving.value = false
  }
}

async function removePackage(name: string) {
  try {
    await deleteOpenCodePlugin(name)
    uiStore.showSuccess(tt('npm 插件已删除', 'npm plugin deleted'))
    await loadPlugins()
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

onMounted(() => {
  void loadPlugins()
})
</script>
