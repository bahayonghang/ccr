<template>
  <transition name="scroll-to-top">
    <div
      v-if="visible"
      class="scroll-to-top"
    >
      <button
        type="button"
        class="scroll-to-top__button"
        data-testid="main-scroll-to-top"
        :aria-label="buttonLabel"
        :title="buttonLabel"
        @click="$emit('click')"
      >
        <SIcon
          name="ChevronUp"
          size="w-4 h-4"
        />
        <span class="scroll-to-top__label">{{ label }}</span>
      </button>
    </div>
  </transition>
</template>

<script setup lang="ts">
import SIcon from '@/components/ui/SIcon.vue'

defineProps<{
  buttonLabel: string
  label: string
  visible: boolean
}>()

defineEmits<{
  click: []
}>()
</script>

<style scoped>
.scroll-to-top {
  position: absolute;
  right: 1rem;
  bottom: 1rem;
  z-index: var(--layer-floating, 30);
  pointer-events: none;
}

.scroll-to-top__button {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
  min-height: 2.9rem;
  padding: 0.7rem 0.95rem;
  border-radius: var(--radius-full);
  border: 1px solid rgb(var(--color-accent-secondary-rgb) / 26%);
  background: rgb(var(--color-bg-elevated-rgb) / 94%);
  box-shadow: var(--shadow-md);
  color: var(--color-accent-secondary);
  font-size: 0.8rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  pointer-events: auto;
  transition:
    transform 180ms ease,
    border-color 180ms ease,
    background-color 180ms ease,
    color 180ms ease,
    box-shadow 180ms ease;
}

.scroll-to-top__button:hover {
  transform: translateY(-2px);
  border-color: rgb(var(--color-accent-secondary-rgb) / 36%);
  background:
    linear-gradient(180deg, rgb(var(--color-bg-elevated-rgb) / 88%), rgb(var(--color-accent-secondary-rgb) / 14%));
  box-shadow:
    0 22px 38px rgb(6 8 18 / 26%),
    0 0 0 1px rgb(var(--color-accent-secondary-rgb) / 12%);
}

.scroll-to-top__button:focus-visible {
  outline: 2px solid rgb(var(--color-accent-secondary-rgb) / 42%);
  outline-offset: 3px;
}

.scroll-to-top__label {
  white-space: nowrap;
}

.scroll-to-top-enter-active,
.scroll-to-top-leave-active {
  transition: opacity 180ms ease, transform 180ms ease;
}

.scroll-to-top-enter-from,
.scroll-to-top-leave-to {
  opacity: 0;
  transform: translateY(12px);
}

@media (width >= 768px) {
  .scroll-to-top {
    right: 1.5rem;
    bottom: 1.5rem;
  }
}

@media (width < 640px) {
  .scroll-to-top__button {
    min-width: 2.9rem;
    justify-content: center;
    padding-inline: 0.82rem;
  }

  .scroll-to-top__label {
    display: none;
  }
}
</style>