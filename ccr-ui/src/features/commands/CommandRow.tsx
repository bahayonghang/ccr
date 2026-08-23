import { memo, useCallback } from 'react'
import { commandBadges, type CommandBadge, type CommandUiInfo } from './commands-model'

interface CommandRowProps {
  command: CommandUiInfo
  active: boolean
  badgeLabel: (badge: CommandBadge) => string
  onSelect: (name: string) => void
}

export const CommandRow = memo(function CommandRow({ command, active, badgeLabel, onSelect }: CommandRowProps) {
  const handleClick = useCallback(() => {
    onSelect(command.name)
  }, [command.name, onSelect])
  return (
    <button
      type="button"
      className={`command-row${active ? ' command-row--active' : ''}${command.executable ? '' : ' command-row--disabled'}`}
      onClick={handleClick}
    >
      <div className="command-row__topline">
        <strong>{command.name}</strong>
        {commandBadges(command).map((badge) => (
          <span key={badge} className={`command-badge command-badge--${badge}`}>{badgeLabel(badge)}</span>
        ))}
      </div>
      <p>{command.description}</p>
    </button>
  )
})
