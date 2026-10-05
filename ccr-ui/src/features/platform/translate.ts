import { translate } from '@/i18n'
import type { TranslateFunction } from '@/utils/tf'

/** 功能面默认 t：走 i18next。组件内请用 useAppT / useResolvedT 以订阅语言切换。 */
export const defaultSurfaceT: TranslateFunction = translate
