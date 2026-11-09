"""Record focused permission-repair checks without running exporters."""
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
COMMANDS = {
    'linux-config': ['cargo', 'test', '-p', 'ccr-config', '--all-features', '--lib'],
    'linux-core-guarded': ['cargo', 'test', '-p', 'ccr-core', '--all-features', 'core::guarded_write::tests', '--', '--test-threads=1'],
    'linux-core-atomic': ['cargo', 'test', '-p', 'ccr-core', '--all-features', 'core::atomic_writer::tests', '--', '--test-threads=1'],
    'linux-clippy': ['cargo', 'clippy', '-p', 'ccr-core', '-p', 'ccr-config', '--all-targets', '--all-features', '--', '-D', 'warnings'],
    'linux-check': ['cargo', 'check', '-p', 'ccr-core', '-p', 'ccr-config', '--all-targets', '--all-features'],
}
label = sys.argv[1]
suffix = '-' + sys.argv[2] if len(sys.argv) > 2 else ''
prefix = HERE / ('remaining-permissions-' + label + suffix)
metadata = prefix.with_suffix('.json')
log = prefix.with_suffix('.log')
if metadata.exists() or log.exists():
    raise SystemExit('Preserve prior check evidence')

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None

def sources():
    paths = json.loads((HERE / 'remaining-final-source-freeze.json').read_text())['files']
    return {name: sha(ROOT / name) for name in sorted(paths)}

def generated():
    folder = ROOT / 'ccr-ui/src/types/generated'
    return {p.relative_to(folder).as_posix(): sha(p) for p in sorted(folder.rglob('*')) if p.is_file()}

env = dict(os.environ)
env.update({'PATH': '/tmp/ccr-architecture-tools-01a0e781/bin:/home/lyh/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin', 'RUSTUP_TOOLCHAIN': '1.98.0', 'CARGO_TARGET_DIR': '/tmp/ccr-architecture-linux-01a0e781', 'CCR_SKIP_ICON_GENERATION': '1'})
command = COMMANDS[label]
record = {'command': command, 'cwd': str(ROOT), 'state': 'running', 'started_at_utc': datetime.now(timezone.utc).isoformat(), 'sources_before': sources(), 'generated_before': generated(), 'runner_sha256': sha(Path(__file__)), 'log': log.name, 'scope': 'Focused reviewer-authored permission repair validation; post-fix sources do not equal the earlier final freeze.', 'environment_overrides': {k: env[k] for k in ['PATH', 'RUSTUP_TOOLCHAIN', 'CARGO_TARGET_DIR', 'CCR_SKIP_ICON_GENERATION']}}
metadata.write_text(json.dumps(record, indent=2) + chr(10))
started = time.monotonic()
with log.open('wb') as stream:
    result = subprocess.run(command, cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
record.update({'state': 'finished', 'exit_code': result.returncode, 'seconds': round(time.monotonic() - started, 3), 'finished_at_utc': datetime.now(timezone.utc).isoformat(), 'sources_after': sources(), 'generated_after': generated(), 'log_sha256': sha(log)})
record['source_changes'] = [n for n, v in record['sources_before'].items() if record['sources_after'][n] != v]
record['generated_changes'] = [n for n in sorted(set(record['generated_before']) | set(record['generated_after'])) if record['generated_before'].get(n) != record['generated_after'].get(n)]
metadata.write_text(json.dumps(record, indent=2) + chr(10))
print(json.dumps({k: record[k] for k in ['command', 'exit_code', 'seconds', 'source_changes', 'generated_changes']}), flush=True)
raise SystemExit(result.returncode)
