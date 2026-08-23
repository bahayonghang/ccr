export function loadCommandsView() {
  return import('./CommandsView').then((mod) => ({ Component: mod.CommandsView }))
}

export function loadSlashCommandsView() {
  return import('./SlashCommandsView').then((mod) => ({ Component: mod.SlashCommandsView }))
}

export const commandsRouteLoaders = {
  commands: loadCommandsView,
  'slash-commands': loadSlashCommandsView,
} as const
