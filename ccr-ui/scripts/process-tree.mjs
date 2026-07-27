import { execFile } from 'node:child_process'

const shutdowns = new WeakMap()

const runFile = (file, args, options) => new Promise((resolve, reject) => {
  execFile(file, args, options, (error, stdout, stderr) => {
    if (error) {
      reject(error)
      return
    }
    resolve({ stdout, stderr })
  })
})

const hasExited = (child) => child.exitCode !== null || child.signalCode !== null

const waitForExit = async (child, timeoutMs) => {
  if (hasExited(child)) return true

  return await new Promise((resolve) => {
    const onExit = () => {
      clearTimeout(timer)
      resolve(true)
    }
    const timer = setTimeout(() => {
      child.off('exit', onExit)
      resolve(hasExited(child))
    }, timeoutMs)
    child.once('exit', onExit)
  })
}

const terminate = async (child, options) => {
  if (!child?.pid || hasExited(child)) return

  const platform = options.platform ?? process.platform
  const graceMs = options.graceMs ?? 3000
  const forceWaitMs = options.forceWaitMs ?? 2000

  if (platform === 'win32') {
    let terminationError
    try {
      await (options.runFile ?? runFile)(
        'taskkill.exe',
        ['/PID', String(child.pid), '/T', '/F'],
        { windowsHide: true },
      )
    } catch (error) {
      terminationError = error
    }

    const exited = await waitForExit(child, forceWaitMs)
    if (!exited && terminationError) throw terminationError
    if (!exited) throw new Error(`Process tree ${child.pid} did not exit after taskkill`)
    return
  }

  child.kill('SIGTERM')
  if (await waitForExit(child, graceMs)) return

  child.kill('SIGKILL')
  if (!(await waitForExit(child, forceWaitMs))) {
    throw new Error(`Process tree ${child.pid} did not exit after SIGKILL`)
  }
}

export const terminateProcessTree = (child, options = {}) => {
  const existing = shutdowns.get(child)
  if (existing) return existing

  const shutdown = terminate(child, options)
  shutdowns.set(child, shutdown)
  return shutdown
}
