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
    id: 'home',
    items: [{ to: '/', labelKey: 'nav.home', icon: 'Home' }],
  },
  {
    id: 'skills',
    titleKey: 'nav.skillsHub',
    items: [
      {
        to: '/skills',
        labelKey: 'nav.skills',
        icon: 'Package',
        iconClass: 'text-accent-primary/85 group-hover:text-accent-primary transition-colors',
      },
      {
        to: '/skills/add',
        labelKey: 'nav.addSkill',
        icon: 'PlusCircle',
        iconClass: 'text-accent-primary/85 group-hover:text-accent-primary transition-colors',
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
        to: '/gemini-cli',
        labelKey: 'nav.gemini',
        icon: 'Sparkles',
        iconClass: 'text-platform-gemini/90 group-hover:text-platform-gemini transition-colors',
      },
      {
        to: '/droid',
        labelKey: 'nav.droid',
        icon: 'Bot',
        iconClass: 'text-accent-secondary/90 group-hover:text-accent-secondary transition-colors',
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
      { to: '/ccr-control', labelKey: 'nav.ccrControl', icon: 'Terminal' },
      { to: '/commands', labelKey: 'nav.commands', icon: 'Terminal' },
      { to: '/checkin', labelKey: 'nav.checkin', icon: 'ClipboardList' },
      { to: '/sync', labelKey: 'nav.sync', icon: 'Cloud' },
      { to: '/usage', labelKey: 'nav.usage', icon: 'Activity' },
    ],
  },
]

export const mainLayoutRouteTitleMap: Record<string, string> = {
  home: 'nav.home',
  settings: 'nav.settings',
  configs: 'nav.configs',
  skills: 'nav.skills',
  'skills-add': 'nav.addSkill',
  market: 'nav.market',
  'skill-detail': 'nav.skills',
  'claude-code': 'nav.claudeCode',
  'claude-code-auth': 'nav.auth',
  'claude-code-settings': 'common.settings',
  'claude-code-profiles': 'nav.profiles',
  codex: 'nav.codex',
  'codex-mcp': 'nav.mcp',
  'codex-profiles': 'nav.profiles',
  'codex-agents': 'nav.agents',
  'codex-sessions': 'nav.sessions',
  'codex-slash-commands': 'nav.slashCommands',
  'codex-auth': 'nav.auth',
  'codex-settings': 'common.settings',
  'gemini-cli': 'nav.gemini',
  'gemini-mcp': 'nav.mcp',
  'gemini-agents': 'nav.agents',
  'gemini-slash-commands': 'nav.slashCommands',
  'gemini-plugins': 'nav.plugins',
  droid: 'nav.droid',
  'droid-mcp': 'nav.mcp',
  'droid-agents': 'nav.agents',
  'droid-slash-commands': 'nav.slashCommands',
  'droid-plugins': 'nav.plugins',
  'droid-models': 'nav.models',
  'droid-profiles': 'nav.profiles',
  'droid-droids': 'nav.droids',
  opencode: 'nav.opencode',
  'opencode-providers': 'nav.providers',
  'opencode-mcp': 'nav.mcp',
  'opencode-agents': 'nav.agents',
  'opencode-commands': 'nav.commands',
  'opencode-skills': 'nav.skills',
  'opencode-plugins': 'nav.plugins',
  'opencode-settings': 'common.settings',
  'ccr-control': 'nav.ccrControl',
  commands: 'nav.commands',
  converter: 'nav.converter',
  sync: 'nav.sync',
  budget: 'nav.budget',
  pricing: 'nav.pricing',
  monitoring: 'nav.monitoring',
  mcp: 'nav.mcp',
  'mcp-unified': 'nav.unifiedMcp',
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
  skills: 'nav.skillsHub',
  tools: 'nav.toolsCenter',
  config: 'nav.configCenter',
  data: 'nav.dataCenter',
  environment: 'nav.environments',
  'claude-code': 'nav.claudeCode',
  codex: 'nav.codex',
  gemini: 'nav.gemini',
  droid: 'nav.droid',
  opencode: 'nav.opencode',
}

export const mainLayoutCachedViews = [
  'HomeView',
  'ConfigsView',
  'CommandsView',
  'ClaudeCodeView',
  'CodexView',
  'ClaudeAuthView',
  'CodexAuthView',
  'CodexProfilesView',
  'CodexMcpView',
  'CodexSessionsView',
  'GeminiCliView',
]
