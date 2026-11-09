await page.getByRole('navigation', { name: 'Module navigation', exact: true }).getByRole('link', { name: '设置', exact: true }).click();
await page.waitForURL('**/codex/settings');
await expect(page.locator('main').first()).toContainText('设置', { timeout: 30000 });
return { main: (await page.locator('main').first().innerText()).slice(0, 6000), buttons: await page.locator('main').first().getByRole('button').evaluateAll(nodes => nodes.map(n=>({text:n.textContent,disabled:n.disabled}))), errors: ccrWebErrors };