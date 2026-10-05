await page.unrouteAll();
await page.reload({waitUntil:'domcontentloaded'});
await expect(page.locator('main')).toContainText('Tauri runtime is unavailable for get_current_environment',{timeout:20000});
return {main:(await page.locator('main').innerText()).slice(0,1800),retry:await page.getByRole('button',{name:'重试',exact:true}).count(),hasFixture:await page.evaluate(()=>Boolean(globalThis.ccrFixture)),errors:ccrWebErrors};