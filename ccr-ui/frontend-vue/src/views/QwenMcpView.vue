<template>
  <Layout>
    <div class="mb-8">
      <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--text-primary)' }">Qwen MCP 服务器</h1>
      <p class="text-base" :style="{ color: 'var(--text-secondary)' }">Qwen MCP 服务器配置管理</p>
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

    <!-- Qwen MCP Servers table -->
    <Card>
      <Table 
        :columns="columns" 
        :data="servers" 
        key-field="name"
        :has-actions="true"
      >
        <template #command="{ value }">
          <code :style="{ color: 'var(--text-primary)' }">{{ value }}</code>
        </template>
        
        <template #actions>
          <div class="flex space-x-2">
            <Button variant="ghost" size="sm">
              <Edit2 class="w-4 h-4" />
            </Button>
            <Button variant="ghost" size="sm">
              <Trash2 class="w-4 h-4" />
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
import { Plus, RefreshCw, Edit2, Trash2 } from 'lucide-vue-next'

const columns = [
  { key: 'name', title: '名称' },
  { key: 'command', title: '命令' },
  { key: 'actions', title: '操作' },
]

const servers = [
  { name: 'qwen-server', command: 'qwen-mcp' },
]
</script>