export const defaultAgentModelOptions = [
  { value: 'claude-sonnet-4-5-20250929', label: 'Claude Sonnet 4.5' },
  { value: 'claude-opus-4-20250514', label: 'Claude Opus 4' },
  { value: 'claude-3-5-sonnet-20241022', label: 'Claude 3.5 Sonnet' },
] as const

export const DEFAULT_AGENT_MODEL = defaultAgentModelOptions[0].value
