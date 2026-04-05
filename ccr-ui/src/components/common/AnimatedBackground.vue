<template>
  <div
    ref="bgRef"
    :class="backgroundLayerClass"
    :style="{ '--animation-state': shouldAnimate ? 'running' : 'paused' }"
  >
    <!-- Variant: Default / Complex - Multi-layer mesh gradient -->
    <template v-if="effectiveVariant === 'default' || effectiveVariant === 'complex'">
      <!-- 1. Base Mesh Gradient (Subtle shifting colors) -->
      <div class="absolute inset-0 opacity-24 dark:opacity-18">
        <div class="absolute top-0 left-0 h-full w-full mesh-glow mesh-glow--primary-top animate-pulse-slow" />
        <div class="absolute bottom-0 right-0 h-full w-full mesh-glow mesh-glow--secondary-bottom animate-pulse-slow delay-1000" />
        <div
          v-if="effectiveVariant === 'complex'"
          class="absolute top-1/2 left-1/2 h-full w-full -translate-x-1/2 -translate-y-1/2 mesh-glow mesh-glow--info-center opacity-40 animate-pulse-slower"
        />
      </div>

      <!-- 2. Cyber Grid Pattern -->
      <div class="absolute inset-0 cyber-grid cyber-grid-mask" />

      <!-- 3. Noise Texture (Film Grain Effect) -->
      <div class="noise-overlay" />
    </template>

    <!-- Variant: Aurora - Northern lights effect -->
    <template v-else-if="effectiveVariant === 'aurora'">
      <div class="absolute inset-0">
        <!-- Aurora waves -->
        <div class="aurora-wave aurora-wave-1" />
        <div class="aurora-wave aurora-wave-2" />
      </div>
      <div class="noise-overlay" />
    </template>

    <!-- Variant: Spotlight - Single focused glow -->
    <template v-else-if="effectiveVariant === 'spotlight'">
      <div
        class="absolute inset-0"
        :class="spotlightColorClass"
      />
      <div class="absolute inset-0 cyber-grid cyber-grid-mask opacity-50" />
      <div class="noise-overlay" />
    </template>

    <!-- Variant: Mesh - Multi-point gradient mesh -->
    <template v-else-if="effectiveVariant === 'mesh'">
      <div class="absolute inset-0 mesh-gradient animate-mesh-shift" />
      <div class="noise-overlay" />
    </template>

    <!-- Variant: Orbs - Floating gradient orbs -->
    <template v-else-if="effectiveVariant === 'orbs'">
      <div class="absolute inset-0">
        <div class="orb orb-1" />
        <div class="orb orb-2" />
      </div>
      <div class="absolute inset-0 cyber-grid opacity-30" />
      <div class="noise-overlay" />
    </template>

    <!-- Variant: Minimal - Subtle single gradient -->
    <template v-else-if="effectiveVariant === 'minimal'">
      <div class="absolute inset-0 minimal-gradient" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAnimationVisibility } from '@/composables/useAnimationVisibility'

type BackgroundVariant = 'default' | 'complex' | 'aurora' | 'spotlight' | 'mesh' | 'orbs' | 'minimal'
type SpotlightColor = 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info'

const props = withDefaults(defineProps<{
  /** Background variant style */
  variant?: BackgroundVariant
  /** Spotlight color (only for spotlight variant) */
  spotlightColor?: SpotlightColor
  /** Render inside the route container instead of owning the viewport */
  contained?: boolean
  /** Legacy prop for backward compatibility */
  complex?: boolean
}>(), {
  variant: 'default',
  spotlightColor: 'primary',
  contained: false,
  complex: false
})

const bgRef = ref<HTMLElement | null>(null)
const { shouldAnimate, prefersReducedMotion } = useAnimationVisibility(bgRef)

// Computed variant considering legacy 'complex' prop and reduced motion preference
const variant = computed(() => {
  if (props.complex) return 'complex'
  return props.variant
})

// 当用户偏好 reduced motion 时，降级为 minimal 变体
const effectiveVariant = computed<BackgroundVariant>(() => {
  if (prefersReducedMotion.value) return 'minimal'
  return variant.value
})

const backgroundLayerClass = computed(() => (
  props.contained
    ? 'background-layer background-layer--contained absolute inset-0 overflow-hidden pointer-events-none -z-10 transition-colors duration-500'
    : 'background-layer fixed inset-0 overflow-hidden pointer-events-none -z-10 transition-colors duration-500'
))

// Spotlight color class mapping
const spotlightColorClass = computed(() => {
  const colorMap: Record<SpotlightColor, string> = {
    primary: 'spotlight-gradient spotlight-gradient--primary',
    secondary: 'spotlight-gradient spotlight-gradient--secondary',
    success: 'spotlight-gradient spotlight-gradient--success',
    warning: 'spotlight-gradient spotlight-gradient--warning',
    danger: 'spotlight-gradient spotlight-gradient--danger',
    info: 'spotlight-gradient spotlight-gradient--info'
  }
  return colorMap[props.spotlightColor]
})
</script>

