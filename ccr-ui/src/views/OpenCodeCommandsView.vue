<template>
  <OpenCodePageShell
    title="Commands"
    description="管理 markdown / JSON 形式的自定义命令模板，并展示 built-in command 覆盖语义。"
    icon="Command"
    tone="amber"
    badge="command"
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
        添加 Command
      </Button>
    </template>

    <div class="grid gap-5 xl:grid-cols-[360px_minmax(0,1fr)]">
      <Card
        variant="glass"
        class="p-5"
      >
        <h2 class="text-lg font-semibold text-text-primary">
          Built-in behavior
        </h2>
        <p class="mt-2 text-sm text-text-secondary">
          自定义命令可以覆盖 `/init`、`/undo`、`/redo`、`/share`、`/help` 等内置命令。Agent / subtask / model 都可在 frontmatter 指定。
        </p>

        <div class="mt-4 space-y-3">
          <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4">
            <strong class="font-mono text-sm text-text-primary">/review &lt;target&gt;</strong>
            <p class="mt-2 text-sm text-text-secondary">
              配合 `agent: plan` 或 `subtask: true`，让命令直接走分析链路。
            </p>
          </div>
          <div class="rounded-2xl border border-border-default/55 bg-bg-base/35 p-4">
            <strong class="font-mono text-sm text-text-primary">!`command` + @file</strong>
            <p class="mt-2 text-sm text-text-secondary">
              OpenCode 命令模板支持注入 shell 输出与文件内容。
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
          <div class="mx-auto h-8 w-8 rounded-full border-2 border-amber-300/25 border-t-amber-300 animate-spin" />
        </Card>

        <Card
          v-else-if="commands.length === 0"
          variant="glass"
          class="p-8 text-center"
        >
          <h2 class="text-lg font-semibold text-text-primary">
            暂无自定义 Command
          </h2>
          <p class="mt-2 text-sm text-text-secondary">
            从测试、review、scaffold 这类高频动作开始封装。
          </p>
        </Card>

        <Card
          v-for="command in commands"
          :key="command.path"
          variant="glass"
          class="p-5"
        >
          <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div class="min-w-0">
              <div class="mb-3 flex flex-wrap items-center gap-2">
                <span class="rounded-full border border-amber-300/20 bg-amber-300/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-amber-200">
                  /{{ command.name }}
                </span>
                <span
                  v-if="command.agent"
                  class="rounded-full bg-bg-base/45 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-text-secondary"
                >
                  {{ command.agent }}
                </span>
                <span
                  v-if="command.subtask"
                  class="rounded-full bg-violet-300/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.16em] text-violet-200"
                >
                  subtask
                </span>
              </div>

              <h2 class="text-lg font-semibold text-text-primary">
                {{ command.description || command.name }}
              </h2>
              <pre class="mt-3 overflow-auto rounded-2xl border border-border-default/55 bg-bg-base/35 p-4 text-xs leading-6 text-text-primary">{{ command.template }}</pre>
            </div>

            <div class="flex flex-wrap gap-2">
              <Button
                variant="secondary"
                surface="status"
                density="compact"
                motion="subtle"
                @click="openEdit(command)"
              >
                编辑
              </Button>
              <Button
                variant="danger"
                surface="status"
                density="compact"
                motion="subtle"
                @click="removeCommand(command)"
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
      :title="editingName ? '编辑 Command' : '添加 Command'"
      description="命令模板支持 $ARGUMENTS、位置参数、shell 输出和文件引用。"
      size="lg"
      content-class="max-w-3xl"
    >
      <div class="space-y-4">
        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">command name *</label>
            <input
              v-model="form.name"
              :disabled="Boolean(editingName)"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="review"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">agent</label>
            <input
              v-model="form.agent"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="plan"
            >
          </div>
        </div>

        <div class="grid gap-4 md:grid-cols-2">
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">description *</label>
            <input
              v-model="form.description"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="Review recent changes"
            >
          </div>
          <div>
            <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">model</label>
            <input
              v-model="form.model"
              class="w-full rounded-2xl border border-border-default/55 bg-bg-base/45 px-4 py-3 text-sm text-text-primary"
              placeholder="anthropic/claude-haiku-4-5"
            >
          </div>
        </div>

        <label class="flex items-center gap-3 rounded-2xl border border-border-default/55 bg-bg-base/35 px-4 py-3 text-sm text-text-primary">
          <input
            v-model="form.subtask"
            type="checkbox"
          >
          强制以 subtask 方式执行
        </label>

        <div>
          <label class="mb-2 block text-xs font-semibold uppercase tracking-[0.16em] text-text-muted">template</label>
          <textarea
            v-model="form.template"
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
            @click="saveCommand"
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
import { addOpenCodeCommand, deleteOpenCodeCommand, listOpenCodeCommands, updateOpenCodeCommand } from '@/api'
import type { OpenCodeCommand, OpenCodeCommandRequest } from '@/types'

const uiStore = useUIStore()
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const editingName = ref('')
const commands = ref<OpenCodeCommand[]>([])

const form = reactive({
  name: '',
  description: '',
  agent: '',
  model: '',
  subtask: false,
  template: '',
})

async function loadCommands() {
  loading.value = true
  try {
    commands.value = await listOpenCodeCommands<OpenCodeCommand[]>()
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
  form.agent = ''
  form.model = ''
  form.subtask = false
  form.template = ''
  showModal.value = true
}

function openEdit(command: OpenCodeCommand) {
  editingName.value = command.name
  form.name = command.name
  form.description = command.description || ''
  form.agent = command.agent || ''
  form.model = command.model || ''
  form.subtask = Boolean(command.subtask)
  form.template = command.template || ''
  showModal.value = true
}

async function saveCommand() {
  if (!form.name.trim() || !form.description.trim()) {
    uiStore.showError('Command name 和 description 为必填项')
    return
  }

  saving.value = true
  try {
    const request: OpenCodeCommandRequest = {
      name: form.name.trim(),
      scope: 'global',
      description: form.description.trim(),
      agent: form.agent.trim() || undefined,
      model: form.model.trim() || undefined,
      subtask: form.subtask,
      template: form.template,
    }

    if (editingName.value) {
      await updateOpenCodeCommand(request)
    } else {
      await addOpenCodeCommand(request)
    }

    uiStore.showSuccess(editingName.value ? 'Command 已更新' : 'Command 已创建')
    showModal.value = false
    await loadCommands()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  } finally {
    saving.value = false
  }
}

async function removeCommand(command: OpenCodeCommand) {
  try {
    await deleteOpenCodeCommand(command.name, { scope: command.scope })
    uiStore.showSuccess('Command 已删除')
    await loadCommands()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : String(error))
  }
}

onMounted(() => {
  void loadCommands()
})
</script>
