const value=await page.evaluate(()=>({updates:ccrFixture.calls.filter(c=>c.cmd==='codex_update_settings'),settings:ccrFixture.settings,unknown:ccrFixture.unknown}));
assert.equal(value.updates.length,1);assert.equal(JSON.stringify(value.updates[0].args.settings),JSON.stringify({model:'codex-after'}));
assert.equal(JSON.stringify(value.settings.tui.notifications),JSON.stringify(['agent-turn-complete','approval-requested']));
assert.equal(value.settings.model_reasoning_effort,'future-effort');
assert.equal((await page.locator('main').innerText()).includes('⟦'),false);
return {value,errors:ccrWebErrors,sourceButton:await page.getByRole('button',{name:'源文件',exact:true}).isVisible()};