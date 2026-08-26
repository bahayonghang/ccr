import { chromium } from 'playwright'
import { mkdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

const REPO_ROOT = join(process.cwd(), '..')
const OUT_DIR = join(process.cwd(), 'tests/__screenshots__')
const NOTES_PATH = join(REPO_ROOT, '.trellis/tasks/08-26-visual-type-profiles/notes.md')
mkdirSync(OUT_DIR, { recursive: true })

const buildInitScript = (theme, flavor) => `
  localStorage.setItem('ccr-theme', '${theme}');
  localStorage.setItem('ccr-flavor', '${flavor}');
  const LONG_URL =
    'https://api.example.com/abcdefghijklmnopqrstuvwxyz0123456789/extra/path/segments';
  const profilesPayload = {
    profiles: [
      {
        name: 'claude-current',
        description: 'Current Claude relay',
        base_url: LONG_URL,
        model: 'claude-sonnet-4-6',
        provider: 'Anthropic',
        auth_mode: 'api_key',
        tags: ['prod'],
        enabled: true,
        is_current: true,
        usage_count: 12,
      },
      {
        name: 'claude-empty-model',
        description: 'No model set',
        base_url: 'https://relay.example.com',
        model: null,
        provider: 'Relay',
        auth_mode: 'subscription',
        tags: [],
        enabled: true,
        is_current: false,
      },
    ],
    current_profile: 'claude-current',
    can_off: true,
  };
  const envPayload = {
    id: 'local',
    env_type: 'local',
    name: 'local',
    display_name: 'Local',
    description: '',
    is_active: true,
  };
  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd) => {
      if (cmd === 'claude_list_profiles') return profilesPayload;
      if (cmd === 'get_current_environment') return envPayload;
      return {};
    },
    transformCallback: (cb) => (typeof cb === 'function' ? 1 : cb),
    unregisterCallback: () => {},
    convertFileSrc: (path) => path,
    runCallback: () => {},
    callbacks: new Map(),
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
  window.__TAURI__ = window.__TAURI__ || {};
`

const combos = [
  { id: 'W1', w: 1440, h: 900, theme: 'light', flavor: 'neutral' },
  { id: 'W2', w: 1440, h: 900, theme: 'light', flavor: 'clay' },
  { id: 'W3', w: 1440, h: 900, theme: 'dark', flavor: 'neutral' },
  { id: 'W4', w: 1440, h: 900, theme: 'dark', flavor: 'clay' },
  { id: 'W5', w: 900, h: 800, theme: 'light', flavor: 'neutral' },
  { id: 'W6', w: 900, h: 800, theme: 'light', flavor: 'clay' },
  { id: 'W7', w: 900, h: 800, theme: 'dark', flavor: 'neutral' },
  { id: 'W8', w: 900, h: 800, theme: 'dark', flavor: 'clay' },
]

const passFail = (ok) => (ok ? 'PASS' : 'FAIL')
const rows = []
const browser = await chromium.launch({ headless: true })

for (const combo of combos) {
  const context = await browser.newContext({ viewport: { width: combo.w, height: combo.h } })
  await context.addInitScript(buildInitScript(combo.theme, combo.flavor))
  const page = await context.newPage()

  let navError = null
  try {
    await page.goto('http://127.0.0.1:5173/claude-code/profiles', {
      waitUntil: 'networkidle',
      timeout: 45000,
    })
    await page.waitForSelector('[data-testid="profiles-page-header"]', { timeout: 20000 })
    await page.waitForSelector('[data-name="claude-current"]', { timeout: 20000 })
  } catch (error) {
    navError = error instanceof Error ? error.message : String(error)
  }

  const shotName = `visual-types-${combo.theme}-${combo.flavor}-${combo.w}x${combo.h}.png`
  const shotPath = join(OUT_DIR, shotName)
  await page.screenshot({ path: shotPath, fullPage: true })

  const evalResult = navError
    ? { navError, bodyText: await page.locator('body').innerText().catch(() => '') }
    : await page.evaluate(() => {
        const header = document.querySelector('[data-testid="profiles-page-header"]')
        const actions = header?.querySelector('.cp-page-header__actions')
        const buttons = actions ? Array.from(actions.querySelectorAll('.ui-btn')) : []
        const primary = buttons.find((btn) => btn.classList.contains('ui-btn--primary'))
        const ghosts = buttons.filter((btn) => btn.classList.contains('ui-btn--ghost'))
        const primaryBg = primary ? getComputedStyle(primary).backgroundColor : ''
        const ghostBgs = ghosts.map((btn) => getComputedStyle(btn).backgroundColor)
        const headerPass =
          Boolean(primary) &&
          ghosts.length >= 3 &&
          primaryBg !== 'rgba(0, 0, 0, 0)' &&
          ghostBgs.every((bg) => bg === 'rgba(0, 0, 0, 0)')

        const banner = document.querySelector('[data-testid="profiles-off-banner"]')
        const bannerStyle = banner ? getComputedStyle(banner) : null
        const card = document.querySelector('.cp-card')
        const cardStyle = card ? getComputedStyle(card) : null
        const bannerBtn = banner?.querySelector('.ui-btn--warning')
        const offPass =
          Boolean(banner) &&
          Boolean(bannerBtn) &&
          Boolean(bannerStyle) &&
          bannerStyle.backgroundColor !== cardStyle?.backgroundColor &&
          bannerStyle.borderTopWidth !== '0px'

        const current = document.querySelector('[data-name="claude-current"]')
        const fields = current ? Array.from(current.querySelectorAll('.cp-card__field')) : []
        const fieldsPass =
          Boolean(fields[0]?.querySelector('.ui-url-text')) &&
          Boolean(fields[1]) &&
          !fields[1].querySelector('.ui-badge') &&
          Boolean(fields[2]?.querySelector('.ui-badge--static')) &&
          Boolean(fields[3]?.querySelector('.ui-badge--static'))

        const dd = current?.querySelector('.cp-card__field dd')
        const grid = document.querySelector('.cp-card-grid')
        const gridRect = grid?.getBoundingClientRect()
        const cardRect = current?.getBoundingClientRect()
        const urlOverflowPass =
          Boolean(dd) &&
          getComputedStyle(dd).overflow === 'hidden' &&
          Boolean(gridRect) &&
          Boolean(cardRect) &&
          cardRect.right <= gridRect.right + 2

        const statusBadge = current?.querySelector('[data-testid="profile-row-status-badge"]')
        const statusPass =
          Boolean(statusBadge) &&
          statusBadge.classList.contains('ui-badge--static') &&
          getComputedStyle(statusBadge).cursor !== 'pointer'

        return {
          headerPass,
          offPass,
          fieldsPass,
          urlOverflowPass,
          statusPass,
          bannerBg: bannerStyle?.backgroundColor ?? null,
          bannerBorder: bannerStyle?.borderColor ?? null,
          primaryBg,
          ghostBgs,
        }
      })

  const cells = navError
    ? {
        header: 'FAIL',
        off: 'FAIL',
        fields: 'FAIL',
        urlOverflow: 'FAIL',
        statusBadge: 'FAIL',
        result: 'FAIL',
        navError,
      }
    : {
        header: passFail(evalResult.headerPass),
        off: passFail(evalResult.offPass),
        fields: passFail(evalResult.fieldsPass),
        urlOverflow: passFail(evalResult.urlOverflowPass),
        statusBadge: passFail(evalResult.statusPass),
        result:
          evalResult.headerPass &&
          evalResult.offPass &&
          evalResult.fieldsPass &&
          evalResult.urlOverflowPass &&
          evalResult.statusPass
            ? 'PASS'
            : 'FAIL',
        evalResult,
      }

  rows.push({
    id: combo.id,
    viewport: `${combo.w}x${combo.h}`,
    theme: combo.theme,
    flavor: combo.flavor,
    screenshot: `ccr-ui/tests/__screenshots__/${shotName}`,
    ...cells,
  })

  await context.close()
}

await browser.close()

const md = `# Profile visual types walkthrough

Web preview with page-start Tauri mock (\`context.addInitScript\`, equivalent to \`Page.addScriptToEvaluateOnNewDocument\`): \`http://127.0.0.1:5173/claude-code/profiles\`

| id | viewport | theme | flavor | header | off-surface | fields | url-overflow | status-badge | result | screenshot |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
${rows
  .map(
    (r) =>
      `| ${r.id} | ${r.viewport} | ${r.theme} | ${r.flavor} | ${r.header} | ${r.off} | ${r.fields} | ${r.urlOverflow} | ${r.statusBadge} | ${r.result} | ${r.screenshot} |`,
  )
  .join('\n')}
`

writeFileSync(NOTES_PATH, md, 'utf8')
console.log(JSON.stringify(rows, null, 2))
