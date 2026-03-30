export interface ModuleSubnavItem {
  label: string
  href: string
  icon: string
}

const moduleSubnavMap: Record<string, ModuleSubnavItem[]> = {
  'claude-code': [
    { label: 'Configs', href: '/configs', icon: 'Settings' },
    { label: 'Profiles 配置', href: '/claude-code/profiles', icon: 'Settings' },
    { label: '云同步', href: '/sync', icon: 'Cloud' },
    { label: 'MCP', href: '/mcp', icon: 'Server' },
    { label: 'Slash Commands', href: '/slash-commands', icon: 'Command' },
    { label: 'Agents', href: '/agents', icon: 'Bot' },
    { label: 'Skills', href: '/skills', icon: 'Book' },
    { label: 'Plugins', href: '/plugins', icon: 'Puzzle' },
  ],
  codex: [
    { label: 'Auth', href: '/codex/auth', icon: 'KeyRound' },
    { label: 'Profiles', href: '/codex/profiles', icon: 'Folders' },
    { label: 'Settings', href: '/codex/settings', icon: 'SlidersHorizontal' },
    { label: 'MCP', href: '/codex/mcp', icon: 'Server' },
    { label: 'Agents', href: '/codex/agents', icon: 'Bot' },
    { label: 'Sessions', href: '/codex/sessions', icon: 'MessagesSquare' },
  ],
  'gemini-cli': [
    { label: 'MCP', href: '/gemini-cli/mcp', icon: 'Server' },
    { label: 'Agents', href: '/gemini-cli/agents', icon: 'Bot' },
    { label: 'Slash Commands', href: '/gemini-cli/slash-commands', icon: 'Command' },
    { label: 'Plugins', href: '/gemini-cli/plugins', icon: 'Puzzle' },
  ],
  qwen: [
    { label: 'MCP', href: '/qwen/mcp', icon: 'Server' },
    { label: 'Agents', href: '/qwen/agents', icon: 'Bot' },
    { label: 'Slash Commands', href: '/qwen/slash-commands', icon: 'Command' },
    { label: 'Plugins', href: '/qwen/plugins', icon: 'Puzzle' },
  ],
  qoder: [
    { label: 'MCP', href: '/qoder/mcp', icon: 'Server' },
    { label: 'Subagents', href: '/qoder/subagents', icon: 'Bot' },
    { label: 'Commands', href: '/qoder/commands', icon: 'Command' },
    { label: 'Hooks', href: '/qoder/hooks', icon: 'Webhook' },
  ],
  droid: [
    { label: 'MCP', href: '/droid/mcp', icon: 'Server' },
    { label: 'Agents', href: '/droid/agents', icon: 'Bot' },
    { label: 'Slash Commands', href: '/droid/slash-commands', icon: 'Command' },
    { label: 'Plugins', href: '/droid/plugins', icon: 'Puzzle' },
  ],
  opencode: [
    { label: 'Providers', href: '/opencode/providers', icon: 'Layers' },
    { label: 'MCP', href: '/opencode/mcp', icon: 'Server' },
    { label: 'Plugins', href: '/opencode/plugins', icon: 'Puzzle' },
    { label: 'Skills', href: '/skills', icon: 'Book' },
  ],
  skills: [
    { label: 'Skills 库', href: '/skills', icon: 'Package' },
    { label: '添加 Skill', href: '/skills/add', icon: 'PlusCircle' },
    { label: 'Market', href: '/skills?tab=marketplace', icon: 'Store' },
  ],
  converter: [{ label: 'CLI 配置转换', href: '/converter', icon: 'ArrowLeftRight' }],
}

export const getModuleSubnavItems = (module: string): ModuleSubnavItem[] => {
  return moduleSubnavMap[module] ?? []
}
