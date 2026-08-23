// Baseline screenshot capture driver. Invoked from browser run scope: require(this)(page)
module.exports = async function capture(page) {
  const fs = require('fs')
  const mod = await import(
    'file:///D:/Documents/Code/Github/ccr/.trellis/tasks/08-22-react-migration/baseline/routes.mjs'
  )
  const routes = mod.BASELINE_ROUTES
  const slug = mod.slug
  const base =
    'D:/Documents/Code/Github/ccr/.trellis/tasks/08-22-react-migration/baseline/screens'
  const out = []
  for (const theme of ['light', 'dark']) {
    fs.mkdirSync(base + '/' + theme, { recursive: true })
    await page.evaluateOnNewDocument((t) => {
      try {
        localStorage.setItem('ccr-theme', t)
      } catch (e) {}
      document.documentElement.addEventListener('DOMContentLoaded', () => {
        document.documentElement.setAttribute('data-theme', t)
        document.documentElement.classList.toggle('dark', t === 'dark')
      })
    }, theme)
    for (const r of routes) {
      const url = 'http://127.0.0.1:4180' + r
      try {
        await page.goto(url, { waitUntil: 'networkidle2', timeout: 20000 })
      } catch (e) {
        // proceed with whatever rendered
      }
      await new Promise((res) => setTimeout(res, 500))
      const file = base + '/' + theme + '/' + slug(r) + '.png'
      await page.screenshot({ path: file, timeout: 15000 })
      out.push(theme + '/' + slug(r))
    }
  }
  return { count: out.length, first: out[0], last: out[out.length - 1] }
}
