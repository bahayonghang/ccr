<template>
  <div 
    class="glass-card relative overflow-hidden rounded-2xl border border-white/10 bg-black/40 shadow-2xl backdrop-blur-xl"
    :class="[paddingClass]"
  >
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps({
  padding: {
    type: String,
    default: 'md' // none, sm, md, lg
  }
})

const paddingClass = computed(() => {
  switch (props.padding) {
    case 'none': return 'p-0'
    case 'sm': return 'p-4'
    case 'lg': return 'p-8'
    case 'md':
    default: return 'p-6'
  }
})
</script>

<style scoped>
.glass-card {
  transform: translateZ(0);
  /* 强制硬件加速并避免子元素复杂重绘 */
}
</style>
