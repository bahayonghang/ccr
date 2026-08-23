import { useCallback, useMemo, useState } from 'react'
import { useForm } from 'react-hook-form'
import type { UnifiedMcpPlatform, UnifiedMcpRequest, UnifiedMcpServer } from '@/types/unifiedMcp'
import { createEmptyForm, stripUnchangedSecretPreviews, toSuccessMessage } from './mcp-constants'

function assembleRequest(input: {
  values: UnifiedMcpRequest
  args: string[]
  includeTools: string[]
  env: Record<string, string> | null
  headers: Record<string, string> | null
  editingServer: UnifiedMcpServer | null
  isHttpMode: boolean
}): UnifiedMcpRequest {
  const request: UnifiedMcpRequest = {
    ...input.values,
    args: input.args,
    include_tools: input.includeTools,
    env: input.env ?? {},
    headers: input.headers ?? {},
  }
  if (request.platform !== 'claude') {
    request.scope = null
    request.headers = null
    request.timeout = null
    request.cwd = null
    request.trust = null
    request.include_tools = null
    request.disabled = null
  }
  if (input.isHttpMode) {
    request.command = null
    request.args = null
  } else {
    request.url = null
  }
  if (!input.editingServer) return request
  if (!input.args.length) request.args = null
  if (!input.includeTools.length) request.include_tools = null
  if (!Object.keys(input.env ?? {}).length) request.env = null
  if (!Object.keys(input.headers ?? {}).length) request.headers = null
  return request
}
import { mcpNotify } from './notify'
import type { UnifiedMcpListApi } from './useUnifiedMcpList'

