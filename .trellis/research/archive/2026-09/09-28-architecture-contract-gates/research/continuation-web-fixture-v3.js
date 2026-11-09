const boot = [
  "import { mockIPC, mockWindows } from '/node_modules/@tauri-apps/api/mocks.js';",
  "globalThis.ccrFixture = { calls: [], unknown: [], settings: { model: 'codex-before', model_reasoning_effort: 'future-effort', tui: {notifications:['agent-turn-complete','approval-requested']} } };",
  "mockIPC((cmd,args) => { const f=globalThis.ccrFixture; f.calls.push({cmd,args}); switch(cmd) {",
  "case 'get_current_environment': return {id:'local',env_type:'local',name:'Synthetic Local',available:true};",
  "case 'list_environments': return [{id:'local',env_type:'local',name:'Synthetic Local',available:true}];",
  "case 'codex_get_settings': return structuredClone(f.settings);",
  "case 'codex_update_settings': Object.assign(f.settings,args.settings); return {message:'synthetic saved'};",
  "case 'codex_get_config_raw_text': return {status:'ok',exists:true,path:'/synthetic/config.toml',token:'v1',content:'model = \"codex-before\"\\n[tui]\\nnotifications = [\"agent-turn-complete\", \"approval-requested\"]\\n'};",
  "case 'codex_list_config_layers': return {layers:[]};",
  "case 'health_check': return {status:'healthy',database:true};",
  "case 'shell_get_preferences': return {confirm_before_exit:true,close_to_tray:false,open_panel_on_tray_click:true,tray_panel:{placement_mode:'anchored',manual_position:null}};",
  "case 'plugin:window|show': case 'append_frontend_logs': return null;",
  "case 'plugin:app|version': return '7.3.0';",
  "case 'plugin:window|is_maximized': return false;",
  "case 'plugin:window|is_focused': return true;",
  "default: f.unknown.push(cmd); throw new Error('Synthetic fixture does not implement '+cmd); } }, {shouldMockEvents:true});",
  "mockWindows('main');",
  "await import('/src/main.tsx?ccr_fixture=1');"
].join(String.fromCharCode(10));
await page.unrouteAll();
await page.route(/\/src\/main\.tsx(?:\?t=\d+)?$/, route => route.fulfill({status:200,contentType:'application/javascript',body:boot}));
await page.reload({waitUntil:'domcontentloaded'});
await expect(page.locator('main').first()).toContainText('Codex 设置',{timeout:30000});
await expect(page.getByDisplayValue ? page.getByDisplayValue('codex-before') : page.locator('input[value="codex-before"]')).toBeVisible({timeout:30000});
return {main:(await page.locator('main').first().innerText()).slice(0,6000), inputs:await page.locator('main input').evaluateAll(nodes=>nodes.map(n=>({id:n.id,value:n.value,type:n.type}))), fixture:await page.evaluate(()=>({calls:ccrFixture.calls.map(c=>c.cmd),unknown:ccrFixture.unknown})),errors:ccrWebErrors};