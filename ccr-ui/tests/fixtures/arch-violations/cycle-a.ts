// [cycle fixture a] 循环依赖自检夹具（check-cycles.mjs --self-check 定向扫描，不进常规 lint）
import { b } from './cycle-b'
export const a = () => b()
