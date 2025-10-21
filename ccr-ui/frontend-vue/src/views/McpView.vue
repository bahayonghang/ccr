<template>
  <Layout>
    <div class="mb-8">
      <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--text-primary)' }">MCP 服务器</h1>
      <p class="text-base" :style="{ color: 'var(--text-secondary)' }">Model Context Protocol 服务器管理</p>
    </div>

    <!-- Action buttons -->
    <div class="flex flex-wrap gap-3 mb-6">
      <Button variant="primary">
        <Plus class="w-4 h-4 mr-2" />
        添加服务器
      </Button>
      <Button variant="secondary">
        <RefreshCw class="w-4 h-4 mr-2" />
        刷新
      </Button>
    </div>

    <!-- MCP Servers table -->
    <Card>
      <Table 
        :columns="columns" 
        :data="servers" 
        key-field="name"
        :has-actions="true"
      >
        <template #disabled="{ value }">
          <span 
            class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium"
            :style="{
              background: value ? 'rgba(239, 68, 68, 0.1)' : 'rgba(16, 185, 129, 0.1)',
              color: value ? '#ef4444' : '#10b981',
              border: value ? '1px solid rgba(239, 68, 68, 0.3)' : '1px solid rgba(16, 185, 129, 0.3)'
            }"
          >
            {{ value ? '已禁用' : '运行中' }}
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
  { key: 'command', title: '命令' },
  { key: 'disabled', title: '状态' },
  { key: 'actions', title: '操作' },
]

const servers = [
  { name: 'claude-mcp', command: 'claude-mcp-server', disabled: false },
  { name: 'codex-mcp', command: 'codex-mcp-server', disabled: true },
  { name: 'gemini-mcp', command: 'gemini-mcp-server', disabled: false },
]
</script>