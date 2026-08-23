import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

/**
 * 原语层类名合并助手（shadcn/ui 约定，08-22-design-system 批次 3）。
 *
 * - `clsx`：过滤 falsy / 条件类名。
 * - `tailwind-merge`：解决工具类冲突（如自定义 className 覆盖默认 padding）。
 *   由于本仓工具类均映射自 CCR token 命名空间（`--color-*` / `--shadow-*` / `--z-*`…），
 *   twMerge 按 Tailwind 类名分组去冲突，对 token 命名的工具类同样生效。
 *
 * 边界：`src/ui/` 只依赖 `utils` / `types` / `shared` 层，本助手不触碰业务逻辑。
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs))
}
