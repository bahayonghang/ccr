<template>
  <div
    class="min-h-screen"
    :style="{ background: 'var(--bg-primary)' }"
  >
    <Navbar />

    <!-- 使用重构后的BaseSlashCommands组件 -->
    <BaseSlashCommands :config="codexConfig" />
  </div>
</template>

<script setup lang="ts">
import { codexConfig } from '@/configs/slashCommands'
import BaseSlashCommands from '@/components/BaseSlashCommands.vue'
import Navbar from '@/components/Navbar.vue'
</script>