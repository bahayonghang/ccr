"""Fixed Node 24.20.0 gates for the approved VSCE 4 migration."""
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
PREFIX = 'resume-2026-10-04-vsix-node24-gate-'
NODE_DIR = Path(os.environ['TEMP']) / 'ccr-mise-node2420' / 'installs' / 'node' / '24.20.0'
NODE = NODE_DIR / 'node.exe'
NPM = NODE_DIR / 'npm.cmd'
EXPECTED_NODE_SHA256 = '5c976096e04e5c2c1f091938926234cc9fbebfe9787ddd149351b3b0ecc707b5'
assert hashlib.sha256(NODE.read_bytes()).hexdigest() == EXPECTED_NODE_SHA256
CHILD_ENV = os.environ.copy()
CHILD_ENV['PATH'] = str(NODE_DIR) + os.pathsep + CHILD_ENV['PATH']
measured_node = subprocess.check_output([str(NODE), '-p', 'process.version'], env=CHILD_ENV).decode().strip()
measured_npm = subprocess.check_output([str(NPM), '--version'], env=CHILD_ENV).decode().strip()
assert measured_node == 'v24.20.0', measured_node


def inputs():
    files = subprocess.check_output(['git', 'ls-files', 'ccr-vscode'], cwd=ROOT).decode().splitlines()
    files.append('justfile')
    return {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in files if (ROOT / path).is_file()}


def save(name, data):
    with (OUT / (PREFIX + name + '.json')).open('x', encoding='utf-8') as stream:
        json.dump(data, stream, indent=2)
        stream.write('\n')


def run(label, argv, cwd):
    before = inputs()
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    timer = time.monotonic()
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
               ['tests ', 'pass ', 'fail ', 'all files', 'file check passed', 'vulnerabilities',
                'Packaged:', 'CI passed', 'ERROR', 'error'])]
    result = {
        'started_utc': started,
        'finished_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
        'elapsed_seconds': round(time.monotonic() - timer, 3),
        'os': platform.platform(),
        'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
        'argv': argv,
        'cwd': str(cwd),
        'exit_code': process.returncode,
        'streams': streams,
        'input_sha256_before': before,
        'input_sha256_after': after,
        'source_bytes_preserved': before == after,
        'measured_node_version': measured_node,
        'measured_npm_version': measured_npm,
        'node_executable': str(NODE),
        'node_sha256': EXPECTED_NODE_SHA256,
        'summary': summary[-40:],
        'boundary': 'Temporary mise data dir only. Node is not activated in a config file. Hosted CI, native activation, and Marketplace remain unverified.',
    }
    if label == 'audit':
        audit = json.loads((OUT / (PREFIX + label + '.stdout.log')).read_bytes())
        result['vulnerability_counts'] = audit.get('metadata', {}).get('vulnerabilities')
    save(label, result)
    print(json.dumps({'label': label, 'exit_code': process.returncode, 'summary': result['summary'][-12:]}, ensure_ascii=False))
    return process.returncode


COMMANDS = {
    'npm-ci': (['npm.cmd', 'ci'], ROOT / 'ccr-vscode'),
    'audit': (['npm.cmd', 'audit', '--json'], ROOT / 'ccr-vscode'),
    'resolve': (['node.exe', '-e', "console.log(require.resolve('@vscode/vsce/package.json'))"], ROOT / 'ccr-vscode'),
    'cli-version': (['npx.cmd', '--no-install', 'vsce', '--version'], ROOT / 'ccr-vscode'),
    'ci': (['just', 'vscode-ci'], ROOT),
    'coverage': (['just', 'vscode-coverage'], ROOT),
    'source-list': (['npx.cmd', '--no-install', 'vsce', 'ls', '--no-dependencies'], ROOT / 'ccr-vscode'),
    'final-vsix': (['node.exe', 'scripts/check-package-files.mjs', '--vsix', 'ccr-vscode.vsix'], ROOT / 'ccr-vscode'),
    'diff-check': (['git', 'diff', '--check', '--', 'ccr-vscode/package.json', 'ccr-vscode/package-lock.json'], ROOT),
}

mode = sys.argv[1]
argv, cwd = COMMANDS[mode]
argv = [str(NODE_DIR / argv[0]) if argv[0] in {'npm.cmd', 'npx.cmd', 'node.exe'} else argv[0], *argv[1:]]
sys.exit(run(mode, argv, cwd))
