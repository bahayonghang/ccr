<template>
  <div
    class="ring-progress-container"
    :style="{ width: `${size}px`, height: `${size}px` }"
  >
    <!-- Background glow effect -->
    <div
      class="absolute inset-0 rounded-full blur-xl opacity-30 transition-opacity duration-500"
      :style="{ background: `radial-gradient(circle, ${glowColor} 0%, transparent 70%)` }"
    />

    <!-- SVG Ring -->
    <svg
      class="ring-svg transform -rotate-90"
      :width="size"
      :height="size"
      :viewBox="`0 0 ${size} ${size}`"
    >
      <!-- Background track -->
      <circle
        class="track"
        :cx="center"
        :cy="center"
        :r="radius"
        fill="none"
        :stroke="trackColor"
        :stroke-width="strokeWidth"
      />

      <!-- Progress arc -->
      <circle
        class="progress-ring"
        :cx="center"
        :cy="center"
        :r="radius"
        fill="none"
        :stroke="progressGradientId ? `url(#${progressGradientId})` : strokeColor"
        :stroke-width="strokeWidth"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="strokeDashoffset"
        stroke-linecap="round"
        :style="{ transition: `stroke-dashoffset ${animationDuration}ms ${easing}` }"
      />

      <!-- Gradient definition -->
      <defs>
        <linearGradient
          :id="progressGradientId"
          x1="0%"
          y1="0%"
          x2="100%"
          y2="0%"
        >
          <stop
            offset="0%"
            :stop-color="gradientStart"
          />
          <stop
            offset="100%"
            :stop-color="gradientEnd"
          />
        </linearGradient>
      </defs>
    </svg>

    <!-- Center content -->
    <div class="center-content absolute inset-0 flex flex-col items-center justify-center">
      <slot>
        <span
          class="value font-mono font-bold tracking-tight"
          :style="{ fontSize: `${size * 0.2}px`, color: textColor }"
        >
          <AnimatedCounter
            :value="displayValue"
            :format="valueFormat"
            :decimals="1"
            :duration="animationDuration"
          />
        </span>
        <span
          v-if="label"
          class="label text-center font-medium opacity-70"
          :style="{ fontSize: `${size * 0.1}px` }"
        >
          {{ label }}
        </span>
      </slot>
    </div>

    <!-- Animated pulse ring on high values -->
    <div
      v-if="value >= 80"
      class="absolute inset-0 rounded-full animate-ping opacity-20"
      :style="{ borderColor: strokeColor, borderWidth: '2px', borderStyle: 'solid' }"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted } from 'vue'
import AnimatedCounter from './AnimatedCounter.vue'

interface Props {
  value: number // 0-100
  size?: number
  strokeWidth?: number
  strokeColor?: string
  trackColor?: string
  textColor?: string
  label?: string
  gradientStart?: string
  gradientEnd?: string
  animationDuration?: number
  easing?: string
  valueFormat?: 'number' | 'compact' | 'percent'
}

const props = withDefaults(defineProps<Props>(), {
  size: 120,
  strokeWidth: 8,
  strokeColor: 'var(--color-accent-primary)',
  trackColor: 'var(--color-border-subtle)',
  textColor: 'var(--color-text-primary)',
  gradientStart: '#8B5CF6',
  gradientEnd: '#06B6D4',
  animationDuration: 1000,
  easing: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
  valueFormat: 'percent'
})

const displayValue = ref(0)
const progressGradientId = computed(() => `ring-gradient-${Math.random().toString(36).substr(2, 9)}`)

const center = computed(() => props.size / 2)
const radius = computed(() => (props.size - props.strokeWidth) / 2)
const circumference = computed(() => 2 * Math.PI * radius.value)

const strokeDashoffset = computed(() => {
  const progress = Math.min(Math.max(displayValue.value, 0), 100) / 100
  return circumference.value * (1 - progress)
})

const glowColor = computed(() => {
  if (props.value >= 80) return 'var(--color-success)'
  if (props.value >= 50) return 'var(--color-warning)'
  return 'var(--color-accent-primary)'
})

// Animate value on mount and change
const animateValue = (from: number, to: number) => {
  const startTime = performance.now()
  const duration = props.animationDuration

  const animate = (currentTime: number) => {
    const elapsed = currentTime - startTime
    const progress = Math.min(elapsed / duration, 1)

    // Ease out expo
    const easeProgress = progress === 1 ? 1 : 1 - Math.pow(2, -10 * progress)
    displayValue.value = from + (to - from) * easeProgress

    if (progress < 1) {
      requestAnimationFrame(animate)
    }
  }

  requestAnimationFrame(animate)
}

watch(
  () => props.value,
  (newVal, oldVal) => {
    animateValue(oldVal || 0, newVal)
  }
)

onMounted(() => {
  animateValue(0, props.value)
})
</script>

<style scoped>
.ring-progress-container {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ring-svg {
  position: absolute;
  top: 0;
  left: 0;
}

.track {
  opacity: 0.3;
}

.progress-ring {
  filter: drop-shadow(0 0 6px var(--color-accent-primary-glow));
  transition: stroke-dashoffset 1s var(--ease-out-back);
}

.center-content {
  z-index: 1;
}

.value {
  line-height: 1;
}

.label {
  margin-top: 2px;
  color: var(--color-text-muted);
}

/* Pulse animation for high values */
@keyframes ring-pulse {
  0%, 100% {
    transform: scale(1);
    opacity: 0.3;
  }
  50% {
    transform: scale(1.05);
    opacity: 0.1;
  }
}
</style>
