// [arch-violation fixture] 门面消费侧绕过：直接导入冻结门面 tauri.ts（规则自检用，常规 lint 已忽略本目录）
import { listConfigs } from '../../../src/api/tauri'
export const configs = listConfigs
