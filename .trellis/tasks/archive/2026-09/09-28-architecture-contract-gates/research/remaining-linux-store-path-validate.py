"""Capture only the approved ccr-store path-test validation commands."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

sys.stdout.reconfigure(encoding='utf-8')
research = Path(__file__).resolve().parent
root = research.parents[3]
label = sys.argv[1]
name = 'remaining-store-path-' + label
metadata = research / (name + '.json')
log = research / (name + '.log')
assert not metadata.exists() and not log.exists(), 'Use a new label; preserve earlier evidence'
exact = 'sessions::providers::tests::restores_codex_live_jsonl_under_tempfile_with_windows_path_shapes'
commands = {
    'linux-exact': ['cargo', 'test', '--locked', '-p', 'ccr-store', '--all-features', '--lib', exact, '--', '--exact', '--nocapture'],
    'linux-lib': ['cargo', 'test', '--locked', '-p', 'ccr-store', '--all-features', '--lib'],
    'linux-clippy': ['cargo', 'clippy', '--locked', '-p', 'ccr-store', '--all-targets', '--all-features', '--', '-D', 'warnings'],
    'windows-exact': ['cargo', 'test', '--locked', '-p', 'ccr-store', '--all-features', '--lib', exact, '--', '--exact', '--nocapture'],
    'format': ['rustfmt', '--edition', '2024', '--check', 'crates/ccr-store/src/sessions/providers.rs'],
    'diff-check': ['git', 'diff', '--check', '--', 'crates/ccr-store/src/sessions/providers.rs'],
}
command = commands[label]
source = 'crates/ccr-store/src/sessions/providers.rs'
paths = ['Cargo.toml', 'Cargo.lock', 'crates/ccr-store/Cargo.toml', source]
def hashes():
    return {path: hashlib.sha256((root / path).read_bytes()).hexdigest() for path in paths}
env = os.environ.copy()
overrides = {}
if label.startswith('linux-'):
    assert sys.platform.startswith('linux')
    overrides = json.loads((research / 'remaining-linux-workspace-quality-after-octal.json').read_text())['environment_overrides']
else:
    assert sys.platform == 'win32'
env.update(overrides)
record = {
    'state': 'running', 'command': command, 'cwd': str(root), 'platform': platform.platform(),
    'started_at_utc': datetime.now(timezone.utc).isoformat(),
    'git_head': subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),
    'sources_before': hashes(), 'environment_overrides': overrides,
    'rustc': subprocess.check_output(['rustc','--version'],cwd=root,env=env,text=True).strip(),
    'runner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
    'scope': 'One approved test, ccr-store lib/strict Clippy, or scoped static check; not an aggregate gate',
    'log': log.name,
}
assert record['sources_before'][source] == '93d816299b3d24abba4cbe02597d0cd91d5d258a865ea84a3fe79267c39c22c0'
with metadata.open('x',encoding='utf-8') as output:
    json.dump(record,output,ensure_ascii=False,indent=2)
print(json.dumps({'label':label,'state':'running','command':command}),flush=True)
started = time.monotonic()
with log.open('xb') as output:
    result = subprocess.run(command,cwd=root,env=env,stdout=output,stderr=subprocess.STDOUT,timeout=900)
record.update({'state':'finished','exit_code':result.returncode,'seconds':round(time.monotonic()-started,3),'finished_at_utc':datetime.now(timezone.utc).isoformat(),'sources_after':hashes(),'log_sha256':hashlib.sha256(log.read_bytes()).hexdigest()})
record['source_changes']=[p for p in paths if record['sources_before'][p]!=record['sources_after'][p]]
metadata.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:record[k] for k in ['state','exit_code','seconds','source_changes','log_sha256']},ensure_ascii=False),flush=True)
print('\n'.join(log.read_text(encoding='utf-8',errors='replace').splitlines()[-16:]),flush=True)
raise SystemExit(result.returncode)
