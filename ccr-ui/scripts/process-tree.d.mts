import type { EventEmitter } from 'node:events'

export interface ProcessTreeChild extends Pick<EventEmitter, 'off' | 'once'> {
  pid?: number
  exitCode: number | null
  signalCode: NodeJS.Signals | null
  kill(signal?: NodeJS.Signals | number): boolean
}

export interface ProcessTreeOptions {
  platform?: NodeJS.Platform
  graceMs?: number
  forceWaitMs?: number
  runFile?: (
    file: string,
    args: string[],
    options: { windowsHide: boolean },
  ) => Promise<unknown>
}

export function terminateProcessTree(
  child: ProcessTreeChild,
  options?: ProcessTreeOptions,
): Promise<void>
