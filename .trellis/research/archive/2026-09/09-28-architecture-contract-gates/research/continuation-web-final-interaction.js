await page.getByLabel('默认模型',{exact:true}).fill('codex-after');
await page.locator('main').first().getByRole('button',{name:'保存',exact:true}).click();
await page.waitForFunction(()=>ccrFixture.calls.filter(c=>c.cmd==='codex_update_settings').length===1);
const calls = await page.evaluate(()=>ccrFixture.calls.filter(c=>c.cmd==='codex_update_settings'));
expect(calls[0].args).toEqual({settings:{model:'codex-after'},confirmationToken:'desktop-confirm:codex_update_settings'});
expect(await page.evaluate(()=>ccrFixture.settings.tui.notifications)).toEqual(['agent-turn-complete','approval-requested']);
expect((await page.locator('body').innerText()).match(/⟦[^⟧]+⟧/g)||[]).toEqual([]);
await page.getByRole('button',{name:'源文件',exact:true}).click();
return {patch:calls[0].args, dialogs:await page.locator('[role="dialog"], [role="alertdialog"]').allTextContents(),buttons:await page.getByRole('button').allTextContents(),text:(await page.locator('body').innerText()).slice(-5000),errors:ccrWebErrors};