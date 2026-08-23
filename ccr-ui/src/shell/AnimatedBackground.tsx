import { cn } from '@/ui/cn'

type BackgroundVariant = 'default' | 'complex' | 'aurora' | 'spotlight' | 'mesh' | 'orbs' | 'minimal'

interface AnimatedBackgroundProps {
  variant?: BackgroundVariant
  contained?: boolean
}

/** 氛围层壳。动画层已收敛为静态基底，保留 props 以免消费方改签名。 */
export function AnimatedBackground({ contained = false }: AnimatedBackgroundProps) {
  return (
    <div
      className={cn(
        'background-layer overflow-hidden pointer-events-none -z-10 transition-colors duration-500',
        contained ? 'absolute inset-0' : 'fixed inset-0',
      )}
      aria-hidden="true"
    />
  )
}
