"""Record the approved T03 local gates without triggering hosted jobs."""
import datetime
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent / 'evidence'
INPUTS = ('.github/dependabot.yml', 'scripts/ci/check_workflow_governance.py',
          'scripts/ci/test_check_workflow_governance.py',
          'ccr-ui/package.json', 'ccr-ui/bun.lock',
          'docs/package.json', 'docs/bun.lock')
CHECKS = (
    ('unit', '.', ['python', '-m', 'unittest', 'scripts.ci.test_check_workflow_governance']),
    ('workflow-governance', '.', ['just', 'workflow-governance-check']),
    ('version-check', '.', ['just', 'version-check']),
    ('docs-frozen', 'docs', ['bun', 'install', '--frozen-lockfile']),
    ('docs-audit', 'docs', ['bun', 'run', 'audit']),
    ('docs-build', 'docs', ['bun', 'run', 'build']),
    ('ui-frozen', 'ccr-ui', ['bun', 'install', '--frozen-lockfile']),
    ('actionlint', '.', ['actionlint']),
    ('diff-check', '.', ['git', 'diff', '--check']),
)


def hashes():
    return {name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest() for name in INPUTS}


def save(name, value):
    (EVIDENCE / name).write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n',
                                 encoding='utf-8', newline='\n')


def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    if any((EVIDENCE / f'{label}.json').exists() for label, _, _ in CHECKS):
        raise RuntimeError('Keep existing receipts; use a new label for a rerun')
    versions = {}
    for tool in ('python', 'bun', 'node', 'just'):
        versions[tool] = subprocess.check_output([shutil.which(tool), '--version'],
                                                text=True, encoding='utf-8').strip()
    save('environment.json', {'platform': platform.platform(), 'versions': versions,
         'observed_local': datetime.datetime.now().astimezone().isoformat(),
         'observed_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
         'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT,
                                         text=True, encoding='utf-8').strip(),
         'hosted_dependabot': 'UNVERIFIED; no dispatch, push, or rerun performed'})
    baseline = subprocess.check_output(['git', 'show', 'HEAD:.github/dependabot.yml'], cwd=ROOT).decode('utf-8')
    expected = baseline
    for directory in ('/ccr-ui', '/docs'):
        expected = expected.replace(f'package-ecosystem: npm\n    directory: {directory}\n',
                                    f'package-ecosystem: bun\n    directory: {directory}\n')
    actual = (ROOT / '.github/dependabot.yml').read_text(encoding='utf-8')
    assert actual == expected
    save('mapping-scope.json', {'only_two_ecosystem_lines_changed': True,
         'preserved': ['Cargo entries', 'GitHub Actions entry', 'directories', 'schedules', 'labels'],
         'javascript_mapping': {'/ccr-ui': 'bun', '/docs': 'bun', '/ccr-vscode': 'npm'}})
    results = []
    for label, cwd, command in CHECKS:
        before = hashes()
        started = datetime.datetime.now(datetime.timezone.utc).isoformat()
        tick = time.monotonic()
        result = subprocess.run([shutil.which(command[0]), *command[1:]], cwd=ROOT / cwd,
                                capture_output=True, check=False)
        (EVIDENCE / f'{label}.stdout.log').write_bytes(result.stdout)
        (EVIDENCE / f'{label}.stderr.log').write_bytes(result.stderr)
        receipt = {'command': command, 'cwd': str(ROOT / cwd), 'started_utc': started,
                   'duration_seconds': round(time.monotonic() - tick, 3),
                   'exit_code': result.returncode, 'input_sha256_before': before,
                   'input_sha256_after': hashes(),
                   'stdout_sha256': hashlib.sha256(result.stdout).hexdigest(),
                   'stderr_sha256': hashlib.sha256(result.stderr).hexdigest()}
        save(f'{label}.json', receipt)
        results.append({'label': label, 'exit_code': result.returncode})
        print(json.dumps(results[-1]), flush=True)
        for output in (result.stdout, result.stderr):
            print('\n'.join(output.decode('utf-8', errors='replace').splitlines()[-8:]), flush=True)
    save('check-results.json', results)
    return int(any(result['exit_code'] for result in results))


if __name__ == '__main__':
    sys.exit(main())
