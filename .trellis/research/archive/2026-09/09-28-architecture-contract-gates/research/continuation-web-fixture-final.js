const install=()=>{
 globalThis.ccrFixture={calls:[],unknown:[],settings:{model:'codex-before',model_reasoning_effort:'future-effort',tui:{notifications:['agent-turn-complete','approval-requested']}}};
 mockIPC((cmd,args)=>{const f=globalThis.ccrFixture;f.calls.push({cmd,args});switch(cmd){
 case 'get_current_environment':return {id:'local',env_type:'local',name:'Synthetic Local',available:true};
 case 'list_environments':return [{id:'local',env_type:'local',name:'Synthetic Local',available:true}];
 case 'codex_get_settings':return structuredClone(f.settings);
 case 'codex_update_settings':Object.assign(f.settings,args.settings);return {message:'synthetic saved'};
 case 'codex_get_config_raw_text':return {status:'ok',exists:true,path:'/synthetic/config.toml',token:'v1',content:['model = '+JSON.stringify(f.settings.model),'[tui]','notifications = ["agent-turn-complete", "approval-requested"]',''].join(String.fromCharCode(10))};
 case 'codex_list_config_layers':return {layers:[]};
 case 'health_check':return {status:'healthy',database:true};
 case 'shell_get_preferences':return {confirm_before_exit:true,close_to_tray:false,open_panel_on_tray_click:true,tray_panel:{placement_mode:'anchored',manual_position:null}};
 case 'plugin:window|show':case 'append_frontend_logs':return null;
 case 'plugin:app|version':return '7.3.0';
 case 'plugin:window|is_maximized':return false;
 case 'plugin:window|is_focused':return true;
 default:f.unknown.push(cmd);throw new Error('Synthetic fixture does not implement '+cmd);
 }},{shouldMockEvents:true});mockWindows('main');
};
const boot="import { mockIPC,mockWindows } from '/node_modules/@tauri-apps/api/mocks.js';\n("+install.toString()+")();\nawait import('/src/main.tsx?ccr_fixture=final');";
globalThis.ccrWebErrors=[];page.on('pageerror',e=>ccrWebErrors.push(String(e)));
await page.route(/\/src\/main\.tsx(?:\?t=\d+)?$/,route=>route.fulfill({status:200,contentType:'application/javascript',body:boot}));
await page.goto('http://127.0.0.1:49173/codex/settings',{waitUntil:'domcontentloaded'});
await expect(page.locator('input[value="codex-before"]')).toBeVisible({timeout:30000});
return {state:await tabbit.observe({frames:'none',maxChars:4600}),fixture:await page.evaluate(()=>({unknown:ccrFixture.unknown,calls:ccrFixture.calls.map(c=>c.cmd)})),errors:ccrWebErrors};