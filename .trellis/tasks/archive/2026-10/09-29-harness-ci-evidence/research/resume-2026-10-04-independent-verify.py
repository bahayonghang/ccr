"""Independently inspect preserved receipts and run read-only narrow checks."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
PARENT = ROOT / '.trellis/tasks/09-29-evergreen-harness-audit/research'
UI = ROOT / '.trellis/tasks/09-29-harness-frontend-security/research/evidence'
OUT = HERE / 'resume-2026-10-04-independent-verification.json'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def read(path):
    return json.loads(path.read_text(encoding='utf-8'))

result = {'recorded_at_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
          'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
          'cargo_receipts': {}, 'checks': []}
expected = {'exact': (1, 16), 'suite': (17, 0), 'platforms': (35, 0),
            'commands': (3, 114), 'default': (17, 0), 'clippy': None}
source_tests = set()
for module in ('general', 'legacy_registry'):
    text = (ROOT / f'crates/ccr/tests/managers/{module}.rs').read_text(encoding='utf-8')
    source_tests.update(f'{module}::{name}' for name in re.findall(r'#\[test\]\s+fn (\w+)', text))
for variant, counts in expected.items():
    path = HERE / f'resume-2026-10-04-manager-{variant}.json'
    data = read(path)
    entry = {'receipt_sha256': sha(path), 'command': data['command'], 'exit_code': data['exit_code'],
             'parallelism': data['parallelism'], 'log_checks': {}, 'source_checks': {},
             'test_counts': data['test_counts']}
    assert data['exit_code'] == 0 and data['status'] == 'PASS'
    assert data['head_at_start'] == data['head_at_end'] == result['head']
    assert data['retries'] == 0 and data['fixture_cleanup_complete']
    assert '--test-threads' not in ' '.join(data['command'])
    for kind in ('stdout', 'stderr'):
        log = HERE / data[kind]
        entry['log_checks'][kind] = {'bytes': log.stat().st_size, 'sha256': sha(log)}
        assert log.stat().st_size == data[f'{kind}_bytes']
        assert sha(log) == data[f'{kind}_sha256']
    assert data['source_hashes_before'] == data['source_hashes_after']
    for name, digest in data['source_hashes_after'].items():
        entry['source_checks'][name] = sha(ROOT / name) == digest
    assert len(entry['source_checks']) == 14 and all(entry['source_checks'].values())
    stdout = (HERE / data['stdout']).read_text(encoding='utf-8')
    parsed = [{'status': status, 'passed': int(passed), 'failed': int(failed),
               'ignored': int(ignored), 'measured': int(measured), 'filtered_out': int(filtered)}
              for status, passed, failed, ignored, measured, filtered in re.findall(
                  r'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out', stdout)]
    assert parsed == data['test_counts']
    if counts:
        assert len(parsed) == 1 and parsed[0]['status'] == 'ok'
        assert (parsed[0]['passed'], parsed[0]['filtered_out']) == counts
        assert parsed[0]['failed'] == parsed[0]['ignored'] == 0
    else:
        assert not parsed
        assert 'Finished `dev` profile' in (HERE / data['stderr']).read_text(encoding='utf-8')
    if variant in ('suite', 'default'):
        executed = set(re.findall(r'^test ([\w:]+) \.\.\. ok$', stdout, re.M))
        assert executed == source_tests and len(executed) == 17
        entry['executed_names_match_source'] = True
    result['cargo_receipts'][variant] = entry

baseline = read(HERE / 'resume-2026-10-04-manager-baseline.json')
ignored = read(HERE / 'resume-2026-10-04-manager-ignore.json')
diagnostic = ROOT / '.trellis/tasks/09-29-harness-ci-evidence/research/tauri-advisory-commands.json'
assert sha(diagnostic) == baseline['diagnostic_json_sha256'] == ignored['diagnostic_json_sha256']
assert diagnostic.stat().st_size == baseline['diagnostic_json_bytes'] == 1164069
candidate = HERE / 'manager-lock-candidate.patch'
assert sha(candidate) == baseline['candidate_sha256']
result['diagnostic_json'] = {'bytes': diagnostic.stat().st_size, 'sha256': sha(diagnostic)}
result['candidate_sha256'] = sha(candidate)
for name, current in baseline['diagnostic_input_sha256'].items():
    for variant in ('exact', 'suite'):
        for phase in ('before', 'after'):
            assert read(HERE / f'manager-lock-{variant}-hashes-{phase}.json')[name] == current
result['baseline_matches_four_historical_input_lists'] = True

for module, count, statement in (
    ('general', 5, '    let _env = crate::setup_ccr_test_env();\n'),
    ('legacy_registry', 1, '    let _env = setup_ccr_test_env();\n')):
    name = f'crates/ccr/tests/managers/{module}.rs'
    original = subprocess.check_output(['git', 'show', f'HEAD:{name}'], cwd=ROOT).decode().replace('\r\n', '\n')
    current = (ROOT / name).read_text(encoding='utf-8').replace('\r\n', '\n')
    assert current.count(statement) == count and current.replace(statement, '') == original
result['source_delta'] = {'general_added_guards': 5, 'legacy_added_guards': 1, 'other_changes': 0}

history = read(PARENT / 'final-ci-retry1.json')
assert history['exit_code'] == 1 and sha(PARENT / history['log']) == history['log_sha256']
result['historical_ci'] = {'exit_code': 1, 'head': history['head'], 'log_sha256': history['log_sha256']}
fresh = read(PARENT / 'resume-2026-10-04-omp-fresh-fixture.json')
generated = read(UI / 'resume-2026-10-04-omp-generated.json')
for data, directory, stem, expected_output in (
    (fresh, PARENT, 'resume-2026-10-04-omp-fresh-fixture', '14 skip'),
    (generated, UI, 'resume-2026-10-04-omp-generated', '14 pass')):
    for kind in ('stdout', 'stderr'):
        assert sha(directory / f'{stem}.{kind}.log') == data[f'{kind}_sha256']
    assert expected_output in (directory / f'{stem}.stderr.log').read_text(encoding='utf-8')
result['omp'] = {'generated': '14 pass, 99 assertions; local generated extension',
                 'source_only': '0 pass, 14 skip, 0 fail; SKIPPED_UNVERIFIED'}

commands = [
    ['git', 'check-ignore', '-v', '--', str(diagnostic.relative_to(ROOT)).replace('\\', '/')],
    ['git', 'diff', '--check'],
    ['python', '.trellis/scripts/task.py', 'validate', '.trellis/tasks/09-29-harness-ci-evidence'],
    ['python', '.trellis/scripts/task.py', 'validate', '.trellis/tasks/09-29-evergreen-harness-audit'],
    ['python', 'scripts/ci/check_workflow_governance.py'],
]
for command in commands:
    check = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
    result['checks'].append({'command': command, 'exit_code': check.returncode,
                             'stdout': check.stdout, 'stderr': check.stderr})
result['receipt_integrity_status'] = 'PASS'
result['boundary'] = 'Independent static and evidence verification. Cargo tests and clippy were not rerun. Full CI and external/native boundaries are owned by the parent.'
with OUT.open('x', encoding='utf-8', newline='\n') as output:
    output.write(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'receipt_integrity_status': result['receipt_integrity_status'],
                  'checks': [{'command': x['command'], 'exit_code': x['exit_code']} for x in result['checks']]}, ensure_ascii=False))
