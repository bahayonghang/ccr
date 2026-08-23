import { useCallback, useRef, useState } from 'react'
import { useForm } from 'react-hook-form'
import { exportCheckinConfig, previewCheckinImport, importCheckinConfig } from '@/api'
import { getErrorMessage } from '@/types/api'
import type { CheckinImportResult, ExportData, ImportPreviewResponse } from '@/types/checkin'
import { checkinNotify } from '../lib/checkinNotify'
import { useCheckinLocale, useTt } from '../hooks/useCheckinT'

interface ExportForm {
  include_plaintext_keys: boolean
  providers_only: boolean
}

interface CheckinImportExportTabProps {
  onRefresh?: () => void
}

export function CheckinImportExportTab({ onRefresh }: CheckinImportExportTabProps) {
  const locale = useCheckinLocale()
  const isZh = locale.startsWith('zh')
  const tt = useTt()
  const exportForm = useForm<ExportForm>({
    defaultValues: { include_plaintext_keys: false, providers_only: false },
  })
  const fileRef = useRef<HTMLInputElement | null>(null)
  const [importData, setImportData] = useState<ExportData | null>(null)
  const [importPreview, setImportPreview] = useState<ImportPreviewResponse | null>(null)
  const [strategy, setStrategy] = useState<'skip' | 'overwrite'>('skip')

  const handleExport = useCallback(async () => {
    try {
      const values = exportForm.getValues()
      const data = await exportCheckinConfig<ExportData>({ ...values })
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = `checkin-config-${new Date().toISOString().slice(0, 10)}.json`
      link.click()
      URL.revokeObjectURL(url)
    } catch (error: unknown) {
      checkinNotify.error(`${tt('导出失败', 'Export failed')}: ${getErrorMessage(error, tt('未知错误', 'Unknown error'))}`)
    }
  }, [exportForm, tt])

  const openFile = useCallback(() => {
    fileRef.current?.click()
  }, [])

  const handleFileSelect = useCallback(
    async (event: { target: { files: FileList | null } }) => {
      const file = event.target.files?.[0]
      if (!file) return
      try {
        const text = await file.text()
        const data = JSON.parse(text) as ExportData
        setImportData(data)
        setImportPreview(await previewCheckinImport<ImportPreviewResponse>(data))
      } catch (error: unknown) {
        checkinNotify.error(
          `${tt('解析文件失败', 'Failed to parse file')}: ${getErrorMessage(error, tt('未知错误', 'Unknown error'))}`,
        )
        setImportData(null)
        setImportPreview(null)
      }
    },
    [tt],
  )

  const handleImport = useCallback(async () => {
    if (!importData) return
    try {
      const result = await importCheckinConfig<CheckinImportResult>(importData, {
        conflict_strategy: strategy,
      })
      checkinNotify.success(
        isZh
          ? `导入完成: 提供商 ${result.providers_imported} 个, 账号 ${result.accounts_imported} 个`
          : `Import complete: ${result.providers_imported} providers, ${result.accounts_imported} accounts`,
      )
      setImportData(null)
      setImportPreview(null)
      onRefresh?.()
    } catch (error: unknown) {
      checkinNotify.error(`${tt('导入失败', 'Import failed')}: ${getErrorMessage(error, tt('未知错误', 'Unknown error'))}`)
    }
  }, [importData, isZh, onRefresh, strategy, tt])

  const onStrategyChange = useCallback((event: { target: { value: string } }) => {
    setStrategy(event.target.value === 'overwrite' ? 'overwrite' : 'skip')
  }, [])

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold text-text-primary">{tt('导入 / 导出', 'Import / Export')}</h2>
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        <div className="rounded-lg border border-border-default bg-bg-surface p-6 shadow-sm">
          <h3 className="mb-4 text-lg font-semibold">{tt('导出配置', 'Export config')}</h3>
          <label className="flex items-center">
            <input type="checkbox" className="h-4 w-4" {...exportForm.register('include_plaintext_keys')} />
            <span className="ml-2 text-sm text-text-secondary">
              {tt('包含明文 API Key (危险)', 'Include plaintext API keys (dangerous)')}
            </span>
          </label>
          <label className="mt-2 flex items-center">
            <input type="checkbox" className="h-4 w-4" {...exportForm.register('providers_only')} />
            <span className="ml-2 text-sm text-text-secondary">{tt('仅导出提供商', 'Export providers only')}</span>
          </label>
          <button
            type="button"
            className="mt-4 w-full rounded-lg bg-accent-primary px-4 py-2 text-text-inverted"
            onClick={handleExport}
          >
            {tt('导出 JSON', 'Export JSON')}
          </button>
        </div>
        <div className="rounded-lg border border-border-default bg-bg-surface p-6 shadow-sm">
          <h3 className="mb-4 text-lg font-semibold">{tt('导入配置', 'Import config')}</h3>
          <input ref={fileRef} type="file" accept=".json" className="hidden" onChange={handleFileSelect} />
          <button type="button" className="w-full text-text-muted" onClick={openFile}>
            {tt('点击选择 JSON 文件', 'Click to choose a JSON file')}
          </button>
          {importPreview ? (
            <div className="mt-3 text-sm text-text-secondary">
              <p>{`${tt('新提供商', 'New providers')}: ${importPreview.new_providers}`}</p>
              <p>{`${tt('新账号', 'New accounts')}: ${importPreview.new_accounts}`}</p>
              <p>{`${tt('冲突项', 'Conflicts')}: ${importPreview.conflicting_providers + importPreview.conflicting_accounts}`}</p>
            </div>
          ) : null}
          <select className="mt-3 w-full rounded-lg border border-border-default bg-bg-surface px-3 py-2" value={strategy} onChange={onStrategyChange}>
            <option value="skip">{tt('跳过冲突项', 'Skip conflicts')}</option>
            <option value="overwrite">{tt('覆盖冲突项', 'Overwrite conflicts')}</option>
          </select>
          <button
            type="button"
            disabled={!importData}
            className="mt-3 w-full rounded-lg bg-accent-success px-4 py-2 text-text-inverted disabled:opacity-50"
            onClick={handleImport}
          >
            {tt('执行导入', 'Run import')}
          </button>
        </div>
      </div>
    </div>
  )
}
