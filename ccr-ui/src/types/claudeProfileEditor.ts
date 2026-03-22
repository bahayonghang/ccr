export const CLAUDE_PROFILE_FORM_SECTION_IDS = ['basic', 'connection', 'auth', 'status'] as const

export type ClaudeProfileFormSectionId = (typeof CLAUDE_PROFILE_FORM_SECTION_IDS)[number]

export interface ClaudeProfileEditorForm {
  name: string
  description: string
  base_url: string
  auth_token: string
  model: string
  small_fast_model: string
  provider: string
  provider_type: string
  account: string
  tagsInput: string
  enabled: boolean
}

export interface ClaudeProfileEditorSummaryItem {
  label: string
  value: string
  icon: string
  mono?: boolean
}

export interface ClaudeProfileEditorSectionItem {
  id: ClaudeProfileFormSectionId
  title: string
  description: string
  icon: string
}