<style scoped>
/* ========== Animation Keyframes ========== */
.animate-pulse-slow {
  animation: pulse-glow 8s ease-in-out infinite;
  animation-play-state: var(--animation-state);
}

.animate-pulse-slower {
  animation: pulse-glow 12s ease-in-out infinite;
  animation-play-state: var(--animation-state);
}

.animate-mesh-shift {
  animation: mesh-shift 20s ease-in-out infinite;
  animation-play-state: var(--animation-state);
}

@keyframes pulse-glow {
  0%, 100% {
    opacity: 0.3;
    transform: scale(1);
  }

  50% {
    opacity: 0.5;
    transform: scale(1.1);
  }
}

@keyframes mesh-shift {
  0%, 100% {
    background-position: 0% 0%, 100% 0%, 0% 100%, 100% 100%;
  }

  25% {
    background-position: 100% 0%, 0% 50%, 50% 100%, 100% 50%;
  }

  50% {
    background-position: 50% 50%, 100% 100%, 0% 0%, 50% 50%;
  }

  75% {
    background-position: 0% 100%, 50% 0%, 100% 50%, 0% 0%;
  }
}

@keyframes aurora-flow {
  0% {
    transform: translateY(0) scaleY(1);
    opacity: 0.3;
  }

  50% {
    transform: translateY(-10%) scaleY(1.2);
    opacity: 0.5;
  }

  100% {
    transform: translateY(0) scaleY(1);
    opacity: 0.3;
  }
}

@keyframes orb-float-1 {
  0%, 100% {
    transform: translate(0, 0) scale(1);
  }

  33% {
    transform: translate(30px, -50px) scale(1.1);
  }

  66% {
    transform: translate(-20px, 20px) scale(0.9);
  }
}

@keyframes orb-float-2 {
  0%, 100% {
    transform: translate(0, 0) scale(1);
  }

  50% {
    transform: translate(-40px, -30px) scale(1.15);
  }
}

@keyframes orb-float-3 {
  0%, 100% {
    transform: translate(0, 0) rotate(0deg);
  }

  50% {
    transform: translate(20px, 40px) rotate(180deg);
  }
}

/* ========== Cyber Grid ========== */
.cyber-grid {
  background-image:
    linear-gradient(to right, rgb(var(--color-accent-primary-rgb) / 4%) 1px, transparent 1px),
    linear-gradient(to bottom, rgb(var(--color-accent-primary-rgb) / 4%) 1px, transparent 1px);
  background-size: 4rem 4rem;
}

[data-theme="dark"] .cyber-grid {
  background-image:
    linear-gradient(to right, rgb(var(--color-accent-primary-rgb) / 6%) 1px, transparent 1px),
    linear-gradient(to bottom, rgb(var(--color-accent-primary-rgb) / 6%) 1px, transparent 1px);
}

.cyber-grid-mask {
  mask-image: radial-gradient(ellipse 120% 140% at 50% 30%, #000 20%, transparent 85%);
}

.background-layer--contained .cyber-grid {
  background-image:
    linear-gradient(to right, rgb(var(--color-accent-primary-rgb) / var(--stage-bg-grid-opacity)) 1px, transparent 1px),
    linear-gradient(to bottom, rgb(var(--color-accent-primary-rgb) / var(--stage-bg-grid-opacity)) 1px, transparent 1px);
}

[data-theme="dark"] .background-layer--contained .cyber-grid {
  background-image:
    linear-gradient(to right, rgb(var(--color-accent-primary-rgb) / var(--stage-bg-grid-opacity-dark)) 1px, transparent 1px),
    linear-gradient(to bottom, rgb(var(--color-accent-primary-rgb) / var(--stage-bg-grid-opacity-dark)) 1px, transparent 1px);
}

.mesh-glow--primary-top {
  background: radial-gradient(
    circle at 50% 0%,
    rgb(var(--color-accent-primary-rgb) / 18%),
    transparent 46%
  );
}

.mesh-glow--secondary-bottom {
  background: radial-gradient(
    circle at 100% 100%,
    rgb(var(--color-accent-secondary-rgb) / 14%),
    transparent 46%
  );
}

.mesh-glow--info-center {
  background: radial-gradient(
    circle at 50% 50%,
    rgb(var(--color-info-rgb) / 10%),
    transparent 56%
  );
}

.background-layer--contained .mesh-glow--primary-top {
  background: radial-gradient(
    circle at 50% 0%,
    rgb(var(--color-accent-primary-rgb) / var(--stage-bg-mesh-primary-opacity)),
    transparent 46%
  );
}

.background-layer--contained .mesh-glow--secondary-bottom {
  background: radial-gradient(
    circle at 100% 100%,
    rgb(var(--color-accent-secondary-rgb) / var(--stage-bg-mesh-secondary-opacity)),
    transparent 46%
  );
}

.background-layer--contained .mesh-glow--info-center {
  background: radial-gradient(
    circle at 50% 50%,
    rgb(var(--color-info-rgb) / var(--stage-bg-mesh-info-opacity)),
    transparent 56%
  );
}

.spotlight-gradient--primary {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-accent-primary-rgb) / 18%), transparent 60%);
}

