<template>
  <Layout>
    <div class="mb-8">
      <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--text-primary)' }">插件管理</h1>
      <p class="text-base" :style="{ color: 'var(--text-secondary)' }">插件启用/禁用和配置管理</p>
    </div>

    <!-- Action buttons -->
    <div class="flex flex-wrap gap-3 mb-6">
      <Button variant="primary">
        <Plus class="w-4 h-4 mr-2" />
        添加插件
      </Button>
      <Button variant="secondary">
        <RefreshCw class="w-4 h-4 mr-2" />
        刷新
      </Button>
    </div>

    <!-- Plugins table -->
    <Card>
      <Table 
        :columns="columns" 
        :data="plugins" 
        key-field="id"
        :has-actions="true"
      >
        <template #enabled="{ value }">
          <span 
            class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium"
            :style="{
              background: value ? 'rgba(16, 185, 129, 0.1)' : 'rgba(239, 68, 68, 0.1)',
              color: value ? '#10b981' : '#ef4444',
              border: value ? '1px solid rgba(16, 185, 129, 0.3)' : '1px solid rgba(239, 68, 68, 0.3)'
            }"
          >
            {{ value ? '已启用' : '已禁用' }}
          </span>
        </template>
        
        <template #version="{ value }">
          <span class="px-2 py-1 rounded-full text-xs font-semibold" :style="{
            backgroundColor: 'var(--bg-secondary)',
            color: 'var(--text-muted)',
            border: '1px solid var(--border-color)'
          }">
            v{{ value }}
          </span>
        </template>
        
        <template #actions>
          <div class="flex space-x-2">
            <Button variant="ghost" size="sm">
              <Edit2 class="w-4 h-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Trash2 class="w-4 h-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Power class="w-4 h-4" />
            </Button>
          </div>
        </template>
      </Table>
    </Card>
  </Layout>
</template>

<script setup lang="ts">
import Layout from '@/components/Layout.vue'
import Card from '@/components/Card.vue'
import Button from '@/components/Button.vue'
import Table from '@/components/Table.vue'
import { Plus, RefreshCw, Edit2, Trash2, Power } from 'lucide-vue-next'

const columns = [
  { key: 'name', title: '名称' },
  { key: 'version', title: '版本' },
  { key: 'enabled', title: '状态' },
  { key: 'actions', title: '操作' },
]

const plugins = [
  { id: 'plugin-1', name: 'Code Formatter', version: '1.2.3', enabled: true },
  { id: 'plugin-2', name: 'Code Review', version: '2.0.1', enabled: false },
  { id: 'plugin-3', name: 'Security Scanner', version: '0.9.5', enabled: true },
  { id: 'plugin-4', name: 'Dependency Checker', version: '1.5.0', enabled: true },
]
</script>