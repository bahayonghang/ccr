import { z } from 'zod'
import type { Hook, HookMap, HookMatcherGroup } from '@/types'
import { tt } from '@/features/claude/locale'

export const ALL_EVENT_KEY = 'All'
export const KNOWN_HOOK_EVENTS = [
  'PermissionRequest', 'PostToolUse', 'PostToolUseFailure', 'PreToolUse', 'Stop', 'SubagentStop',
  'TaskCompleted', 'UserPromptSubmit', 'ConfigChange', 'Elicitation', 'ElicitationResult',
  'InstructionsLoaded', 'Notification', 'PostCompact', 'PreCompact', 'SessionEnd', 'SessionStart',
  'StopFailure', 'SubagentStart', 'TeammateIdle', 'WorktreeCreate', 'WorktreeRemove',
]
export const KNOWN_HANDLER_TYPES = ['command', 'http', 'prompt', 'agent']

export const hookHandlerSchema = z.object({
  type: z.string(),
  command: z.string(),
  url: z.string(),
  prompt: z.string(),
  model: z.string(),
  timeout: z.string(),
  statusMessage: z.string(),
  headersJson: z.string(),
  allowedEnvVarsText: z.string(),
  asyncEnabled: z.boolean(),
  extraJson: z.string(),
})

export const hookGroupSchema = z.object({
  event: z.string(),
  matcher: z.string(),
  groupExtraJson: z.string(),
  handlers: z.array(hookHandlerSchema).min(1),
})

export type HookHandlerForm = z.infer<typeof hookHandlerSchema>
export type HookGroupForm = z.infer<typeof hookGroupSchema>

export function emptyHandlerForm(type = 'command'): HookHandlerForm {
  return {
    type,
    command: '',
    url: '',
    prompt: '',
    model: '',
    timeout: '',
    statusMessage: '',
    headersJson: '',
    allowedEnvVarsText: '',
    asyncEnabled: false,
    extraJson: '',
  }
}

export function emptyGroupForm(event = ''): HookGroupForm {
  return { event, matcher: '', groupExtraJson: '', handlers: [emptyHandlerForm()] }
}

export function cloneHookMap(source: HookMap): HookMap {
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
    throw new Error(tt(`${label} 必须是 JSON 对象`, `${label} must be a JSON object`))
  }
  return parsed as Record<string, unknown>
}

function parseTimeout(timeout: string): number | undefined {
  const trimmed = timeout.trim()
  if (!trimmed) return undefined
  const value = Number.parseInt(trimmed, 10)
  if (!Number.isFinite(value) || value < 0) {
    throw new Error(tt('超时必须是非负整数', 'Timeout must be a non-negative integer'))
  }
  return value
}

export function groupExtraKeys(group: HookMatcherGroup): string[] {
  return Object.keys(group).filter((key) => key !== 'matcher' && key !== 'hooks')
}

export function handlerExtraKeys(handler: Hook): string[] {
  return Object.keys(handler).filter(
    (key) => !['type', 'command', 'url', 'prompt', 'model', 'timeout', 'statusMessage', 'allowedEnvVars', 'headers', 'async'].includes(key),
  )
}

export function getEventColor(eventName: string): string {
  const palette: Record<string, string> = {
    PreToolUse: 'border-accent-secondary/20 bg-accent-secondary/10 text-accent-secondary',
    PostToolUse: 'border-accent-success/20 bg-accent-success/10 text-accent-success',
    Stop: 'border-accent-danger/20 bg-accent-danger/10 text-accent-danger',
    UserPromptSubmit: 'border-accent-primary/20 bg-accent-primary/10 text-accent-primary',
    Notification: 'border-accent-warning/20 bg-accent-warning/10 text-accent-warning',
  }
  return palette[eventName] || 'border-border-default bg-bg-elevated text-text-secondary'
}

export function getHandlerSummary(handler: Hook): string {
  if (handler.type === 'command') return handler.command || tt('(缺少命令)', '(missing command)')
  if (handler.type === 'http') return handler.url || tt('(缺少 URL)', '(missing url)')
  if (handler.type === 'prompt' || handler.type === 'agent') return handler.prompt || tt('(缺少提示词)', '(missing prompt)')
  return JSON.stringify(handler)
}

