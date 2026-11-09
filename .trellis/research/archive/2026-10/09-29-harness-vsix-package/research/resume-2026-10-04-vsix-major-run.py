"""Approved actual VSCE 4 migration and fixed Node receipts."""
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
PREFIX = 'resume-2026-10-04-vsix-major-'
fixed = json.loads((OUT / 'resume-2026-10-04-vsix-node24-receipt.json').read_bytes())
NODE = Path(fixed['temporary_executable'])
assert hashlib.sha256(NODE.read_bytes()).hexdigest() == fixed['binary_sha256']
CHILD_ENV = os.environ.copy()
CHILD_ENV['PATH'] = str(NODE.parent) + os.pathsep + CHILD_ENV['PATH']
assert subprocess.check_output([str(NODE), '--version']).strip() == b'v24.20.0'


def inputs():
    files = subprocess.check_output(['git', 'ls-files', 'ccr-vscode'], cwd=ROOT).decode().splitlines()
    files += ['justfile']
    return {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in files if (ROOT / path).is_file()}


def save(name, data):
    with (OUT / (PREFIX + name + '.json')).open('x', encoding='utf-8') as stream:
        json.dump(data, stream, indent=2)
        stream.write('\n')


def run(label, argv, cwd):
    before = inputs()
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    timer = time.monotonic()
    argv = [shutil.which(argv[0], path=CHILD_ENV['PATH']), *argv[1:]]
    with (OUT / (PREFIX + label + '.stdout.log')).open('xb') as stdout:
        with (OUT / (PREFIX + label + '.stderr.log')).open('xb') as stderr:
            process = subprocess.run(argv, cwd=cwd, env=CHILD_ENV, stdout=stdout, stderr=stderr)
    streams = {}
    combined = ''
    for channel in ['stdout', 'stderr']:
        path = OUT / (PREFIX + label + '.' + channel + '.log')
        raw = path.read_bytes()
        streams[channel] = {'path': path.name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
        combined += raw.decode('utf-8', errors='replace')
    after = inputs()
    summary = [line for line in combined.splitlines() if any(marker in line for marker in
               ['tests ', 'pass ', 'fail ', 'all files', 'file check passed', 'vulnerabilities', 'Packaged:'])]
    result = {'started_utc': started, 'finished_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
              'elapsed_seconds': round(time.monotonic() - timer, 3), 'os': platform.platform(),
              'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
              'argv': argv, 'cwd': str(cwd), 'exit_code': process.returncode, 'streams': streams,
              'input_sha256_before': before, 'input_sha256_after': after,
              'actual_node_executable': str(NODE), 'actual_node_version': 'v24.20.0',
              'npm_version': subprocess.check_output([shutil.which('npm', path=CHILD_ENV['PATH']), '--version'], env=CHILD_ENV).decode().strip(),
              'summary': summary, 'thresholds': {'lines': 70, 'functions': 70},
              'boundary': 'local fixed Node actual VSCE migration; default test parallelism retained'}
    if label == 'audit':
        audit = json.loads((OUT / (PREFIX + label + '.stdout.log')).read_bytes())
        result['vulnerability_counts'] = audit.get('metadata', {}).get('vulnerabilities')
    save(label, result)
    print(json.dumps({'label': label, 'exit_code': process.returncode, 'summary': summary}))
    assert before == after, 'Validation changed extension source bytes'
    return process.returncode


mode = sys.argv[1]
if mode == 'apply':
    before = inputs()
    prepare = json.loads((OUT / 'resume-2026-10-04-vsix-final-verification.json').read_bytes())
    assert before['ccr-vscode/package-lock.json'] == prepare['actual_lock_sha256']
    assert before['ccr-vscode/package.json'] == prepare['unchanged_extension_source_sha256']['ccr-vscode/package.json']
    candidate = OUT / 'resume-2026-10-04-vsix-candidate'
    metadata = json.loads((OUT / 'resume-2026-10-04-vsix-candidate-diff.json').read_bytes())
    for name in ['package.json', 'package-lock.json']:
        data = (candidate / name).read_bytes()
        assert hashlib.sha256(data).hexdigest() == metadata['candidate_sha256'][name]
        (ROOT / 'ccr-vscode' / name).write_bytes(data)
    after = inputs()
    assert [path for path in before if before[path] != after[path]] == ['ccr-vscode/package-lock.json', 'ccr-vscode/package.json']
    save('apply', {'date_utc': dt.datetime.now(dt.timezone.utc).isoformat(), 'os': platform.platform(),
                   'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
                   'input_sha256_before': before, 'input_sha256_after': after,
                   'changed_product_files': ['ccr-vscode/package.json', 'ccr-vscode/package-lock.json'],
                   'candidate_diff': 'resume-2026-10-04-vsix-candidate-diff.json',
                   'approval_basis': 'User explicit VSCE 4 migration approval; source freeze released after first full CI'})
    print('Applied approved VSCE 4 candidate to exactly two actual product files')
else:
    commands = {
        'npm-ci': (['npm', 'ci'], ROOT / 'ccr-vscode'),
        'audit': (['npm', 'audit', '--json'], ROOT / 'ccr-vscode'),
        'ci': (['just', 'vscode-ci'], ROOT),
        'coverage': (['just', 'vscode-coverage'], ROOT),
        'source-list': (['npx', '--no-install', 'vsce', 'ls', '--no-dependencies'], ROOT / 'ccr-vscode'),
        'final-vsix': (['node', 'scripts/check-package-files.mjs', '--vsix', 'ccr-vscode.vsix'], ROOT / 'ccr-vscode'),
        'diff-check': (['git', 'diff', '--check', '--', 'ccr-vscode/package.json', 'ccr-vscode/package-lock.json'], ROOT),
    }
    argv, cwd = commands[mode]
    sys.exit(run(mode, argv, cwd))
