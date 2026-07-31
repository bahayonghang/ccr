<template>
  <div
    class="claude-background"
    aria-hidden="true"
  />
</template>

<style scoped>
/* 氛围层收敛：静态不透明基底（halo / grain / 顶部洗色带已随配色系统重构移除） */
.claude-background {
  position: fixed;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
  z-index: var(--layer-background);
  background: var(--color-bg-base);
}
</style>
