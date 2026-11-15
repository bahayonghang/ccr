import type { CommandInfo } from '@/types'
import type { CliClient } from '@/types/router'
import type { TranslateFunction } from '@/utils/tf'

export interface CommandClient {
  id: CliClient
  name: string
  icon: string
  executable: boolean
}

export interface CommandUiInfo extends CommandInfo {
  category: string
  dangerous: boolean
  readOnly: boolean
  requiresArgs: boolean
  executable: boolean
}

export type CommandBadge = 'safe' | 'danger' | 'readonly' | 'args' | 'blocked'
export type LedgerChannel = 'stdout' | 'stderr' | 'system'
export type CommandCollection = 'catalog' | 'favorites' | 'history'

export const MAX_LEDGER_LINES = 2000
export const CLI_CLIENTS: CommandClient[] = [
  { id: 'ccr', name: 'CCR', icon: 'Zap', executable: true },
  { id: 'claude', name: 'Claude Code', icon: 'Code2', executable: false },
  { id: 'gemini', name: 'Antigravity CLI', icon: 'Sparkles', executable: false },
]

export const fallbackCommandRegistry: Record<CliClient, CommandInfo[]> = {
  ccr: [
    { name: 'status', description: 'Inspect current CCR status.', usage: 'ccr status', examples: ['ccr status', 'ccr status --json'], category: 'read', risk: 'safe', executable: true },
    { name: 'show', description: 'Compatibility alias for the current status view.', usage: 'ccr show', examples: ['ccr show', 'ccr show --json'], category: 'read', risk: 'safe', executable: true },
    { name: 'validate', description: 'Validate global registry and Claude, Codex, and Grok configurations and runtime settings.', usage: 'ccr validate', examples: ['ccr validate', 'ccr validate --json'], category: 'diagnostic', risk: 'safe', executable: true },
    { name: 'history', description: 'Inspect recent CCR configuration operations.', usage: 'ccr history -l 20', examples: ['ccr history', 'ccr history -l 50 -t switch'], category: 'diagnostic', risk: 'safe', executable: true },
    { name: 'version', description: 'Inspect the installed CCR version.', usage: 'ccr version', examples: ['ccr version'], category: 'read', risk: 'safe', executable: true },
    { name: 'help', description: 'Show CCR help for a command path.', usage: 'ccr help <command>', examples: ['ccr help', 'ccr help validate'], category: 'read', risk: 'safe', executable: true },
  ],
  claude: [
    { name: 'help', description: 'Preview only. Claude Code execution is not wired to the CCR whitelist.', usage: 'claude --help', examples: ['claude --help'], category: 'blocked' },
    { name: 'version', description: 'Preview only. Claude Code execution is not wired to the CCR whitelist.', usage: 'claude --version', examples: ['claude --version'], category: 'blocked' },
  ],
  gemini: [
    { name: 'help', description: 'Preview only. Antigravity CLI execution is not wired to the CCR whitelist.', usage: 'agy --help', examples: ['agy --help'], category: 'blocked' },
    { name: 'version', description: 'Preview only. Antigravity CLI execution is not wired to the CCR whitelist.', usage: 'agy --version', examples: ['agy --version'], category: 'blocked' },
  ],
}

function riskOf(command: CommandInfo): string {
  return command.risk ?? (command.category === 'danger' ? 'destructive' : 'safe')
}

function categoryOf(command: CommandInfo, dangerous: boolean, risk: string): string {
  if (command.category) return command.category
  if (dangerous) return 'danger'
  if (risk === 'writes_config') return 'write'
  return 'read'
}

export function normalizeCommand(command: CommandInfo, client: CliClient, t: TranslateFunction): CommandUiInfo {
  const risk = riskOf(command)
  const executable = client === 'ccr' ? command.executable ?? true : false
  const dangerous = Boolean(command.requiresConfirmation) || risk === 'destructive'
  const category = categoryOf(command, dangerous, risk)
  const clientLabel = CLI_CLIENTS.find((item) => item.id === client)?.name ?? client
  const description = client === 'ccr' && command.description
    ? command.description
    : t('commands.clientPreviewCommandDescription', { client: clientLabel })
  return {
    ...command,
    description,
    usage: command.usage || `ccr ${command.name}`,
    examples: command.examples || [`ccr ${command.name}`],
    category,
    dangerous,
    readOnly: risk === 'safe' || category === 'read' || category === 'diagnostic',
    requiresArgs: command.args?.some((arg) => arg.required) ?? false,
    executable,
  }
}

export function commandBadges(command: CommandUiInfo): CommandBadge[] {
  const badges: CommandBadge[] = []
  if (!command.executable) badges.push('blocked')
  if (command.dangerous) badges.push('danger')
  if (command.readOnly) badges.push('readonly')
  if (command.requiresArgs) badges.push('args')
  if (badges.length === 0) badges.push('safe')
  return badges
}

export function splitArgs(value: string): string[] {
  return value.split(' ').map((arg) => arg.trim()).filter((arg) => arg.length > 0)
}

export function resolvedCommandName(command: string): string {
  return command.trim().split(/\s+/)[0] ?? ''
}
