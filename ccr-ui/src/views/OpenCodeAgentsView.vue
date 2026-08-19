<template>
  <OpenCodePageShell
    :title="tt('Agents', 'Agents')"
    :description="tt('把 OpenCode 的 built-in agent 模式和自定义 agents 放在同一张操作面板里。', 'Bring OpenCode built-in agent modes and custom agents onto one control surface.')"
    icon="Bot"
    tone="violet"
    badge="agent"
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
        {{ tt('添加 Agent', 'Add agent') }}
      </Button>
    </template>

    <div class="grid gap-5 xl:grid-cols-[360px_minmax(0,1fr)]">
      <Card
        variant="glass"
        class="p-5"
      >
        <h2 class="text-lg font-semibold text-text-primary">
          {{ tt('Built-in layout', 'Built-in layout') }}
        </h2>
        <p class="mt-2 text-sm text-text-secondary">
          {{ tt('OpenCode 内置两个 primary agent 和两个 subagent，页面重点是展示自定义 agent 如何挂在这个体系上。', 'OpenCode ships with two primary agents and two subagents. This page focuses on how custom agents attach to that layout.') }}
        </p>

        <div class="mt-4 space-y-3">
          <div
            v-for="item in builtInAgents"
            :key="item.name"
            class="rounded-2xl border border-border-default/55 bg-bg-base p-4"
          >
            <div class="flex items-center justify-between gap-3">
              <strong class="text-sm text-text-primary">{{ item.name }}</strong>
              <span
                class="rounded-full px-3 py-1 text-xs font-semibold"
                :class="item.mode === 'primary' ? 'bg-violet-300/10 text-violet-200' : 'bg-cyan-300/10 text-cyan-200'"
              >
                {{ item.mode }}
              </span>
            </div>
            <p class="mt-2 text-sm leading-6 text-text-secondary">
              {{ item.description }}
            </p>
          </div>
        </div>
      </Card>

      <div class="space-y-4">
        <Card
          v-if="loading"
          variant="glass"
          class="p-8 text-center"
        >
          <div class="mx-auto h-8 w-8 rounded-full border-2 border-violet-300/25 border-t-violet-300 animate-spin" />
        </Card>

        <Card
          v-else-if="agents.length === 0"
          variant="glass"
          class="p-8 text-center"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            {{ tt('暂无自定义 Agent', 'No custom agents yet') }}
          </h2>
          <p class="mt-2 text-sm text-text-secondary">
            {{ tt('新建 primary / subagent，用于计划、评审、文档或其它专项工作流。', 'Create a primary agent or subagent for planning, review, documentation, or other focused workflows.') }}
          </p>
        </Card>

        <Card
          v-for="agent in agents"
          :key="agent.path"
          variant="glass"
          class="p-5"
        >
          <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div class="min-w-0">
              <div class="mb-3 flex flex-wrap items-center gap-2">
                <span class="rounded-full border border-violet-300/20 bg-violet-300/10 px-3 py-1 text-xs font-semibold text-violet-200">
                  {{ agent.name }}
                </span>
                <span class="rounded-full bg-bg-base px-3 py-1 text-xs font-semibold text-text-secondary">
                  {{ agent.mode || 'all' }}
                </span>
                <span class="rounded-full bg-bg-base px-3 py-1 text-xs font-semibold text-text-secondary">
                  {{ agent.scope }}
                </span>
                <span
                  v-if="agent.hidden"
                  class="rounded-full bg-amber-300/10 px-3 py-1 text-xs font-semibold text-amber-200"
                >
                  {{ tt('隐藏', 'hidden') }}
                </span>
              </div>

              <h2 class="text-lg font-semibold text-text-primary">
                {{ agent.description || agent.name }}
              </h2>
              <p class="mt-2 text-sm leading-7 text-text-secondary">
                {{ agent.body || tt('未配置 body prompt。', 'No body prompt configured.') }}
              </p>

              <div class="mt-4 grid gap-3 md:grid-cols-3">
                <div class="rounded-2xl border border-border-default/55 bg-bg-base p-3">
                  <span class="text-[11px] font-semibold text-text-muted">{{ tt('模型', 'model') }}</span>
                  <p class="mt-2 text-sm text-text-primary">
                    {{ agent.model || tt('继承', 'inherit') }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border-default/55 bg-bg-base p-3">
                  <span class="text-[11px] font-semibold text-text-muted">{{ tt('步数', 'steps') }}</span>
                  <p class="mt-2 text-sm text-text-primary">
                    {{ agent.steps ?? tt('不限', 'unlimited') }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border-default/55 bg-bg-base p-3">
                  <span class="text-[11px] font-semibold text-text-muted">{{ tt('路径', 'path') }}</span>
                  <p class="mt-2 break-all font-mono text-xs text-text-primary">
                    {{ agent.path }}
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
                @click="openEdit(agent)"
              >
                {{ tt('编辑', 'Edit') }}
              </Button>
              <Button
                variant="danger"
                surface="status"
                density="compact"
                motion="subtle"
                @click="removeAgent(agent)"
              >
                {{ tt('删除', 'Delete') }}
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <BaseModal
      v-model="showModal"
      :title="editingName ? tt('编辑 Agent', 'Edit agent') : tt('添加 Agent', 'Add agent')"
      :description="tt('写 frontmatter，写 body prompt，然后交给 OpenCode 的 Task / agent runtime 使用。', 'Define frontmatter and a body prompt, then hand the result to the OpenCode task / agent runtime.')"
      size="lg"
      content-class="max-w-3xl"
    >
      <div class="space-y-4">
        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('name *', 'name *') }}</label>
            <input
              v-model="form.name"
              :disabled="Boolean(editingName)"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              placeholder="code-reviewer"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('mode', 'mode') }}</label>
            <select
              v-model="form.mode"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
            >
              <option value="primary">
                {{ tt('primary', 'primary') }}
              </option>
              <option value="subagent">
                {{ tt('subagent', 'subagent') }}
              </option>
              <option value="all">
                {{ tt('all', 'all') }}
              </option>
            </select>
          </div>
        </div>

        <div>
          <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('description *', 'description *') }}</label>
          <input
            v-model="form.description"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
            placeholder="Reviews code for risks and maintainability"
          >
        </div>

        <div class="grid gap-4 md:grid-cols-4">
          <div class="md:col-span-2">
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('模型', 'model') }}</label>
            <input
              v-model="form.model"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              placeholder="anthropic/claude-sonnet-4-5"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('temperature', 'temperature') }}</label>
            <input
              v-model="form.temperature"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              placeholder="0.1"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('steps', 'steps') }}</label>
            <input
              v-model="form.steps"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary"
              placeholder="5"
            >
          </div>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary">
            <input
              v-model="form.hidden"
              type="checkbox"
            >
            {{ tt('隐藏 subagent', 'hidden subagent') }}
          </label>
          <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 text-sm text-text-primary">
            <input
              v-model="form.disable"
              type="checkbox"
            >
            {{ tt('已禁用', 'disabled') }}
          </label>
        </div>

        <div>
          <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('permission JSON', 'permission JSON') }}</label>
          <textarea
            v-model="form.permissionJson"
            rows="6"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
          />
        </div>

        <div>
          <label class="mb-2 block text-xs font-semibold text-text-muted">{{ tt('body prompt', 'body prompt') }}</label>
          <textarea
            v-model="form.body"
            rows="12"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base px-4 py-3 font-mono text-sm text-text-primary"
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
            {{ tt('取消', 'Cancel') }}
          </Button>
          <Button
            variant="primary"
            surface="card"
            density="compact"
            motion="standard"
            :disabled="saving"
            @click="saveAgent"
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
import { computed, onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getErrorMessage } from '@/utils/errorHandler'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import { useUIStore } from '@/stores/ui'
import { addOpenCodeAgent, deleteOpenCodeAgent, listOpenCodeAgents, updateOpenCodeAgent } from '@/api'
import type { OpenCodeAgent, OpenCodeAgentRequest } from '@/types'
import { formatJsonInput, parseJsonInput } from '@/utils/opencode'

