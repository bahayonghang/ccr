<template>
  <div
    class="rounded-xl p-6 space-y-4"
    :style="{
      background: 'var(--glass-bg)',
      border: '1px solid var(--border-color)'
    }"
  >
    <!-- 头部：标题和状态 -->
    <div class="flex items-center justify-between">
      <Skeleton
        variant="text"
        width="40%"
        class="h-6"
      />
      <Skeleton
        variant="rectangular"
        width="60px"
        height="24px"
        class="rounded-full"
      />
    </div>

    <!-- 内容区域 -->
    <div class="space-y-3">
      <Skeleton
        variant="text"
        width="100%"
      />
      <Skeleton
        variant="text"
        width="80%"
      />
      <Skeleton
        variant="text"
        width="60%"
      />
    </div>

    <!-- 底部：操作按钮 -->
    <div class="flex items-center gap-3 pt-2">
      <Skeleton
        variant="rectangular"
        width="80px"
        height="32px"
        class="rounded-lg"
      />
      <Skeleton
        variant="rectangular"
        width="80px"
        height="32px"
        class="rounded-lg"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import Skeleton from './Skeleton.vue'
</script>
