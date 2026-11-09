"""Verify parent gate receipts without rerunning the gates."""
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
UI = ROOT / '.trellis/tasks/09-29-harness-frontend-security/research/evidence'
VSIX = ROOT / '.trellis/tasks/09-29-harness-vsix-package/research'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def read(path):
    return json.loads(path.read_text(encoding='utf-8'))

result = {'parent_gates': {}, 'vsix': {}, 'boundary': 'Existing receipts verified; no gate rerun.'}
for variant in ('workspace-test', 'lint-strict'):
    stem = f'resume-2026-10-04-{variant}'
    data = read(UI / f'{stem}.json')
    assert data['exit_code'] == 0
    assert data['input_sha256_before'] == data['input_sha256_after']
    for name, digest in data['input_sha256_after'].items():
        assert sha(ROOT / name) == digest
    for kind in ('stdout', 'stderr'):
        assert sha(UI / f'{stem}.{kind}.log') == data[f'{kind}_sha256']
    stdout = (UI / f'{stem}.stdout.log').read_text(encoding='utf-8')
    stderr = (UI / f'{stem}.stderr.log').read_text(encoding='utf-8')
    entry = {'command': data['command'], 'exit_code': 0, 'receipt_sha256': sha(UI / f'{stem}.json'),
             'stdout_sha256': data['stdout_sha256'], 'stderr_sha256': data['stderr_sha256'],
             'input_snapshot_scope': list(data['input_sha256_after'])}
    if variant == 'workspace-test':
        parsed = re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out', stdout)
        assert parsed and all(row[0] == 'ok' and int(row[2]) == 0 for row in parsed)
        entry['test_counts'] = {'passed': sum(int(x[1]) for x in parsed), 'failed': 0,
                                'ignored': sum(int(x[3]) for x in parsed), 'result_blocks': len(parsed)}
        assert 'cargo test --workspace --all-features -- --skip export_bindings' in stderr
        assert '--test-threads' not in stderr
    else:
        assert 'cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::unwrap_used' in stderr
        assert 'Finished `dev` profile' in stderr
    result['parent_gates'][variant] = entry

original = json.loads(subprocess.check_output(['git', 'show', 'HEAD:ccr-vscode/package-lock.json'], cwd=ROOT))
current = read(ROOT / 'ccr-vscode/package-lock.json')
differences = []
def compare(before, after, path=()):
    if isinstance(before, dict) and isinstance(after, dict):
        assert before.keys() == after.keys()
        for key in before:
            compare(before[key], after[key], path + (key,))
    elif before != after:
        differences.append({'path': list(path), 'before': before, 'after': after})
compare(original, current)
expected_nodes = {'node_modules/brace-expansion', 'node_modules/fast-uri'}
assert len(differences) == 6
assert all(len(x['path']) == 3 and x['path'][0] == 'packages' and x['path'][1] in expected_nodes and x['path'][2] in {'version', 'resolved', 'integrity'} for x in differences)
result['vsix']['lock_differences'] = differences
for variant in ('audit-before', 'audit-after', 'npm-ci', 'ci'):
    path = VSIX / f'resume-2026-10-04-vsix-{variant}.json'
    if not path.exists():
        continue
    data = read(path)
    for info in data['streams'].values():
        assert sha(VSIX / info['path']) == info['sha256']
        assert (VSIX / info['path']).stat().st_size == info['bytes']
    result['vsix'][variant] = {'exit_code': data['exit_code'], 'receipt_sha256': sha(path),
                              'vulnerability_counts': data.get('vulnerability_counts')}
    if variant.startswith('audit'):
        raw = read(VSIX / data['streams']['stdout']['path'])
        assert raw['metadata']['vulnerabilities'] == data['vulnerability_counts']

checks = [
    ['git', '-c', 'core.whitespace=blank-at-eol,blank-at-eof,space-before-tab,cr-at-eol', 'diff', '--check'],
    ['git', 'diff', '--check', '--', '.gitignore', 'crates/ccr/tests/managers/general.rs',
     'crates/ccr/tests/managers/legacy_registry.rs', '.trellis/spec/ccr/backend/test-fixtures.md', 'ccr-vscode/package-lock.json'],
]
result['whitespace_checks'] = []
for command in checks:
    check = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    result['whitespace_checks'].append({'command': command, 'exit_code': check.returncode,
                                      'stdout': check.stdout, 'stderr': check.stderr})
assert all(x['exit_code'] == 0 for x in result['whitespace_checks'])
with (HERE / 'resume-2026-10-04-independent-parent-verification.json').open('x', encoding='utf-8', newline='\n') as output:
    output.write(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'workspace': result['parent_gates']['workspace-test']['test_counts'],
                  'vsix': {key: value for key, value in result['vsix'].items() if key != 'lock_differences'},
                  'lock_changed_fields': len(differences)}, ensure_ascii=False))
