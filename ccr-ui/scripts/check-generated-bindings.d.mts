import type { BindingRunner } from './generate-bindings.mjs'
export { diffSnapshots, snapshotDirectory } from './bindings-transaction.mjs'
export function checkBindings(options?: {
  directory?: string
  runStep?: BindingRunner
  writeError?: (message: string) => void
}): Promise<number>
