await page.getByRole('link', { name: 'Codex', exact: true }).click();
await page.waitForURL('**/codex');
return { observation: await tabbit.observe({ frames: 'none', maxChars: 6500 }), errors: ccrWebErrors };