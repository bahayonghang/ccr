import { useCallback, useEffect, useState } from 'react'
import { Link } from 'react-router'
import { detectSkillportApp, isTauriEnvironment, openSkillportApp, type SkillportAppStatus } from '@/api/domains/system'
import skillportBadgeUrl from '@/assets/skillport-badge.svg'
import { tt } from '@/features/claude/locale'
import { PageHeader, PageShell } from '@/ui'
import { logger } from '@/utils/logger'

const SKILLPORT_REPO = 'https://github.com/bahayonghang/skills-manage-windows'

const EMPTY_STATUS: SkillportAppStatus = {
  supported: false,
  installed: false,
  platform: 'other',
  source: 'unsupported',
}

function statusMeta(isDetecting: boolean, status: SkillportAppStatus) {
  if (isDetecting) {
    return {
      pill: tt('正在检测', 'Detecting'),
      className: 'border-border-default/60 bg-bg-surface text-text-primary',
      summary: tt('正在检查本机是否已经安装 skillport。', 'Checking whether skillport is installed locally.'),
    }
  }
  if (status.installed) {
    return {
      pill: tt('已检测到安装', 'Installed'),
      className: 'border-accent-success/20 bg-accent-success/10 text-accent-success',
      summary: tt('已检测到本机安装，可以直接从这里拉起独立应用。', 'Skillport is installed locally and can be launched from here.'),
    }
  }
  if (status.supported) {
    return {
      pill: tt('未检测到安装', 'Not installed'),
      className: 'border-accent-warning/20 bg-accent-warning/10 text-accent-warning',
      summary: tt('当前没有检测到本机安装，请先前往仓库查看最新安装说明。', 'No local install was detected. Check the repository instructions first.'),
    }
  }
  return {
    pill: tt('当前环境不支持', 'Unsupported environment'),
    className: 'border-border-default/50 bg-bg-base text-text-secondary',
    summary: tt('当前运行环境暂不支持自动检测，请直接前往仓库查看说明。', 'The current environment does not support auto-detection. Check the repository instructions directly.'),
  }
}

