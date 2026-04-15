<template>
  <McpCreatePanel
    :is-editing="true"
    :form-data="formData"
    :is-http-mode="isHttpMode"
    :arg-input="argInput"
    :env-key="envKey"
    :env-value="envValue"
    :header-key="headerKey"
    :header-value="headerValue"
    :platforms="platforms"
    :platform-meta="platformMeta"
    @submit="$emit('submit')"
    @cancel="$emit('cancel')"
    @update:is-http-mode="$emit('update:isHttpMode', $event)"
    @update:arg-input="$emit('update:argInput', $event)"
    @update:env-key="$emit('update:envKey', $event)"
    @update:env-value="$emit('update:envValue', $event)"
    @update:header-key="$emit('update:headerKey', $event)"
    @update:header-value="$emit('update:headerValue', $event)"
    @update-field="(field: keyof UnifiedMcpRequest, value: unknown) => $emit('updateField', field, value)"
    @add-env="$emit('addEnv')"
    @remove-env="$emit('removeEnv', $event)"
    @add-header="$emit('addHeader')"
    @remove-header="$emit('removeHeader', $event)"
  />
</template>

<script setup lang="ts">
import McpCreatePanel from './McpCreatePanel.vue'
import type { UnifiedMcpRequest, PlatformMeta, UnifiedMcpPlatform } from '@/types/unifiedMcp'

defineProps<{
  formData: UnifiedMcpRequest
  isHttpMode: boolean
  argInput: string
  envKey: string
  envValue: string
  headerKey: string
  headerValue: string
  platforms: UnifiedMcpPlatform[]
  platformMeta: Record<string, PlatformMeta>
}>()

defineEmits<{
  submit: []
  cancel: []
  'update:isHttpMode': [value: boolean]
  'update:argInput': [value: string]
  'update:envKey': [value: string]
  'update:envValue': [value: string]
  'update:headerKey': [value: string]
  'update:headerValue': [value: string]
  'updateField': [field: keyof UnifiedMcpRequest, value: unknown]
  addEnv: []
  removeEnv: [key: string]
  addHeader: []
  removeHeader: [key: string]
}>()
</script>
