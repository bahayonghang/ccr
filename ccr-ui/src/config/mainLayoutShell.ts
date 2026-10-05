export interface MainLayoutNavItem {
  to: string
  labelKey: string
  icon: string
  iconClass?: string
}

export interface MainLayoutNavSection {
  id: string
  titleKey?: string
  items: MainLayoutNavItem[]
}

export const mainLayoutNavSections: MainLayoutNavSection[] = [
  {
    id: 'dashboard',
    items: [{ to: '/', labelKey: 'nav.dashboard', icon: 'Activity' }],
  },
  {
    id: 'workspace',
    titleKey: 'nav.configCenter',
    items: [
      {
        to: '/agent-sessions',
        labelKey: 'nav.agentSessions',
        icon: 'MessagesSquare',
        iconClass: 'text-accent-primary/85 group-hover:text-accent-primary transition-colors',
      },
      {
        to: '/mcp-manager',
        labelKey: 'nav.mcpManager',
        icon: 'Server',
        iconClass: 'text-accent-secondary/85 group-hover:text-accent-secondary transition-colors',
      },
    ],
  },
  {
    id: 'modules',
    titleKey: 'nav.mainModules',
    items: [
      {
        to: '/claude-code',
        labelKey: 'nav.claudeCode',
        icon: 'Code2',
        iconClass: 'text-platform-claude/90 group-hover:text-platform-claude transition-colors',
      },
      {
        to: '/codex',
        labelKey: 'nav.codex',
        icon: 'Settings',
        iconClass: 'text-platform-codex/90 group-hover:text-platform-codex transition-colors',
      },
      {
        to: '/grok',
        labelKey: 'nav.grok',
        icon: 'Zap',
        iconClass: 'text-platform-grok/90 group-hover:text-platform-grok transition-colors',
      },
      {
        to: '/antigravity',
        labelKey: 'nav.gemini',
        icon: 'Sparkles',
        iconClass: 'text-[color:color-mix(in_srgb,var(--color-platform-antigravity)_90%,transparent)] group-hover:text-[color:var(--color-platform-antigravity)] transition-colors',
      },
      {
        to: '/opencode',
        labelKey: 'nav.opencode',
        icon: 'TerminalSquare',
        iconClass: 'text-text-muted group-hover:text-text-primary transition-colors',
      },
    ],
  },
  {
    id: 'tools',
    titleKey: 'nav.toolsCenter',
    items: [
      { to: '/commands', labelKey: 'nav.commands', icon: 'Terminal' },
      { to: '/checkin', labelKey: 'nav.checkin', icon: 'ClipboardList' },
      { to: '/sync', labelKey: 'nav.sync', icon: 'Cloud' },
      { to: '/usage', labelKey: 'nav.usage', icon: 'Activity' },
    ],
  },
]

export const mainLayoutRouteTitleMap: Record<string, string> = {
  dashboard: 'nav.dashboard',
  settings: 'nav.settings',
  configs: 'nav.configs',
  skills: 'nav.skillsMigration',
  market: 'nav.skillsMigration',
  'claude-code': 'nav.claudeCode',
  'claude-code-auth': 'nav.auth',
  'claude-code-settings': 'common.settings',
  'claude-code-profiles': 'nav.profiles',
  codex: 'nav.codex',
  grok: 'nav.grok',
  'grok-auth': 'nav.auth',
  'grok-profiles': 'nav.profiles',
  'grok-settings': 'common.settings',
  'codex-mcp': 'nav.mcp',
  'codex-profiles': 'nav.profiles',
  'codex-agents': 'nav.agents',
  'codex-sessions': 'nav.sessions',
  'codex-slash-commands': 'nav.slashCommands',
  'codex-auth': 'nav.auth',
  'codex-settings': 'common.settings',
  antigravity: 'nav.gemini',
  'gemini-cli': 'nav.gemini',
  'gemini-mcp': 'nav.mcp',
  'gemini-agents': 'nav.agents',
  'gemini-slash-commands': 'nav.slashCommands',
  'gemini-plugins': 'nav.plugins',
  opencode: 'nav.opencode',
  'opencode-providers': 'nav.providers',
  'opencode-mcp': 'nav.mcp',
  'opencode-agents': 'nav.agents',
  'opencode-commands': 'nav.commands',
  'opencode-skills': 'nav.skillsMigration',
  'opencode-plugins': 'nav.plugins',
  'opencode-settings': 'common.settings',
  'ccr-control': 'nav.commands',
  commands: 'nav.commands',
  converter: 'nav.converter',
  sync: 'nav.sync',
  budget: 'nav.budget',
  pricing: 'nav.pricing',
  monitoring: 'nav.monitoring',
  mcp: 'nav.mcp',
  'mcp-unified': 'nav.mcpManager',
  'mcp-manager': 'nav.mcpManager',
  'agent-sessions': 'nav.agentSessions',
  'slash-commands': 'nav.slashCommands',
  agents: 'nav.agents',
  'agent-detail': 'nav.agents',
  plugins: 'nav.plugins',
  hooks: 'nav.hooks',
  'output-styles': 'nav.outputStyles',
  statusline: 'nav.statusline',
  checkin: 'nav.checkin',
  'checkin-account-dashboard': 'checkin.account_manager.dashboard',
  usage: 'nav.usage',
  'wsl-management': 'nav.wsl',
  'ssh-management': 'nav.ssh',
}

export const mainLayoutGroupTitleMap: Record<string, string> = {
  settings: 'nav.settings',
  tools: 'nav.toolsCenter',
  config: 'nav.configCenter',
  data: 'nav.dataCenter',
  environment: 'nav.environments',
  'claude-code': 'nav.claudeCode',
  codex: 'nav.codex',
  grok: 'nav.grok',
  gemini: 'nav.gemini',
  opencode: 'nav.opencode',
}

export const mainLayoutCachedViews = [
  'DashboardView',
  'ConfigsView',
  'CommandsView',
  'ClaudeCodeView',
  'CodexView',
  'ClaudeAuthView',
  'CodexAuthView',
  'GrokAuthView',
  'CodexProfilesView',
  'CodexMcpView',
  'CodexSessionsView',
  'GeminiCliView',
  'McpManagerView',
  'AgentSessionsView',
]
