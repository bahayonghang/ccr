<template>
  <div
    class="min-h-screen"
    :style="{ background: 'var(--bg-primary)' }"
  >
    <Navbar />

    <!-- 使用重构后的BaseSlashCommands组件 -->
    <BaseSlashCommands :config="iflowConfig" />
  </div>
</template>

<script setup lang="ts">
import { iflowConfig } from '@/configs/slashCommands'
import BaseSlashCommands from '@/components/BaseSlashCommands.vue'
import Navbar from '@/components/Navbar.vue'
</script>