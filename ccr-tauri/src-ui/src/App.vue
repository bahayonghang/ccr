<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import {
  listConfigs,
  getCurrentConfig,
  switchConfig,
  getHistory,
  getSystemInfo,
  createConfig,
  updateConfig,
  deleteConfig
} from './api'
import type { ConfigInfo, HistoryEntry, SystemInfo } from './types'

// ============================================================================
// 🎯 状态管理
// ============================================================================

const configs = ref<ConfigInfo[]>([])
const currentConfig = ref<ConfigInfo | null>(null)
const history = ref<HistoryEntry[]>([])
const systemInfo = ref<SystemInfo | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

// 当前标签页
const activeTab = ref<'configs' | 'history'>('configs')

// 配置过滤类型
const filterType = ref<'all' | 'official_relay' | 'third_party_model' | 'uncategorized'>('all')

// 主题
const theme = ref<'light' | 'dark'>('light')

// 模态框状态
const showConfigModal = ref(false)
const editingConfig = ref<ConfigInfo | null>(null)

// 表单数据
const formData = ref({
  name: '',
  description: '',
  base_url: '',
  auth_token: '',
  model: '',
  small_fast_model: '',
  provider_type: '',
  provider: '',
  account: '',
  tags: ''
})

// ============================================================================
// 🎨 计算属性
// ============================================================================

const filteredConfigs = computed(() => {
  if (filterType.value === 'all') return configs.value
  if (filterType.value === 'official_relay') {
    return configs.value.filter(c =>
      c.provider_type === 'OfficialRelay' || c.provider_type === 'official_relay'
    )
  }
  if (filterType.value === 'third_party_model') {
    return configs.value.filter(c =>
      c.provider_type === 'ThirdPartyModel' || c.provider_type === 'third_party_model'
    )
  }
  if (filterType.value === 'uncategorized') {
    return configs.value.filter(c => !c.provider_type)
  }
  return configs.value
})

const statsTotal = computed(() => configs.value.length)
const statsHistory = computed(() => history.value.length)

// ============================================================================
// 📡 数据加载
// ============================================================================

async function loadData() {
  try {
    loading.value = true
    error.value = null

    const [configList, currentConfigData, historyData, sysInfo] = await Promise.all([
      listConfigs(),
      getCurrentConfig(),
      getHistory(50),
      getSystemInfo()
    ])

    configs.value = configList.configs
    currentConfig.value = currentConfigData
    history.value = historyData
    systemInfo.value = sysInfo
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    console.error('加载数据失败:', e)
  } finally {
    loading.value = false
  }
}

// ============================================================================
// 🎭 主题切换
// ============================================================================

function toggleTheme() {
  theme.value = theme.value === 'light' ? 'dark' : 'light'
  document.documentElement.setAttribute('data-theme', theme.value)
  localStorage.setItem('ccr-theme', theme.value)
}

function initTheme() {
  const savedTheme = localStorage.getItem('ccr-theme') as 'light' | 'dark' || 'light'
  theme.value = savedTheme
  document.documentElement.setAttribute('data-theme', savedTheme)
}

// ============================================================================
// 🔄 配置操作
// ============================================================================

async function handleSwitch(name: string) {
  if (!confirm(`确定切换到配置 "${name}" 吗？`)) return

  try {
    loading.value = true
    await switchConfig(name)
    await loadData()
    showNotification(`✓ 成功切换到配置 "${name}"`, 'success')
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    showNotification(`切换失败: ${error.value}`, 'error')
  } finally {
    loading.value = false
  }
}

function openAddModal() {
  editingConfig.value = null
  resetForm()
  showConfigModal.value = true
}

function openEditModal(config: ConfigInfo) {
  editingConfig.value = config
  formData.value = {
    name: config.name,
    description: config.description || '',
    base_url: config.base_url || '',
    auth_token: config.auth_token || '',
    model: config.model || '',
    small_fast_model: config.small_fast_model || '',
    provider_type: config.provider_type || '',
    provider: config.provider || '',
    account: config.account || '',
    tags: config.tags ? config.tags.join(', ') : ''
  }
  showConfigModal.value = true
}

