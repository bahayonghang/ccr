"""Record a required Linux gate without changing its recipe."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
FREEZE = HERE / 'continuation-platform-final-source-freeze.json'
TOOLS = '/tmp/ccr-architecture-tools-01a0e781/bin'
COMMANDS = {
    'tauri-ci': ['just', 'tauri-ci'],
    'coverage-tauri': ['just', 'coverage-tauri'],
    'native-build': ['cargo', 'build', '--manifest-path', 'ccr-ui/src-tauri/Cargo.toml', '--bin', 'ccr-desktop', '--features', 'custom-protocol'],
}
COMMANDS['native-after-permissions'] = COMMANDS['native-build']
COMMANDS['tauri-ci-after-permissions'] = COMMANDS['tauri-ci']
COMMANDS['coverage-tauri-after-permissions'] = COMMANDS['coverage-tauri']
name = sys.argv[1]
if name in ('native-after-permissions', 'tauri-ci-after-permissions', 'coverage-tauri-after-permissions'):
    FREEZE = HERE / 'remaining-permissions-source-freeze.json'
command = COMMANDS[name]
prefix = HERE / ('remaining-linux-' + name)
metadata = prefix.with_suffix('.json')
log_path = prefix.with_suffix('.log')
if metadata.exists() or log_path.exists():
    raise SystemExit('Existing evidence must not be overwritten')

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None

def sources():
    names = set(json.loads(FREEZE.read_text())['files'])
    names.update(['ccr-ui/.tmp-desktop-probe.mjs', 'ccr-ui/.tmp-insights-visual.mjs'])
    return {name: sha(ROOT / name) for name in sorted(names)}

def generated():
    folder = ROOT / 'ccr-ui/src/types/generated'
    return {p.relative_to(folder).as_posix(): sha(p) for p in sorted(folder.rglob('*')) if p.is_file()}

env = dict(os.environ)
env.update({'PATH': TOOLS + ':/home/lyh/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin', 'RUSTUP_TOOLCHAIN': '1.98.0', 'CARGO_TARGET_DIR': '/tmp/ccr-architecture-linux-01a0e781', 'CCR_SKIP_ICON_GENERATION': '1'})
env.pop('TAURI_CONFIG', None)
record = {'command': command, 'cwd': str(ROOT), 'started_at_utc': datetime.now(timezone.utc).isoformat(), 'state': 'running', 'git_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'parent_source_freeze': FREEZE.name, 'parent_source_freeze_sha256': sha(FREEZE), 'sources_before': sources(), 'generated_before': generated(), 'environment_overrides': {key: env[key] for key in ['PATH', 'RUSTUP_TOOLCHAIN', 'CARGO_TARGET_DIR', 'CCR_SKIP_ICON_GENERATION']}, 'rustc': subprocess.check_output(['rustc', '--version'], env=env, text=True).strip(), 'log': log_path.name}
if name in ('native-after-permissions', 'tauri-ci-after-permissions', 'coverage-tauri-after-permissions'):
    record['source_freeze_reference'] = FREEZE.name
    record['source_freeze_sha256'] = sha(FREEZE)
    lineage = json.loads(FREEZE.read_text())
    record['parent_source_freeze'] = lineage['parent_source_freeze']
    record['parent_source_freeze_sha256'] = lineage['parent_source_freeze_sha256']
if name in ('native-build', 'native-after-permissions'):
    folder = ROOT / 'ccr-ui/dist'
    record['frontend_dist'] = {p.relative_to(folder).as_posix(): sha(p) for p in sorted(folder.rglob('*')) if p.is_file()}
    if not record['frontend_dist']:
        raise SystemExit('Native build requires the actual frontend dist')
    record['csp_config_sha256'] = sha(ROOT / 'ccr-ui/src-tauri/tauri.conf.json')
metadata.write_text(json.dumps(record, indent=2) + chr(10))
print(json.dumps({'state':'running', 'command':command,'metadata':str(metadata)}), flush=True)
start = time.monotonic()
with log_path.open('wb') as log:
    result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
record.update({'state':'finished','exit_code':result.returncode,'finished_at_utc':datetime.now(timezone.utc).isoformat(),'seconds':round(time.monotonic()-start,3),'sources_after':sources(),'generated_after':generated(),'log_sha256':sha(log_path)})
record['source_changes'] = [p for p, h in record['sources_before'].items() if h != record['sources_after'].get(p)]
record['generated_changes'] = [p for p in sorted(set(record['generated_before']) | set(record['generated_after'])) if record['generated_before'].get(p) != record['generated_after'].get(p)]
if name in ('coverage-tauri', 'coverage-tauri-after-permissions'):
    record['coverage_report_sha256'] = sha(ROOT / 'ccr-ui/src-tauri/target/coverage-tauri.json')
if name in ('native-build', 'native-after-permissions'):
    record['binary'] = env['CARGO_TARGET_DIR'] + '/debug/ccr-desktop'
    record['binary_sha256'] = sha(Path(record['binary']))
    folder = ROOT / 'ccr-ui/dist'
    record['frontend_dist_after'] = {p.relative_to(folder).as_posix(): sha(p) for p in sorted(folder.rglob('*')) if p.is_file()}
    record['frontend_dist_changed'] = record['frontend_dist'] != record['frontend_dist_after']
metadata.write_text(json.dumps(record, indent=2) + chr(10))
print(json.dumps({key:record[key] for key in ['state','exit_code','seconds','source_changes','generated_changes']}), flush=True)
raise SystemExit(result.returncode)
