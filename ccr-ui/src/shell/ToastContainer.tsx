import { createPortal } from 'react-dom'
import { AnimatePresence, motion } from 'motion/react'
import { useUIStore, type Toast } from '@/shell/stores/ui'
import { SIcon } from '@/ui/s-icon'

const TOAST_ICON: Record<Toast['type'], string> = {
  success: 'CheckCircle',
  error: 'XCircle',
  warning: 'AlertTriangle',
  info: 'Info',
}

export function ToastContainer() {
  const toasts = useUIStore((state) => state.toasts)
  const removeToast = useUIStore((state) => state.removeToast)
  if (typeof document === 'undefined') return null

  return createPortal(
    <div className="toast-container">
      <AnimatePresence>
        {toasts.map((toast) => (
          <motion.div
            key={toast.id}
            className={`toast toast-${toast.type}`}
            initial={{ opacity: 0, x: 24 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 24 }}
            transition={{ duration: 0.2 }}
            onClick={() => removeToast(toast.id)}
          >
            <SIcon name={TOAST_ICON[toast.type]} className="toast-icon" />
            <span className="toast-message">{toast.message}</span>
            <SIcon name="X" className="toast-close" size="w-4 h-4" />
          </motion.div>
        ))}
      </AnimatePresence>
    </div>,
    document.body,
  )
}
