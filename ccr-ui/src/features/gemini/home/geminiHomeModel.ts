import { t } from '../locale'

export type GeminiTagTone = 'gemini' | 'command' | 'neutral' | 'capability'
export type GeminiModuleTone = 'gemini' | 'command' | 'capability' | 'plugin'

export interface GeminiHeroTag {
  key: string
  icon: string
  label: string
  tone: GeminiTagTone
}

export interface GeminiModuleCard {
  key: string
  to: string
  icon: string
  tone: GeminiModuleTone
  title: string
  description: string
  badge: string
  hint: string
  status: string
  spotlight?: boolean
}

export interface GeminiTerminalSnippet {
  label: string
  command: string
}

export interface GeminiQuickCard {
  key: string
  icon: string
  kicker: string
  title: string
  items: string[]
}

export const geminiHeroTags = (): GeminiHeroTag[] => [
  { key: 'mcp', icon: 'Server', label: t('gemini.overview.tags.mcp'), tone: 'gemini' },
  { key: 'commands', icon: 'Command', label: t('gemini.overview.tags.commands'), tone: 'command' },
  { key: 'settings', icon: 'Settings', label: t('gemini.overview.tags.settings'), tone: 'neutral' },
  { key: 'boundary', icon: 'ShieldCheck', label: t('gemini.overview.tags.boundary'), tone: 'capability' },
]

export const geminiTerminalSnippets = (): GeminiTerminalSnippet[] => [
  { label: t('gemini.overview.terminal.helpLabel'), command: 'agy --help' },
  { label: t('gemini.overview.terminal.versionLabel'), command: 'agy --version' },
  { label: t('gemini.overview.terminal.importLabel'), command: 'agy plugin import gemini' },
]

export const geminiConfigPreview = () => [
  { label: t('gemini.overview.terminal.settingsPath'), value: '~/.gemini/antigravity-cli/settings.json' },
  { label: t('gemini.overview.terminal.mcpPath'), value: '~/.gemini/antigravity-cli/mcp_config.json' },
  { label: t('gemini.overview.terminal.skillsPath'), value: '~/.gemini/antigravity-cli/skills' },
  { label: t('gemini.overview.terminal.workspacePath'), value: '.agents/{mcp_config.json,skills}' },
]

export const geminiModuleCards = (): GeminiModuleCard[] => [
  {
    key: 'mcp',
    to: '/antigravity/mcp',
    icon: 'Server',
    tone: 'gemini',
    spotlight: true,
    title: t('gemini.mcp.title'),
    description: t('gemini.overview.modules.mcpDescription'),
    badge: t('gemini.overview.modules.supportedBadge'),
    hint: t('gemini.overview.modules.mcpHint'),
    status: t('gemini.overview.modules.mcpStatus'),
  },
  {
    key: 'slash-commands',
    to: '/antigravity/slash-commands',
    icon: 'Command',
    tone: 'command',
    spotlight: true,
    title: t('gemini.slashCommands.title'),
    description: t('gemini.overview.modules.commandsDescription'),
    badge: t('gemini.overview.modules.supportedBadge'),
    hint: t('gemini.overview.modules.commandsHint'),
    status: t('gemini.overview.modules.commandsStatus'),
  },
  {
    key: 'agents',
    to: '/antigravity/agents',
    icon: 'Bot',
    tone: 'capability',
    title: t('gemini.agents.title'),
    description: t('gemini.overview.modules.agentsDescription'),
    badge: t('gemini.overview.modules.boundaryBadge'),
    hint: t('gemini.overview.modules.agentsHint'),
    status: t('gemini.overview.modules.agentsStatus'),
  },
  {
    key: 'plugins',
    to: '/antigravity/plugins',
    icon: 'Puzzle',
    tone: 'plugin',
    title: t('gemini.plugins.title'),
    description: t('gemini.overview.modules.pluginsDescription'),
    badge: t('gemini.overview.modules.boundaryBadge'),
    hint: t('gemini.overview.modules.pluginsHint'),
    status: t('gemini.overview.modules.pluginsStatus'),
  },
]

export const geminiQuickCards = (): GeminiQuickCard[] => [
  {
    key: 'paths',
    icon: 'Workflow',
    kicker: t('gemini.overview.quick.pathsKicker'),
    title: t('gemini.overview.quick.pathsTitle'),
    items: [t('gemini.overview.quick.pathMcp'), t('gemini.overview.quick.pathCommands'), t('gemini.overview.quick.pathSkills')],
  },
  {
    key: 'config',
    icon: 'FolderOpen',
    kicker: t('gemini.overview.quick.configKicker'),
    title: t('gemini.overview.quick.configTitle'),
    items: [
      t('gemini.overview.quick.configSettings'),
      t('gemini.overview.quick.configProjectCommands'),
      t('gemini.overview.quick.configUserCommands'),
    ],
  },
  {
    key: 'tips',
    icon: 'Lightbulb',
    kicker: t('gemini.overview.quick.tipsKicker'),
    title: t('gemini.overview.quick.tipsTitle'),
    items: [
      t('gemini.overview.quick.tipSafeCommands'),
      t('gemini.overview.quick.tipBoundaries'),
      t('gemini.overview.quick.tipNoBackendChange'),
    ],
  },
]

export const geminiTagClass: Record<GeminiTagTone, string> = {
  gemini: 'border-[color:color-mix(in_srgb,var(--platform-gemini)_22%,transparent)] bg-[color:color-mix(in_srgb,var(--platform-gemini)_10%,transparent)] text-[color:var(--platform-gemini)]',
  command: 'border-accent-secondary/22 bg-accent-secondary/10 text-accent-secondary',
  neutral: 'border-[color:var(--stage-chip-neutral-border)] bg-[var(--stage-chip-neutral-bg)] text-[color:var(--stage-chip-neutral-text)]',
  capability: 'border-accent-warning/22 bg-accent-warning/10 text-accent-warning',
}

export const geminiModuleClass: Record<GeminiModuleTone, string> = {
  gemini: 'border-[color:color-mix(in_srgb,var(--platform-gemini)_22%,transparent)]',
  command: 'border-accent-secondary/22',
  capability: 'border-accent-warning/22',
  plugin: 'border-accent-info/22',
}
