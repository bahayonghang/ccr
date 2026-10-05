import { memo, useCallback } from 'react'
import { useForm } from 'react-hook-form'
import { SIcon } from '@/ui/s-icon'
import { mcpFieldInputClass, mcpGhostBtnClass, mcpKvRowClass } from './mcp-classes'
import { maskSecret } from './mcp-format'

interface McpKvRowProps {
  entryKey: string
  maskedValue: string
  onRemove: (key: string) => void
}

const McpKvRow = memo(function McpKvRow({ entryKey, maskedValue, onRemove }: McpKvRowProps) {
  const handleRemove = useCallback(() => {
    onRemove(entryKey)
  }, [entryKey, onRemove])

  return (
    <div className={mcpKvRowClass}>
      <span className="font-mono text-xs font-semibold text-text-primary">{entryKey}</span>
      <span className="min-w-0 flex-1 overflow-hidden text-ellipsis font-mono text-xs text-text-muted">
        {maskedValue}
      </span>
      <button type="button" className="shrink-0 text-text-muted hover:text-danger" onClick={handleRemove}>
        <SIcon name="X" size="w-3 h-3" />
      </button>
    </div>
  )
})

export interface McpKvEditorProps {
  entries: Record<string, string>
  keyValue: string
  valueValue: string
  onKeyChange: (value: string) => void
  onValueChange: (value: string) => void
  onAdd: () => void
  onRemove: (key: string) => void
  keyPlaceholder: string
  valuePlaceholder: string
}

export function McpKvEditor({
  entries,
  keyValue,
  valueValue,
  onKeyChange,
  onValueChange,
  onAdd,
  onRemove,
  keyPlaceholder,
  valuePlaceholder,
}: McpKvEditorProps) {
  const extra = useForm({ values: { keyValue, valueValue } })

  const handleKeyChange = useCallback(
    (event: { target: EventTarget | null }) => {
      const target = event.target as HTMLInputElement
      onKeyChange(target.value)
    },
    [onKeyChange],
  )

  const handleValueChange = useCallback(
    (event: { target: EventTarget | null }) => {
      const target = event.target as HTMLInputElement
      onValueChange(target.value)
    },
    [onValueChange],
  )

  return (
    <div className="flex flex-col gap-1.5">
      {Object.entries(entries).map(([entryKey, entryValue]) => (
        <McpKvRow
          key={entryKey}
          entryKey={entryKey}
          maskedValue={maskSecret(String(entryValue))}
          onRemove={onRemove}
        />
      ))}
      <div className="flex items-center gap-1.5">
        <input
          {...extra.register('keyValue', { onChange: handleKeyChange })}
          type="text"
          className={mcpFieldInputClass}
          placeholder={keyPlaceholder}
        />
        <input
          {...extra.register('valueValue', { onChange: handleValueChange })}
          type="text"
          className={mcpFieldInputClass}
          placeholder={valuePlaceholder}
        />
        <button type="button" className={mcpGhostBtnClass} disabled={!keyValue || !valueValue} onClick={onAdd}>
          <SIcon name="Plus" size="w-4 h-4" />
        </button>
      </div>
    </div>
  )
}