export function handlerToForm(handler: Hook): HookHandlerForm {
  const { type, command, url, prompt, model, timeout, statusMessage, allowedEnvVars, headers, async: asyncFlag, ...other } = handler
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
    asyncEnabled: asyncFlag === true,
    extraJson: formatJsonObject(other),
  }
}

export function groupToForm(eventName: string, group: HookMatcherGroup): HookGroupForm {
  const { matcher, hooks, ...other } = group
  return {
    event: eventName,
    matcher: matcher ?? '',
    groupExtraJson: formatJsonObject(other),
    handlers: hooks.length > 0 ? hooks.map(handlerToForm) : [emptyHandlerForm()],
  }
}

function parseHeaders(headersJson: string): Record<string, string> | undefined {
  const headers = parseJsonObject(headersJson, 'Headers JSON')
  const entries = Object.entries(headers)
  return entries.length > 0 ? Object.fromEntries(entries.map(([key, value]) => [key, String(value)])) : undefined
}

function parseAllowedEnvVars(input: string): string[] | undefined {
  const values = input.split(',').map((value) => value.trim()).filter(Boolean)
  return values.length > 0 ? values : undefined
}

export function buildHandler(handlerForm: HookHandlerForm): Hook {
  const type = handlerForm.type.trim()
  if (!type) throw new Error(tt('处理器类型不能为空', 'Handler type is required'))
  const extra = parseJsonObject(handlerForm.extraJson, 'Handler advanced JSON')
  const handler: Hook = { ...extra, type }
  const command = handlerForm.command.trim()
  const url = handlerForm.url.trim()
  const prompt = handlerForm.prompt.trim()
  const model = handlerForm.model.trim()
  const statusMessage = handlerForm.statusMessage.trim()
  if (type === 'command') {
    if (!command) throw new Error(tt('命令型处理器必须填写命令', 'Command handlers require a command'))
    handler.command = command
    if (handlerForm.asyncEnabled) handler.async = true
  } else if (type === 'http') {
    if (!url) throw new Error(tt('HTTP 处理器必须填写 URL', 'HTTP handlers require a URL'))
    handler.url = url
    const headers = parseHeaders(handlerForm.headersJson)
    if (headers) handler.headers = headers
    const allowedEnvVars = parseAllowedEnvVars(handlerForm.allowedEnvVarsText)
    if (allowedEnvVars) handler.allowedEnvVars = allowedEnvVars
    if (handlerForm.asyncEnabled) handler.async = true
  } else {
    if (!prompt) throw new Error(tt(`${type} 处理器必须填写提示词`, `${type} handlers require a prompt`))
    handler.prompt = prompt
    if (model) handler.model = model
  }
  const timeout = parseTimeout(handlerForm.timeout)
  if (timeout != null) handler.timeout = timeout
  if (statusMessage) handler.statusMessage = statusMessage
  return handler
}

export function buildGroupFromForm(form: HookGroupForm): { event: string; group: HookMatcherGroup } {
  const eventName = form.event.trim()
  if (!eventName) throw new Error(tt('事件不能为空', 'Event is required'))
  if (form.handlers.length === 0) throw new Error(tt('至少需要一个处理器', 'At least one handler is required'))
  const groupExtra = parseJsonObject(form.groupExtraJson, 'Group advanced JSON')
  const matcher = form.matcher.trim()
  const group: HookMatcherGroup = { ...groupExtra, hooks: form.handlers.map(buildHandler) }
  if (matcher) group.matcher = matcher
  return { event: eventName, group }
}

export function groupKey(eventName: string, group: HookMatcherGroup): string {
  return `${eventName}::${JSON.stringify(group)}`
}

export function applyEditedGroup(input: {
  source: HookMap
  editing: { event: string; groupIndex: number } | null
  event: string
  group: HookMatcherGroup
}): HookMap {
  const nextHooks = cloneHookMap(input.source)
  if (!input.editing) {
    nextHooks[input.event] = [...(nextHooks[input.event] ?? []), input.group]
    return nextHooks
  }
  nextHooks[input.editing.event]?.splice(input.editing.groupIndex, 1)
  const leftover = nextHooks[input.editing.event] ?? []
  if (leftover.length === 0) delete nextHooks[input.editing.event]
  nextHooks[input.event] = [...(nextHooks[input.event] ?? []), input.group]
  return nextHooks
}
