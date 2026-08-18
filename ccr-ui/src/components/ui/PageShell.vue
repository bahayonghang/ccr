<template>
  <div class="page-shell">
    <div class="page-shell__inner">
      <div
        v-if="$slots.header"
        class="page-shell__header"
      >
        <slot name="header" />
      </div>

      <div
        v-if="$slots.subnav"
        class="page-shell__subnav"
      >
        <slot name="subnav" />
      </div>

      <div class="page-shell__content">
        <slot />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 页壳：页头槽 + 可选 subnav + 内容栅格。表面按 workspace，无装饰渐变 / inset 高光。
</script>

<style scoped>
.page-shell {
  position: relative;
  min-height: 100%;
  padding: 1rem 1.25rem;
}

.page-shell__inner {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
  max-width: 1480px;
  margin-inline: auto;
}

.page-shell__header,
.page-shell__subnav {
  min-width: 0;
}

.page-shell__content {
  display: grid;
  gap: 1.25rem;
  min-width: 0;
}

@media (width >= 640px) {
  .page-shell {
    padding: 1.5rem;
  }
}
</style>