.spotlight-gradient--secondary {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-accent-secondary-rgb) / 18%), transparent 60%);
}

.spotlight-gradient--success {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-success-rgb) / 18%), transparent 60%);
}

.spotlight-gradient--warning {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-warning-rgb) / 18%), transparent 60%);
}

.spotlight-gradient--danger {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-danger-rgb) / 18%), transparent 60%);
}

.spotlight-gradient--info {
  background: radial-gradient(circle at 50% 0%, rgb(var(--color-info-rgb) / 18%), transparent 60%);
}

.minimal-gradient {
  background: radial-gradient(
    ellipse 78% 46% at 50% -18%,
    rgb(var(--color-accent-primary-rgb) / 10%),
    transparent
  );
}

/* ========== Noise Overlay ========== */
.noise-overlay {
  position: absolute;
  inset: 0;
  opacity: 0.02;
  mix-blend-mode: overlay;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)' opacity='1'/%3E%3C/svg%3E");
}

[data-theme="dark"] .noise-overlay {
  opacity: 0.035;
}

.background-layer--contained .noise-overlay {
  opacity: var(--stage-bg-noise-opacity);
}

[data-theme="dark"] .background-layer--contained .noise-overlay {
  opacity: var(--stage-bg-noise-opacity-dark);
}

/* ========== Aurora Waves ========== */
.aurora-wave {
  position: absolute;
  width: 200%;
  height: 60%;
  top: -20%;
  left: -50%;
  filter: blur(28px);
  animation: aurora-flow 10s ease-in-out infinite;
  animation-play-state: var(--animation-state);
}

.aurora-wave-1 {
  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / 16%) 0%, transparent 100%);
  animation-delay: 0s;
}

.aurora-wave-2 {
  background: linear-gradient(180deg, rgb(var(--color-accent-secondary-rgb) / 12%) 0%, transparent 100%);
  animation-delay: 2s;
  top: -10%;
}

.background-layer--contained .aurora-wave-1 {
  background: linear-gradient(180deg, rgb(var(--color-accent-primary-rgb) / var(--stage-bg-aurora-primary-opacity)) 0%, transparent 100%);
}

.background-layer--contained .aurora-wave-2 {
  background: linear-gradient(180deg, rgb(var(--color-accent-secondary-rgb) / var(--stage-bg-aurora-secondary-opacity)) 0%, transparent 100%);
}

/* ========== Mesh Gradient ========== */
.mesh-gradient {
  background:
    radial-gradient(at 40% 20%, rgb(var(--color-accent-primary-rgb) / 12%) 0, transparent 50%),
    radial-gradient(at 80% 0%, rgb(var(--color-info-rgb) / 10%) 0, transparent 50%),
    radial-gradient(at 0% 50%, rgb(var(--color-accent-secondary-rgb) / 10%) 0, transparent 50%),
    radial-gradient(at 80% 50%, rgb(var(--color-success-rgb) / 8%) 0, transparent 50%);
  background-size: 100% 100%;
}

[data-theme="dark"] .mesh-gradient {
  background:
    radial-gradient(at 40% 20%, rgb(var(--color-accent-primary-rgb) / 18%) 0, transparent 50%),
    radial-gradient(at 80% 0%, rgb(var(--color-info-rgb) / 15%) 0, transparent 50%),
    radial-gradient(at 0% 50%, rgb(var(--color-accent-secondary-rgb) / 15%) 0, transparent 50%),
    radial-gradient(at 80% 50%, rgb(var(--color-success-rgb) / 12%) 0, transparent 50%);
}

/* ========== Floating Orbs ========== */
.orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(34px);
}

.orb-1 {
  top: -10%;
  left: -10%;
  width: 35vw;
  height: 35vw;
  background: rgb(var(--color-accent-primary-rgb) / 12%);
  animation: orb-float-1 24s ease-in-out infinite;
  animation-play-state: var(--animation-state);
}

.orb-2 {
  bottom: -10%;
  right: -10%;
  width: 30vw;
  height: 30vw;
  background: rgb(var(--color-accent-secondary-rgb) / 10%);
  animation: orb-float-2 28s ease-in-out infinite;
  animation-play-state: var(--animation-state);
}

.background-layer--contained .orb-1 {
  background: rgb(var(--color-accent-primary-rgb) / var(--stage-bg-orb-primary-opacity));
}

.background-layer--contained .orb-2 {
  background: rgb(var(--color-accent-secondary-rgb) / var(--stage-bg-orb-secondary-opacity));
}

[data-theme="dark"] .orb-1 {
  background: rgb(var(--color-accent-primary-rgb) / 16%);
  mix-blend-mode: screen;
}

[data-theme="dark"] .orb-2 {
  background: rgb(var(--color-accent-secondary-rgb) / 14%);
  mix-blend-mode: screen;
}
</style>
