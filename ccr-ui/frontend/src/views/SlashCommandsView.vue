<template>
  <div class="min-h-screen transition-colors duration-300">
    <Navbar />

    <!-- 使用重构后的BaseSlashCommands组件 -->
    <BaseSlashCommands :config="claudeCodeConfig" />
  </div>
</template>

<script setup lang="ts">
import { claudeCodeConfig } from '@/configs/slashCommands'
import BaseSlashCommands from '@/components/BaseSlashCommands.vue'
import Navbar from '@/components/Navbar.vue'
</script>