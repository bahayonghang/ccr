"""Keep local generated-extension and missing-extension contract receipts separate."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import time

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
PREFIX = 'resume-2026-10-04-omp-adjustment-'
SOURCE = ROOT / 'scripts/trellis/omp-context.test.ts'
EXTENSION = ROOT / '.omp/extensions/trellis/index.ts'
PATHS = [SOURCE, EXTENSION, ROOT / 'docs/agents/harnesses.md', ROOT / 'docs/en/agents/harnesses.md']


def hashes():
    return {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else None for path in PATHS}


def run(label, argv, cwd):
    before = hashes()
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    timer = time.monotonic()
    streams = {}
    with (OUT / (PREFIX + label + '.stdout.log')).open('xb') as stdout:
        with (OUT / (PREFIX + label + '.stderr.log')).open('xb') as stderr:
            process = subprocess.run([shutil.which(argv[0]), *argv[1:]], cwd=cwd, stdout=stdout, stderr=stderr)
    combined = ''
    for channel in ['stdout', 'stderr']:
        path = OUT / (PREFIX + label + '.' + channel + '.log')
        raw = path.read_bytes()
        streams[channel] = {'path': path.name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
        combined += raw.decode('utf-8', errors='replace')
    after = hashes()
    assert before == after
    if label == 'local':
        assert process.returncode == 0 and '14 pass' in combined and '99 expect() calls' in combined
        assert 'SKIPPED_UNVERIFIED' not in combined
    elif label == 'absent':
        assert process.returncode == 0 and '14 skip' in combined and '0 pass' in combined
        assert 'SKIPPED_UNVERIFIED' in combined and 'All 14 contract tests remain unexecuted.' in combined
    else:
        assert process.returncode == 0
    result = {'started_utc': started, 'finished_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
              'elapsed_seconds': round(time.monotonic() - timer, 3), 'argv': argv, 'cwd': str(cwd),
              'os': platform.platform(), 'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
              'bun_version': subprocess.check_output([shutil.which('bun'), '--version'], cwd=ROOT).decode().strip(),
              'input_sha256_before': before, 'input_sha256_after': after, 'exit_code': process.returncode,
              'streams': streams, 'summary': [line for line in combined.splitlines() if any(marker in line for marker in
                         [' pass', ' skip', ' fail', 'expect() calls', 'SKIPPED_UNVERIFIED'])],
              'native_loading_trust': 'UNVERIFIED',
              'boundary': 'local generated source exercised' if label == 'local' else 'source-only missing-extension fixture' if label == 'absent' else 'scoped whitespace check'}
    with (OUT / (PREFIX + label + '.json')).open('x', encoding='utf-8') as stream:
        json.dump(result, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'label': label, 'exit_code': process.returncode, 'summary': result['summary']}))


assert EXTENSION.exists(), 'local generated extension missing; do not install automatically'
run('local', ['bun', 'test', 'scripts/trellis/omp-context.test.ts'], ROOT)
with tempfile.TemporaryDirectory(prefix='ccr-omp-contract-source-only-') as temporary:
    fixture_root = Path(temporary).resolve()
    assert fixture_root.is_relative_to(Path(tempfile.gettempdir()).resolve())
    fixture_source = fixture_root / 'scripts/trellis/omp-context.test.ts'
    fixture_source.parent.mkdir(parents=True)
    fixture_source.write_bytes(SOURCE.read_bytes())
    assert not (fixture_root / '.omp').exists()
    assert hashlib.sha256(fixture_source.read_bytes()).hexdigest() == hashlib.sha256(SOURCE.read_bytes()).hexdigest()
    run('absent', ['bun', 'test', 'scripts/trellis/omp-context.test.ts'], fixture_root)
run('diff-check', ['git', 'diff', '--check', '--', 'scripts/trellis/omp-context.test.ts', 'docs/agents/harnesses.md', 'docs/en/agents/harnesses.md'], ROOT)
