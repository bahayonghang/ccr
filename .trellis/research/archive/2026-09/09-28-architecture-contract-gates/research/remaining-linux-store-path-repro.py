import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time

root = Path('/mnt/d/Documents/Code/Github/ccr')
research = root / '.trellis/tasks/09-28-architecture-contract-gates/research'
source = 'crates/ccr-store/src/sessions/providers.rs'
prior = json.loads((research / 'remaining-linux-workspace-quality-after-octal.json').read_text())
name = 'remaining-linux-store-path-repro'
log = research / (name + '.log')
receipt = research / (name + '.json')
assert not receipt.exists() and not log.exists()
env = os.environ.copy()
env.update(prior['environment_overrides'])
paths = ['Cargo.toml', 'Cargo.lock', 'crates/ccr-store/Cargo.toml', source]
def hashes():
    return {path: hashlib.sha256((root / path).read_bytes()).hexdigest() for path in paths}
command = ['cargo', 'test', '--locked', '-p', 'ccr-store', '--all-features', '--lib', 'sessions::providers::tests::restores_codex_live_jsonl_under_tempfile_with_windows_path_shapes', '--', '--exact', '--nocapture']
original_binary = Path('/tmp/ccr-architecture-linux-01a0e781/debug/deps/ccr_store-3139bc386fa2fa49')
record = {
    'command': command,
    'cwd': str(root),
    'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'platform': platform.platform(),
    'git_head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
    'git_status_target_before': subprocess.check_output(['git', 'status', '--porcelain=v1', '--', source], cwd=root, text=True),
    'environment_overrides': prior['environment_overrides'],
    'sources_before': hashes(),
    'historical_receipt': 'remaining-linux-workspace-quality-after-octal.json',
    'historical_receipt_sha256': hashlib.sha256((research / 'remaining-linux-workspace-quality-after-octal.json').read_bytes()).hexdigest(),
    'historical_freeze_includes_target': source in prior['sources_before'],
    'historical_identity_boundary': 'The prior 268-path receipt does not bind providers.rs. This new receipt binds only the new exact-test run and does not amend prior evidence.',
    'original_quality_test_binary_path': str(original_binary),
    'original_quality_test_binary_sha256_before': hashlib.sha256(original_binary.read_bytes()).hexdigest(),
    'runner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
}
start = time.monotonic()
with log.open('xb') as output:
    completed = subprocess.run(command, cwd=root, env=env, stdout=output, stderr=subprocess.STDOUT, timeout=600)
record.update({'exit_code': completed.returncode, 'seconds': round(time.monotonic()-start, 3), 'finished_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'sources_after': hashes(), 'git_status_target_after': subprocess.check_output(['git', 'status', '--porcelain=v1', '--', source], cwd=root, text=True), 'log': log.name, 'log_sha256': hashlib.sha256(log.read_bytes()).hexdigest()})
record['source_changes'] = [path for path in paths if record['sources_before'][path] != record['sources_after'][path]]
with receipt.open('x', encoding='utf-8') as output:
    json.dump(record, output, ensure_ascii=False, indent=2)
print(json.dumps(record, ensure_ascii=False, indent=2))
print(log.read_text(errors='replace'))
