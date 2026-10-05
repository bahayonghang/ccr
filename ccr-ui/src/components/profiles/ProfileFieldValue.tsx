import type { ProfileFieldKind } from '@/configs/profilePresentation'
import { Badge, UrlText } from '@/ui'

export interface ProfileFieldValueProps {
  kind?: ProfileFieldKind
  value: string
  placeholder: string
}

/** 卡片 / 表格字段值：按 presentation kind 渲染，不含平台名分支。 */
export function ProfileFieldValue({ kind = 'text', value, placeholder }: ProfileFieldValueProps) {
  const display = value || placeholder

  switch (kind) {
    case 'url':
      if (!value) return <span>{placeholder}</span>
      return <UrlText value={value} />
    case 'chip':
      return (
        <Badge mode="static" tone="neutral">
          {display}
        </Badge>
      )
    case 'text':
      return <span>{display}</span>
    default: {
      const _exhaustive: never = kind
      return _exhaustive
    }
  }
}