function resetForm() {
  formData.value = {
    name: '',
    description: '',
    base_url: '',
    auth_token: '',
    model: '',
    small_fast_model: '',
    provider_type: '',
    provider: '',
    account: '',
    tags: ''
  }
}

async function saveConfig() {
  try {
    loading.value = true

    const tagsArray = formData.value.tags
      ? formData.value.tags.split(',').map(t => t.trim()).filter(t => t)
      : undefined

    if (editingConfig.value) {
      // 更新配置
      await updateConfig({
        old_name: editingConfig.value.name,
        new_name: formData.value.name,
        description: formData.value.description || undefined,
        base_url: formData.value.base_url || undefined,
        auth_token: formData.value.auth_token || undefined,
        model: formData.value.model || undefined,
        small_fast_model: formData.value.small_fast_model || undefined,
        provider_type: formData.value.provider_type || undefined,
        provider: formData.value.provider || undefined,
        account: formData.value.account || undefined,
        tags: tagsArray
      })
      showNotification('✓ 配置更新成功', 'success')
    } else {
      // 创建配置
      await createConfig({
        name: formData.value.name,
        description: formData.value.description || undefined,
        base_url: formData.value.base_url || undefined,
        auth_token: formData.value.auth_token || undefined,
        model: formData.value.model || undefined,
        small_fast_model: formData.value.small_fast_model || undefined,
        provider_type: formData.value.provider_type || undefined,
        provider: formData.value.provider || undefined,
        account: formData.value.account || undefined,
        tags: tagsArray
      })
      showNotification('✓ 配置添加成功', 'success')
    }

    showConfigModal.value = false
    await loadData()
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    showNotification(`保存失败: ${error.value}`, 'error')
  } finally {
    loading.value = false
  }
}

async function handleDelete(name: string) {
  if (!confirm(`确定删除配置 "${name}" 吗？此操作不可恢复！`)) return

  try {
    loading.value = true
    await deleteConfig(name)
    showNotification(`✓ 配置 "${name}" 删除成功`, 'success')
    await loadData()
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
    showNotification(`删除失败: ${error.value}`, 'error')
  } finally {
    loading.value = false
  }
}

// ============================================================================
// 📍 配置导航跳转
// ============================================================================

function scrollToConfig(configName: string) {
  // 确保在配置列表标签页
  activeTab.value = 'configs'

  // 延迟一小段时间确保 DOM 已更新
  setTimeout(() => {
    const element = document.getElementById(`config-${configName}`)
    if (element) {
      element.scrollIntoView({
        behavior: 'smooth',
        block: 'center'
      })

      // 高亮效果
      element.style.transform = 'scale(1.02)'
      setTimeout(() => {
        element.style.transform = 'scale(1)'
      }, 300)
    }
  }, 100)
}

// ============================================================================
// 🔔 通知系统
// ============================================================================

const notification = ref<{ message: string; type: string; show: boolean }>({
  message: '',
  type: 'success',
  show: false
})

let notificationTimer: number | null = null

function showNotification(message: string, type: 'success' | 'error' | 'warning' = 'success') {
  notification.value = { message, type, show: true }

  if (notificationTimer) clearTimeout(notificationTimer)

  notificationTimer = window.setTimeout(() => {
    notification.value.show = false
  }, 4000)
}

// ============================================================================
// 🚀 生命周期
// ============================================================================

onMounted(async () => {
  initTheme()
  await loadData()

  // 每 5 秒更新系统信息
  setInterval(async () => {
    try {
      systemInfo.value = await getSystemInfo()
    } catch (e) {
      console.error('更新系统信息失败:', e)
    }
  }, 5000)
})
</script>

