"""Exercise packaged Tauri assets in an isolated native Linux WebView."""
import base64
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import sys
import tempfile
import time
import traceback
import urllib.error
import urllib.request
from urllib.parse import urlsplit

HERE = Path(__file__).resolve().parent
TOOLS = Path('/tmp/ccr-architecture-tools-01a0e781')
attempt = sys.argv[1]
if not attempt.isdigit():
    raise SystemExit('Expected numeric attempt identifier')
PREFIX = HERE / ('remaining-native-webview-' + attempt)
OUTPUT = PREFIX.with_suffix('.json')
if OUTPUT.exists():
    raise SystemExit('Preserve existing evidence')
build_name = sys.argv[2] if len(sys.argv) > 2 else 'remaining-linux-native-build.json'
if build_name not in ('remaining-linux-native-build.json', 'remaining-linux-native-after-csp.json'):
    raise SystemExit('Expected a recorded native build')
build = json.loads((HERE / build_name).read_text())
binary = Path(build['binary'])
binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
if build['exit_code'] != 0 or binary_hash != build['binary_sha256']:
    raise SystemExit('Native executable must match a successful build record')
fixture = Path(tempfile.mkdtemp(prefix='ccr-native-fixture-'))
home = fixture / 'home'
for path in [home, home / '.claude', home / '.codex', home / '.ccr', fixture / 'runtime']:
    path.mkdir(parents=True, exist_ok=True)
    path.chmod(0o700)
settings = home / '.claude/settings.json'
original = {'model':'sonnet','env':{'CCR_NATIVE_FIXTURE':'preserve'},'native_fixture':{'unknown':True,'count':7}}
settings.write_text(json.dumps(original, indent=2) + chr(10))
env = {
    'HOME':str(home), 'USERPROFILE':str(home), 'PATH':str(TOOLS / 'bin') + ':/usr/bin:/bin',
    'CCR_ROOT':str(home / '.ccr'), 'CCR_DATA_DIR':str(home / '.ccr'),
    'CLAUDE_CONFIG_DIR':str(home / '.claude'), 'CODEX_HOME':str(home / '.codex'),
    'LLMUSAGE_HOME':str(home / '.llmusage'), 'XDG_CONFIG_HOME':str(home / '.config'),
    'XDG_DATA_HOME':str(home / '.local/share'), 'XDG_CACHE_HOME':str(home / '.cache'),
    'XDG_STATE_HOME':str(home / '.local/state'), 'XDG_RUNTIME_DIR':str(fixture / 'runtime'),
    'TMPDIR':str(fixture), 'DISPLAY':':0', 'GDK_BACKEND':'x11', 'LANG':'C.UTF-8', 'LC_ALL':'C.UTF-8',
}
def port():
    with socket.socket() as sock:
        sock.bind(('127.0.0.1',0))
        return sock.getsockname()[1]
driver_port = port()
native_port = port()
while native_port == driver_port:
    native_port = port()
base = f'http://127.0.0.1:{driver_port}'
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
session = None
process = None
record = {'started_at_utc':datetime.now(timezone.utc).isoformat(),'state':'running','fixture_root':str(fixture),'environment':env,'binary':str(binary),'binary_sha256':binary_hash,'build_record':'remaining-linux-native-build.json','checks':{},'scope':'Native Linux WebKitGTK under WSLg; production custom-protocol assets/CSP; synthetic local settings; no account authentication'}
record['harness_sha256'] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
record['build_record'] = build_name
record['build_record_sha256'] = hashlib.sha256((HERE / build_name).read_bytes()).hexdigest()
def save():
    OUTPUT.write_text(json.dumps(record, indent=2, ensure_ascii=False) + chr(10))
def request(method, path, payload=None, timeout=45):
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(base + path, data=data, method=method, headers={'Content-Type':'application/json'})
    try:
        with opener.open(req, timeout=timeout) as response:
            result = json.load(response)
    except urllib.error.HTTPError as error:
        raise RuntimeError(error.read().decode()) from error
    value = result.get('value', result)
    if isinstance(value,dict) and value.get('error'):
        raise RuntimeError(json.dumps(value))
    return value
def script(code, *args):
    return request('POST', f'/session/{session}/execute/sync', {'script':code,'args':list(args)})
