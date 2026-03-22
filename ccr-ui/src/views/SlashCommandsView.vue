<template>
  <div class="min-h-full transition-colors duration-300">
    <!-- 使用重构后的BaseSlashCommands组件 -->
    <BaseSlashCommands :config="claudeCodeConfig" />
  </div>
</template>

<script setup lang="ts">
import { claudeCodeConfig } from '@/configs/slashCommands'
import BaseSlashCommands from '@/components/BaseSlashCommands.vue'
</script>
