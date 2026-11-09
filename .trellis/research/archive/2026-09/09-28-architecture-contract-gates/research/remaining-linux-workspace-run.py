"""Capture the existing Linux root workflow recipes and exact source inputs."""
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
FREEZE = HERE / 'remaining-final-source-freeze.json'
COMMANDS = {
    'workspace-quality-final': ['just', 'version-check', 'workflow-governance-check', 'fmt-check', 'lint-strict', 'test', 'check-workspace'],
    'workspace-coverage-final': ['just', 'coverage-rust'],
}
label = sys.argv[1]
command = COMMANDS[label]
prefix = HERE / ('remaining-linux-' + label)
metadata = prefix.with_suffix('.json')
log_path = prefix.with_suffix('.log')
if metadata.exists() or log_path.exists():
    raise SystemExit('Preserve existing evidence; choose a new explicit run label')

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None

def source_map():
    return {name: sha(ROOT / name) for name in sorted(json.loads(FREEZE.read_text())['files'])}

def generated_map():
    folder = ROOT / 'ccr-ui/src/types/generated'
    return {p.relative_to(folder).as_posix(): sha(p) for p in sorted(folder.rglob('*')) if p.is_file()}

env = dict(os.environ)
env.update({'PATH': '/tmp/ccr-architecture-tools-01a0e781/bin:/home/lyh/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin', 'RUSTUP_TOOLCHAIN': '1.98.0', 'CARGO_TARGET_DIR': '/tmp/ccr-architecture-linux-01a0e781', 'CCR_SKIP_ICON_GENERATION': '1'})
env.pop('TAURI_CONFIG', None)
record = {
    'command': command, 'cwd': str(ROOT), 'state': 'running',
    'started_at_utc': datetime.now(timezone.utc).isoformat(),
    'git_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'source_freeze_reference': FREEZE.name, 'source_freeze_sha256': sha(FREEZE),
    'parent_source_freeze': json.loads(FREEZE.read_text())['parent_source_freeze'],
    'parent_source_freeze_sha256': json.loads(FREEZE.read_text())['parent_source_freeze_sha256'],
    'sources_before': source_map(), 'generated_before': generated_map(),
    'environment_overrides': {key: env[key] for key in ['PATH', 'RUSTUP_TOOLCHAIN', 'CARGO_TARGET_DIR', 'CCR_SKIP_ICON_GENERATION']},
    'rustc': subprocess.check_output(['rustc', '--version'], env=env, text=True).strip(),
    'workflow_path': '.github/workflows/ci.yml', 'workflow_sha256': sha(ROOT / '.github/workflows/ci.yml'),
    'runner_sha256': sha(Path(__file__)), 'log': log_path.name,
    'scope': 'Local WSL Ubuntu 24.04 execution of existing root workflow recipes; not a hosted GitHub required status',
}
assert record['sources_before'] == json.loads(FREEZE.read_text())['files']
metadata.write_text(json.dumps(record, indent=2) + chr(10))
print(json.dumps({'state': 'running', 'command': command, 'metadata': str(metadata)}), flush=True)
start = time.monotonic()
with log_path.open('wb') as log:
    result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
record.update({'state': 'finished', 'exit_code': result.returncode, 'seconds': round(time.monotonic() - start, 3), 'finished_at_utc': datetime.now(timezone.utc).isoformat(), 'sources_after': source_map(), 'generated_after': generated_map(), 'log_sha256': sha(log_path)})
record['source_changes'] = [name for name, digest in record['sources_before'].items() if record['sources_after'].get(name) != digest]
record['generated_changes'] = [name for name in sorted(set(record['generated_before']) | set(record['generated_after'])) if record['generated_before'].get(name) != record['generated_after'].get(name)]
if label == 'workspace-coverage-final':
    report = ROOT / 'target/coverage-workspace.json'
    record['coverage_report_sha256'] = sha(report)
    if report.is_file():
        copy = HERE / 'remaining-linux-coverage-workspace-report.json'
        if copy.exists():
            raise SystemExit('Preserve existing coverage evidence')
        copy.write_bytes(report.read_bytes())
        record['coverage_report_copy'] = copy.name
metadata.write_text(json.dumps(record, indent=2) + chr(10))
print(json.dumps({key: record[key] for key in ['state', 'exit_code', 'seconds', 'source_changes', 'generated_changes']}), flush=True)
raise SystemExit(result.returncode)
