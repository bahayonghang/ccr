<template>
  <div
    class="min-h-screen"
    :style="{ background: 'var(--bg-primary)' }"
  >
    <Navbar />

    <!-- 使用重构后的BaseSlashCommands组件 -->
    <BaseSlashCommands :config="geminiConfig" />
  </div>
</template>

<script setup lang="ts">
import { geminiConfig } from '@/configs/slashCommands'
import BaseSlashCommands from '@/components/BaseSlashCommands.vue'
import Navbar from '@/components/Navbar.vue'
</script>