<template>
  <div class="app">
    <!-- 动态背景 -->
    <div class="bg-effect"></div>

    <div class="container">
      <!-- 顶部导航 -->
      <nav class="navbar">
        <div class="brand-section">
          <div class="logo-container">
            <div class="logo">⚡ CCR</div>
            <div class="logo-subtitle">Desktop</div>
          </div>
          <div class="divider"></div>
          <div class="project-info">
            <div class="project-title">Claude Code Configuration Switcher</div>
            <div class="project-meta" v-if="systemInfo">
              <div class="meta-item">
                <span class="meta-dot"></span>
                <span>{{ systemInfo.username }}@{{ systemInfo.hostname }}</span>
              </div>
            </div>
          </div>
        </div>
        <div class="nav-actions">
          <button class="theme-toggle" @click="toggleTheme" title="切换主题">
            <span class="theme-icon">{{ theme === 'dark' ? '🌙' : '☀️' }}</span>
          </button>
          <button class="btn btn-secondary" @click="loadData">
            🔄 刷新
          </button>
          <button class="btn btn-primary" @click="openAddModal">
            ➕ 添加配置
          </button>
        </div>
      </nav>

      <!-- 主内容 -->
      <div class="main-content">
        <!-- 左侧边栏 -->
        <aside class="sidebar-left">
          <!-- 当前配置 -->
          <div class="sidebar-section">
            <div class="sidebar-title">当前配置</div>
            <div class="current-config-display">
              <div class="config-status">
                <span class="status-dot"></span>
                <span style="font-size: 11px; color: var(--text-secondary);">ACTIVE</span>
              </div>
              <div class="config-name">
                {{ currentConfig?.name || '-' }}
              </div>
            </div>
          </div>

          <!-- 系统信息 -->
          <div class="sidebar-section" v-if="systemInfo">
            <div class="sidebar-title">系统信息</div>
            <div class="system-info-card">
              <div class="system-info-item">
                <div class="system-info-icon">💻</div>
                <div class="system-info-content">
                  <div class="system-info-label">主机</div>
                  <div class="system-info-value">{{ systemInfo.hostname }}</div>
                </div>
              </div>
              <div class="system-info-item">
                <div class="system-info-icon">🖥️</div>
                <div class="system-info-content">
                  <div class="system-info-label">系统</div>
                  <div class="system-info-value">{{ systemInfo.os }}</div>
                </div>
              </div>
              <div class="system-info-item">
                <div class="system-info-icon">👤</div>
                <div class="system-info-content">
                  <div class="system-info-label">用户</div>
                  <div class="system-info-value">{{ systemInfo.username }}</div>
                </div>
              </div>
            </div>
          </div>

          <!-- 统计 -->
          <div class="sidebar-section">
            <div class="sidebar-title">统计</div>
            <div class="stats-grid">
              <div class="stat-card">
                <div class="stat-value">{{ statsTotal }}</div>
                <div class="stat-label">总配置</div>
              </div>
              <div class="stat-card">
                <div class="stat-value">{{ statsHistory }}</div>
                <div class="stat-label">历史</div>
              </div>
            </div>
          </div>
        </aside>

        <!-- 配置区域 -->
        <main class="configs-section">
          <!-- 标签页导航 -->
          <div class="tab-nav">
            <button
              class="tab-btn"
              :class="{ active: activeTab === 'configs' }"
              @click="activeTab = 'configs'"
            >
              配置列表
            </button>
            <button
              class="tab-btn"
              :class="{ active: activeTab === 'history' }"
              @click="activeTab = 'history'"
            >
              历史记录
            </button>
          </div>

          <!-- 配置列表 -->
          <div v-show="activeTab === 'configs'" class="tab-content active">
            <!-- 配置类型过滤 -->
            <div class="config-type-filter">
              <button
                class="type-filter-btn"
                :class="{ active: filterType === 'all' }"
                @click="filterType = 'all'"
              >
                📋 全部配置
              </button>
              <button
                class="type-filter-btn"
                :class="{ active: filterType === 'official_relay' }"
                @click="filterType = 'official_relay'"
              >
                🔄 官方中转
              </button>
              <button
                class="type-filter-btn"
                :class="{ active: filterType === 'third_party_model' }"
                @click="filterType = 'third_party_model'"
              >
                🤖 第三方模型
              </button>
              <button
                class="type-filter-btn"
                :class="{ active: filterType === 'uncategorized' }"
                @click="filterType = 'uncategorized'"
              >
                ❓ 未分类
              </button>
            </div>

            <!-- 加载状态 -->
            <div v-if="loading" class="loading-container" style="text-align: center; padding: 40px;">
              <div class="loading"></div>
              <p style="margin-top: 10px; color: var(--text-secondary);">加载中...</p>
            </div>

            <!-- 配置卡片 -->
            <div v-else-if="filteredConfigs.length > 0">
              <div
                v-for="config in filteredConfigs"
                :key="config.name"
                :id="`config-${config.name}`"
                class="config-card"
                :class="{ active: config.is_current }"
              >
                <div class="config-header">
                  <div class="config-info">
                    <h3 class="config-title">
                      <span class="config-name">{{ config.name }}</span>
                      <span v-if="config.is_current" class="badge badge-active">当前</span>
                      <span v-if="config.is_default" class="badge badge-default">默认</span>
                    </h3>
                    <div class="config-description" v-if="config.description">
                      <span class="desc-icon">📝</span>
                      <span class="desc-text">{{ config.description }}</span>
                    </div>
                  </div>
                </div>
                <div class="config-details">
                  <div class="config-field" v-if="config.base_url">
                    <div class="field-label">Base URL</div>
                    <div class="field-value">{{ config.base_url }}</div>
                  </div>
                  <div class="config-field" v-if="config.auth_token">
                    <div class="field-label">Auth Token</div>
                    <div class="field-value">{{ config.auth_token.substring(0, 8) }}...</div>
                  </div>
                  <div class="config-field" v-if="config.model">
                    <div class="field-label">Model</div>
                    <div class="field-value">{{ config.model }}</div>
                  </div>
                  <div class="config-field" v-if="config.provider">
                    <div class="field-label">Provider</div>
                    <div class="field-value">{{ config.provider }}</div>
                  </div>
                </div>
                <div class="config-actions">
                  <button
                    v-if="!config.is_current"
                    class="btn btn-primary btn-small"
                    @click="handleSwitch(config.name)"
                    :disabled="loading"
                  >
                    切换
                  </button>
                  <button
                    class="btn btn-secondary btn-small"
                    @click="openEditModal(config)"
                    :disabled="loading"
                  >
                    编辑
                  </button>
                  <button
                    v-if="!config.is_current && !config.is_default"
                    class="btn btn-danger btn-small"
                    @click="handleDelete(config.name)"
                    :disabled="loading"
                  >
                    删除
                  </button>
                </div>
              </div>
            </div>

            <!-- 空状态 -->
            <div v-else style="text-align: center; color: var(--text-muted); padding: 40px;">
              暂无配置
            </div>
          </div>

          <!-- 历史记录 -->
          <div v-show="activeTab === 'history'" class="tab-content active">
            <div v-if="history.length > 0">
              <div v-for="entry in history" :key="entry.timestamp" class="history-item">
                <div class="history-header">
                  <span class="history-op">{{ entry.operation }}</span>
                  <span class="history-time">{{ new Date(entry.timestamp).toLocaleString() }}</span>
                </div>
                <div class="history-details">
                  操作者: {{ entry.actor }}<br>
                  <span v-if="entry.from_config && entry.to_config">
                    从 {{ entry.from_config }} 切换到 {{ entry.to_config }}
                  </span>
                </div>
              </div>
            </div>
            <div v-else style="text-align: center; color: var(--text-muted); padding: 40px;">
              暂无历史记录
            </div>
          </div>
        </main>

        <!-- 右侧配置导航 -->
        <aside class="sidebar-right">
          <div class="sidebar-title">配置目录</div>
          <ul class="config-nav">
            <li v-if="filteredConfigs.length === 0" class="config-nav-item">
              <span style="font-size: 12px; color: var(--text-muted);">暂无配置</span>
            </li>
            <li v-for="config in filteredConfigs" :key="config.name" class="config-nav-item">
              <a
                href="#"
                class="config-nav-link"
                :class="{ active: config.is_current }"
                @click.prevent="scrollToConfig(config.name)"
              >
                <span class="config-nav-badge" :class="{
                  current: config.is_current,
                  default: config.is_default && !config.is_current
                }"></span>
                {{ config.name }}
              </a>
            </li>
          </ul>
        </aside>
      </div>
    </div>

    <!-- 配置模态框 -->
    <div v-if="showConfigModal" class="modal show">
      <div class="modal-content">
        <div class="modal-header">
          <h2 class="modal-title">{{ editingConfig ? '编辑配置' : '添加配置' }}</h2>
          <button class="close-btn" @click="showConfigModal = false">&times;</button>
        </div>
        <form @submit.prevent="saveConfig">
          <div class="form-group">
            <label class="form-label">配置名称 *</label>
            <input
              v-model="formData.name"
              type="text"
              class="form-input"
              required
              :disabled="!!editingConfig"
            >
          </div>
          <div class="form-group">
            <label class="form-label">描述</label>
            <input v-model="formData.description" type="text" class="form-input">
          </div>
          <div class="form-group">
            <label class="form-label">Base URL *</label>
            <input v-model="formData.base_url" type="url" class="form-input" required>
          </div>
          <div class="form-group">
            <label class="form-label">Auth Token *</label>
            <input v-model="formData.auth_token" type="text" class="form-input" required>
          </div>
          <div class="form-group">
            <label class="form-label">Model</label>
            <input v-model="formData.model" type="text" class="form-input">
          </div>
          <div class="form-group">
            <label class="form-label">Small Fast Model</label>
            <input v-model="formData.small_fast_model" type="text" class="form-input">
          </div>
          <div class="form-group">
            <label class="form-label">提供商类型</label>
            <select v-model="formData.provider_type" class="form-input">
              <option value="">未分类</option>
              <option value="official_relay">🔄 官方中转</option>
              <option value="third_party_model">🤖 第三方模型</option>
            </select>
          </div>
          <div class="form-group">
            <label class="form-label">提供商名称</label>
            <input v-model="formData.provider" type="text" class="form-input" placeholder="如: anyrouter, glm, moonshot">
            <div class="form-hint">用于标识同一提供商的不同配置</div>
          </div>
          <div class="form-group">
            <label class="form-label">账号标识</label>
            <input v-model="formData.account" type="text" class="form-input" placeholder="如: github_5953, personal, work">
            <div class="form-hint">用于区分同一提供商的不同账号</div>
          </div>
          <div class="form-group">
            <label class="form-label">标签</label>
            <input v-model="formData.tags" type="text" class="form-input" placeholder="用逗号分隔，如: free, stable, backup">
            <div class="form-hint">用于灵活分类和筛选，多个标签用逗号分隔</div>
          </div>
          <div style="display: flex; gap: 10px; justify-content: flex-end;">
            <button type="button" class="btn btn-secondary" @click="showConfigModal = false">
              取消
            </button>
            <button type="submit" class="btn btn-primary" :disabled="loading">
              保存
            </button>
          </div>
        </form>
      </div>
    </div>

    <!-- 通知 -->
    <div v-if="notification.show" class="notification" :class="[notification.type, 'show']">
      {{ notification.message }}
    </div>
  </div>
