import * as systemPromptsClient from '../generated/systemPrompts'
import type { UnsupportedEnvironment } from './configRawTypes'
import type { OpenJsonValueDto } from '@/types/generated/common/OpenJsonValueDto'

export interface SystemPromptFile {
  id: string
  labelKey: string
  path: string
  exists: boolean
  size: number | null
  mtime: number | null
  editable: boolean
  limitHint: number | null
}

export interface SystemPromptRule {
  name: string
  path: string
  size: number | null
}

export type SystemPromptsListResult =
  | { status: 'ok'; files: SystemPromptFile[]; rules: SystemPromptRule[] }
  | UnsupportedEnvironment

export type SystemPromptGetResult =
  | {
      status: 'ok'
      content: string
      token: string
      path: string
      exists: boolean
      limitHint: number | null
    }
  | UnsupportedEnvironment

export type SystemPromptWriteResult =
  | { status: 'saved'; token: string; warning?: 'size'; limitHint?: number }
  | { status: 'conflict' }
  | UnsupportedEnvironment

const objectValue = (value: OpenJsonValueDto): Record<string, OpenJsonValueDto | undefined> => {
  if (value === null || Array.isArray(value) || typeof value !== 'object') {
    throw new Error('System prompts response is not an object')
  }
  return value
}

const optionalNumber = (value: OpenJsonValueDto | undefined): number | null =>
  typeof value === 'number' ? value : null

const unsupportedEnvironment = (
  source: Record<string, OpenJsonValueDto | undefined>,
): UnsupportedEnvironment | null => {
  if (source.status === 'unsupported_environment' && typeof source.envType === 'string') {
    return { status: source.status, envType: source.envType }
  }
  return null
}

const promptFileFrom = (value: OpenJsonValueDto): SystemPromptFile => {
  const source = objectValue(value)
  if (
    typeof source.id !== 'string'
    || typeof source.labelKey !== 'string'
    || typeof source.path !== 'string'
    || typeof source.exists !== 'boolean'
    || typeof source.editable !== 'boolean'
  ) {
    throw new Error('System prompt file response is invalid')
  }
  return {
    id: source.id,
    labelKey: source.labelKey,
    path: source.path,
    exists: source.exists,
    size: optionalNumber(source.size),
    mtime: optionalNumber(source.mtime),
    editable: source.editable,
    limitHint: optionalNumber(source.limitHint),
  }
}

const promptRuleFrom = (value: OpenJsonValueDto): SystemPromptRule => {
  const source = objectValue(value)
  if (typeof source.name !== 'string' || typeof source.path !== 'string') {
    throw new Error('System prompt rule response is invalid')
  }
  return { name: source.name, path: source.path, size: optionalNumber(source.size) }
}

const listResultFrom = (value: OpenJsonValueDto): SystemPromptsListResult => {
  const source = objectValue(value)
  const unsupported = unsupportedEnvironment(source)
  if (unsupported) return unsupported
  if (source.status !== 'ok' || !Array.isArray(source.files) || !Array.isArray(source.rules)) {
    throw new Error('System prompts list response is invalid')
  }
  return {
    status: source.status,
    files: source.files.map(promptFileFrom),
    rules: source.rules.map(promptRuleFrom),
  }
}

const getResultFrom = (value: OpenJsonValueDto): SystemPromptGetResult => {
  const source = objectValue(value)
  const unsupported = unsupportedEnvironment(source)
  if (unsupported) return unsupported
  if (
    source.status !== 'ok'
    || typeof source.content !== 'string'
    || typeof source.token !== 'string'
    || typeof source.path !== 'string'
    || typeof source.exists !== 'boolean'
  ) {
    throw new Error('System prompt get response is invalid')
  }
  return {
    status: source.status,
    content: source.content,
    token: source.token,
    path: source.path,
    exists: source.exists,
    limitHint: optionalNumber(source.limitHint),
  }
}

const writeResultFrom = (value: OpenJsonValueDto): SystemPromptWriteResult => {
  const source = objectValue(value)
  const unsupported = unsupportedEnvironment(source)
  if (unsupported) return unsupported
  if (source.status === 'conflict') return { status: source.status }
  if (source.status !== 'saved' || typeof source.token !== 'string') {
    throw new Error('System prompt write response is invalid')
  }
  return {
    status: source.status,
    token: source.token,
    warning: source.warning === 'size' ? source.warning : undefined,
    limitHint: typeof source.limitHint === 'number' ? source.limitHint : undefined,
  }
}

export const listSystemPrompts = async (platform: string): Promise<SystemPromptsListResult> => {
  return listResultFrom(await systemPromptsClient.listSystemPrompts(platform))
}

export const getSystemPrompt = async (
  platform: string,
  id: string,
): Promise<SystemPromptGetResult> => {
  return getResultFrom(await systemPromptsClient.getSystemPrompt(platform, id))
}

export const saveSystemPrompt = async (
  platform: string,
  id: string,
  content: string,
  token: string,
): Promise<SystemPromptWriteResult> => {
  return writeResultFrom(await systemPromptsClient.saveSystemPrompt(platform, id, content, token))
}

export const createSystemPrompt = async (
  platform: string,
  id: string,
): Promise<SystemPromptWriteResult> => {
  return writeResultFrom(await systemPromptsClient.createSystemPrompt(platform, id))
}