const { locale } = useI18n()
const isZh = computed(() => locale.value.startsWith('zh'))
const tt = (zh: string, en: string) => (isZh.value ? zh : en)

const builtInAgents = [
  { name: 'build', mode: 'primary', description: '默认 primary agent，拥有完整工具访问。' },
  { name: 'plan', mode: 'primary', description: '受限的 planning / analysis agent，默认写入和 bash 走 ask。' },
  { name: 'general', mode: 'subagent', description: '全功能 subagent，适合并行多步任务。' },
  { name: 'explore', mode: 'subagent', description: '只读探索 agent，适合快速搜索与代码定位。' },
]

const uiStore = useUIStore()
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const editingName = ref('')
const agents = ref<OpenCodeAgent[]>([])

const form = reactive({
  name: '',
  description: '',
  mode: 'subagent' as 'primary' | 'subagent' | 'all',
  model: '',
  temperature: '',
  steps: '',
  hidden: false,
  disable: false,
  permissionJson: '{}',
  body: '',
})

async function loadAgents() {
  loading.value = true
  try {
    agents.value = await listOpenCodeAgents()
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  } finally {
    loading.value = false
  }
}

function openCreate() {
  editingName.value = ''
  form.name = ''
  form.description = ''
  form.mode = 'subagent'
  form.model = ''
  form.temperature = ''
  form.steps = ''
  form.hidden = false
  form.disable = false
  form.permissionJson = '{}'
  form.body = ''
  showModal.value = true
}