export function SkillsMigrationView() {
  const [isDetecting, setIsDetecting] = useState(false)
  const [isOpening, setIsOpening] = useState(false)
  const [launchError, setLaunchError] = useState('')
  const [appStatus, setAppStatus] = useState<SkillportAppStatus>(EMPTY_STATUS)

  const refreshAppStatus = useCallback(async () => {
    logger.info('[skills-migration] 开始探测 skillport 状态')
    setIsDetecting(true)
    setLaunchError('')
    try {
      if (!isTauriEnvironment()) {
        setAppStatus(EMPTY_STATUS)
        return
      }
      setAppStatus(await detectSkillportApp())
    } catch (error) {
      setAppStatus({ supported: true, installed: false, platform: 'other', source: 'not_found' })
      setLaunchError(tt('自动检测失败，请先查看仓库说明后再重试。', 'Auto-detection failed. Check the repository instructions first, then retry.'))
      logger.warn('[skills-migration] 探测 skillport 状态失败', error)
    } finally {
      setIsDetecting(false)
    }
  }, [])

  useEffect(() => {
    void refreshAppStatus()
  }, [refreshAppStatus])

  const handlePrimaryAction = useCallback(async () => {
    if (!appStatus.installed) return
    setIsOpening(true)
    setLaunchError('')
    try {
      await openSkillportApp()
    } catch (error) {
      setLaunchError(
        tt(
          '已检测到 skillport，但拉起失败。请先查看仓库说明，确认安装是否完整。',
          'Skillport was detected, but launch failed. Check the repository instructions and confirm the install is complete.',
        ),
      )
      logger.error('[skills-migration] 打开 skillport 失败', error)
    } finally {
      setIsOpening(false)
    }
  }, [appStatus.installed])

  const meta = statusMeta(isDetecting, appStatus)
  const primaryClass =
    'inline-flex min-h-11 items-center justify-center gap-2 rounded-2xl bg-accent-primary px-5 py-3 text-sm font-semibold text-[color:var(--color-accent-primary-contrast)] hover:bg-accent-primary/90 disabled:cursor-not-allowed disabled:opacity-70'
  const secondaryClass =
    'inline-flex min-h-11 items-center justify-center rounded-2xl border border-border-default/60 bg-bg-base px-5 py-3 text-sm font-semibold text-text-primary hover:bg-bg-surface/70 disabled:cursor-not-allowed disabled:opacity-70'

  return (
    <PageShell
      className="skills-migration-view"
      header={
        <PageHeader
          title={tt('Skills 已从 CCR UI 下线', 'Skills has been removed from CCR UI')}
          description={tt(
            'CCR UI 现在只保留 CLI 配置管理主线，不再内置 skills 安装、市场和来源管理。',
            'CCR UI now only keeps the CLI configuration management path and no longer embeds skills installation, marketplace, or source management.',
          )}
          status={
            <span
              className={`inline-flex w-fit items-center rounded-full border px-3 py-1 text-xs font-semibold ${meta.className}`}
              data-testid="skills-migration-status"
            >
              {meta.pill}
            </span>
          }
        />
      }
    >
      <section className="rounded-xl border border-border-default/60 bg-bg-surface p-6">
        <p className="max-w-3xl text-sm leading-7 text-text-secondary">
          {tt('之后请改用独立应用', 'Use the standalone app instead')}{' '}
          <a href={SKILLPORT_REPO} target="_blank" rel="noreferrer" className="font-semibold text-accent-primary hover:underline">
            skillport
          </a>{' '}
          {tt('处理 skills。', 'to handle skills.')}
        </p>
        <p className="mt-4 text-sm leading-7 text-text-secondary">{meta.summary}</p>
        <div className="mt-6 flex flex-wrap gap-3">
          {isDetecting ? (
            <button type="button" className={`${primaryClass} bg-accent-primary/70`} data-testid="skills-migration-primary" disabled>
              {tt('检测 skillport…', 'Detecting skillport...')}
            </button>
          ) : appStatus.installed ? (
            <button type="button" className={primaryClass} data-testid="skills-migration-primary" disabled={isOpening} onClick={handlePrimaryAction}>
              <img src={skillportBadgeUrl} alt="" className="h-5 w-5 rounded-lg" />
              <span>{isOpening ? tt('正在打开…', 'Opening...') : tt('打开 skillport', 'Open skillport')}</span>
            </button>
          ) : (
            <a href={SKILLPORT_REPO} target="_blank" rel="noreferrer" className={primaryClass} data-testid="skills-migration-primary">
              {tt('前往 skillport 仓库', 'Go to the skillport repository')}
            </a>
          )}
          <button type="button" className={secondaryClass} data-testid="skills-migration-refresh" disabled={isDetecting} onClick={refreshAppStatus}>
            {tt('重新检测', 'Recheck')}
          </button>
          <Link to="/configs" className={secondaryClass}>
            {tt('返回配置管理', 'Back to config management')}
          </Link>
        </div>
        {launchError ? (
          <p className="mt-4 rounded-xl border border-accent-danger/20 bg-accent-danger/10 px-4 py-3 text-sm leading-6 text-accent-danger" data-testid="skills-migration-error">
            {launchError}
          </p>
        ) : null}
        <div className="mt-4 flex flex-wrap gap-4">
          <a href={SKILLPORT_REPO} target="_blank" rel="noreferrer" className="text-sm font-medium text-text-secondary underline-offset-4 hover:text-text-primary hover:underline">
            {tt('查看仓库说明', 'View repository instructions')}
          </a>
        </div>
      </section>
      <section className="mt-4 grid gap-4 md:grid-cols-3">
        <article className="rounded-xl border border-border-default/50 bg-bg-elevated p-5">
          <h2 className="text-sm font-semibold text-text-primary">{tt('为什么下线', 'Why it was removed')}</h2>
          <p className="mt-3 text-sm leading-7 text-text-secondary">
            {tt(
              '这一层功能和 CCR UI 的核心定位不一致。继续保留会让路由、状态和桌面后端持续膨胀。',
              'This layer did not fit CCR UI’s core scope. Keeping it would keep inflating routes, state, and the desktop backend.',
            )}
          </p>
        </article>
        <article className="rounded-xl border border-border-default/50 bg-bg-elevated p-5">
          <h2 className="text-sm font-semibold text-text-primary">{tt('现在去哪里', 'Where to go now')}</h2>
          <p className="mt-3 text-sm leading-7 text-text-secondary">
            {tt(
              'skills 的浏览、安装和管理统一改到 skillport。CCR UI 只负责 CLI 配置、运行态和数据面。',
              'Browse, install, and manage skills in skillport. CCR UI only handles CLI config, runtime state, and data surfaces.',
            )}
          </p>
        </article>
        <article className="rounded-xl border border-border-default/50 bg-bg-elevated p-5">
          <h2 className="text-sm font-semibold text-text-primary">{tt('怎么开始', 'How to start')}</h2>
          <p className="mt-3 text-sm leading-7 text-text-secondary">
            {tt(
              '如果本机已安装 skillport，这里会直接显示打开按钮。还没安装时，请先去仓库查看最新安装说明。',
              'If skillport is already installed, this page will show an open button. Otherwise, check the repository for install instructions first.',
            )}
          </p>
        </article>
      </section>
    </PageShell>
  )
}
