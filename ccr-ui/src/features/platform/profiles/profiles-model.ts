import type { ProfilesConfig, ProfileRecord } from '@/configs/profiles'
import type { ProfileRowDescriptor } from '@/utils/profileDescriptors'
import type { TranslateFunction } from '@/utils/tf'

export function makeProfileRowDescriptor(
  config: ProfilesConfig,
  t: TranslateFunction,
): ProfileRowDescriptor<ProfileRecord> {
  return {
    baseUrl: (profile) => profile.baseUrl ?? '',
    model: (profile) => profile.model ?? '',
    authMode: (profile) => profile.authMode ?? '',
    editIcon: config.editIcon,
    labels: {
      apply: t(`${config.i18nPrefix}.actions.apply`),
      edit: t(`${config.i18nPrefix}.actions.edit`),
      delete: t(`${config.i18nPrefix}.actions.delete`),
    },
  }
}
