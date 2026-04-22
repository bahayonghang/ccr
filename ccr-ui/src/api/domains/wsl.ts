/**
 * WSL Domain —— Windows Subsystem for Linux 发行版探测与配置读写 API
 *
 * WSL 命令实现位于 `../runtime/wsl`（历史阶段性落地），此 domain 作为统一门面
 * 对齐其它 domain 的 import 风格：`import { wslApi } from '@/api'` 将来可用。
 *
 * 对应后端 commands::wsl::* 命令（Windows 专属）。
 */

export {
  getWslCacheStatus,
  listWslDistros,
  refreshWslDistros,
  clearWslCache,
  detectWslCli,
  readWslConfig,
  syncWslConfig,
  type WslDistro,
  type WslCacheStatus,
  type WslCliStatus,
  type ReadWslConfigRequest,
  type SyncWslConfigRequest,
} from '../runtime/wsl'
