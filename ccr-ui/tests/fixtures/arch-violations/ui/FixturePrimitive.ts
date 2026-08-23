// [arch-violation fixture] 跨层导入：UI 原语导入 feature 域（check:arch-boundaries 定向自检用）
import { marker } from '../features/claude'
export const primitive = marker
