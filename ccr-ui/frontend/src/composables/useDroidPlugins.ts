import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUIStore } from '@/stores/ui'
import {
  listDroidPlugins,
  addDroidPlugin,
  updateDroidPlugin,
  deleteDroidPlugin,
  type DroidPlugin
} from '@/api/client'

// ============ Types ============

/** Droid Plugin 数据结构 */
export interface DroidPluginItem extends DroidPlugin {
  _expanded?: boolean // UI状态：是否展开
}

/** 表单数据接口 */
interface DroidPluginFormData {
  id: string
  data: string // JSON 字符串格式
}

// ============ Composable ============

export function useDroidPlugins() {
  const { t } = useI18n()
  const uiStore = useUIStore()

  // State
  const plugins = ref<DroidPluginItem[]>([])
  const loading = ref(false)
  const showForm = ref(false)
  const editingPlugin = ref<DroidPluginItem | null>(null)
  const formData = ref<DroidPluginFormData>({
    id: '',
    data: '{}'
  })

  // Computed
  const moduleColor = computed(() => '#8b5cf6') // Purple for Droid Plugins

  const i18nPrefix = computed(() => 'droid.plugins')

  const parentPath = computed(() => '/droid')

  const sidebarModule = computed(() => 'droid')

  // Methods

  /** 加载插件列表 */
  async function loadPlugins(): Promise<void> {
    loading.value = true
    try {
      const data = await listDroidPlugins()
      plugins.value = data.map((p) => ({ ...p, _expanded: false }))
    } catch (error) {
      uiStore.showError(t('droid.plugins.loadFailed'))
      console.error('Failed to load Droid plugins:', error)
    } finally {
      loading.value = false
    }
  }

  /** 打开添加表单 */
  function openAddForm(): void {
    editingPlugin.value = null
    formData.value = {
      id: '',
      data: '{}'
    }
    showForm.value = true
  }

  /** 打开编辑表单 */
  function openEditForm(plugin: DroidPluginItem): void {
    editingPlugin.value = plugin
    formData.value = {
      id: plugin.id,
      data: JSON.stringify(plugin.data, null, 2)
    }
    showForm.value = true
  }

  /** 关闭表单 */
  function closeForm(): void {
    showForm.value = false
    editingPlugin.value = null
    formData.value = {
      id: '',
      data: '{}'
    }
  }

  /** 提交表单 */
  async function submitForm(): Promise<void> {
    // 验证 ID
    if (!formData.value.id.trim()) {
      uiStore.showError(t('droid.plugins.idRequired'))
      return
    }

    // 解析 JSON 数据
    let parsedData: Record<string, unknown>
    try {
      parsedData = JSON.parse(formData.value.data)
    } catch {
      uiStore.showError(t('droid.plugins.invalidJson'))
      return
    }

    try {
      if (editingPlugin.value) {
        // 更新
        await updateDroidPlugin(editingPlugin.value.id, parsedData)
        uiStore.showSuccess(t('droid.plugins.updateSuccess'))
      } else {
        // 添加
        await addDroidPlugin(formData.value.id, parsedData)
        uiStore.showSuccess(t('droid.plugins.addSuccess'))
      }
      closeForm()
      await loadPlugins()
    } catch (error: any) {
      uiStore.showError(error.response?.data?.error || t('droid.plugins.saveFailed'))
    }
  }

  /** 删除插件 */
  async function deletePlugin(plugin: DroidPluginItem): Promise<void> {
    if (!confirm(t('droid.plugins.deleteConfirm', { id: plugin.id }))) {
      return
    }

    try {
      await deleteDroidPlugin(plugin.id)
      uiStore.showSuccess(t('droid.plugins.deleteSuccess'))
      await loadPlugins()
    } catch (error: any) {
      uiStore.showError(error.response?.data?.error || t('droid.plugins.deleteFailed'))
    }
  }

  /** 切换展开状态 */
  function toggleExpanded(plugin: DroidPluginItem): void {
    plugin._expanded = !plugin._expanded
  }

  return {
    // State
    plugins,
    loading,
    showForm,
    editingPlugin,
    formData,
    // Computed
    moduleColor,
    i18nPrefix,
    parentPath,
    sidebarModule,
    // Methods
    loadPlugins,
    openAddForm,
    openEditForm,
    closeForm,
    submitForm,
    deletePlugin,
    toggleExpanded
  }
}
