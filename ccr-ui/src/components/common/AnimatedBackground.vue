<template>
  <div
    ref="bgRef"
    :class="backgroundLayerClass"
    :style="{ '--animation-state': shouldAnimate ? 'running' : 'paused' }"
  >
    <div
      class="background-layer__base"
      :class="`background-layer__base--${effectiveVariant}`"
    />
    <div class="background-layer__grain" />
    <div
      v-if="effectiveVariant !== 'minimal'"
      class="background-layer__halo background-layer__halo--primary"
      :class="`background-layer__halo--${effectiveVariant}`"
    />
    <div
      v-if="effectiveVariant === 'default' || effectiveVariant === 'complex' || effectiveVariant === 'mesh'"
      class="background-layer__halo background-layer__halo--secondary"
      :class="`background-layer__halo--${effectiveVariant}`"
    />
    <div
      v-if="effectiveVariant === 'spotlight'"
      class="background-layer__spotlight"
      :class="spotlightColorClass"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useAnimationVisibility } from '@/composables/useAnimationVisibility'

type BackgroundVariant = 'default' | 'complex' | 'aurora' | 'spotlight' | 'mesh' | 'orbs' | 'minimal'
type SpotlightColor = 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info'

const props = withDefaults(defineProps<{
  variant?: BackgroundVariant
  spotlightColor?: SpotlightColor
  contained?: boolean
  complex?: boolean
}>(), {
  variant: 'default',
  spotlightColor: 'primary',
  contained: false,
  complex: false,
})

const bgRef = ref<HTMLElement | null>(null)
const { shouldAnimate, prefersReducedMotion } = useAnimationVisibility(bgRef)

const variant = computed(() => (props.complex ? 'complex' : props.variant))
const effectiveVariant = computed<BackgroundVariant>(() => (prefersReducedMotion.value ? 'minimal' : variant.value))

const backgroundLayerClass = computed(() => (
  props.contained
    ? 'background-layer background-layer--contained absolute inset-0 overflow-hidden pointer-events-none -z-10 transition-colors duration-500'
    : 'background-layer fixed inset-0 overflow-hidden pointer-events-none -z-10 transition-colors duration-500'
))

const spotlightColorClass = computed(() => `background-layer__spotlight--${props.spotlightColor}`)
</script>

<style scoped>
.background-layer {
  isolation: isolate;
  background: transparent;
}

.background-layer__base,
.background-layer__halo,
.background-layer__grain,
.background-layer__spotlight {
  position: absolute;
  inset: 0;
}

.background-layer__base {
  background:
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 82%), rgb(var(--color-bg-base-rgb) / 100%));
}

.background-layer__base--default,
.background-layer__base--complex {
  background:
    radial-gradient(circle at 12% 0%, rgb(var(--color-accent-primary-rgb) / 6%), transparent 28%),
    radial-gradient(circle at 92% 10%, rgb(var(--color-accent-secondary-rgb) / 5%), transparent 22%),
    linear-gradient(180deg, rgb(var(--color-premium-pink-rgb) / 34%), transparent 30%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 78%), rgb(var(--color-bg-base-rgb) / 100%));
}

.background-layer__base--aurora {
  background:
    linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 5%), transparent 24%),
    radial-gradient(circle at 82% 14%, rgb(var(--color-accent-secondary-rgb) / 4%), transparent 22%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 82%), rgb(var(--color-bg-base-rgb) / 100%));
}

.background-layer__base--mesh,
.background-layer__base--orbs {
  background:
    radial-gradient(circle at 14% 12%, rgb(var(--color-premium-blue-rgb) / 44%), transparent 28%),
    radial-gradient(circle at 86% 0%, rgb(var(--color-accent-primary-rgb) / 6%), transparent 22%),
    linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 76%), rgb(var(--color-bg-base-rgb) / 100%));
}

.background-layer__base--minimal {
  background: linear-gradient(180deg, rgb(var(--color-bg-base-rgb) / 88%), rgb(var(--color-bg-base-rgb) / 100%));
}

.background-layer__grain {
  opacity: 0.024;
  background-image: radial-gradient(rgb(var(--color-text-primary-rgb) / 22%) 0.8px, transparent 0.8px);
  background-size: 18px 18px;
  mask-image: linear-gradient(180deg, rgb(0 0 0 / 72%), transparent);
}

.background-layer__halo {
  filter: blur(88px);
  animation: ambient-drift 20s ease-in-out infinite;
  animation-play-state: var(--animation-state);
}

.background-layer__halo--primary {
  inset: -12% auto auto -10%;
  width: 34vw;
  height: 34vw;
  border-radius: 50%;
  background: rgb(var(--color-accent-primary-rgb) / 9%);
}

.background-layer__halo--secondary {
  inset: auto -12% -18% auto;
  width: 30vw;
  height: 30vw;
  border-radius: 50%;
  background: rgb(var(--color-premium-blue-rgb) / 54%);
  animation-duration: 24s;
}

.background-layer__halo--aurora {
  background: rgb(var(--color-accent-secondary-rgb) / 8%);
}

.background-layer__halo--mesh,
.background-layer__halo--orbs {
  background: rgb(var(--color-premium-blue-rgb) / 64%);
}

.background-layer__spotlight {
  mask-image: radial-gradient(circle at 50% 0%, rgb(0 0 0 / 88%), transparent 52%);
}

.background-layer__spotlight--primary {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-accent-primary-rgb) / 12%), transparent 52%);
}

.background-layer__spotlight--secondary {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-accent-secondary-rgb) / 12%), transparent 52%);
}

.background-layer__spotlight--success {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-success-rgb) / 12%), transparent 52%);
}

.background-layer__spotlight--warning {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-warning-rgb) / 12%), transparent 52%);
}

.background-layer__spotlight--danger {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-danger-rgb) / 12%), transparent 52%);
}

.background-layer__spotlight--info {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-info-rgb) / 12%), transparent 52%);
}

[data-theme='dark'] .background-layer__grain {
  opacity: 0.04;
}

[data-theme='dark'] .background-layer__halo--primary {
  background: rgb(var(--color-accent-primary-rgb) / 11%);
}

[data-theme='dark'] .background-layer__halo--secondary,
[data-theme='dark'] .background-layer__halo--mesh,
[data-theme='dark'] .background-layer__halo--orbs {
  background: rgb(var(--color-premium-blue-rgb) / 72%);
}

@keyframes ambient-drift {
  0%,
  100% {
    transform: translate3d(0, 0, 0) scale(1);
    opacity: 0.8;
  }

  50% {
    transform: translate3d(18px, -18px, 0) scale(1.08);
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .background-layer__halo {
    display: none;
  }
}
</style>
