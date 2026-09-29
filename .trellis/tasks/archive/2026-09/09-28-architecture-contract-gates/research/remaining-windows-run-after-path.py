"""Capture one unmodified repository gate and its input/output evidence."""
from __future__ import annotations

import base64
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import threading
import time

sys.stdout.reconfigure(encoding='utf-8')
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
FREEZE = HERE / 'continuation-final-source-freeze.json'
SCRIPTS = ['ccr-ui/.tmp-desktop-probe.mjs', 'ccr-ui/.tmp-insights-visual.mjs']


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None


def sources():
    frozen = json.loads(FREEZE.read_text(encoding='utf-8'))['files']
    return {p: sha(ROOT / p) for p in sorted(set(frozen) | set(SCRIPTS))}


def generated():
    root = ROOT / 'ccr-ui/src/types/generated'
    return {p.relative_to(root).as_posix(): p.read_bytes() for p in sorted(root.rglob('*')) if p.is_file()}


def exporter_binaries():
    rows = {}
    for folder, pattern in [('target/debug/deps', 'ccr_cli-*.exe'), ('target/debug/deps', 'ccr_usage-*.exe'), ('ccr-ui/src-tauri/target/debug/deps', 'ccr_desktop-*.exe')]:
        for path in sorted((ROOT / folder).glob(pattern)):
            rows[path.relative_to(ROOT).as_posix()] = {'resolved_path': str(path.resolve()), 'sha256': sha(path), 'bytes': path.stat().st_size}
    return rows


def changed(before, after):
    return [{'path': p, 'before': before.get(p), 'after': after.get(p)} for p in sorted(set(before) | set(after)) if before.get(p) != after.get(p)]


if len(sys.argv) != 2 or sys.argv[1] not in ('frontend-check', 'ci', 'ci-after-doctor', 'ci-after-path', 'frontend-final'):
    raise SystemExit('Usage: remaining-windows-run-gate.py frontend-check|ci|ci-after-doctor|ci-after-path|frontend-final')
label = sys.argv[1]
gate = {'ci-after-doctor': 'ci', 'ci-after-path': 'ci', 'frontend-final': 'frontend-check'}.get(label, label)
if label == 'ci-after-doctor':
    FREEZE = HERE / 'remaining-source-freeze.json'
if label in ('ci-after-path', 'frontend-final'):
    FREEZE = HERE / 'remaining-platform-tests-source-freeze.json'
prefix = HERE / ('remaining-windows-' + label)
metadata = prefix.with_suffix('.json')
paths = {name: Path(str(prefix) + suffix) for name, suffix in [('stdout', '.stdout.log'), ('stderr', '.stderr.log'), ('combined', '.log')]}
if metadata.exists() or any(path.exists() for path in paths.values()):
    raise SystemExit('Preserve existing evidence; this gate already has a capture.')
before = sources()
before_generated = generated()
snapshot = Path(str(prefix) + '.generated-before.json')
snapshot.write_text(json.dumps({p: {'sha256': hashlib.sha256(b).hexdigest(), 'bytes_base64': base64.b64encode(b).decode('ascii')} for p, b in before_generated.items()}, indent=2) + '\n', encoding='utf-8')
before_binaries = exporter_binaries() if gate == 'ci' else {}
argv = [shutil.which('just') or 'just', gate]
record = {
    'command': ['just', gate], 'argv': argv, 'cwd': str(ROOT),
    'started_at_utc': datetime.now(timezone.utc).isoformat(),
    'head_before': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'environment_overrides': {'CCR_SKIP_ICON_GENERATION': '1'},
    'source_freeze_reference': FREEZE.name, 'source_freeze_sha256': sha(FREEZE),
    'sources_before': before,
    'generated_before_snapshot': snapshot.name, 'generated_before_count': len(before_generated),
    'exporter_binaries_before': before_binaries,
    'logs': {name: path.name for name, path in paths.items()},
    'combined_log_order': 'Raw chunks in capture acquisition order; separate stdout and stderr logs preserve each stream byte-for-byte.',
    'state': 'running',
}
metadata.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'started': gate, 'metadata': str(metadata)}), flush=True)
started = time.monotonic()
lock = threading.Lock()
with paths['stdout'].open('wb') as out, paths['stderr'].open('wb') as err, paths['combined'].open('wb') as combined:
    proc = subprocess.Popen(argv, cwd=ROOT, env=dict(os.environ, CCR_SKIP_ICON_GENERATION='1'), stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    def copy(pipe, destination):
        while block := pipe.read1(16384):
            destination.write(block)
            destination.flush()
            with lock:
                combined.write(block)
                combined.flush()

    threads = [threading.Thread(target=copy, args=(proc.stdout, out)), threading.Thread(target=copy, args=(proc.stderr, err))]
    for thread in threads:
        thread.start()
    exit_code = proc.wait()
    for thread in threads:
        thread.join()
after = sources()
after_generated = generated()
before_hashes = {p: hashlib.sha256(b).hexdigest() for p, b in before_generated.items()}
after_hashes = {p: hashlib.sha256(b).hexdigest() for p, b in after_generated.items()}
record.update({
    'state': 'finished', 'finished_at_utc': datetime.now(timezone.utc).isoformat(),
    'exit_code': exit_code, 'seconds': round(time.monotonic() - started, 3),
    'head_after': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'sources_after': after, 'source_changes': changed(before, after),
    'generated_after_count': len(after_generated), 'generated_changes': changed(before_hashes, after_hashes),
    'exporter_binaries_after': exporter_binaries() if gate == 'ci' else {},
    'log_sha256': {name: sha(path) for name, path in paths.items()},
})
metadata.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps({k: record[k] for k in ['state', 'exit_code', 'seconds', 'source_changes', 'generated_after_count', 'generated_changes']}), flush=True)
print('\n'.join(paths['stdout'].read_text(encoding='utf-8', errors='replace').splitlines()[-18:]), flush=True)
print('\n'.join(paths['stderr'].read_text(encoding='utf-8', errors='replace').splitlines()[-10:]), flush=True)
raise SystemExit(exit_code)
