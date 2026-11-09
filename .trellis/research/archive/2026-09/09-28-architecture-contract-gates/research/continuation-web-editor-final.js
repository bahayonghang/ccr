await page.getByRole('dialog',{name:'查看原始配置',exact:true}).getByRole('button',{name:'继续查看',exact:true}).click();
await expect(page.locator('.cm-editor')).toBeVisible({timeout:20000});
const content=await page.locator('.cm-content').innerText();
assert.equal(content.includes('model = "codex-after"'),true);
assert.equal(content.includes('notifications = ["agent-turn-complete", "approval-requested"]'),true);
assert.equal(content.includes(String.fromCharCode(92)+'n'),false);
const fixture=await page.evaluate(()=>({unknown:ccrFixture.unknown,reads:ccrFixture.calls.filter(c=>c.cmd==='codex_get_config_raw_text'),updates:ccrFixture.calls.filter(c=>c.cmd==='codex_update_settings')}));
assert.equal(fixture.unknown.length,0);assert.equal(fixture.reads.length,1);assert.equal(fixture.updates.length,1);assert.equal(ccrWebErrors.length,0);
return {content,fixture,main:(await page.locator('main').innerText()).slice(0,2500),errors:ccrWebErrors,styles:await page.locator('.cm-scroller').evaluate(e=>({display:getComputedStyle(e).display,editorTop:e.getBoundingClientRect().top,gutterTop:e.querySelector('.cm-gutters')?.getBoundingClientRect().top,contentTop:e.querySelector('.cm-content')?.getBoundingClientRect().top}))};