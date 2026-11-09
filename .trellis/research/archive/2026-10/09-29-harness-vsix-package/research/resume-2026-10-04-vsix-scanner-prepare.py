"""VSCE 4 secret and .env checks use synthetic packages and a temporary fixed Node."""
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile
import time

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
PREFIX = 'resume-2026-10-04-vsix-scanner-'
STATE = OUT / (PREFIX + 'state.json')
fixed = json.loads((OUT / 'resume-2026-10-04-vsix-node24-receipt.json').read_bytes())
NODE = Path(fixed['temporary_executable'])
assert hashlib.sha256(NODE.read_bytes()).hexdigest() == fixed['binary_sha256']
CHILD_ENV = os.environ.copy()
CHILD_ENV['PATH'] = str(NODE.parent) + os.pathsep + CHILD_ENV['PATH']


def inputs():
    return {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in
            ['ccr-vscode/package.json', 'ccr-vscode/package-lock.json', 'scripts/trellis/omp-context.test.ts', '.omp/extensions/trellis/index.ts']}


def run(label, argv, cwd):
    before = inputs()
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    timer = time.monotonic()
    with (OUT / (PREFIX + label + '.stdout.log')).open('xb') as stdout:
        with (OUT / (PREFIX + label + '.stderr.log')).open('xb') as stderr:
            process = subprocess.run(argv, cwd=cwd, env=CHILD_ENV, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr)
    streams = {}
    combined = ''
    for channel in ['stdout', 'stderr']:
        path = OUT / (PREFIX + label + '.' + channel + '.log')
        raw = path.read_bytes()
        streams[channel] = {'path': path.name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
        combined += raw.decode('utf-8', errors='replace')
    after = inputs()
    with (OUT / (PREFIX + label + '.json')).open('x', encoding='utf-8') as stream:
        json.dump({'started_utc': started, 'finished_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
                   'elapsed_seconds': round(time.monotonic() - timer, 3), 'os': platform.platform(),
                   'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
                   'argv': argv, 'cwd': str(cwd), 'exit_code': process.returncode, 'streams': streams,
                   'input_sha256_before': before, 'input_sha256_after': after,
                   'node_version': '24.20.0', 'boundary': 'temporary synthetic VSCE 4 packaging; no account operation'}, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'label': label, 'exit_code': process.returncode, 'output': combined[-6500:]}))
    assert before == after
    return process.returncode


mode = sys.argv[1]
if mode == 'install':
    temporary_root = Path(tempfile.mkdtemp(prefix='ccr-vsce4-scanner-')).resolve()
    assert temporary_root.is_relative_to(Path(tempfile.gettempdir()).resolve())
    tools_root = temporary_root / 'cli'
    tools_root.mkdir()
    (tools_root / 'package.json').write_text('{"name":"ccr-vsce4-scanner-tools","private":true}\n', encoding='utf-8')
    projects = {}
    for name in ['clean', 'secret', 'dotenv']:
        project = temporary_root / name
        project.mkdir()
        manifest = {'name': 'ccr-synthetic-scanner-' + name, 'displayName': 'CCR Synthetic Scanner Fixture',
                    'version': '0.0.0', 'publisher': 'ccr-review-fixture', 'license': 'MIT',
                    'engines': {'vscode': '^1.85.0'}, 'main': './extension.js',
                    'repository': {'type': 'git', 'url': 'https://example.invalid/ccr-synthetic'}}
        manifest['activationEvents'] = []
        (project / 'package.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
        (project / 'README.md').write_text('# Synthetic CCR scanner fixture\n\nTemporary test package.\n', encoding='utf-8')
        (project / 'LICENSE').write_text('MIT License\n\nCopyright (c) 2026 Synthetic Fixture\n', encoding='utf-8')
        code = 'exports.activate = () => {}\n'
        if name == 'secret':
            code += "const syntheticToken = 'ghp_0123456789012345678901234567890123456789'\n"
        (project / 'extension.js').write_text(code, encoding='utf-8')
        if name == 'dotenv':
            (project / '.env').write_text('SYNTHETIC_FIXTURE_VALUE=example\n', encoding='utf-8')
        projects[name] = str(project)
    with STATE.open('x', encoding='utf-8') as stream:
        json.dump({'tools_root': str(tools_root), 'projects': projects, 'temporary_root': str(temporary_root)}, stream, indent=2)
        stream.write('\n')
    code = run('install', [shutil.which('npm'), 'install', '--ignore-scripts', '--no-audit', '--no-fund',
                          '--save-exact', '@vscode/vsce@4.0.0'], tools_root)
    if code:
        sys.exit(code)
    cli = tools_root / 'node_modules/@vscode/vsce/vsce'
    assert run('version', [str(NODE), str(cli), '--version'], tools_root) == 0
    sys.exit(run('help', [str(NODE), str(cli), 'package', '--help'], tools_root))
elif mode == 'repair-install':
    state = json.loads(STATE.read_bytes())
    tools_root = Path(state['tools_root'])
    assert run('install-retry', [shutil.which('npm'), 'ci', '--ignore-scripts', '--no-audit', '--no-fund'], tools_root) == 0
    cli = tools_root / 'node_modules/@vscode/vsce/vsce'
    assert run('version', [str(NODE), str(cli), '--version'], tools_root) == 0
    sys.exit(run('help', [str(NODE), str(cli), 'package', '--help'], tools_root))
else:
    state = json.loads(STATE.read_bytes())
    project_name = 'clean' if mode == 'clean-activation' else mode
    project = Path(state['projects'][project_name])
    manifest_path = project / 'package.json'
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    if 'activationEvents' not in manifest:
        manifest['activationEvents'] = []
        manifest_path.write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    cli = Path(state['tools_root']) / 'node_modules/@vscode/vsce/vsce'
    code = run(mode, [str(NODE), str(cli), 'package', '--no-dependencies', '-o', 'synthetic.vsix'], project)
    expected = 0 if project_name == 'clean' else 1
    assert code == expected, (mode, code)
    assert (project / 'synthetic.vsix').exists() == (project_name == 'clean')