</template>

<style scoped>
.app {
  width: 100%;
  min-height: 100vh;
}

.container {
  max-width: 1600px;
  margin: 0 auto;
  padding: 20px;
}

/* 导航栏 */
.navbar {
  background: var(--bg-card);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 20px 28px;
  margin-bottom: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: var(--shadow-medium);
  position: relative;
  overflow: hidden;
}

.navbar::before {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: 2px;
  background: linear-gradient(90deg, transparent, var(--accent-primary), var(--accent-secondary), transparent);
  opacity: 0.5;
}

.brand-section {
  display: flex;
  align-items: center;
  gap: 20px;
}

.logo-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.logo {
  font-size: 28px;
  font-weight: 700;
  background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  letter-spacing: -1px;
  line-height: 1;
}

.logo-subtitle {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 500;
  letter-spacing: 2px;
  text-transform: uppercase;
}

.divider {
  width: 1px;
  height: 50px;
  background: linear-gradient(180deg, transparent, var(--border-color), transparent);
}

.project-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.project-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1;
}

.project-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 11px;
  color: var(--text-secondary);
}

.meta-dot {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: var(--text-muted);
}

.nav-actions {
  display: flex;
  gap: 10px;
}

.theme-toggle {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--bg-tertiary);
  border: 2px solid var(--border-color);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  transition: all 0.3s ease;
  padding: 0;
}

