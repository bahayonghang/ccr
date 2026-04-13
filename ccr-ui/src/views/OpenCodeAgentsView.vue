<template>
  <OpenCodePageShell
    title="Agents"
    description="把 OpenCode 的 built-in agent 模式和自定义 agents 放在同一张操作面板里。"
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
        添加 Agent
      </Button>
    </template>

    <div class="grid gap-5 xl:grid-cols-[360px_minmax(0,1fr)]">
      <Card
        variant="glass"
        class="p-5"
      >
        <h2 class="text-lg font-semibold text-text-primary">
          Built-in layout
        </h2>
        <p class="mt-2 text-sm text-text-secondary">
          OpenCode 内置两个 primary agent 和两个 subagent，页面重点是展示自定义 agent 如何挂在这个体系上。
        </p>

        <div class="mt-4 space-y-3">
          <div
            v-for="item in builtInAgents"
            :key="item.name"
            class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4"
          >
            <div class="flex items-center justify-between gap-3">
              <strong class="text-sm text-text-primary">{{ item.name }}</strong>
              <span
                class="rounded-full px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em]"
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
            暂无自定义 Agent
          </h2>
          <p class="mt-2 text-sm text-text-secondary">
            新建 primary / subagent，用于计划、评审、文档或其它专项工作流。
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
                <span class="rounded-full border border-violet-300/20 bg-violet-300/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-violet-200">
                  {{ agent.name }}
                </span>
                <span class="rounded-full bg-bg-base/45 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-text-secondary">
                  {{ agent.mode || 'all' }}
                </span>
                <span class="rounded-full bg-bg-base/45 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-text-secondary">
                  {{ agent.scope }}
                </span>
                <span
                  v-if="agent.hidden"
                  class="rounded-full bg-amber-300/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-amber-200"
                >
                  hidden
                </span>
              </div>

              <h2 class="text-lg font-semibold text-text-primary">
                {{ agent.description || agent.name }}
              </h2>
              <p class="mt-2 text-sm leading-7 text-text-secondary">
                {{ agent.body || 'No body prompt configured.' }}
              </p>

              <div class="mt-4 grid gap-3 md:grid-cols-3">
                <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted">model</span>
                  <p class="mt-2 text-sm text-text-primary">
                    {{ agent.model || 'inherit' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted">steps</span>
                  <p class="mt-2 text-sm text-text-primary">
                    {{ agent.steps ?? 'unlimited' }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-3">
                  <span class="text-[11px] font-semibold uppercase tracking-[0.16em] text-text-muted">path</span>
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
                编辑
              </Button>
              <Button
                variant="danger"
                surface="status"
                density="compact"
                motion="subtle"
                @click="removeAgent(agent)"
              >
                删除
              </Button>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <BaseModal
      v-model="showModal"
      :title="editingName ? '编辑 Agent' : '添加 Agent'"
      description="写 frontmatter，写 body prompt，然后交给 OpenCode 的 Task / agent runtime 使用。"
      size="lg"
      content-class="max-w-3xl"
    >
      <div class="space-y-4">
        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">name *</label>
            <input
              v-model="form.name"
              :disabled="Boolean(editingName)"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="code-reviewer"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">mode</label>
            <select
              v-model="form.mode"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
            >
              <option value="primary">
                primary
              </option>
              <option value="subagent">
                subagent
              </option>
              <option value="all">
                all
              </option>
            </select>
          </div>
        </div>

        <div>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">description *</label>
          <input
            v-model="form.description"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
            placeholder="Reviews code for risks and maintainability"
          >
        </div>

        <div class="grid gap-4 md:grid-cols-4">
          <div class="md:col-span-2">
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">model</label>
            <input
              v-model="form.model"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="anthropic/claude-sonnet-4-5"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">temperature</label>
            <input
              v-model="form.temperature"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="0.1"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">steps</label>
            <input
              v-model="form.steps"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="5"
            >
          </div>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 px-4 py-3 text-sm text-text-primary">
            <input
              v-model="form.hidden"
              type="checkbox"
            >
            hidden subagent
          </label>
          <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 px-4 py-3 text-sm text-text-primary">
            <input
              v-model="form.disable"
              type="checkbox"
            >
            disabled
          </label>
        </div>

        <div>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">permission JSON</label>
          <textarea
            v-model="form.permissionJson"
            rows="6"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
          />
        </div>

        <div>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">body prompt</label>
          <textarea
            v-model="form.body"
            rows="12"
            class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 font-mono text-sm text-text-primary"
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
            保存
          </Button>
        </div>
      </div>
    </BaseModal>
  </OpenCodePageShell>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import Card from '@/components/ui/Card.vue'
import Button from '@/components/ui/Button.vue'
import SIcon from '@/components/ui/SIcon.vue'
import BaseModal from '@/components/common/BaseModal.vue'
import OpenCodePageShell from '@/components/opencode/OpenCodePageShell.vue'
import { useUIStore } from '@/stores/ui'
import { addOpenCodeAgent, deleteOpenCodeAgent, listOpenCodeAgents, updateOpenCodeAgent } from '@/api'
import type { OpenCodeAgent, OpenCodeAgentRequest } from '@/types'
import { formatJsonInput, parseJsonInput } from '@/utils/opencode'

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
    agents.value = await listOpenCodeAgents<OpenCodeAgent[]>()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
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
    uiStore.showError('Agent name 和 description 为必填项')
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

    uiStore.showSuccess(editingName.value ? 'Agent 已更新' : 'Agent 已创建')
    showModal.value = false
    await loadAgents()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    saving.value = false
  }
}

async function removeAgent(agent: OpenCodeAgent) {
  try {
    await deleteOpenCodeAgent(agent.name, { scope: agent.scope })
    uiStore.showSuccess('Agent 已删除')
    await loadAgents()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

onMounted(() => {
  void loadAgents()
})
</script>
