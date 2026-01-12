<template>
  <span class="animated-counter font-mono tabular-nums">
    <span
      v-for="(char, index) in displayChars"
      :key="index"
      class="char-wrapper inline-block"
      :style="{ animationDelay: `${index * 30}ms` }"
    >
      <span
        v-if="isDigit(char)"
        class="digit"
        :class="{ 'animate-roll': isAnimating }"
      >
        {{ char }}
      </span>
      <span
        v-else
        class="separator"
      >{{ char }}</span>
    </span>
  </span>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue'

interface Props {
  value: number
  duration?: number
  format?: 'number' | 'compact' | 'percent'
  decimals?: number
}

const props = withDefaults(defineProps<Props>(), {
  duration: 1000,
  format: 'compact',
  decimals: 1
})

const displayValue = ref(0)
const isAnimating = ref(false)
let animationFrame: number | null = null

const formatValue = (val: number): string => {
  if (props.format === 'percent') {
    return `${val.toFixed(props.decimals)}%`
  }
  if (props.format === 'compact') {
    if (val >= 1000000) {
      return `${(val / 1000000).toFixed(props.decimals)}M`
    }
    if (val >= 1000) {
      return `${(val / 1000).toFixed(props.decimals)}K`
    }
    return val.toFixed(0)
  }
  return new Intl.NumberFormat('en-US').format(Math.round(val))
}

const displayChars = computed(() => {
  return formatValue(displayValue.value).split('')
})

const isDigit = (char: string): boolean => {
  return /[0-9]/.test(char)
}

const animateValue = (from: number, to: number) => {
  if (animationFrame) {
    cancelAnimationFrame(animationFrame)
  }

  const startTime = performance.now()
  isAnimating.value = true

  const animate = (currentTime: number) => {
    const elapsed = currentTime - startTime
    const progress = Math.min(elapsed / props.duration, 1)

    // Easing function: easeOutExpo
    const easeProgress = progress === 1 ? 1 : 1 - Math.pow(2, -10 * progress)

    displayValue.value = from + (to - from) * easeProgress

    if (progress < 1) {
      animationFrame = requestAnimationFrame(animate)
    } else {
      displayValue.value = to
      isAnimating.value = false
    }
  }

  animationFrame = requestAnimationFrame(animate)
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
.animated-counter {
  display: inline-flex;
  align-items: baseline;
}

.char-wrapper {
  position: relative;
  overflow: hidden;
}

.digit {
  display: inline-block;
  transition: transform 0.15s var(--ease-out);
}

.digit.animate-roll {
  animation: digit-roll 0.3s var(--ease-out-back);
}

.separator {
  opacity: 0.7;
}

@keyframes digit-roll {
  0% {
    transform: translateY(-100%);
    opacity: 0;
  }
  50% {
    opacity: 0.5;
  }
  100% {
    transform: translateY(0);
    opacity: 1;
  }
}
</style>