.theme-toggle:hover {
  transform: rotate(180deg) scale(1.1);
  border-color: var(--accent-primary);
  box-shadow: 0 0 20px var(--glow-primary);
}

/* 主内容区 */
.main-content {
  display: grid;
  grid-template-columns: 260px 1fr 220px;
  gap: 16px;
}

@media (max-width: 1200px) {
  .main-content {
    grid-template-columns: 1fr;
  }
}

/* 侧边栏 */
.sidebar-left,
.sidebar-right {
  background: var(--bg-card);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 20px;
  height: fit-content;
  position: sticky;
  top: 16px;
  box-shadow: var(--shadow-small);
}

.sidebar-right {
  max-height: calc(100vh - 120px);
  overflow-y: auto;
}

.sidebar-section {
  margin-bottom: 24px;
}

.sidebar-section:last-child {
  margin-bottom: 0;
}

.sidebar-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 1px;
  margin-bottom: 12px;
}

/* 当前配置 */
.current-config-display {
  background: linear-gradient(135deg, var(--bg-tertiary), var(--bg-secondary));
  border: 1px solid var(--accent-primary);
  border-radius: 10px;
  padding: 16px;
}

.config-status {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 10px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-success);
  box-shadow: 0 0 10px var(--glow-success);
  animation: pulse 2s infinite;
}

