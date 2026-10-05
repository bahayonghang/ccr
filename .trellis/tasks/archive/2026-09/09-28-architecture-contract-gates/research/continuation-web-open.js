globalThis.ccrWebErrors = [];
page.on('pageerror', error => ccrWebErrors.push(String(error.message).slice(0, 400)));
await page.goto('http://127.0.0.1:49173/', { waitUntil: 'domcontentloaded' });
await expect(page.locator('body')).toContainText('CCR', { timeout: 30000 });
return { observation: await tabbit.observe({ frames: 'none', maxChars: 6500 }), errors: ccrWebErrors };