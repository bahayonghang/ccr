export interface ModuleSubnavItem {
  label: string
  labelKey?: string
  href: string
  icon: string
}

const moduleSubnavMap: Record<string, ModuleSubnavItem[]> = {
  'claude-code': [
    { label: 'Configurations', labelKey: 'nav.configs', href: '/configs', icon: 'Settings' },
    { label: 'Auth', labelKey: 'nav.auth', href: '/claude-code/auth', icon: 'KeyRound' },
    { label: 'Profiles', labelKey: 'nav.profiles', href: '/claude-code/profiles', icon: 'Settings' },
    { label: 'Sync', labelKey: 'nav.sync', href: '/sync', icon: 'Cloud' },
    { label: 'MCP', labelKey: 'nav.mcp', href: '/mcp-manager', icon: 'Server' },
    { label: 'Slash Commands', labelKey: 'nav.slashCommands', href: '/slash-commands', icon: 'Command' },
    { label: 'Agents', labelKey: 'nav.agents', href: '/agents', icon: 'Bot' },
    { label: 'Plugins', labelKey: 'nav.plugins', href: '/plugins', icon: 'Puzzle' },
  ],
  codex: [
    { label: 'Auth', labelKey: 'nav.auth', href: '/codex/auth', icon: 'KeyRound' },
    { label: 'Profiles', labelKey: 'nav.profiles', href: '/codex/profiles', icon: 'Folders' },
    { label: 'Settings', labelKey: 'common.settings', href: '/codex/settings', icon: 'SlidersHorizontal' },
    { label: 'MCP', labelKey: 'nav.mcp', href: '/codex/mcp', icon: 'Server' },
    { label: 'Agents', labelKey: 'nav.agents', href: '/codex/agents', icon: 'Bot' },
    { label: 'Sessions', labelKey: 'nav.sessions', href: '/codex/sessions', icon: 'MessagesSquare' },
  ],
  antigravity: [
    { label: 'MCP', labelKey: 'nav.mcp', href: '/antigravity/mcp', icon: 'Server' },
    { label: 'Agents', labelKey: 'nav.agents', href: '/antigravity/agents', icon: 'Bot' },
    { label: 'Slash Commands', labelKey: 'nav.slashCommands', href: '/antigravity/slash-commands', icon: 'Command' },
    { label: 'Plugins', labelKey: 'nav.plugins', href: '/antigravity/plugins', icon: 'Puzzle' },
  ],
  'gemini-cli': [
    { label: 'MCP', labelKey: 'nav.mcp', href: '/antigravity/mcp', icon: 'Server' },
    { label: 'Agents', labelKey: 'nav.agents', href: '/antigravity/agents', icon: 'Bot' },
    { label: 'Slash Commands', labelKey: 'nav.slashCommands', href: '/antigravity/slash-commands', icon: 'Command' },
    { label: 'Plugins', labelKey: 'nav.plugins', href: '/antigravity/plugins', icon: 'Puzzle' },
  ],
  opencode: [
    { label: 'Providers', labelKey: 'nav.providers', href: '/opencode/providers', icon: 'Layers' },
    { label: 'MCP', labelKey: 'nav.mcp', href: '/opencode/mcp', icon: 'Server' },
    { label: 'Agents', labelKey: 'nav.agents', href: '/opencode/agents', icon: 'Bot' },
    { label: 'Commands', labelKey: 'nav.commands', href: '/opencode/commands', icon: 'Command' },
    { label: 'Plugins', labelKey: 'nav.plugins', href: '/opencode/plugins', icon: 'Puzzle' },
    { label: 'Settings', labelKey: 'common.settings', href: '/opencode/settings', icon: 'SlidersHorizontal' },
  ],
  converter: [{ label: 'Converter', labelKey: 'nav.converter', href: '/converter', icon: 'ArrowLeftRight' }],
}

export const getModuleSubnavItems = (module: string): ModuleSubnavItem[] => {
  return moduleSubnavMap[module] ?? []
}