export function useUnifiedMcpForm(list: UnifiedMcpListApi) {
  const form = useForm<UnifiedMcpRequest>({ defaultValues: createEmptyForm() })
  const formData = form.watch()
  const [showForm, setShowForm] = useState(false)
  const [editingServer, setEditingServer] = useState<UnifiedMcpServer | null>(null)
  const [isHttpMode, setIsHttpMode] = useState(false)
  const [argInput, setArgInput] = useState('')
  const [envKey, setEnvKey] = useState('')
  const [envValue, setEnvValue] = useState('')
  const [headerKey, setHeaderKey] = useState('')
  const [headerValue, setHeaderValue] = useState('')
  const [includeToolInput, setIncludeToolInput] = useState('')

  const currentCapability = useMemo(() => {
    if (!formData.platform) return null
    return list.capabilities.find((c) => c.platform === formData.platform) ?? null
  }, [formData.platform, list.capabilities])

  const closeForm = useCallback(() => {
    setShowForm(false)
    setEditingServer(null)
  }, [])

  const resetFormInputs = useCallback(() => {
    setArgInput('')
    setEnvKey('')
    setEnvValue('')
    setHeaderKey('')
    setHeaderValue('')
    setIncludeToolInput('')
  }, [])

  const validateForm = useCallback(() => {
    const values = form.getValues()
    if (!values.name) {
      mcpNotify.warning('服务器名称不能为空')
      return false
    }
    if (!isHttpMode && !values.command) {
      mcpNotify.warning('STDIO 模式必须提供 command')
      return false
    }
    if (isHttpMode && !values.url) {
      mcpNotify.warning('HTTP 模式必须提供 url')
      return false
    }
    return true
  }, [form, isHttpMode])

  const buildRequest = useCallback((): UnifiedMcpRequest => {
    const values = form.getValues()
    const args = argInput.split(' ').map((a) => a.trim()).filter(Boolean)
    const includeTools = includeToolInput.split(',').map((item) => item.trim()).filter(Boolean)
    const env = values.env ? { ...values.env } : null
    const headers = values.headers ? { ...values.headers } : null
    if (editingServer) {
      stripUnchangedSecretPreviews(env, editingServer.env ?? {})
      stripUnchangedSecretPreviews(headers, editingServer.headers ?? {})
    }
    return assembleRequest({ values, args, includeTools, env, headers, editingServer, isHttpMode })
  }, [argInput, editingServer, form, includeToolInput, isHttpMode])

  const addServer = useCallback(async () => {
    if (!validateForm()) return false
    try {
      const message = await list.addMutation.mutateAsync(buildRequest())
      mcpNotify.success(toSuccessMessage(message, '添加成功'))
      await list.reloadServers()
      closeForm()
      return true
    } catch (err) {
      mcpNotify.error(`添加失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
      return false
    }
  }, [buildRequest, closeForm, list, validateForm])

  const updateServer = useCallback(async () => {
    if (!editingServer || !validateForm()) return false
    const request = buildRequest()
    if (!request.scope && typeof editingServer.scope === 'string') request.scope = editingServer.scope
    try {
      const message = await list.updateMutation.mutateAsync({
        platform: editingServer.platform,
        name: editingServer.name,
        request,
      })
      mcpNotify.success(toSuccessMessage(message, '更新成功'))
      await list.reloadServers()
      closeForm()
      return true
    } catch (err) {
      mcpNotify.error(`更新失败: ${err instanceof Error ? err.message : 'Unknown error'}`)
      return false
    }
  }, [buildRequest, closeForm, editingServer, list, validateForm])

  const openAddForm = useCallback(
    (platform?: UnifiedMcpPlatform, scope: UnifiedMcpRequest['scope'] = 'user') => {
      setEditingServer(null)
      setIsHttpMode(false)
      form.reset(createEmptyForm())
      if (platform) {
        form.setValue('platform', platform)
        form.setValue('scope', platform === 'claude' ? scope : null)
      } else {
        form.setValue('scope', form.getValues('platform') === 'claude' ? scope : null)
      }
      resetFormInputs()
      setShowForm(true)
    },
    [form, resetFormInputs],
  )

  const openEditForm = useCallback((server: UnifiedMcpServer) => {
    setEditingServer(server)
    setIsHttpMode(!!server.url)
    form.reset({
      platform: server.platform,
      name: server.name,
      scope: server.scope ?? 'user',
      command: server.command,
      url: server.url,
      args: server.args?.length ? server.args : null,
      env: server.env && Object.keys(server.env).length > 0 ? { ...server.env } : null,
      headers: server.headers ? { ...server.headers } : null,
      timeout: server.timeout,
      disabled: server.disabled,
      cwd: server.cwd,
      trust: server.trust,
      include_tools: server.include_tools ? [...server.include_tools] : null,
    })
    setArgInput(server.args?.join(' ') ?? '')
    setIncludeToolInput(server.include_tools?.join(', ') ?? '')
    setEnvKey('')
    setEnvValue('')
    setHeaderKey('')
    setHeaderValue('')
    setShowForm(true)
  }, [form])

  const submitForm = useCallback(
    () => (editingServer ? updateServer() : addServer()),
    [addServer, editingServer, updateServer],
  )

  const addEnvVar = useCallback(() => {
    if (!envKey || !envValue) return
    form.setValue('env', { ...(form.getValues('env') ?? {}), [envKey]: envValue })
    setEnvKey('')
    setEnvValue('')
  }, [envKey, envValue, form])

  const removeEnvVar = useCallback((key: string) => {
    const current = form.getValues('env')
    if (!current) return
    const next = { ...current }
    delete next[key]
    form.setValue('env', Object.keys(next).length > 0 ? next : null)
  }, [form])

  const addHeader = useCallback(() => {
    if (!headerKey || !headerValue) return
    form.setValue('headers', { ...(form.getValues('headers') ?? {}), [headerKey]: headerValue })
    setHeaderKey('')
    setHeaderValue('')
  }, [form, headerKey, headerValue])

  const removeHeader = useCallback((key: string) => {
    const current = form.getValues('headers')
    if (!current) return
    const next = { ...current }
    delete next[key]
    form.setValue('headers', Object.keys(next).length > 0 ? next : null)
  }, [form])

  return {
    showForm,
    editingServer,
    isHttpMode,
    formData,
    formApi: form,
    argInput,
    envKey,
    envValue,
    headerKey,
    headerValue,
    includeToolInput,
    setArgInput,
    setEnvKey,
    setEnvValue,
    setHeaderKey,
    setHeaderValue,
    setIncludeToolInput,
    setShowForm,
    setIsHttpMode,
    currentCapability,
    openAddForm,
    openEditForm,
    closeForm,
    submitForm,
    addEnvVar,
    removeEnvVar,
    addHeader,
    removeHeader,
  }
}