def wait_for(callback, label, seconds=30):
    deadline = time.monotonic() + seconds
    last_error = None
    while time.monotonic() < deadline:
        if process and process.poll() is not None:
            raise RuntimeError(f'driver exited: {process.returncode}')
        try:
            result = callback()
            if result:
                return result
        except (OSError, RuntimeError) as error:
            last_error = str(error)
        time.sleep(0.2)
    raise RuntimeError(f'Timed out waiting for {label}: {last_error}')
def element(selector):
    value = request('POST', f'/session/{session}/element', {'using':'css selector','value':selector})
    return value['element-6066-11e4-a52e-4f735466cecf']
save()
try:
    command = ['/usr/bin/dbus-run-session','--',str(TOOLS / 'bin/tauri-driver'),'--port',str(driver_port),'--native-port',str(native_port),'--native-driver',str(TOOLS / 'webkit/usr/bin/WebKitWebDriver')]
    record['driver_command'] = command
    with PREFIX.with_suffix('.log').open('wb') as log:
        process = subprocess.Popen(command, cwd=fixture, env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
    record['driver_pid'] = process.pid
    wait_for(lambda:request('GET','/status',timeout=2).get('ready') is True, 'driver readiness', 20)
    result = request('POST','/session',{'capabilities':{'alwaysMatch':{'tauri:options':{'application':str(binary)},'timeouts':{'implicit':0,'pageLoad':30000,'script':30000}}}})
    session = result['sessionId']
    record['capabilities'] = result.get('capabilities')
    wait_for(lambda:script('return document.readyState === "complete" && !!document.querySelector("#app")'), 'native page')
    initial = script('return {url:location.href,native:typeof window.__TAURI_INTERNALS__?.invoke === "function",title:document.title}')
    record['checks']['native_boot'] = initial
    parsed = urlsplit(initial['url'])
    assert initial['native'] and (parsed.scheme == 'tauri' or parsed.hostname == 'tauri.localhost'), initial
    origin = parsed.scheme + '://' + parsed.netloc
    request('POST',f'/session/{session}/url',{'url':origin + '/claude-code/settings'})
    wait_for(lambda:script('return [...document.querySelectorAll("button")].some(b => !b.disabled && b.textContent.trim() === "源文件")'), 'settings source action')
    source_buttons = script('return [...document.querySelectorAll("button")].filter(b => !b.disabled && b.textContent.trim() === "源文件").map(b => b.textContent.trim())')
    assert len(source_buttons) == 1, source_buttons
    script('window.__ccrNativeStyleCspEvents=[]; window.__ccrNativeStyleAdditions=[]; new MutationObserver(records=>{for(const r of records) for(const n of r.addedNodes) if(n.nodeName === "STYLE") window.__ccrNativeStyleAdditions.push({nonce:n.nonce,text:n.textContent.slice(0,1200),parent:n.parentElement?.tagName})}).observe(document.documentElement,{subtree:true,childList:true}); document.addEventListener("securitypolicyviolation",e=>{if(e.effectiveDirective.startsWith("style-src")) window.__ccrNativeStyleCspEvents.push({directive:e.effectiveDirective,blockedURI:e.blockedURI,sourceFile:e.sourceFile,lineNumber:e.lineNumber,columnNumber:e.columnNumber,sample:e.sample,originalPolicy:e.originalPolicy})}); return true')
    source_element = script('return [...document.querySelectorAll("button")].find(b => !b.disabled && b.textContent.trim() === "源文件")')['element-6066-11e4-a52e-4f735466cecf']
    request('POST',f'/session/{session}/element/{source_element}/click',{})
    wait_for(lambda:script('return [...document.querySelectorAll("button")].some(b => !b.disabled && b.textContent.trim() === "继续查看")'), 'raw configuration confirmation')
    record['checks']['plaintext_confirmation'] = script('return {visible:document.body.innerText.includes("查看原始配置"),editorCount:document.querySelectorAll(".cm-editor").length}')
    assert record['checks']['plaintext_confirmation'] == {'visible':True,'editorCount':0}
    record['styles_before_plaintext_confirmation'] = script('return {events:window.__ccrNativeStyleCspEvents,styles:window.__ccrNativeStyleAdditions}')
    scroll_lock = script('const s=[...document.querySelectorAll("style")].find(s=>s.textContent.includes("body[data-scroll-locked]")); return {nonce:s?.nonce,pageNonce:document.querySelector("style[nonce]")?.nonce,sheetReadable:!!s?.sheet,rules:s?.sheet?.cssRules.length,bodyOverflow:getComputedStyle(document.body).overflow,locked:document.body.hasAttribute("data-scroll-locked")}')
    record['checks']['native_scroll_lock'] = scroll_lock
    assert scroll_lock['nonce'] and scroll_lock['nonce'] == scroll_lock['pageNonce'], scroll_lock
    assert scroll_lock['sheetReadable'] and scroll_lock['rules'] > 0, scroll_lock
    assert scroll_lock['bodyOverflow'] == 'hidden' and scroll_lock['locked'], scroll_lock
    confirm_element = script('return [...document.querySelectorAll("button")].find(b => !b.disabled && b.textContent.trim() === "继续查看")')['element-6066-11e4-a52e-4f735466cecf']
    request('POST',f'/session/{session}/element/{confirm_element}/click',{})
    wait_for(lambda:script('return !document.body.hasAttribute("data-scroll-locked")'), 'confirmation scroll lock release')
    wait_for(lambda:script('return document.querySelector(".cm-content")?.textContent.includes("native_fixture")'), 'CodeMirror fixture')
    record['checks']['native_editor'] = script('const e=document.querySelector(".cm-editor"),c=document.querySelector(".cm-content"),s=getComputedStyle(e); return {text:c.innerText,editorHeight:e.getBoundingClientRect().height,editorDisplay:s.display,editorMinHeight:parseFloat(s.minHeight),contentMinHeight:parseFloat(getComputedStyle(c).minHeight),rootFontSize:parseFloat(getComputedStyle(document.documentElement).fontSize),editorFontSize:s.fontSize,styleCspViolations:window.__ccrNativeStyleCspEvents,styleSheets:document.styleSheets.length,noncedStyles:[...document.querySelectorAll("style")].filter(x=>!!x.nonce).length,tokenSpans:document.querySelectorAll(".cm-line span").length,url:location.href}')
    assert record['checks']['native_editor']['editorHeight'] > 0
    assert record['checks']['native_editor']['tokenSpans'] > 0
    editor_style = record['checks']['native_editor']
    record['editor_runtime_style'] = script('const s=[...document.querySelectorAll("style")].find(s=>s.textContent.includes(".cm-scroller")),c=document.querySelector(".cm-content"),g=document.querySelector(".cm-gutters"); return {nonce:s?.nonce,pageNonce:document.querySelector("style[nonce]")?.nonce,sheetReadable:!!s?.sheet,rules:s?.sheet?.cssRules.length,scrollerDisplay:getComputedStyle(document.querySelector(".cm-scroller")).display,contentTop:c.getBoundingClientRect().top,gutterTop:g.getBoundingClientRect().top}')
    runtime_style = record['editor_runtime_style']
    assert runtime_style['nonce'] and runtime_style['nonce'] == runtime_style['pageNonce'], runtime_style
    assert runtime_style['sheetReadable'] and runtime_style['rules'] > 0 and runtime_style['scrollerDisplay'] == 'flex', runtime_style
    assert abs(runtime_style['contentTop'] - runtime_style['gutterTop']) < 1, runtime_style
    record['styles_after_editor_mount'] = script('return {events:window.__ccrNativeStyleCspEvents,styles:window.__ccrNativeStyleAdditions}')
    assert editor_style['editorDisplay'] == 'flex', editor_style
    assert editor_style['editorFontSize'] == '13px', editor_style
    assert abs(editor_style['editorMinHeight'] - 28 * editor_style['rootFontSize']) < 1, editor_style
    assert abs(editor_style['contentMinHeight'] - 28 * editor_style['rootFontSize']) < 1, editor_style
    assert editor_style['styleCspViolations'] == [], editor_style
    PREFIX.with_suffix('.png').write_bytes(base64.b64decode(request('GET',f'/session/{session}/screenshot')))
    script('window.__ccrNativeCspEvents=[]; window.__ccrNativeCspProbe=false; document.addEventListener("securitypolicyviolation",e=>window.__ccrNativeCspEvents.push({directive:e.effectiveDirective,blockedURI:e.blockedURI})); const s=document.createElement("script"); s.textContent="window.__ccrNativeCspProbe=true"; document.head.appendChild(s); return true')
    wait_for(lambda:script('return window.__ccrNativeCspEvents.length > 0'), 'production CSP rejection', 5)
    record['checks']['production_csp'] = script('return {inlineExecuted:window.__ccrNativeCspProbe,events:window.__ccrNativeCspEvents,meta:[...document.querySelectorAll("meta[http-equiv]")].map(e=>({name:e.httpEquiv,content:e.content}))}')
    assert record['checks']['production_csp']['inlineExecuted'] is False
    assert any(e['directive'].startswith('script-src') for e in record['checks']['production_csp']['events'])
    replacement = dict(original, model='haiku')
    content = json.dumps(replacement, indent=2)
    editor = element('.cm-content')
    request('POST',f'/session/{session}/element/{editor}/click',{})
    keys = chr(0xE009) + 'a' + chr(0xE000) + content
    request('POST',f'/session/{session}/element/{editor}/value',{'text':keys,'value':list(keys)})
    wait_for(lambda:script('return document.querySelector(".config-source-panel__button--primary")?.disabled === false'), 'dirty editor save')
    save_button = element('.config-source-panel__button--primary')
    request('POST',f'/session/{session}/element/{save_button}/click',{})
    wait_for(lambda:json.loads(settings.read_text()).get('model') == 'haiku', 'native settings persistence')
    actual = json.loads(settings.read_text())
    assert actual == replacement, actual
    record['checks']['native_save_roundtrip'] = {'expected':replacement,'actual':actual,'path':str(settings),'sha256':hashlib.sha256(settings.read_bytes()).hexdigest()}
    wait_for(lambda:script('return !document.querySelector(".cm-editor") && [...document.querySelectorAll("button")].some(b => !b.disabled && b.textContent.trim() === "源文件")'), 'saved settings view')
    record['styles_after_save'] = script('return window.__ccrNativeStyleCspEvents')
    assert record['styles_after_save'] == [], record['styles_after_save']
    record['state'] = 'passed'
except Exception as error:
    record['state'] = 'failed'
    record['error'] = str(error)
    record['traceback'] = traceback.format_exc()
    if session:
        try:
            record['failure_dom'] = script('return {url:location.href,body:document.body.innerText.slice(0,6000)}')
        except Exception as diagnostic_error:
            record['diagnostic_error'] = str(diagnostic_error)
finally:
    if session:
        try:
            request('DELETE',f'/session/{session}',timeout=8)
        except Exception as close_error:
            record['session_close_error'] = str(close_error)
            record['state'] = 'failed_cleanup'
    if process:
        try:
            os.killpg(process.pid,signal.SIGTERM)
            process.wait(timeout=8)
        except ProcessLookupError:
            pass
        except subprocess.TimeoutExpired:
            os.killpg(process.pid,signal.SIGKILL)
            process.wait(timeout=5)
        record['driver_exit_code'] = process.returncode
        def live_group_members():
            members = []
            for stat_path in Path('/proc').glob('[0-9]*/stat'):
                try:
                    fields = stat_path.read_text().rsplit(')',1)[1].split()
                    if int(fields[2]) == process.pid and fields[0] != 'Z':
                        members.append(int(stat_path.parent.name))
                except (OSError, ValueError, IndexError):
                    continue
            return members
        deadline = time.monotonic() + 5
        while live_group_members() and time.monotonic() < deadline:
            time.sleep(0.1)
        record['live_process_group_members_after_cleanup'] = live_group_members()
        if record['live_process_group_members_after_cleanup']:
            os.killpg(process.pid, signal.SIGKILL)
            record['state'] = 'failed_cleanup'
    record['fixture_files'] = [{'path':p.relative_to(fixture).as_posix(),'bytes':p.stat().st_size} for p in fixture.rglob('*') if p.is_file()]
    record['finished_at_utc'] = datetime.now(timezone.utc).isoformat()
    save()
print(json.dumps({'state':record['state'],'checks':list(record['checks']),'error':record.get('error'),'evidence':str(OUTPUT)}), flush=True)
raise SystemExit(0 if record['state'] == 'passed' else 1)
