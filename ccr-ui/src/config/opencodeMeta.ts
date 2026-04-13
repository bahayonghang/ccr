export interface OpenCodeCapabilityCard {
  id: string
  title: string
  description: string
  href: string
  icon: string
  tone: 'lime' | 'violet' | 'cyan' | 'amber' | 'emerald'
}

export interface OpenCodeCliCommandMeta {
  command: string
  description: string
  note?: string
}

export interface OpenCodeToolMeta {
  id: string
  description: string
  permissionKey: string
  availability?: string
}

export interface OpenCodeTopologyMeta {
  title: string
  path: string
  description: string
}

export const opencodeCapabilityCards: OpenCodeCapabilityCard[] = [
  {
    id: 'providers',
    title: 'Providers',
    description: '配置 provider、模型、认证与默认模型策略。',
    href: '/opencode/providers',
    icon: 'Layers',
    tone: 'lime',
  },
  {
    id: 'mcp',
    title: 'MCP',
    description: '管理 local / remote MCP 服务器与 CLI 认证动作。',
    href: '/opencode/mcp',
    icon: 'Server',
    tone: 'cyan',
  },
  {
    id: 'agents',
    title: 'Agents',
    description: '查看内置 agent 模式，并管理自定义 primary / subagent。',
    href: '/opencode/agents',
    icon: 'Bot',
    tone: 'violet',
  },
  {
    id: 'commands',
    title: 'Commands',
    description: '维护自定义命令模板、绑定 agent 和模型。',
    href: '/opencode/commands',
    icon: 'Command',
    tone: 'amber',
  },
  {
    id: 'skills',
    title: 'Skills',
    description: '复用统一 Skills Hub，但默认聚焦 OpenCode 平台。',
    href: '/opencode/skills',
    icon: 'BookOpen',
    tone: 'emerald',
  },
  {
    id: 'plugins',
    title: 'Plugins',
    description: '同时管理 npm 插件和本地插件文件目录。',
    href: '/opencode/plugins',
    icon: 'Puzzle',
    tone: 'emerald',
  },
  {
    id: 'settings',
    title: 'Settings',
    description: '拆分 opencode.json 与 tui.json，集中管理 runtime、theme、keybinds。',
    href: '/opencode/settings',
    icon: 'SlidersHorizontal',
    tone: 'lime',
  },
]

export const opencodeCliCommands: OpenCodeCliCommandMeta[] = [
  { command: 'opencode agent', description: '管理 agents；支持 create / list。' },
  { command: 'opencode mcp', description: '管理 MCP；支持 add / list / auth / logout / debug。' },
  { command: 'opencode models', description: '列出 provider/model 组合，支持 refresh。' },
  { command: 'opencode run', description: '非交互运行 prompt，可绑定 command / agent / model。' },
  { command: 'opencode serve', description: '启动 headless HTTP server。', note: '配合 server.port / hostname / cors。' },
  { command: 'opencode web', description: '启动 web 界面的 headless server。' },
  { command: 'opencode acp', description: '启动 ACP server，走 stdin/stdout 或端口模式。' },
  { command: 'opencode session', description: '管理会话；支持 list。' },
  { command: 'opencode stats', description: '查看 token / cost / model 维度使用统计。' },
]

export const opencodeBuiltInTools: OpenCodeToolMeta[] = [
  { id: 'bash', description: '执行 shell 命令。', permissionKey: 'bash' },
  { id: 'edit', description: '精确替换已有文件内容。', permissionKey: 'edit' },
  { id: 'write', description: '创建或覆盖文件。', permissionKey: 'edit' },
  { id: 'read', description: '读取文件内容。', permissionKey: 'read' },
  { id: 'grep', description: '基于 ripgrep 做正则搜索。', permissionKey: 'grep' },
  { id: 'glob', description: '按 glob 查找文件。', permissionKey: 'glob' },
  { id: 'list', description: '列出目录内容。', permissionKey: 'list' },
  { id: 'apply_patch', description: '应用 patch 修改文件。', permissionKey: 'edit' },
  { id: 'skill', description: '按需加载 SKILL.md。', permissionKey: 'skill' },
  { id: 'todowrite', description: '维护 todo 列表。', permissionKey: 'todowrite' },
  { id: 'webfetch', description: '读取指定网页内容。', permissionKey: 'webfetch' },
  {
    id: 'websearch',
    description: '使用 Exa 搜索网络信息。',
    permissionKey: 'websearch',
    availability: '需要 OpenCode provider 或 OPENCODE_ENABLE_EXA。',
  },
  { id: 'question', description: '执行中向用户提问。', permissionKey: 'question' },
]

export const opencodeConfigTopology: OpenCodeTopologyMeta[] = [
  {
    title: 'Global config',
    path: '~/.config/opencode/opencode.json',
    description: '用户级 server/runtime/provider 配置。',
  },
  {
    title: 'Global TUI',
    path: '~/.config/opencode/tui.json',
    description: '主题、keybinds、TUI 表现层配置。',
  },
  {
    title: 'Project config',
    path: 'opencode.json',
    description: '项目根目录配置，覆盖 global 默认值。',
  },
  {
    title: 'Project modules',
    path: '.opencode/{agents,commands,plugins,skills}',
    description: '项目级 agents / commands / plugins / skills 目录。',
  },
  {
    title: 'Compatibility',
    path: '.claude/skills + .agents/skills',
    description: 'OpenCode 会兼容发现 Claude / Agents 风格的 skills。',
  },
]
