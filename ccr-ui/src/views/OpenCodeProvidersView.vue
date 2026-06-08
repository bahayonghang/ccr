<template>
  <OpenCodePageShell
    title="Providers"
    description="按官方 provider schema 管理认证、baseURL、模型和启用状态。"
    icon="Layers"
    tone="lime"
    badge="provider"
  >
    <template #actions>
      <Button
        variant="primary"
        surface="card"
        density="compact"
        motion="standard"
        @click="openCreate()"
      >
        <template #leading>
          <SIcon
            name="Plus"
            size="w-4 h-4"
          />
        </template>
        添加 Provider
      </Button>
    </template>

    <div class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_320px]">
      <div class="space-y-4">
        <Card
          v-if="loading"
          variant="glass"
          class="p-8 text-center"
        >
          <div class="mx-auto h-8 w-8 rounded-full border-2 border-lime-300/25 border-t-lime-300 animate-spin" />
        </Card>

        <Card
          v-else-if="providers.length === 0"
          variant="glass"
          class="p-8 text-center"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            暂无 Provider
          </h2>
          <p class="mt-2 text-sm text-text-secondary">
            从 Anthropic、OpenAI、Google 或自定义 OpenAI-compatible provider 开始。
          </p>
        </Card>

        <Card
          v-for="provider in providers"
          :key="provider.id"
          variant="glass"
          class="p-5"
        >
          <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div class="min-w-0">
              <div class="mb-3 flex flex-wrap items-center gap-2">
                <span class="rounded-full border border-lime-300/20 bg-lime-300/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-lime-200">
                  {{ provider.id }}
                </span>
                <span
                  class="rounded-full px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em]"
                  :class="providerEnabled(provider.id) ? 'bg-emerald-300/10 text-emerald-200' : 'bg-amber-300/10 text-amber-200'"
                >
                  {{ providerEnabled(provider.id) ? 'enabled' : 'disabled' }}
                </span>
              </div>

              <h2 class="text-lg font-semibold text-text-primary">
                {{ provider.name || provider.id }}
              </h2>
              <div class="mt-3 grid gap-3 md:grid-cols-3">
                <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted">API key</span>
                  <p class="mt-2 text-sm text-text-primary">
                    {{ maskSecret(provider.options?.apiKey) }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted">baseURL</span>
                  <p class="mt-2 break-all text-sm text-text-primary">
                    {{ provider.options?.baseURL || 'default' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted">models</span>
                  <p class="mt-2 text-sm text-text-primary">
                    {{ Object.keys(provider.models || {}).length }}
                  </p>
                </div>
              </div>
            </div>

            <div class="flex flex-wrap gap-2">
              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                @click="toggleEnabled(provider)"
              >
                <template #leading>
                  <SIcon
                    :name="providerEnabled(provider.id) ? 'PauseCircle' : 'PlayCircle'"
                    size="w-4 h-4"
                  />
                </template>
                {{ providerEnabled(provider.id) ? '停用' : '启用' }}
              </Button>
              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                @click="openEdit(provider)"
              >
                <template #leading>
                  <SIcon
                    name="Pencil"
                    size="w-4 h-4"
                  />
                </template>
                编辑
              </Button>
              <Button
                variant="danger"
                surface="status"
                density="compact"
                motion="subtle"
                @click="removeProvider(provider)"
              >
                <template #leading>
                  <SIcon
                    name="Trash2"
                    size="w-4 h-4"
                  />
                </template>
                删除
              </Button>
            </div>
          </div>
        </Card>
      </div>

      <Card
        variant="glass"
        class="p-5"
      >
        <h2 class="text-lg font-semibold text-text-primary">
          Provider templates
        </h2>
        <p class="mt-2 text-sm text-text-secondary">
          搜索内置或自定义的非敏感模板，一次性填写 provider id、npm、baseURL 和模型 JSON。
        </p>

        <ProviderTemplateSelector
          class="mt-4"
          platform="opencode"
          :selected-template-id="selectedProviderTemplate"
          :selected-endpoint="selectedProviderEndpoint"
          :draft-context="openCodeTemplateDraft"
          label="Template"
          helper="Templates never store apiKey; credentials stay in this provider form."
          @select="handleOpenCodeTemplateSelect"
          @manual="useManualOpenCodeTemplate"
        />
      </Card>
    </div>

    <BaseModal
      v-model="showModal"
      :title="editingId ? '编辑 Provider' : '添加 Provider'"
      description="直接编辑 OpenCode provider 配置。"
      size="lg"
      content-class="max-w-2xl max-h-[90vh] overflow-y-auto"
    >
      <div class="space-y-4">
        <ProviderTemplateSelector
          v-if="!editingId"
          platform="opencode"
          :selected-template-id="selectedProviderTemplate"
          :selected-endpoint="selectedProviderEndpoint"
          :draft-context="openCodeTemplateDraft"
          label="Provider template"
          helper="Apply another non-secret template before saving this provider."
          @select="applyOpenCodeProviderTemplate"
          @manual="useManualOpenCodeTemplate"
        />

        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">provider id *</label>
            <input
              v-model="form.id"
              :disabled="Boolean(editingId)"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="anthropic"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">display name</label>
            <input
              v-model="form.name"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="Anthropic"
            >
          </div>
          <div class="md:col-span-2">
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">npm package</label>
            <input
              v-model="form.npm"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="@ai-sdk/openai-compatible"
            >
          </div>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">api key</label>
            <input
              v-model="form.apiKey"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="{env:ANTHROPIC_API_KEY}"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">baseURL</label>
            <input
              v-model="form.baseURL"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="https://api.example.com"
            >
          </div>
        </div>

        <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 px-4 py-3 text-sm text-text-primary">
          <input
            v-model="form.enabled"
            type="checkbox"
          >
          该 provider 默认启用
        </label>

        <div>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">models JSON</label>
          <textarea
            v-model="form.modelsJson"
            rows="8"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
            placeholder="{&#10;  &quot;claude-sonnet-4-5&quot;: { &quot;name&quot;: &quot;claude-sonnet-4-5&quot; }&#10;}"
          />
        </div>

        <div>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">extra options JSON</label>
          <textarea
            v-model="form.extraOptionsJson"
            rows="6"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
            placeholder="{&#10;  &quot;timeout&quot;: 600000&#10;}"
          />
        </div>

        <div>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">root extra JSON</label>
          <textarea
            v-model="form.rootExtraJson"
            rows="5"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
            placeholder="{&#10;  &quot;api&quot;: &quot;chat&quot;&#10;}"
          />
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
            variant="primary"
            surface="card"
            density="compact"
            motion="standard"
            :disabled="saving"
            @click="saveProvider"
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
import { computed, onMounted, reactive, ref } from 'vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import ProviderTemplateSelector from '@/components/provider-templates/ProviderTemplateSelector.vue'
import { useUIStore } from '@/stores/ui'
import { addOpenCodeProvider, deleteOpenCodeProvider, getOpenCodeConfig, listOpenCodeProviders, updateOpenCodeConfig } from '@/api'
import type {
  OpenCodeConfig,
  OpenCodeModelConfig,
  OpenCodeProviderConfig,
  OpenCodeProviderOptions,
  OpenCodeProviderRequest,
} from '@/types'
import type { ProviderTemplateDraftContext, ProviderTemplateSelection } from '@/types/providerTemplates'
import { formatJsonInput, maskSecret, parseJsonInput } from '@/utils/opencode'
import { mapTemplateToOpenCodeProviderPatch } from '@/utils/providerTemplates'

const uiStore = useUIStore()
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const editingId = ref('')
const providers = ref<OpenCodeProviderConfig[]>([])
const configState = ref<OpenCodeConfig>({})
const selectedProviderTemplate = ref<string | null>(null)
const selectedProviderEndpoint = ref('')

const form = reactive({
  id: '',
  name: '',
  npm: '',
  apiKey: '',
  baseURL: '',
  enabled: true,
  modelsJson: '{}',
  extraOptionsJson: '{}',
  rootExtraJson: '{}',
})

const providerManagedRootKeys = new Set([
  'id',
  'name',
  'npm',
  'options',
  'models',
])

function parseJsonInputSafe<T>(value: string, fallback: T): T {
  try {
    return parseJsonInput<T>(value, fallback)
  } catch {
    return fallback
  }
}

const disabledProviders = computed(() => new Set(configState.value.disabled_providers || []))
const enabledProviders = computed(() => configState.value.enabled_providers || [])
const openCodeTemplateDraft = computed<ProviderTemplateDraftContext>(() => ({
  platform: 'opencode',
  defaultName: form.name || form.id || 'OpenCode provider',
  name: form.name || form.id,
  category: form.baseURL.trim() ? 'third_party' : 'official',
  baseUrls: form.baseURL.trim() ? [form.baseURL.trim()] : [],
  modelCatalog: Object.keys(parseJsonInputSafe<Record<string, unknown>>(form.modelsJson, {})),
  platformOverride: {
    id: form.id,
    name: form.name,
    npm: form.npm,
    baseURL: form.baseURL,
    models: parseJsonInputSafe<Record<string, unknown>>(form.modelsJson, {}),
    extraOptions: parseJsonInputSafe<Record<string, unknown>>(form.extraOptionsJson, {}),
    rootExtra: parseJsonInputSafe<Record<string, unknown>>(form.rootExtraJson, {}),
  },
}))

function providerEnabled(id: string) {
  if (disabledProviders.value.has(id)) return false
  if (enabledProviders.value.length > 0) return enabledProviders.value.includes(id)
  return true
}

async function loadProviders() {
  loading.value = true
  try {
    const [providerList, config] = await Promise.all([
      listOpenCodeProviders<OpenCodeProviderConfig[]>(),
      getOpenCodeConfig<OpenCodeConfig>(),
    ])
    providers.value = providerList
    configState.value = config
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    loading.value = false
  }
}

function resetCreateForm() {
  editingId.value = ''
  form.id = ''
  form.name = ''
  form.npm = ''
  form.apiKey = ''
  form.baseURL = ''
  form.enabled = true
  form.modelsJson = '{}'
  form.extraOptionsJson = '{}'
  form.rootExtraJson = '{}'
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
}

function openCreate() {
  resetCreateForm()
  showModal.value = true
}

function useManualOpenCodeTemplate() {
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
  if (!showModal.value) {
    openCreate()
  }
}

function applyOpenCodeProviderTemplate(selection: ProviderTemplateSelection) {
  const patch = mapTemplateToOpenCodeProviderPatch(selection.template, selection.endpoint)

  selectedProviderTemplate.value = selection.template.id
  selectedProviderEndpoint.value = selection.endpoint || ''
  form.id = patch.id || ''
  form.name = patch.name || selection.template.name
  form.npm = patch.npm || ''
  form.baseURL = patch.baseURL || ''
  form.modelsJson = patch.modelsJson || '{}'
  form.extraOptionsJson = patch.extraOptionsJson || '{}'
  form.rootExtraJson = patch.rootExtraJson || '{}'
  form.apiKey = ''
  form.enabled = true
}

function handleOpenCodeTemplateSelect(selection: ProviderTemplateSelection) {
  resetCreateForm()
  applyOpenCodeProviderTemplate(selection)
  showModal.value = true
}

function openEdit(provider: OpenCodeProviderConfig) {
  editingId.value = provider.id
  form.id = provider.id
  form.name = provider.name || ''
  form.npm = provider.npm || ''
  form.apiKey = String(provider.options?.apiKey || '')
  form.baseURL = String(provider.options?.baseURL || '')
  form.enabled = providerEnabled(provider.id)
  form.modelsJson = formatJsonInput(provider.models || {})
  const extraOptions = { ...(provider.options || {}) }
  delete extraOptions.apiKey
  delete extraOptions.baseURL
  form.extraOptionsJson = formatJsonInput(extraOptions)
  form.rootExtraJson = formatJsonInput(rootExtraFields(provider))
  selectedProviderTemplate.value = null
  selectedProviderEndpoint.value = ''
  showModal.value = true
}

function rootExtraFields(provider: OpenCodeProviderConfig) {
  return Object.fromEntries(
    Object.entries(provider).filter(([key]) => !providerManagedRootKeys.has(key)),
  )
}

async function syncProviderVisibility(id: string, enabled: boolean) {
  const nextDisabled = new Set(configState.value.disabled_providers || [])
  const nextEnabled = new Set(configState.value.enabled_providers || [])

  if (enabled) {
    nextDisabled.delete(id)
    if (nextEnabled.size > 0) nextEnabled.add(id)
  } else {
    nextDisabled.add(id)
    nextEnabled.delete(id)
  }

  const patch: Record<string, unknown> = {
    disabled_providers: [...nextDisabled],
  }
  if ((configState.value.enabled_providers || []).length > 0) {
    patch.enabled_providers = [...nextEnabled]
  }
  configState.value = await updateOpenCodeConfig<OpenCodeConfig>(patch)
}

async function saveProvider() {
  if (!form.id.trim()) {
    uiStore.showError('Provider id 不能为空')
    return
  }

  saving.value = true
  try {
    const extraOptions = parseJsonInput<Record<string, unknown>>(form.extraOptionsJson, {})
    const rootExtra = parseJsonInput<Record<string, unknown>>(form.rootExtraJson, {})
    const models = parseJsonInput<Record<string, OpenCodeModelConfig>>(form.modelsJson, {})
    const options: OpenCodeProviderOptions = { ...extraOptions }
    if (form.apiKey.trim()) options.apiKey = form.apiKey.trim()
    if (form.baseURL.trim()) options.baseURL = form.baseURL.trim()

    const request: OpenCodeProviderRequest = {
      ...rootExtra,
      id: form.id.trim(),
      name: form.name.trim() || undefined,
      npm: form.npm.trim() || undefined,
      options,
      models,
    }
    const { id, ...providerConfig } = request

    await addOpenCodeProvider(id, providerConfig)

    await syncProviderVisibility(id, form.enabled)
    uiStore.showSuccess(editingId.value ? 'Provider 已更新' : 'Provider 已创建')
    showModal.value = false
    await loadProviders()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    saving.value = false
  }
}

async function toggleEnabled(provider: OpenCodeProviderConfig) {
  try {
    await syncProviderVisibility(provider.id, !providerEnabled(provider.id))
    uiStore.showSuccess(providerEnabled(provider.id) ? 'Provider 已启用' : 'Provider 已停用')
    await loadProviders()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

async function removeProvider(provider: OpenCodeProviderConfig) {
  try {
    await deleteOpenCodeProvider(provider.id)
    configState.value = await updateOpenCodeConfig<OpenCodeConfig>({
      disabled_providers: (configState.value.disabled_providers || []).filter((item) => item !== provider.id),
      enabled_providers: (configState.value.enabled_providers || []).filter((item) => item !== provider.id),
    })
    uiStore.showSuccess('Provider 已删除')
    await loadProviders()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

onMounted(() => {
  void loadProviders()
})
</script>
