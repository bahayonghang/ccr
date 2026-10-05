const calls = await page.evaluate(()=>ccrFixture.calls.filter(c=>c.cmd==='codex_update_settings'));
expect(calls).toHaveLength(1);
expect(calls[0].args.settings).toEqual({model:'codex-after'});
expect(calls[0].args.confirmationToken).toBe('desktop-confirm:codex_update_settings');
expect(await page.evaluate(()=>ccrFixture.settings.tui.notifications)).toEqual(['agent-turn-complete','approval-requested']);
expect(await page.getByLabel('默认模型',{exact:true}).inputValue()).toBe('codex-after');
await page.getByRole('button',{name:'源文件',exact:true}).click();
await expect(page.locator('.cm-editor')).toBeVisible();
return {patch:calls[0].args, editorCount:await page.locator('.cm-editor').count(), main:(await page.locator('main').first().innerText()).slice(0,6500), untranslated:await page.locator('body').innerText().then(t=>t.match(/⟦[^⟧]+⟧/g)||[]),errors:ccrWebErrors};