const { spawnSync } = require('node:child_process')
const { existsSync } = require('node:fs')
const { resolve } = require('node:path')

const uiRoot = resolve(__dirname, '..')

const requiredAssets = [
  'public/icons/icon.svg',
  'public/icons/logo.svg',
  'src/assets/favicon.svg',
  'src-tauri/icons/32x32.png',
  'src-tauri/icons/128x128.png',
  'src-tauri/icons/icon.png',
  'src-tauri/icons/icon@2x.png',
  'src-tauri/icons/icon.ico',
  'src-tauri/icons/icon.icns',
]

function writeLine(stream, message) {
  stream.write(`${message}\n`)
}

function info(message) {
  writeLine(process.stdout, message)
}

function warn(message) {
  writeLine(process.stderr, message)
}

function error(message) {
  writeLine(process.stderr, message)
}

function missingAssets() {
  return requiredAssets.filter((asset) => !existsSync(resolve(uiRoot, asset)))
}

function runIconGeneration() {
  const result = spawnSync('uv', ['run', './scripts/generate_icons.py'], {
    cwd: uiRoot,
    stdio: 'inherit',
  })

  if (result.error) {
    throw result.error
  }

  return result.status ?? 1
}

function canUseUv() {
  const result = spawnSync('uv', ['--version'], {
    cwd: uiRoot,
    stdio: 'ignore',
  })

  return !result.error && result.status === 0
}

function main() {
  if (process.env.CCR_SKIP_ICON_GENERATION === '1') {
    const missing = missingAssets()
    if (missing.length > 0) {
      error('CCR_SKIP_ICON_GENERATION=1 but required icon assets are missing:')
      for (const asset of missing) {
        error(`- ${asset}`)
      }
      process.exit(1)
    }

    info('Skipping icon generation because CCR_SKIP_ICON_GENERATION=1')
    return
  }

  if (canUseUv()) {
    process.exit(runIconGeneration())
  }

  const missing = missingAssets()
  if (missing.length === 0) {
    warn('uv not found, using committed icon assets')
    return
  }

  error('uv not found and required icon assets are missing:')
  for (const asset of missing) {
    error(`- ${asset}`)
  }
  error('Install uv or regenerate icons before building.')
  process.exit(1)
}

main()