function openEdit(agent: OpenCodeAgent) {
  editingName.value = agent.name
  form.name = agent.name
  form.description = agent.description || ''
  form.mode = agent.mode || 'all'
  form.model = agent.model || ''
  form.temperature = agent.temperature != null ? String(agent.temperature) : ''
  form.steps = agent.steps != null ? String(agent.steps) : ''
  form.hidden = Boolean(agent.hidden)
  form.disable = Boolean(agent.disable)
  form.permissionJson = formatJsonInput(agent.permission || {})
  form.body = agent.body || ''
  showModal.value = true
}

async function saveAgent() {
  if (!form.name.trim() || !form.description.trim()) {
    uiStore.showError(tt('Agent name 和 description 为必填项', 'Agent name and description are required'))
    return
  }

  saving.value = true
  try {
    const request: OpenCodeAgentRequest = {
      name: form.name.trim(),
      scope: 'global',
      description: form.description.trim(),
      mode: form.mode,
      model: form.model.trim() || undefined,
      temperature: form.temperature.trim() ? Number(form.temperature.trim()) : undefined,
      steps: form.steps.trim() ? Number(form.steps.trim()) : undefined,
      hidden: form.hidden,
      disable: form.disable,
      permission: parseJsonInput<Record<string, unknown>>(form.permissionJson, {}),
      body: form.body,
    }

    if (editingName.value) {
      await updateOpenCodeAgent(request)
    } else {
      await addOpenCodeAgent(request)
    }

    uiStore.showSuccess(editingName.value ? tt('Agent 已更新', 'Agent updated') : tt('Agent 已创建', 'Agent created'))
    showModal.value = false
    await loadAgents()
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  } finally {
    saving.value = false
  }
}

async function removeAgent(agent: OpenCodeAgent) {
  try {
    await deleteOpenCodeAgent(agent.name, { scope: agent.scope })
    uiStore.showSuccess(tt('Agent 已删除', 'Agent deleted'))
    await loadAgents()
  } catch (error) {
    uiStore.showError(getErrorMessage(error))
  }
}

onMounted(() => {
  void loadAgents()
})
</script>
