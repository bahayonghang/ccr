<template>
  <div class="hooks-view">
    <div class="mb-6" />
    <div class="mx-auto max-w-[1600px]">
      <div class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div class="flex items-center gap-4">
          <h2 class="flex items-center text-xl font-bold text-text-primary sm:text-2xl">
            <SIcon
              name="Webhook"
              size="w-6 h-6"
              class="mr-2 text-accent-secondary sm:w-7 sm:h-7"
            />
            Hooks Management
          </h2>
          <span class="rounded-full border border-accent-secondary/20 bg-accent-secondary/10 px-3 py-1 text-sm font-medium text-accent-secondary">
            {{ totalHandlers }}
          </span>
        </div>
        <button
          class="hooks-primary-button"
          @click="openCreateGroup()"
        >
          <SIcon
            name="Plus"
            size="w-5 h-5"
            class="mr-2"
          />
          Add Hook Group
        </button>
      </div>

      <Card
        variant="glass"
        pattern
        class="mb-6"
      >
        <div class="space-y-3 p-5">
          <p class="text-sm text-text-secondary">
            Claude Code hooks use the official grouped format:
            <code class="font-mono text-xs">event -&gt; matcher groups -&gt; handlers</code>.
          </p>
          <p class="text-xs text-text-muted">
            Individual hooks cannot be toggled off in the official schema. Remove a handler or matcher group to disable it.
          </p>
        </div>
      </Card>

      <div class="mb-6 flex gap-2 overflow-x-auto pb-2 scrollbar-thin md:flex-wrap md:overflow-x-visible md:pb-0">
        <button
          v-for="eventName in eventTabs"
          :key="eventName"
          class="min-h-[44px] whitespace-nowrap rounded-lg px-4 py-2 text-sm font-medium transition-colors"
          :class="selectedEvent === eventName ? 'bg-accent-secondary text-white shadow-md' : 'border border-border-default bg-bg-elevated text-text-secondary hover:bg-bg-surface'"
          @click="selectedEvent = eventName"
        >
          {{ eventName }}
          <span class="ml-2 opacity-70">({{ eventGroupCount(eventName) }})</span>
        </button>
      </div>

      <div
        v-if="loading"
        class="py-20 text-center text-text-muted"
      >
        <div class="loading-spinner mx-auto mb-4 h-8 w-8 border-accent-secondary/30 border-t-accent-secondary" />
        <span>Loading...</span>
      </div>

      <div
        v-else-if="visibleEventEntries.length === 0"
        class="py-20 text-center text-text-muted"
      >
        <div class="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full bg-bg-elevated">
          <SIcon
            name="Webhook"
            size="w-10 h-10"
            class="opacity-50"
          />
        </div>
        <p class="text-lg font-medium">
          No hook groups found
        </p>
      </div>

      <div
        v-else
        class="space-y-6"
      >
        <Card
          v-for="[eventName, groups] in visibleEventEntries"
          :key="eventName"
          variant="glass"
          pattern
        >
          <div class="space-y-5 p-5">
            <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
              <div class="space-y-1">
                <div class="flex flex-wrap items-center gap-2">
                  <h3 class="text-lg font-bold text-text-primary">
                    {{ eventName }}
                  </h3>
                  <span
                    class="rounded-md border px-2 py-0.5 text-xs font-medium"
                    :class="getEventColor(eventName)"
                  >
                    {{ groups.length }} group{{ groups.length === 1 ? '' : 's' }}
                  </span>
                </div>
                <p class="text-xs text-text-muted">
                  {{ handlerCountForEvent(eventName) }} handler{{ handlerCountForEvent(eventName) === 1 ? '' : 's' }}
                </p>
              </div>
              <button
                class="hooks-secondary-button"
                @click="openCreateGroup(eventName)"
              >
                <SIcon
                  name="Plus"
                  size="w-4 h-4"
                  class="mr-2"
                />
                Add Group
              </button>
            </div>

            <div class="space-y-4">
              <div
                v-for="(group, groupIndex) in groups"
                :key="`${eventName}-${groupIndex}`"
                class="rounded-2xl border border-border-default/60 bg-bg-surface/60 p-4"
              >
                <div class="mb-4 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                  <div class="space-y-2">
                    <div class="flex flex-wrap items-center gap-2">
                      <span class="rounded-md bg-bg-elevated px-2 py-1 text-xs font-semibold uppercase tracking-wide text-text-muted">Matcher</span>
                      <code class="break-all rounded-md border border-border-default bg-bg-elevated px-2 py-1 font-mono text-xs text-text-primary">
                        {{ group.matcher || 'All matches' }}
                      </code>
                    </div>
                    <p
                      v-if="groupExtraKeys(group).length > 0"
                      class="text-xs text-text-muted"
                    >
                      Advanced group fields: {{ groupExtraKeys(group).join(', ') }}
                    </p>
                  </div>

                  <div class="flex items-center gap-2">
                    <button
                      class="hooks-icon-button hooks-icon-button--accent"
                      @click="openEditGroup(eventName, groupIndex)"
                    >
                      <SIcon
                        name="Edit2"
                        size="w-4 h-4"
                      />
                    </button>
                    <button
                      class="hooks-icon-button hooks-icon-button--danger"
                      @click="handleDeleteGroup(eventName, groupIndex)"
                    >
                      <SIcon
                        name="Trash2"
                        size="w-4 h-4"
                      />
                    </button>
                  </div>
                </div>

                <div class="space-y-3">
                  <div
                    v-for="(handler, handlerIndex) in group.hooks"
                    :key="`${eventName}-${groupIndex}-${handlerIndex}`"
                    class="rounded-xl border border-border-default/50 bg-bg-elevated/50 p-3"
                  >
                    <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                      <div class="space-y-2">
                        <div class="flex flex-wrap items-center gap-2">
                          <span class="rounded-md border border-accent-secondary/20 bg-accent-secondary/10 px-2 py-1 text-xs font-semibold uppercase tracking-wide text-accent-secondary">{{ handler.type }}</span>
                          <span
                            v-if="handler.model"
                            class="rounded-md border border-border-default px-2 py-1 text-xs text-text-secondary"
                          >model: {{ handler.model }}</span>
                          <span
                            v-if="typeof handler.timeout === 'number'"
                            class="rounded-md border border-border-default px-2 py-1 text-xs text-text-secondary"
                          >timeout: {{ handler.timeout }}s</span>
                          <span
                            v-if="handler.async === true"
                            class="rounded-md border border-border-default px-2 py-1 text-xs text-text-secondary"
                          >async</span>
                        </div>
                        <code class="block break-all font-mono text-xs text-text-primary">{{ getHandlerSummary(handler) }}</code>
                        <p
                          v-if="handlerExtraKeys(handler).length > 0"
                          class="text-xs text-text-muted"
                        >
                          Advanced handler fields: {{ handlerExtraKeys(handler).join(', ') }}
                        </p>
                      </div>

                      <button
                        class="hooks-icon-button hooks-icon-button--danger"
                        @click="handleDeleteHandler(eventName, groupIndex, handlerIndex)"
                      >
                        <SIcon
                          name="Trash2"
                          size="w-4 h-4"
                        />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Card>
      </div>
      <Teleport to="body">
        <div
          v-if="showModal"
          class="hooks-modal-overlay"
          @click="closeModal"
        >
          <div
            class="hooks-modal-panel"
            @click.stop
          >
            <button
              class="hooks-modal-close"
              @click="closeModal"
            >
              <SIcon
                name="X"
                size="w-5 h-5"
              />
            </button>

            <h3 class="mb-6 flex items-center text-2xl font-bold text-text-primary">
              <SIcon
                :name="editingTarget ? 'Edit2' : 'Plus'"
                size="w-6 h-6"
                class="mr-2 text-accent-secondary"
              />
              {{ editingTarget ? 'Edit Hook Group' : 'Add Hook Group' }}
            </h3>

            <div class="space-y-6">
              <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div>
                  <label class="hooks-field-label">Event</label>
                  <input
                    v-model="groupForm.event"
                    list="known-hook-events"
                    type="text"
                    class="hooks-input"
                    placeholder="PreToolUse"
                  >
                </div>
                <div>
                  <label class="hooks-field-label">Matcher</label>
                  <input
                    v-model="groupForm.matcher"
                    type="text"
                    class="hooks-input"
                    placeholder="Write|Edit"
                  >
                </div>
              </div>

              <div>
                <label class="hooks-field-label">Group Advanced JSON</label>
                <textarea
                  v-model="groupForm.groupExtraJson"
                  rows="4"
                  class="hooks-input hooks-input--mono hooks-input--textarea"
                  placeholder="{&#10;  &quot;source&quot;: &quot;user&quot;&#10;}"
                />
              </div>

              <div class="space-y-4">
                <div class="flex items-center justify-between">
                  <h4 class="text-lg font-semibold text-text-primary">
                    Handlers
                  </h4>
                  <button
                    class="hooks-secondary-button"
                    @click="addHandlerForm()"
                  >
                    <SIcon
                      name="Plus"
                      size="w-4 h-4"
                      class="mr-2"
                    />
                    Add Handler
                  </button>
                </div>

                <div
                  v-for="(handler, handlerIndex) in groupForm.handlers"
                  :key="handlerIndex"
                  class="space-y-4 rounded-2xl border border-border-default/60 bg-bg-surface/60 p-4"
                >
                  <div class="flex items-center justify-between">
                    <h5 class="text-sm font-semibold uppercase tracking-wide text-text-muted">
                      Handler {{ handlerIndex + 1 }}
                    </h5>
                    <button
                      class="hooks-icon-button hooks-icon-button--danger"
                      :disabled="groupForm.handlers.length === 1"
                      @click="removeHandlerForm(handlerIndex)"
                    >
                      <SIcon
                        name="Trash2"
                        size="w-4 h-4"
                      />
                    </button>
                  </div>

                  <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <div>
                      <label class="hooks-field-label">Type</label>
                      <input
                        v-model="handler.type"
                        list="known-handler-types"
                        type="text"
                        class="hooks-input"
                        placeholder="command"
                      >
                    </div>
                    <div>
                      <label class="hooks-field-label">Timeout (seconds)</label>
                      <input
                        v-model="handler.timeout"
                        type="text"
                        class="hooks-input"
                        placeholder="30"
                      >
                    </div>
                  </div>

                  <div
                    v-if="handler.type === 'command'"
                    class="space-y-4"
                  >
                    <div>
                      <label class="hooks-field-label">Command</label>
                      <input
                        v-model="handler.command"
                        type="text"
                        class="hooks-input hooks-input--mono"
                        placeholder="./scripts/check-style.sh"
                      >
                    </div>
                    <label class="hooks-checkbox">
                      <input
                        v-model="handler.asyncEnabled"
                        type="checkbox"
                        class="h-4 w-4 rounded border-border-default text-accent-secondary focus:ring-accent-secondary"
                      >
                      <span class="hooks-checkbox__label">Run asynchronously</span>
                    </label>
                  </div>

                  <div
                    v-else-if="handler.type === 'http'"
                    class="space-y-4"
                  >
                    <div>
                      <label class="hooks-field-label">URL</label>
                      <input
                        v-model="handler.url"
                        type="text"
                        class="hooks-input hooks-input--mono"
                        placeholder="https://example.com/hooks"
                      >
                    </div>
                    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                      <div>
                        <label class="hooks-field-label">Headers JSON</label>
                        <textarea
                          v-model="handler.headersJson"
                          rows="4"
                          class="hooks-input hooks-input--mono hooks-input--textarea"
                          placeholder="{&#10;  &quot;Authorization&quot;: &quot;Bearer ...&quot;&#10;}"
                        />
                      </div>
                      <div>
                        <label class="hooks-field-label">Allowed Env Vars</label>
                        <textarea
                          v-model="handler.allowedEnvVarsText"
                          rows="4"
                          class="hooks-input hooks-input--mono hooks-input--textarea"
                          placeholder="OPENAI_API_KEY, GITHUB_TOKEN"
                        />
                      </div>
                    </div>
                    <label class="hooks-checkbox">
                      <input
                        v-model="handler.asyncEnabled"
                        type="checkbox"
                        class="h-4 w-4 rounded border-border-default text-accent-secondary focus:ring-accent-secondary"
                      >
                      <span class="hooks-checkbox__label">Run asynchronously</span>
                    </label>
                  </div>

                  <div
                    v-else
                    class="space-y-4"
                  >
                    <div>
                      <label class="hooks-field-label">Prompt</label>
                      <textarea
                        v-model="handler.prompt"
                        rows="4"
                        class="hooks-input hooks-input--mono hooks-input--textarea"
                        placeholder="Evaluate this action and return JSON"
                      />
                    </div>
                    <div>
                      <label class="hooks-field-label">Model</label>
                      <input
                        v-model="handler.model"
                        type="text"
                        class="hooks-input"
                        placeholder="claude-haiku-4-5"
                      >
                    </div>
                  </div>

                  <div>
                    <label class="hooks-field-label">Status Message</label>
                    <input
                      v-model="handler.statusMessage"
                      type="text"
                      class="hooks-input"
                      placeholder="Checking style..."
                    >
                  </div>

                  <div>
                    <label class="hooks-field-label">Handler Advanced JSON</label>
                    <textarea
                      v-model="handler.extraJson"
                      rows="4"
                      class="hooks-input hooks-input--mono hooks-input--textarea"
                      placeholder="{&#10;  &quot;custom&quot;: true&#10;}"
                    />
                  </div>
                </div>
              </div>

              <div class="hooks-footer-actions">
                <button
                  class="hooks-footer-button"
                  @click="closeModal"
                >
                  Cancel
                </button>
                <button
                  class="hooks-footer-button hooks-footer-button--primary"
                  :disabled="saving"
                  @click="saveGroup"
                >
                  {{ saving ? 'Saving...' : editingTarget ? 'Save Group' : 'Add Group' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </Teleport>

      <datalist id="known-hook-events">
        <option
          v-for="eventName in knownEventOptions"
          :key="eventName"
          :value="eventName"
        />
      </datalist>
      <datalist id="known-handler-types">
        <option
          v-for="handlerType in knownHandlerTypes"
          :key="handlerType"
          :value="handlerType"
        />
      </datalist>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import SIcon from '@/components/ui/SIcon.vue'
import Card from '@/components/ui/Card.vue'
import { listHooks, updateHooks } from '@/api'
import { useUIStore } from '@/stores/ui'
import type { Hook, HookMap, HookMatcherGroup } from '@/types'
import { logger } from '@/utils/logger'

interface HookHandlerForm {
  type: string
  command: string
  url: string
  prompt: string
  model: string
  timeout: string
  statusMessage: string
  headersJson: string
  allowedEnvVarsText: string
  asyncEnabled: boolean
  extraJson: string
}

interface HookGroupFormState {
  event: string
  matcher: string
  groupExtraJson: string
  handlers: HookHandlerForm[]
}

interface EditingTarget {
  event: string
  groupIndex: number
}

const knownHookEvents = ['PermissionRequest', 'PostToolUse', 'PostToolUseFailure', 'PreToolUse', 'Stop', 'SubagentStop', 'TaskCompleted', 'UserPromptSubmit', 'ConfigChange', 'Elicitation', 'ElicitationResult', 'InstructionsLoaded', 'Notification', 'PostCompact', 'PreCompact', 'SessionEnd', 'SessionStart', 'StopFailure', 'SubagentStart', 'TeammateIdle', 'WorktreeCreate', 'WorktreeRemove']
const knownHandlerTypes = ['command', 'http', 'prompt', 'agent']

const uiStore = useUIStore()
const hooksConfig = ref<HookMap>({})
const loading = ref(false)
const saving = ref(false)
const showModal = ref(false)
const selectedEvent = ref('All')
const editingTarget = ref<EditingTarget | null>(null)
const groupForm = ref<HookGroupFormState>(createEmptyGroupForm())

const sortedEventNames = computed(() => Object.keys(hooksConfig.value).sort((left, right) => left.localeCompare(right)))
const eventTabs = computed(() => ['All', ...sortedEventNames.value])
const visibleEventEntries = computed(() => {
  const entries = Object.entries(hooksConfig.value).sort(([left], [right]) => left.localeCompare(right))
  return selectedEvent.value === 'All' ? entries : entries.filter(([eventName]) => eventName === selectedEvent.value)
})
const totalHandlers = computed(() => Object.values(hooksConfig.value).reduce((count, groups) => count + groups.reduce((sum, group) => sum + group.hooks.length, 0), 0))
const knownEventOptions = computed(() => Array.from(new Set([...knownHookEvents, ...sortedEventNames.value])).sort((left, right) => left.localeCompare(right)))

function createEmptyHandlerForm(type = 'command'): HookHandlerForm {
  return { type, command: '', url: '', prompt: '', model: '', timeout: '', statusMessage: '', headersJson: '', allowedEnvVarsText: '', asyncEnabled: false, extraJson: '' }
}

function createEmptyGroupForm(event = ''): HookGroupFormState {
  return { event, matcher: '', groupExtraJson: '', handlers: [createEmptyHandlerForm()] }
}

function cloneHookMap(source: HookMap): HookMap {
  return JSON.parse(JSON.stringify(source)) as HookMap
}

function formatJsonObject(value: Record<string, unknown>): string {
  return Object.keys(value).length > 0 ? JSON.stringify(value, null, 2) : ''
}

function parseJsonObject(input: string, label: string): Record<string, unknown> {
  const trimmed = input.trim()
  if (!trimmed) return {}
  const parsed = JSON.parse(trimmed) as unknown
  if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
    throw new Error(`${label} must be a JSON object`)
  }
  return parsed as Record<string, unknown>
}

function parseTimeout(timeout: string): number | undefined {
  const trimmed = timeout.trim()
  if (!trimmed) return undefined
  const value = Number.parseInt(trimmed, 10)
  if (!Number.isFinite(value) || value < 0) throw new Error('Timeout must be a non-negative integer')
  return value
}

function parseHeaders(headersJson: string): Record<string, string> | undefined {
  const headers = parseJsonObject(headersJson, 'Headers JSON')
  const entries = Object.entries(headers)
  return entries.length > 0 ? Object.fromEntries(entries.map(([key, value]) => [key, String(value)])) : undefined
}

function parseAllowedEnvVars(input: string): string[] | undefined {
  const values = input.split(',').map(value => value.trim()).filter(Boolean)
  return values.length > 0 ? values : undefined
}

function groupExtraKeys(group: HookMatcherGroup): string[] {
  return Object.keys(group).filter(key => key !== 'matcher' && key !== 'hooks')
}

function handlerExtraKeys(handler: Hook): string[] {
  return Object.keys(handler).filter(key => !['type', 'command', 'url', 'prompt', 'model', 'timeout', 'statusMessage', 'allowedEnvVars', 'headers', 'async'].includes(key))
}

function getEventColor(eventName: string): string {
  const palette: Record<string, string> = {
    PreToolUse: 'border-accent-secondary/20 bg-accent-secondary/10 text-accent-secondary',
    PostToolUse: 'border-accent-success/20 bg-accent-success/10 text-accent-success',
    Stop: 'border-accent-danger/20 bg-accent-danger/10 text-accent-danger',
    UserPromptSubmit: 'border-accent-primary/20 bg-accent-primary/10 text-accent-primary',
    Notification: 'border-accent-warning/20 bg-accent-warning/10 text-accent-warning',
  }
  return palette[eventName] || 'border-border-default bg-bg-elevated text-text-secondary'
}

function eventGroupCount(eventName: string): number {
  if (eventName === 'All') {
    return Object.values(hooksConfig.value).reduce((count, groups) => count + groups.length, 0)
  }
  return hooksConfig.value[eventName]?.length ?? 0
}

function handlerCountForEvent(eventName: string): number {
  return (hooksConfig.value[eventName] ?? []).reduce((count, group) => count + group.hooks.length, 0)
}

function getHandlerSummary(handler: Hook): string {
  if (handler.type === 'command') return handler.command || '(missing command)'
  if (handler.type === 'http') return handler.url || '(missing url)'
  if (handler.type === 'prompt' || handler.type === 'agent') return handler.prompt || '(missing prompt)'
  return JSON.stringify(handler)
}

function handlerToForm(handler: Hook): HookHandlerForm {
  const { type, command, url, prompt, model, timeout, statusMessage, allowedEnvVars, headers, async, ...other } = handler
  return {
    type: String(type ?? 'command'),
    command: typeof command === 'string' ? command : '',
    url: typeof url === 'string' ? url : '',
    prompt: typeof prompt === 'string' ? prompt : '',
    model: typeof model === 'string' ? model : '',
    timeout: typeof timeout === 'number' ? String(timeout) : '',
    statusMessage: typeof statusMessage === 'string' ? statusMessage : '',
    headersJson: headers ? JSON.stringify(headers, null, 2) : '',
    allowedEnvVarsText: Array.isArray(allowedEnvVars) ? allowedEnvVars.join(', ') : '',
    asyncEnabled: async === true,
    extraJson: formatJsonObject(other),
  }
}

function groupToForm(eventName: string, group: HookMatcherGroup): HookGroupFormState {
  const { matcher, hooks, ...other } = group
  return {
    event: eventName,
    matcher: matcher ?? '',
    groupExtraJson: formatJsonObject(other),
    handlers: hooks.length > 0 ? hooks.map(handlerToForm) : [createEmptyHandlerForm()],
  }
}

function buildHandler(handlerForm: HookHandlerForm): Hook {
  const type = handlerForm.type.trim()
  if (!type) throw new Error('Handler type is required')

  const extra = parseJsonObject(handlerForm.extraJson, 'Handler advanced JSON')
  const handler: Hook = { ...extra, type }

  const command = handlerForm.command.trim()
  const url = handlerForm.url.trim()
  const prompt = handlerForm.prompt.trim()
  const model = handlerForm.model.trim()
  const statusMessage = handlerForm.statusMessage.trim()

  if (type === 'command') {
    if (!command) throw new Error('Command handlers require a command')
    handler.command = command
    if (handlerForm.asyncEnabled) handler.async = true
  } else if (type === 'http') {
    if (!url) throw new Error('HTTP handlers require a URL')
    handler.url = url
    const headers = parseHeaders(handlerForm.headersJson)
    if (headers) handler.headers = headers
    const allowedEnvVars = parseAllowedEnvVars(handlerForm.allowedEnvVarsText)
    if (allowedEnvVars) handler.allowedEnvVars = allowedEnvVars
    if (handlerForm.asyncEnabled) handler.async = true
  } else {
    if (!prompt) throw new Error(`${type} handlers require a prompt`)
    handler.prompt = prompt
    if (model) handler.model = model
  }

  const timeout = parseTimeout(handlerForm.timeout)
  if (timeout != null) handler.timeout = timeout
  if (statusMessage) handler.statusMessage = statusMessage
  return handler
}

function buildGroupFromForm(): { event: string; group: HookMatcherGroup } {
  const eventName = groupForm.value.event.trim()
  if (!eventName) throw new Error('Event is required')
  if (groupForm.value.handlers.length === 0) throw new Error('At least one handler is required')

  const groupExtra = parseJsonObject(groupForm.value.groupExtraJson, 'Group advanced JSON')
  const matcher = groupForm.value.matcher.trim()
  const group: HookMatcherGroup = { ...groupExtra, hooks: groupForm.value.handlers.map(buildHandler) }
  if (matcher) group.matcher = matcher
  return { event: eventName, group }
}

async function loadHooks() {
  loading.value = true
  try {
    hooksConfig.value = await listHooks<HookMap>()
    if (selectedEvent.value !== 'All' && !hooksConfig.value[selectedEvent.value]) selectedEvent.value = 'All'
  } catch (error) {
    logger.error('Failed to load hooks:', error)
    uiStore.showError(error instanceof Error ? error.message : 'Failed to load hooks')
  } finally {
    loading.value = false
  }
}

function openCreateGroup(eventName = '') {
  editingTarget.value = null
  groupForm.value = createEmptyGroupForm(eventName)
  showModal.value = true
}

function openEditGroup(eventName: string, groupIndex: number) {
  const group = hooksConfig.value[eventName]?.[groupIndex]
  if (!group) return
  editingTarget.value = { event: eventName, groupIndex }
  groupForm.value = groupToForm(eventName, group)
  showModal.value = true
}

function closeModal() {
  showModal.value = false
}

function addHandlerForm() {
  groupForm.value.handlers.push(createEmptyHandlerForm())
}

function removeHandlerForm(index: number) {
  if (groupForm.value.handlers.length === 1) return
  groupForm.value.handlers.splice(index, 1)
}

async function persistHooks(nextHooks: HookMap, successMessage: string) {
  saving.value = true
  try {
    hooksConfig.value = await updateHooks<HookMap>(nextHooks)
    uiStore.showSuccess(successMessage)
    if (selectedEvent.value !== 'All' && !hooksConfig.value[selectedEvent.value]) selectedEvent.value = 'All'
  } catch (error) {
    logger.error('Failed to save hooks:', error)
    uiStore.showError(error instanceof Error ? error.message : 'Failed to save hooks')
    throw error
  } finally {
    saving.value = false
  }
}

async function saveGroup() {
  try {
    const { event, group } = buildGroupFromForm()
    const nextHooks = cloneHookMap(hooksConfig.value)

    if (editingTarget.value) {
      const { event: originalEvent, groupIndex } = editingTarget.value
      nextHooks[originalEvent]?.splice(groupIndex, 1)
      if ((nextHooks[originalEvent] ?? []).length === 0) delete nextHooks[originalEvent]
    }

    nextHooks[event] = [...(nextHooks[event] ?? []), group]
    await persistHooks(nextHooks, editingTarget.value ? 'Hook group updated successfully' : 'Hook group added successfully')
    closeModal()
  } catch (error) {
    uiStore.showError(error instanceof Error ? error.message : 'Failed to save hook group')
  }
}

async function handleDeleteGroup(eventName: string, groupIndex: number) {
  const confirmed = await uiStore.requestConfirm({
    title: 'Delete hook group',
    message: `Delete matcher group ${groupIndex + 1} from "${eventName}"?`,
    confirmText: 'Delete',
    cancelText: 'Cancel',
    type: 'danger',
  })
  if (!confirmed) return

  const nextHooks = cloneHookMap(hooksConfig.value)
  nextHooks[eventName]?.splice(groupIndex, 1)
  if ((nextHooks[eventName] ?? []).length === 0) delete nextHooks[eventName]
  await persistHooks(nextHooks, 'Hook group deleted successfully')
}

async function handleDeleteHandler(eventName: string, groupIndex: number, handlerIndex: number) {
  const confirmed = await uiStore.requestConfirm({
    title: 'Delete handler',
    message: `Delete handler ${handlerIndex + 1} from "${eventName}"?`,
    confirmText: 'Delete',
    cancelText: 'Cancel',
    type: 'danger',
  })
  if (!confirmed) return

  const nextHooks = cloneHookMap(hooksConfig.value)
  const group = nextHooks[eventName]?.[groupIndex]
  if (!group) return

  group.hooks.splice(handlerIndex, 1)
  if (group.hooks.length === 0) nextHooks[eventName]?.splice(groupIndex, 1)
  if ((nextHooks[eventName] ?? []).length === 0) delete nextHooks[eventName]
  await persistHooks(nextHooks, 'Handler deleted successfully')
}

onMounted(() => {
  loadHooks()
})
</script>

<style scoped>
.hooks-view {
  @apply min-h-full p-5 transition-colors duration-300;
}

.hooks-primary-button {
  @apply flex w-full items-center justify-center rounded-lg bg-accent-secondary px-4 py-2 font-medium text-white shadow-md;
  @apply transition-[color,background-color,border-color,transform] hover:scale-105 hover:shadow-lg sm:w-auto;

  min-height: 44px;
}

.hooks-secondary-button {
  @apply inline-flex items-center justify-center rounded-lg border border-accent-secondary/20 bg-accent-secondary/10 px-3 py-2 text-sm font-medium text-accent-secondary;
  @apply transition-colors hover:bg-accent-secondary/15;

  min-height: 44px;
}

.hooks-icon-button {
  @apply flex items-center justify-center rounded-md transition-colors;

  min-height: 44px;
  min-width: 44px;
}

.hooks-icon-button--accent {
  @apply text-accent-secondary hover:bg-accent-secondary/10;
}

.hooks-icon-button--danger {
  @apply text-accent-danger hover:bg-accent-danger/10;
}

.hooks-modal-overlay {
  @apply fixed inset-0 z-50 flex items-center justify-center bg-black/20 p-4 backdrop-blur-md;
}

.hooks-modal-panel {
  @apply relative max-h-[88vh] w-full max-w-4xl overflow-y-auto rounded-2xl border border-border-default bg-bg-elevated p-8 shadow-2xl;
}

.hooks-modal-close {
  @apply absolute right-4 top-4 flex items-center justify-center rounded-full text-text-muted transition-colors hover:bg-bg-surface;

  min-height: 44px;
  min-width: 44px;
}

.hooks-field-label {
  @apply mb-1.5 block text-sm font-semibold text-text-secondary;
}

.hooks-input {
  @apply w-full rounded-lg border border-border-default bg-bg-surface px-4 py-2.5 outline-none transition-colors;
  @apply focus:border-accent-secondary focus:ring-1 focus:ring-accent-secondary;
}

.hooks-input--mono {
  @apply font-mono text-sm;
}

.hooks-input--textarea {
  @apply resize-y py-3;
}

.hooks-checkbox {
  @apply flex cursor-pointer items-center gap-3;
}

.hooks-checkbox__label {
  @apply text-sm font-semibold text-text-secondary;
}

.hooks-footer-actions {
  @apply mt-8 flex gap-4 border-t border-border-default pt-6;
}

.hooks-footer-button {
  @apply flex-1 rounded-lg border border-border-default bg-bg-surface px-6 py-3 font-medium text-text-secondary transition-colors hover:bg-bg-elevated;

  min-height: 44px;
}

.hooks-footer-button--primary {
  @apply border-transparent bg-accent-secondary text-white shadow-md;
  @apply transition-[color,background-color,border-color,transform] hover:-translate-y-0.5 hover:shadow-lg;
}
</style>
