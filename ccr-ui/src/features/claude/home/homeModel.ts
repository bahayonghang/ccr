import { t } from '@/features/claude/locale'

export const CURRENT_CONFIG_COMMAND = 'ccr current'
export const SWITCH_CONFIG_COMMAND = 'ccr switch'
export const LIST_CONFIGS_COMMAND = 'ccr list'

export function heroChips(): string[] {
  return [
    t('claudeCode.features.configManagement'),
    t('claudeCode.features.authReady'),
    t('claudeCode.features.localSettings'),
  ]
}

export function featureTags() {
  return [
    { label: t('claudeCode.features.mcpSupport'), icon: 'Server', className: 'border-accent-info/30 bg-accent-info/10 text-accent-info' },
    { label: t('claudeCode.features.aiAgents'), icon: 'Bot', className: 'border-accent-success/30 bg-accent-success/10 text-accent-success' },
    { label: t('claudeCode.features.slashCommands'), icon: 'Terminal', className: 'border-accent-warning/30 bg-accent-warning/10 text-accent-warning' },
    { label: t('claudeCode.features.localSettings'), icon: 'SlidersHorizontal', className: 'border-[color:var(--stage-chip-neutral-border)] bg-[var(--stage-chip-neutral-bg)] text-[color:var(--stage-text-secondary)]' },
  ]
}

export function coreModules() {
  return [
    {
      to: '/claude-code/profiles',
      icon: 'Settings',
      title: t('claudeCode.modules.profiles.title'),
      desc: t('claudeCode.modules.profiles.desc'),
      badge: t('claudeCode.modules.profiles.badge'),
      cardClass: 'border-accent-info/20 hover:border-accent-info/40',
    },
    {
      to: '/claude-code/auth',
      icon: 'KeyRound',
      title: t('claudeCode.modules.auth.title'),
      desc: t('claudeCode.modules.auth.desc'),
      badge: t('claudeCode.modules.auth.badge'),
      cardClass: 'border-accent-warning/20 hover:border-accent-warning/40',
    },
    {
      to: '/claude-code/settings',
      icon: 'SlidersHorizontal',
      title: t('claudeCode.modules.settings.title'),
      desc: t('claudeCode.modules.settings.desc'),
      badge: t('claudeCode.modules.settings.badge'),
      cardClass: 'border-accent-secondary/20 hover:border-accent-secondary/40',
    },
  ]
}

export function extensionModules() {
  return [
    {
      to: '/mcp-manager',
      icon: 'Server',
      title: t('claudeCode.modules.mcpServers.title'),
      desc: t('claudeCode.modules.mcpServers.desc'),
      badge: t('claudeCode.modules.mcpServers.badge'),
      iconClass: 'bg-accent-info/10 text-accent-info',
    },
    {
      to: '/agents',
      icon: 'Users',
      title: t('claudeCode.modules.agents.title'),
      desc: t('claudeCode.modules.agents.desc'),
      badge: t('claudeCode.modules.agents.badge'),
      iconClass: 'bg-accent-success/10 text-accent-success',
    },
    {
      to: '/plugins',
      icon: 'Puzzle',
      title: t('claudeCode.modules.plugins.title'),
      desc: t('claudeCode.modules.plugins.desc'),
      badge: t('claudeCode.modules.plugins.badge'),
      iconClass: 'bg-accent-primary/10 text-accent-primary',
    },
    {
      to: '/slash-commands',
      icon: 'Terminal',
      title: t('claudeCode.modules.slashCommands.title'),
      desc: t('claudeCode.modules.slashCommands.desc'),
      badge: t('claudeCode.modules.slashCommands.badge'),
      iconClass: 'bg-accent-warning/10 text-accent-warning',
    },
  ]
}

export function commonCommands() {
  return [
    { label: t('claudeCode.quickActions.viewCurrentConfig'), cmd: CURRENT_CONFIG_COMMAND },
    { label: t('claudeCode.quickActions.switchConfig'), cmd: SWITCH_CONFIG_COMMAND },
    { label: t('claudeCode.quickActions.listAllConfigs'), cmd: LIST_CONFIGS_COMMAND },
  ]
}

export function resources() {
  return [
    { label: t('claudeCode.quickActions.officialDocs'), url: 'https://docs.anthropic.com/en/docs/claude-code', icon: 'BookOpen' },
    { label: t('claudeCode.quickActions.mcpProtocol'), url: 'https://modelcontextprotocol.io', icon: 'Server' },
    { label: t('claudeCode.quickActions.settingsReference'), url: 'https://docs.anthropic.com/en/docs/claude-code/settings', icon: 'SlidersHorizontal' },
  ]
}
