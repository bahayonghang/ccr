import type { CliType, ConverterRequest } from '@/types'

export const CLI_DEFINITIONS: { value: CliType; label: string; descriptionKey: string }[] = [
  { value: 'claude-code', label: 'Claude Code', descriptionKey: 'converter.formatDescriptions.claudeCode' },
  { value: 'codex', label: 'Codex', descriptionKey: 'converter.formatDescriptions.codex' },
  { value: 'gemini', label: 'Antigravity CLI', descriptionKey: 'converter.formatDescriptions.gemini' },
]

export const converterFormSchemaShape = {
  sourceFormat: true,
  targetFormat: true,
  configData: true,
  convertMcp: true,
  convertCommands: true,
  convertAgents: true,
} as const

export interface ConverterFormValues {
  sourceFormat: CliType
  targetFormat: CliType
  configData: string
  convertMcp: boolean
  convertCommands: boolean
  convertAgents: boolean
}

export const emptyConverterForm = (): ConverterFormValues => ({
  sourceFormat: 'claude-code',
  targetFormat: 'codex',
  configData: '',
  convertMcp: true,
  convertCommands: true,
  convertAgents: true,
})

export const CONVERTER_EXAMPLE = `{
  "mcpServers": {
    "context7": {
      "command": "npx",
      "args": ["-y", "@upstash/context7-mcp"],
      "env": {
        "API_KEY": "your-api-key-here"
      }
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/files"]
    }
  }
}`

export function toConverterRequest(values: ConverterFormValues): ConverterRequest {
  return {
    source_format: values.sourceFormat,
    target_format: values.targetFormat,
    config_data: values.configData,
    convert_mcp: values.convertMcp,
    convert_commands: values.convertCommands,
    convert_agents: values.convertAgents,
  }
}

export function resultExtension(format: string | undefined): string {
  if (format === 'json') return 'json'
  if (format === 'toml') return 'toml'
  return 'txt'
}

export function cliLabelOf(value: CliType): string {
  return CLI_DEFINITIONS.find((type) => type.value === value)?.label || value
}
