import { useCallback, useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { Link, useNavigate, useParams } from 'react-router'
import { deleteAgent, getAgent, toggleAgent, updateAgent } from '@/api'
import { surfaceNotify } from '@/configs/surfaceNotify'
import { PlatformSubnav } from '@/features/platform/PlatformSubnav'
import { defaultSurfaceT } from '@/features/platform/translate'
import type { Agent, AgentRequest } from '@/types'
import { PageHeader, PageShell, SIcon } from '@/ui'
import { copyText } from '@/utils/clipboard'
import { getErrorMessage } from '@/utils/errorHandler'
import { logger } from '@/utils/logger'
import { AgentDetailContent } from './AgentDetailContent'
import { DEFAULT_AGENT_MODEL } from './agentModels'
import { AgentEditModal, type AgentEditForm } from './AgentEditModal'

const emptyForm = (): AgentEditForm => ({
  model: DEFAULT_AGENT_MODEL,
  systemPrompt: '',
  toolDraft: '',
  toolsText: '',
})

/** 跨平台 Agent 详情。当前路由 `/agents/:name` 仍走 Claude agents API。 */
export function AgentDetailView() {
  const t = defaultSurfaceT
  const navigate = useNavigate()
  const params = useParams()
  const name = typeof params.name === 'string' ? params.name : ''
  const [agent, setAgent] = useState<Agent | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [showEdit, setShowEdit] = useState(false)
  const [saving, setSaving] = useState(false)
  const [copied, setCopied] = useState(false)
  const form = useForm<AgentEditForm>({ defaultValues: emptyForm() })

  useEffect(() => {
    if (!name) {
      setError('Invalid agent name parameter')
      setLoading(false)
      return
    }
    void (async () => {
      try {
        const next = await getAgent(name)
        if (!next) {
          setError(t('agents.loadError'))
          return
        }
        setAgent(next)
      } catch (err: unknown) {
        logger.error('Failed to load agent:', err)
        setError(getErrorMessage(err) || 'Failed to load agent')
      } finally {
        setLoading(false)
      }
    })()
  }, [name, t])

  const closeEdit = useCallback(() => setShowEdit(false), [])

  const handleEdit = useCallback(() => {
    if (!agent) return
    form.reset({
      model: agent.model || DEFAULT_AGENT_MODEL,
      systemPrompt: agent.system_prompt || '',
      toolDraft: '',
      toolsText: (agent.tools || []).join('\n'),
    })
    setShowEdit(true)
  }, [agent, form])

  const handleSave = useCallback(async () => {
    if (!agent) return
    setSaving(true)
    const values = form.getValues()
    const tools = values.toolsText
      .split('\n')
      .map((item) => item.trim())
      .filter(Boolean)
    const request: AgentRequest = {
      name: agent.name,
      model: values.model,
      tools: tools.length > 0 ? tools : undefined,
      system_prompt: values.systemPrompt || undefined,
      disabled: agent.disabled || false,
    }
    try {
      await updateAgent(agent.name, request)
      setAgent({
        ...agent,
        model: values.model,
        tools,
        system_prompt: values.systemPrompt,
      })
      setShowEdit(false)
    } catch (err) {
      logger.error('Failed to update agent:', err)
      surfaceNotify.error(t('common.operationFailed'))
    } finally {
      setSaving(false)
    }
  }, [agent, form, t])

  const handleToggle = useCallback(async () => {
    if (!agent) return
    try {
      await toggleAgent(agent.name)
      setAgent({ ...agent, disabled: !agent.disabled })
    } catch (err) {
      logger.error('Failed to toggle agent:', err)
      surfaceNotify.error(t('common.operationFailed'))
    }
  }, [agent, t])

  const handleDelete = useCallback(async () => {
    if (!agent) return
    const confirmed = await surfaceNotify.confirm({
      title: t('common.delete'),
      message: t('agents.deleteConfirm', { name: agent.name }),
      confirmText: t('common.delete'),
      cancelText: t('common.cancel'),
      type: 'danger',
    })
    if (!confirmed) return
    try {
      await deleteAgent(agent.name)
      void navigate('/agents')
    } catch (err) {
      logger.error('Failed to delete agent:', err)
      surfaceNotify.error(t('common.deleteFailed'))
    }
  }, [agent, navigate, t])

  const copySystemPrompt = useCallback(async () => {
    if (!agent?.system_prompt) return
    const ok = await copyText(agent.system_prompt)
    if (!ok) {
      logger.error('Failed to copy:', new Error('clipboard copy failed'))
      return
    }
    setCopied(true)
    window.setTimeout(() => setCopied(false), 2000)
  }, [agent?.system_prompt])

  const header = (
    <PageHeader
      title={agent?.name || t('common.loading')}
      description={agent?.folder || undefined}
      status={
        agent ? (
          <div className="flex flex-wrap gap-2">
            {agent.model ? (
              <span className="inline-flex items-center rounded-md border border-border-default bg-bg-surface px-3 py-1 text-xs text-text-secondary">
                {agent.model}
              </span>
            ) : null}
            <span className="inline-flex items-center rounded-md border border-border-default bg-bg-elevated px-3 py-1 text-xs font-medium text-text-secondary">
              {agent.disabled ? t('agents.disabledBadge') : t('agents.enabledBadge')}
            </span>
          </div>
        ) : null
      }
      actions={
        <div className="flex flex-wrap gap-2">
          <Link
            to="/agents"
            className="inline-flex min-h-11 items-center gap-2 rounded-lg border border-border-default bg-bg-elevated px-4 py-2 text-sm font-medium text-text-secondary"
          >
            <SIcon name="ArrowLeft" size="w-4 h-4" />
            {t('common.back')}
          </Link>
          {agent ? (
            <>
              <button
                type="button"
                className="inline-flex items-center gap-2 rounded-lg bg-bg-elevated px-4 py-2 text-sm font-medium text-text-secondary"
                onClick={handleToggle}
              >
                <SIcon name={agent.disabled ? 'PowerOff' : 'Power'} size="w-4 h-4" />
                {agent.disabled ? t('agents.enable') : t('agents.disable')}
              </button>
              <button
                type="button"
                className="inline-flex items-center gap-2 rounded-lg bg-bg-elevated px-4 py-2 text-sm font-medium text-text-secondary"
                onClick={handleEdit}
              >
                <SIcon name="Edit2" size="w-4 h-4" />
                {t('common.edit')}
              </button>
              <button
                type="button"
                className="inline-flex items-center gap-2 rounded-lg bg-accent-danger/10 px-4 py-2 text-sm font-medium text-accent-danger"
                onClick={handleDelete}
              >
                <SIcon name="Trash2" size="w-4 h-4" />
                {t('common.delete')}
              </button>
            </>
          ) : null}
        </div>
      }
    />
  )

  return (
    <PageShell header={header} subnav={<PlatformSubnav module="claude-code" />}>
      <AgentDetailContent
        loading={loading}
        error={error}
        agent={agent}
        copied={copied}
        t={t}
        onCopy={copySystemPrompt}
      />
      <AgentEditModal
        open={showEdit}
        name={agent?.name ?? ''}
        saving={saving}
        t={t}
        form={form}
        onClose={closeEdit}
        onSave={handleSave}
      />
    </PageShell>
  )
}
