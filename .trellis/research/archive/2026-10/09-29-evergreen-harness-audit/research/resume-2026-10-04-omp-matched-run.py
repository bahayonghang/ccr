"""Exact official CLI generation in an owned temporary workspace only."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import tempfile
import time

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
PREFIX = 'resume-2026-10-04-omp-matched-'
STATE = OUT / (PREFIX + 'state.json')
PRODUCT_PATHS = ['scripts/trellis/omp-context.test.ts', '.omp/extensions/trellis/index.ts',
                 'docs/agents/harnesses.md', 'docs/en/agents/harnesses.md', 'ccr-vscode/package-lock.json']


def inputs():
    return {path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest() for path in PRODUCT_PATHS}


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
            process = subprocess.run(argv, cwd=cwd, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr)
    streams = {}
    combined = ''
    for channel in ['stdout', 'stderr']:
        path = OUT / (PREFIX + label + '.' + channel + '.log')
        raw = path.read_bytes()
        streams[channel] = {'path': path.name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
        combined += raw.decode('utf-8', errors='replace')
    after = inputs()
    assert before == after, 'Real product source changed during isolated command'
    save(label, {'started_utc': started, 'finished_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
                 'elapsed_seconds': round(time.monotonic() - timer, 3), 'argv': argv, 'cwd': str(cwd),
                 'os': platform.platform(), 'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
                 'input_sha256_before': before, 'input_sha256_after': after, 'streams': streams,
                 'exit_code': process.returncode, 'boundary': 'owned temporary exact-version generation; native client not exercised'})
    print(json.dumps({'label': label, 'exit_code': process.returncode, 'output': combined[-9000:]}))
    return process.returncode


mode = sys.argv[1]
if mode == 'install':
    temporary_root = Path(tempfile.mkdtemp(prefix='ccr-omp-matched-')).resolve()
    assert temporary_root.is_relative_to(Path(tempfile.gettempdir()).resolve())
    tools_root = temporary_root / 'cli'
    project = temporary_root / 'project'
    tools_root.mkdir()
    project.mkdir()
    (tools_root / 'package.json').write_text('{"name":"ccr-omp-matched-cli-fixture","private":true}\n', encoding='utf-8')
    save('state', {'temporary_root': str(temporary_root), 'tools_root': str(tools_root),
                   'project': str(project), 'requested_version': '0.7.0-beta.4', 'product_sha256': inputs()})
    exit_code = run('install', [shutil.which('npm'), 'install', '--ignore-scripts', '--no-audit', '--no-fund',
                              '--save-exact', '@mindfoldhq/trellis@0.7.0-beta.4'], tools_root)
    if exit_code:
        sys.exit(exit_code)
    installed = json.loads((tools_root / 'node_modules/@mindfoldhq/trellis/package.json').read_bytes())
    assert installed['version'] == '0.7.0-beta.4'
    cli = tools_root / 'node_modules/@mindfoldhq/trellis/bin/trellis.js'
    assert run('version', [shutil.which('node'), str(cli), '--version'], project) == 0
    sys.exit(run('help', [shutil.which('node'), str(cli), 'init', '--help'], project))
else:
    state = json.loads(STATE.read_bytes())
    project = Path(state['project'])
    cli = Path(state['tools_root']) / 'node_modules/@mindfoldhq/trellis/bin/trellis.js'
    if mode == 'init':
        argv = [shutil.which('node'), str(cli), 'init', *sys.argv[2:]]
    elif mode in ['test', 'candidate-test']:
        source = project / 'scripts/trellis/omp-context.test.ts'
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_bytes((ROOT / 'scripts/trellis/omp-context.test.ts').read_bytes())
        extension = project / '.omp/extensions/trellis/index.ts'
        save('generated-inputs' if mode == 'test' else 'candidate-generated-inputs', {'fixture_test_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
                                 'fixture_extension_exists': extension.exists(),
                                 'fixture_extension_sha256': hashlib.sha256(extension.read_bytes()).hexdigest() if extension.exists() else None,
                                 'generated_project_version': (project / '.trellis/.version').read_text(encoding='utf-8').strip() if (project / '.trellis/.version').exists() else None,
                                 'original_product_sha256': inputs(), 'native_loading_trust': 'UNVERIFIED'})
        argv = [shutil.which('bun'), 'test', 'scripts/trellis/omp-context.test.ts']
    elif mode in ['patch-check', 'patch-apply']:
        patch = OUT / (PREFIX + 'candidate.patch')
        argv = [shutil.which('git'), 'apply', *(['--check'] if mode == 'patch-check' else []), str(patch)]
    else:
        raise ValueError(mode)
    sys.exit(run(mode, argv, project))
