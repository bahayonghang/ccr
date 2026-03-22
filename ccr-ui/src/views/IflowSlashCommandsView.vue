<template>
  <div
    class="min-h-full"
    :style="{ background: 'var(--bg-primary)' }"
  >
    <!-- 使用重构后的BaseSlashCommands组件 -->
    <BaseSlashCommands :config="iflowConfig" />
  </div>
</template>

<script setup lang="ts">
import { iflowConfig } from '@/configs/slashCommands'
import BaseSlashCommands from '@/components/BaseSlashCommands.vue'
</script>
