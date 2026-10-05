import type { ReactNode } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import { cn } from './cn'
import { SIcon } from './s-icon'

interface MultiSelectFloatingBarProps {
  selectedCount: number
  totalCount: number
  showDelete?: boolean
  countLabel?: string
  deleteLabel?: string
  deleteAriaLabel?: string
  onDelete?: () => void
  children?: ReactNode
  className?: string
}

export function MultiSelectFloatingBar({
  selectedCount,
  totalCount,
  showDelete = true,
  countLabel,
  deleteLabel,
  deleteAriaLabel,
  onDelete,
  children,
  className,
}: MultiSelectFloatingBarProps) {
  return (
    <AnimatePresence>
      {selectedCount > 0 ? (
        <motion.div
          className={cn('multi-select-bar', className)}
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 16 }}
          transition={{ duration: 0.2 }}
        >
          <span className="multi-select-bar__count">
            {countLabel || `${selectedCount} / ${totalCount} selected`}
          </span>
          <div className="multi-select-bar__actions">
            {children}
            {showDelete ? (
              <button
                type="button"
                className="multi-select-bar__btn multi-select-bar__btn--danger"
                aria-label={deleteAriaLabel || `Delete ${selectedCount} items`}
                onClick={onDelete}
              >
                <SIcon name="Trash2" size="w-4 h-4" />
                <span>{deleteLabel || 'Delete'}</span>
              </button>
            ) : null}
          </div>
        </motion.div>
      ) : null}
    </AnimatePresence>
  )
}
