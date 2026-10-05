import type { ReactNode } from 'react'
import type { UseFormRegister } from 'react-hook-form'
import { SIcon } from '@/ui'
import { t, tt } from '../locale'
import type { ConfigFormValues } from '../lib/configForm'

const inputClass =
  'w-full rounded-xl border border-border-default bg-bg-elevated py-2.5 pr-4 pl-10 text-sm text-text-primary outline-none focus:ring-2 focus:ring-accent-primary/20'

interface LabeledFieldProps {
  label: string
  hint?: string
  icon?: string
  leading?: ReactNode
  children: ReactNode
}

function LabeledField({ label, hint, icon, leading, children }: LabeledFieldProps) {
  return (
    <label className="block w-full">
      <span className="mb-1.5 ml-1 block text-xs font-bold tracking-wider text-text-muted uppercase">{label}</span>
      <div className="relative">
        <span className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3 text-text-muted">
          {leading ?? (icon ? <SIcon name={icon} size="w-4 h-4" /> : null)}
        </span>
        {children}
      </div>
      {hint ? <span className="mt-1 block text-xs text-text-muted">{hint}</span> : null}
    </label>
  )
}

interface ConfigFormFieldsProps {
  register: UseFormRegister<ConfigFormValues>
  showName?: boolean
  showToken?: boolean
  onToggleToken?: () => void
}

export function ConfigFormFields({
  register,
  showName = false,
  showToken = false,
  onToggleToken,
}: ConfigFormFieldsProps) {
  return (
    <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
      {showName ? (
        <LabeledField label={t('configs.addConfig.name')} leading={<span>#</span>}>
          <input className={inputClass} placeholder={t('configs.addConfig.namePlaceholder')} {...register('name')} />
        </LabeledField>
      ) : null}
      <LabeledField label={t('configs.addConfig.description')} icon="FileText">
        <input
          className={inputClass}
          placeholder={t('configs.addConfig.descriptionPlaceholder')}
          {...register('description')}
        />
      </LabeledField>
      <div className="md:col-span-2">
        <LabeledField label="Base URL" icon="Globe">
          <input className={inputClass} placeholder="https://api.example.com" {...register('base_url')} />
        </LabeledField>
      </div>
      <div className="relative md:col-span-2">
        <LabeledField label="Auth Token" icon="KeyRound">
          <input
            className={inputClass}
            type={showToken ? 'text' : 'password'}
            placeholder={t('configs.addConfig.tokenPlaceholder')}
            autoComplete="off"
            {...register('auth_token')}
          />
        </LabeledField>
        {onToggleToken ? (
          <button
            type="button"
            className="absolute top-[2.125rem] right-3 rounded-md p-1 text-text-muted hover:bg-bg-overlay hover:text-text-primary"
            title={showToken ? 'Hide token' : 'Show token'}
            onClick={onToggleToken}
          >
            <SIcon name={showToken ? 'EyeOff' : 'Eye'} size="w-4 h-4" />
          </button>
        ) : null}
      </div>
      <LabeledField label="Model" icon="Bot">
        <input className={inputClass} placeholder={t('configs.addConfig.modelPlaceholder')} {...register('model')} />
      </LabeledField>
      <LabeledField label="Fast Model" icon="Zap">
        <input
          className={inputClass}
          placeholder={t('configs.addConfig.smallModelPlaceholder')}
          {...register('small_fast_model')}
        />
      </LabeledField>
      <div className="w-full">
        <span className="mb-1.5 ml-1 block text-xs font-bold tracking-wider text-text-muted uppercase">
          {t('configs.addConfig.providerType')}
        </span>
        <div className="relative">
          <select
            className="w-full appearance-none rounded-xl border border-border-default bg-bg-elevated px-4 py-2.5 text-sm text-text-primary focus:ring-2 focus:ring-accent-primary/20 focus:outline-none"
            {...register('provider_type')}
          >
            <option value="">{t('configs.addConfig.providerUncategorized')}</option>
            <option value="official_relay">{t('configs.addConfig.providerOfficialRelay')}</option>
            <option value="third_party_model">{t('configs.addConfig.providerThirdParty')}</option>
          </select>
          <div className="pointer-events-none absolute inset-y-0 right-3 flex items-center text-text-muted">
            <SIcon name="ChevronDown" size="w-4 h-4" />
          </div>
        </div>
      </div>
      <LabeledField label={t('configs.addConfig.providerName')} icon="Building2" hint={t('configs.addConfig.providerNameHint')}>
        <input
          className={inputClass}
          placeholder={t('configs.addConfig.providerNamePlaceholder')}
          {...register('provider')}
        />
      </LabeledField>
      <LabeledField label={t('configs.addConfig.account')} icon="User" hint={t('configs.addConfig.accountHint')}>
        <input
          className={inputClass}
          placeholder={t('configs.addConfig.accountPlaceholder')}
          {...register('account')}
        />
      </LabeledField>
      <LabeledField label={t('configs.addConfig.tags')} icon="Tags" hint={t('configs.addConfig.tagsHint')}>
        <input className={inputClass} placeholder={t('configs.addConfig.tagsPlaceholder')} {...register('tagsInput')} />
      </LabeledField>
      <p className="sr-only">{tt('标签用逗号分隔', 'Tags are comma separated')}</p>
    </div>
  )
}