.config-name {
  font-size: 18px;
  font-weight: 700;
  font-family: 'Monaco', 'Consolas', monospace;
  letter-spacing: 1px;
  text-transform: uppercase;
  word-break: break-all;
}

/* 系统信息 */
.system-info-card {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.system-info-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 8px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.system-info-icon {
  font-size: 20px;
  width: 24px;
  text-align: center;
}

.system-info-content {
  flex: 1;
}

.system-info-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
  font-weight: 600;
}

.system-info-value {
  font-size: 12px;
  color: var(--text-primary);
  font-weight: 500;
  font-family: 'Monaco', 'Consolas', monospace;
}

/* 统计卡片 */
.stats-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.stat-card {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 10px;
  text-align: center;
}

.stat-value {
  font-size: 18px;
  font-weight: 700;
  color: var(--accent-primary);
}

.stat-label {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
}

/* 配置区域 */
.configs-section {
  background: var(--bg-card);
  backdrop-filter: blur(20px);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 24px;
  box-shadow: var(--shadow-small);
}

/* 标签页 */
.tab-nav {
  display: flex;
  gap: 6px;
  margin-bottom: 20px;
  background: var(--bg-tertiary);
  padding: 5px;
  border-radius: 10px;
}

.tab-btn {
  flex: 1;
  padding: 8px;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 600;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.tab-btn.active {
  background: var(--accent-primary);
  color: white;
}

/* 配置过滤 */
.config-type-filter {
  display: flex;
  gap: 8px;
  margin-bottom: 20px;
  padding: 8px;
  background: var(--bg-tertiary);
  border-radius: 10px;
  border: 1px solid var(--border-color);
}

.type-filter-btn {
  flex: 1;
  padding: 10px 16px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.type-filter-btn:hover {
  background: var(--bg-secondary);
  border-color: var(--accent-primary);
  color: var(--accent-primary);
}

.type-filter-btn.active {
  background: linear-gradient(135deg, var(--accent-primary), var(--accent-secondary));
  border-color: var(--accent-primary);
  color: white;
  box-shadow: 0 0 15px var(--glow-primary);
}

/* 配置卡片 */
.config-card {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 16px;
  margin-bottom: 14px;
  transition: all 0.3s ease;
  scroll-margin-top: 20px;
}

.config-card:hover {
  border-color: var(--accent-primary);
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.config-card.active {
  border-color: var(--accent-primary);
  background: linear-gradient(135deg, rgba(139, 92, 246, 0.1), rgba(168, 85, 247, 0.1));
  box-shadow: 0 0 20px var(--glow-primary);
}

.config-header {
  margin-bottom: 12px;
}

.config-title {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 8px;
}

.config-description {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: rgba(139, 92, 246, 0.08);
  border-left: 3px solid var(--accent-primary);
  border-radius: 6px;
}

.desc-text {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.badge {
  padding: 3px 10px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
}

.badge-active {
  background: var(--accent-success);
  color: white;
}

.badge-default {
  background: var(--accent-warning);
  color: white;
}

/* 配置详情 */
.config-details {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  margin-bottom: 12px;
}

.config-field {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 8px;
}

.field-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 3px;
}

.field-value {
  font-size: 12px;
  font-family: 'Monaco', 'Consolas', monospace;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.config-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

/* 配置导航 */
.config-nav {
  list-style: none;
}

.config-nav-item {
  margin-bottom: 8px;
}

.config-nav-link {
  display: block;
  padding: 8px 12px;
  color: var(--text-secondary);
  text-decoration: none;
  border-radius: 8px;
  transition: all 0.3s ease;
  font-size: 13px;
  position: relative;
  cursor: pointer;
}

.config-nav-link:hover {
  background: rgba(139, 92, 246, 0.1);
  color: var(--accent-primary);
}

.config-nav-link.active {
  background: linear-gradient(90deg, rgba(139, 92, 246, 0.2), transparent);
  color: var(--accent-primary);
  font-weight: 600;
}

.config-nav-badge {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 6px;
}

.config-nav-badge.current {
  background: var(--accent-success);
  box-shadow: 0 0 8px var(--glow-success);
}

.config-nav-badge.default {
  background: var(--accent-warning);
}

/* 历史记录 */
.history-item {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 14px;
  margin-bottom: 10px;
}

.history-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
}

.history-op {
  font-size: 13px;
  font-weight: 600;
  color: var(--accent-primary);
}

.history-time {
  font-size: 11px;
  color: var(--text-muted);
}

.history-details {
  font-size: 12px;
  color: var(--text-secondary);
}

/* 模态框 */
.modal {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(10px);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.modal-content {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 28px;
  max-width: 550px;
  width: 90%;
  max-height: 85vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.modal-title {
  font-size: 20px;
  font-weight: 700;
}

.close-btn {
  background: var(--bg-tertiary);
  border: none;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  cursor: pointer;
  color: var(--text-secondary);
  font-size: 18px;
  transition: all 0.3s ease;
  padding: 0;
}

.close-btn:hover {
  background: var(--accent-danger);
  color: white;
  transform: rotate(90deg);
}

/* 表单 */
.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.form-input {
  width: 100%;
  padding: 10px 14px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-primary);
  font-size: 13px;
  transition: all 0.3s ease;
}

.form-input:focus {
  outline: none;
  border-color: var(--accent-primary);
  box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.1);
}

select.form-input {
  cursor: pointer;
}

.form-hint {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 4px;
}

/* 通知 */
.notification {
  position: fixed;
  top: 20px;
  right: 20px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 14px 18px;
  min-width: 280px;
  box-shadow: var(--shadow-large);
  z-index: 2000;
  animation: slideIn 0.3s ease;
}

.notification.success {
  border-color: var(--accent-success);
}

.notification.error {
  border-color: var(--accent-danger);
}

.notification.warning {
  border-color: var(--accent-warning);
}
</style>
