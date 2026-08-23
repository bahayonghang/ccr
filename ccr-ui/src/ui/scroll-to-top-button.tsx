import { AnimatePresence, motion } from 'motion/react'
import { cn } from './cn'
import { SIcon } from './s-icon'

interface ScrollToTopButtonProps {
  visible: boolean
  buttonLabel: string
  label: string
  onClick: () => void
  className?: string
}

export function ScrollToTopButton({
  visible,
  buttonLabel,
  label,
  onClick,
  className,
}: ScrollToTopButtonProps) {
  return (
    <AnimatePresence>
      {visible ? (
        <motion.div
          className={cn('scroll-to-top', className)}
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 12 }}
          transition={{ duration: 0.18 }}
        >
          <button
            type="button"
            className="scroll-to-top__button"
            data-testid="main-scroll-to-top"
            aria-label={buttonLabel}
            title={buttonLabel}
            onClick={onClick}
          >
            <SIcon name="ChevronUp" size="w-4 h-4" />
            <span className="scroll-to-top__label">{label}</span>
          </button>
        </motion.div>
      ) : null}
    </AnimatePresence>
  )
}
