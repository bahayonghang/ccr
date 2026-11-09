import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
TASK = Path(__file__).resolve().parents[1]
TOOLBIN = Path('/home/lyh/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin')
ENV = os.environ.copy()
ENV.update(RUSTC=str(TOOLBIN / 'rustc'), RUSTDOC=str(TOOLBIN / 'rustdoc'),
           RUSTUP_TOOLCHAIN='stable', CARGO_TARGET_DIR='/tmp/ccr-architecture-linux-01a0e781')
ENV['PATH'] = str(TOOLBIN) + ':' + ENV.get('PATH', '')
for name in ('RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'):
    ENV.pop(name, None)
FILES = ('Cargo.lock', 'crates/ccr-core/src/core/atomic_writer.rs',
         'crates/ccr-core/src/core/guarded_write.rs', 'crates/ccr-core/src/core/write_journal.rs',
         'crates/ccr-config/src/managers/config/repository.rs')

def fingerprints():
    return {name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest() for name in FILES}

results = []
label = sys.argv[sys.argv.index('--label') + 1] if '--label' in sys.argv else ''
if label and not re.fullmatch(r'[A-Za-z0-9_-]+', label):
    raise SystemExit('Invalid evidence label')
runs = [('write_journal', ['--features', 'test-support'], 'write-journal-faults')] if '--faults' in sys.argv else [
    ('write_journal', [], 'write-journal'), ('guarded_write', [], 'guarded-write')]
if '--guarded-only' in sys.argv:
    runs = [('guarded_write', [], 'guarded-write')]
if '--post-alias' in sys.argv:
    runs = [('write_journal', ['--features', 'test-support'], 'write-journal-faults'),
            ('guarded_write', [], 'guarded-write'),
            ('atomic_writer', [], 'atomic-writer'),
            ('repository', [], 'repository')]
for test_filter, extra, suffix in runs:
    package = 'ccr-config' if test_filter == 'repository' else 'ccr-core'
    command = [str(TOOLBIN / 'cargo'), 'test', '--offline', '--locked', '-p', package,
               *extra, test_filter, '--', '--test-threads=1']
    before = fingerprints()
    started = time.monotonic()
    result = subprocess.run(command, cwd=ROOT, env=ENV, capture_output=True, text=True, timeout=600)
    output = result.stdout + result.stderr
    after = fingerprints()
    record = {'command': command, 'exit_code': result.returncode,
              'elapsed_seconds': round(time.monotonic() - started, 2),
              'fingerprints_before': before, 'fingerprints_after': after,
              'source_unchanged': before == after,
              'summaries': re.findall(r'test result: .*', output),
              'toolchain': str(TOOLBIN), 'output': output}
    stem = 'root-linux-' + suffix + ('-' + label if label else '')
    (TASK / (stem + '.log')).write_text(output, encoding='utf-8')
    (TASK / (stem + '.json')).write_text(json.dumps(record, indent=2) + chr(10), encoding='utf-8')
    summary = {key: value for key, value in record.items() if key != 'output'}
    print(json.dumps(summary), flush=True)
    results.append(record)
raise SystemExit(0 if all(r['exit_code'] == 0 and r['source_unchanged'] for r in results) else 1)
