<template>
  <div class="master-detail">
    <!-- 左侧列表面板 -->
    <div
      class="master-detail__list"
      :style="listStyle"
    >
      <slot name="list" />
    </div>

    <!-- 右侧详情面板 -->
    <div class="master-detail__detail">
      <slot name="detail" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  /** 左侧面板宽度 (CSS 值) */
  listWidth?: string
}>(), {
  listWidth: '20rem',
})

const listStyle = computed(() => ({
  width: props.listWidth,
}))
</script>

<style scoped>
.master-detail {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.master-detail__list {
  position: relative;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  border-right: 1px solid var(--surface-workspace-border, rgb(var(--color-border-default-rgb) / 45%));
  overflow: hidden;
}

.master-detail__detail {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  position: relative;
}

/* 移动端：隐藏列表，仅显示详情 (或反之) — 后续可扩展 */
@media (width <= 768px) {
  .master-detail {
    flex-direction: column;
  }

  .master-detail__list {
    width: 100% !important;
    max-height: 40vh;
    border-right: none;
    border-bottom: 1px solid var(--surface-workspace-border, rgb(var(--color-border-default-rgb) / 45%));
  }

  .master-detail__detail {
    flex: 1;
  }
}
</style>
