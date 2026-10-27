import { createRequire } from 'node:module'
import { fileURLToPath, pathToFileURL } from 'node:url'
import path from 'node:path'

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..')
const root = path.join(repo, 'ccr-ui')
const require = createRequire(path.join(root, 'package.json'))
const { createServer } = await import(pathToFileURL(require.resolve('vite')).href)
const server = await createServer({
  root,
  configFile: path.join(root, 'vite.config.ts'),
  plugins: [{ name: 'task-baseline-watch-override', configResolved(config) { config.server.watch = null } }],
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
    watch: null,
  },
})
await server.listen()
console.log(JSON.stringify({ pid: process.pid, url: 'http://127.0.0.1:5173', mode: 'task-only watch:null workaround; restart after source edits; no product config changed' }))
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, async () => { await server.close(); process.exit(0) })
