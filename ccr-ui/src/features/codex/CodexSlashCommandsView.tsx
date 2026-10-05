import { memo, useMemo } from 'react'
import { Link } from 'react-router'
import { PageHeader, PageShell, SIcon, buttonClass } from '@/ui'
import { CodexSubnav } from './CodexSubnav'
import { panelCardClass } from './ui-classes'
import { useCodexLocale } from './useCodexLocale'

interface ShortcutItem {
  title: string
  description: string
  to: string
  icon: string
}

const ShortcutLink = memo(function ShortcutLink({ item }: { item: ShortcutItem }) {
  return (
    <Link
      to={item.to}
      className="flex items-start gap-3 rounded-2xl border border-border-default/15 bg-bg-elevated p-4 transition-all duration-200 hover:border-accent-warning/25 hover:bg-bg-elevated/80"
    >
      <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-xl border border-border-default/15 bg-bg-elevated text-text-secondary">
        <SIcon name={item.icon} size="w-4 h-4" />
      </div>
      <div className="min-w-0 flex-1">
        <p className="text-sm font-semibold text-text-primary">{item.title}</p>
        <p className="mt-1 text-sm leading-6 text-text-muted">{item.description}</p>
      </div>
      <SIcon name="ArrowRight" size="w-4 h-4" className="text-text-ghost" />
    </Link>
  )
})

export function CodexSlashCommandsView() {
  const { tt } = useCodexLocale()
  const shortcuts = useMemo<ShortcutItem[]>(
    () => [
      {
        title: tt('Sessions', 'Sessions'),
        description: tt(
          '查看最近会话、导出上下文、克隆或删除本地 session 记录。',
          'Inspect recent sessions, export context, and clone or delete local session records.',
        ),
        to: '/codex/sessions',
        icon: 'MessagesSquare',
      },
      {
        title: tt('Agents', 'Agents'),
        description: tt(
          '管理 Codex 专用 agents，复用现有 agent 配置能力。',
          'Manage Codex-specific agents while reusing the existing agent configuration flow.',
        ),
        to: '/codex/agents',
        icon: 'Bot',
      },
      {
        title: 'MCP',
        description: tt(
          '继续扩展本地工具链，把 Codex 接到更多外部能力上。',
          'Keep extending the local toolchain and connect Codex to more external capabilities.',
        ),
        to: '/codex/mcp',
        icon: 'Server',
      },
    ],
    [tt],
  )

  return (
    <PageShell
      className="min-w-0"
      header={
        <PageHeader
          title={tt('Codex 目前没有可管理的 Slash Commands', 'Codex currently has no manageable slash commands')}
          eyebrow={tt('仅兼容入口', 'Compatibility Only')}
          description={tt(
            '这个页面保留为兼容入口，用来解释为什么 Codex 模块没有接入 Slash Commands 管理。 当前工作流重点已经切到 Sessions、Agents、Profiles 和 MCP。',
            'This page remains as a compatibility entry so it can explain why the Codex module does not expose slash-command management. The active workflow focus has shifted to Sessions, Agents, Profiles, and MCP.',
          )}
          actions={
            <div className="flex flex-wrap gap-2">
              <Link to="/codex/sessions" className={buttonClass({ variant: 'primary' })}>
                <SIcon name="MessagesSquare" size="w-4 h-4" />
                <span>{tt('打开 Sessions', 'Open Sessions')}</span>
              </Link>
              <Link to="/codex/agents" className={buttonClass({ variant: 'secondary' })}>
                <SIcon name="Bot" size="w-4 h-4" />
                <span>{tt('管理 Agents', 'Manage Agents')}</span>
              </Link>
            </div>
          }
        />
      }
      subnav={<CodexSubnav />}
    >
      <div className="grid gap-4 xl:grid-cols-2">
        <section className={panelCardClass}>
          <div className="mb-4 flex items-center gap-3">
            <div className="flex h-11 w-11 items-center justify-center rounded-2xl border border-accent-danger/20 bg-accent-danger/10 text-accent-danger">
              <SIcon name="AlertTriangle" size="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-semibold text-text-primary">{tt('当前状态', 'Current state')}</h2>
              <p className="text-sm text-text-muted">
                {tt('后端没有对应的 Slash Commands 命令集', 'The backend has no matching slash-command command set')}
              </p>
            </div>
          </div>
          <div className="rounded-2xl border border-border-default/15 bg-bg-elevated p-4">
            <p className="text-sm font-semibold text-text-primary">
              {tt('为什么不继续沿用通用页面', 'Why not keep the generic page')}
            </p>
            <p className="mt-2 text-sm leading-7 text-text-secondary">
              {tt(
                'Codex 在 CCR 中没有对应的 slash command CRUD 能力，之前的页面只是复用通用容器后返回“平台不支持”。现在把它降级成说明页，避免把一个不存在的能力放进主导航。',
                'CCR does not expose slash-command CRUD for Codex. The old page only reused the generic container and returned “platform not supported”. It is now downgraded into an explainer page so a nonexistent capability does not stay in the main navigation.',
              )}
            </p>
          </div>
        </section>

        <section className={panelCardClass}>
          <div className="mb-4 flex items-center gap-3">
            <div className="flex h-11 w-11 items-center justify-center rounded-2xl border border-accent-success/20 bg-accent-success/10 text-accent-success">
              <SIcon name="Workflow" size="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-semibold text-text-primary">{tt('推荐入口', 'Recommended entries')}</h2>
              <p className="text-sm text-text-muted">
                {tt('真实可用的 Codex 工作面板', 'The Codex surfaces that actually work')}
              </p>
            </div>
          </div>
          <div className="space-y-3">
            {shortcuts.map((item) => (
              <ShortcutLink key={item.to} item={item} />
            ))}
          </div>
        </section>
      </div>
    </PageShell>
  )
}
