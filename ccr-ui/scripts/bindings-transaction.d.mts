export type DirectorySnapshot = {
  exists: boolean
  directories: string[]
  files: Map<string, Buffer>
}

export function snapshotDirectory(directory: string): Promise<DirectorySnapshot>
export function restoreDirectory(directory: string, snapshot: DirectorySnapshot): Promise<void>
export function diffSnapshots(before: Map<string, Buffer>, after: Map<string, Buffer>): string[]
export function withGeneratedDirectory(
  directory: string,
  operation: (before: DirectorySnapshot) => number | Promise<number>,
  options?: {
    restoreAlways?: boolean
    restore?: (directory: string, snapshot: DirectorySnapshot) => void | Promise<void>
  },
): Promise<number>
