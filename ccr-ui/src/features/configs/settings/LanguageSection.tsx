import { t } from '../locale'
import { ChoiceButton } from './ChoiceButton'

interface LanguageSectionProps {
  locale: string
  onSelect: (value: string) => void
}

export function LanguageSection({ locale, onSelect }: LanguageSectionProps) {
  return (
    <section>
      <div className="app-settings-card">
        <div className="app-settings-card__header">
          <div>
            <p className="app-settings-card__eyebrow">{t('settings.language.eyebrow')}</p>
            <h2 className="app-settings-card__title">{t('settings.language.title')}</h2>
          </div>
          <p className="app-settings-card__description">{t('settings.language.description')}</p>
        </div>
        <div className="app-settings-option-grid app-settings-option-grid--compact">
          <ChoiceButton
            value="zh-CN"
            active={locale === 'zh-CN'}
            testId="settings-language-zh-CN"
            leading="CN"
            title={t('language.chinese')}
            caption={t('settings.language.chineseDescription')}
            status={locale === 'zh-CN' ? t('settings.active') : t('settings.language.instant')}
            onSelect={onSelect}
          />
          <ChoiceButton
            value="en-US"
            active={locale === 'en-US'}
            testId="settings-language-en-US"
            leading="US"
            title={t('language.english')}
            caption={t('settings.language.englishDescription')}
            status={locale === 'en-US' ? t('settings.active') : t('settings.language.instant')}
            onSelect={onSelect}
          />
        </div>
      </div>
    </section>
  )
}
