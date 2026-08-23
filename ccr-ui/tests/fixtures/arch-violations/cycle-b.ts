// [cycle fixture b] 循环依赖自检夹具（check-cycles.mjs --self-check 定向扫描，不进常规 lint）
import { a } from './cycle-a'
export const b = () => a()
