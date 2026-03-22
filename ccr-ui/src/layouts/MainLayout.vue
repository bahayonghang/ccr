<template>
  <MainLayout />
  <span class="hidden">{{ appVersionLabel }}</span>
</template>

<script setup lang="ts">
import { APP_VERSION_LABEL } from '@/config/appMeta'
import MainLayout from '@/components/MainLayout.vue'

const appVersionLabel = APP_VERSION_LABEL
</script>
