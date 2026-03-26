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
        iconClass: 'text-fuchsia-400 group-hover:text-fuchsia-300 transition-colors',
      },
      {
        to: '/skills/add',
        labelKey: 'nav.addSkill',
        icon: 'PlusCircle',
        iconClass: 'text-fuchsia-400 group-hover:text-fuchsia-300 transition-colors',
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
        iconClass: 'text-platform-claude group-hover:text-platform-claude/80 transition-colors',
      },
      {
        to: '/codex',
        labelKey: 'nav.codex',
        icon: 'Settings',
        iconClass: 'text-platform-codex group-hover:text-platform-codex/80 transition-colors',
      },
      {
        to: '/gemini-cli',
        labelKey: 'nav.gemini',
        icon: 'Sparkles',
        iconClass: 'text-platform-gemini group-hover:text-platform-gemini/80 transition-colors',
      },
      {
        to: '/qwen',
        labelKey: 'nav.qwen',
        icon: 'Zap',
        iconClass: 'text-platform-qwen group-hover:text-platform-qwen/80 transition-colors',
      },
      {
        to: '/qoder',
        labelKey: 'nav.qoder',
        icon: 'Activity',
        iconClass: 'text-platform-qoder group-hover:text-platform-qoder/80 transition-colors',
      },
      {
        to: '/droid',
        labelKey: 'nav.droid',
        icon: 'Bot',
        iconClass: 'text-accent-secondary group-hover:text-accent-secondary/80 transition-colors',
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
  configs: 'nav.configs',
  skills: 'nav.skills',
  'skills-add': 'nav.addSkill',
  market: 'nav.market',
  'skill-detail': 'nav.skills',
  'claude-code': 'nav.claudeCode',
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
  qwen: 'nav.qwen',
  'qwen-mcp': 'nav.mcp',
  'qwen-agents': 'nav.agents',
  'qwen-slash-commands': 'nav.slashCommands',
  'qwen-plugins': 'nav.plugins',
  qoder: 'nav.qoder',
  'qoder-mcp': 'nav.mcp',
  'qoder-subagents': 'nav.subagents',
  'qoder-commands': 'nav.commands',
  'qoder-hooks': 'nav.hooks',
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
  'opencode-plugins': 'nav.plugins',
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
  skills: 'nav.skillsHub',
  tools: 'nav.toolsCenter',
  config: 'nav.configCenter',
  data: 'nav.dataCenter',
  environment: 'nav.environments',
  'claude-code': 'nav.claudeCode',
  codex: 'nav.codex',
  gemini: 'nav.gemini',
  qwen: 'nav.qwen',
  qoder: 'nav.qoder',
  droid: 'nav.droid',
  opencode: 'nav.opencode',
}

export const mainLayoutCachedViews = [
  'HomeView',
  'ConfigsView',
  'CommandsView',
  'ClaudeCodeView',
  'CodexView',
  'CodexAuthView',
  'CodexProfilesView',
  'CodexMcpView',
  'CodexSessionsView',
  'GeminiCliView',
  'QwenView',
  'QoderView',
]
