import {
  invoke as tauriInvoke,
  type InvokeArgs,
  type InvokeOptions,
} from '@tauri-apps/api/core'

import { COMMAND_MANIFEST } from '@/api/generated/commandCapabilities'

type CommandCapability = (typeof COMMAND_MANIFEST.commands)[number]

const COMMANDS: ReadonlyMap<string, CommandCapability> = new Map(
  COMMAND_MANIFEST.commands.map(command => [command.id, command]),
)

const withCapabilityConfirmation = (command: string, args?: InvokeArgs): InvokeArgs | undefined => {
  const capability = COMMANDS.get(command)
  if (capability?.confirmation !== 'user_gesture') return args

  if (
    args instanceof ArrayBuffer
    || args instanceof Uint8Array
    || Array.isArray(args)
  ) {
    throw new TypeError(`Command ${command} requires a JSON confirmation payload`)
  }

  return {
    ...(args ?? {}),
    confirmationToken: `desktop-confirm:${command}`,
  }
}

export const invoke = <T>(
  command: string,
  args?: InvokeArgs,
  options?: InvokeOptions,
): Promise<T> => {
  const payload = withCapabilityConfirmation(command, args)
  return options === undefined
    ? tauriInvoke<T>(command, payload)
    : tauriInvoke<T>(command, payload, options)
}
