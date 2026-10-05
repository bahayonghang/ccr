// [arch-violation fixture] 反向依赖：utils 导入 store（规则自检用，常规 lint 已忽略本目录）
import { useSomeStore } from './store'
export const getter = useSomeStore
