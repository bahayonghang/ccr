"""Record baseline dispositions and append a separate current-source audit."""

import hashlib
import json
import re
import subprocess
from collections import Counter
from pathlib import Path

from review_calls import calls

PARENT = Path('.trellis/tasks/10-06-cli-output-presentation/research')
AUTH = Path('.trellis/tasks/10-06-auth-cli-feedback/checks')
CHECKS = Path('.trellis/tasks/10-06-cli-feedback-rollout/checks')
STATUS = {'success', 'info', 'warning', 'error', 'step', 'format_status'}


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8'))


def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


def owner(filename):
    if '/auth/' in filename or filename.endswith('/grok/auth.rs'):
        return 'C2'
    if 'ccr-core/' in filename or filename.endswith('/main.rs'):
        return 'C1'
    return 'C3'


def compact(source):
    return re.sub(r'\s+', ' ', source).strip()


inventory = read_json(PARENT / 'output-callsite-inventory.json')
baseline_rows = inventory['callsites']
baseline_head = inventory['head']
paths = sorted({row['file'] for row in baseline_rows} | {
    'crates/ccr-cli/src/commands/doctor_cmd.rs',
    'crates/ccr-cli/src/commands/check_cmd.rs',
    'crates/ccr-cli/src/commands/codex/auth/off.rs',
    'crates/ccr-cli/src/commands/common/feedback.rs',
    'crates/ccr-cli/src/commands/common/mode.rs',
    'crates/ccr/src/main.rs',
})
current = {path: Path(path).read_text(encoding='utf-8') for path in paths}
baseline = {
    path: subprocess.check_output(['git', 'show', f'{baseline_head}:{path}']).decode('utf-8')
    for path in {row['file'] for row in baseline_rows}
}
baseline_calls = {}
for path, source in baseline.items():
    for start, end, method, line in calls(source):
        baseline_calls[path, line, method] = compact(source[start:end])

auth_calls = {
    (row['file'], row['line'], row['method']): row
    for row in read_json(AUTH / 'auth-callsite-disposition.json')['callsites']
}
auth_semantics = {
    (row['file'], row['line']): row
    for row in read_json(AUTH / 'auth-semantic-disposition.json')['entries']
}


def disposition(row, semantic=False):
    filename = row['file']
    role = owner(filename)
    auth_key = (filename, row['line']) if semantic else (filename, row['line'], row['method'])
    auth_review = (auth_semantics if semantic else auth_calls).get(auth_key)
    if role == 'C2' and auth_review:
        return auth_review['disposition'], auth_review['reason'], role
    method = row.get('method')
    if semantic:
        source = row['source'].splitlines()[0].strip()
        match = re.search(r'ColorOutput::(\w+)\(', source)
        method = match.group(1) if match else None
    else:
        source = baseline_calls.get((filename, row['line'], method), '')
    if method and method not in STATUS:
        return 'preserved', 'Keep public title/separator/banner/table/confirmation/masking layout and signatures.', role
    if filename.endswith('/cli/definitions.rs') or filename.endswith('/cli/dispatch.rs'):
        return 'preserved' if not method else 'shared-covered', 'Keep command/help catalogue and dispatcher error/exit behavior; shared status renderer covers messages.', role
    if source and compact(source) in compact(current.get(filename, Path(filename).read_text(encoding='utf-8'))):
        if not method:
            return 'preserved', 'Retain input hints, table legends, and custom help catalogue under the approved layout/input boundary.', role
        return 'shared-covered', 'Message remains a status/ordinary explanation or input hint; C1 removes legacy labels and whole-message styling.', role
    return 'migrated', 'Reviewed handler entrance: use neutral fields/counts/cancellation, plain status arguments, and contextual command blocks; keep branches/data/streams.', role


for row in baseline_rows:
    row['disposition'], row['reason'], row['owner'] = disposition(row)
    row['snapshot'] = 'baseline'

coverage = []
for filename, source in current.items():
    for start, end, method, line in calls(source):
        snippet = compact(source[start:end])
        unchanged = snippet in compact(baseline.get(filename, ''))
        coverage.append({
            'file': filename, 'line': line, 'method': method, 'snapshot': 'current',
            'source': snippet, 'owner': owner(filename),
            'disposition': ('shared-covered' if method in STATUS else 'preserved') if unchanged else 'migrated',
            'reason': 'Shared status rendering; caller retains its original stream and semantic branch.' if method in STATUS else 'Field or retained public layout/masking/confirmation helper.',
        })
    for start, end, _, line in calls(source, r'\b(print_next_steps)\('):
        if source[max(0, start - 3):start] == 'fn ':
            continue
        coverage.append({
            'file': filename, 'line': line, 'method': 'print_next_steps', 'snapshot': 'current',
            'source': compact(source[start:end]), 'owner': owner(filename),
            'disposition': 'migrated', 'reason': 'Contextual action and complete command on separate lines; no execution or new service checks.',
        })
    for start, end, method, line in calls(source, r'\b(println!|print!)\('):
        snippet = compact(source[start:end])
        if not re.search(r'提示|建议|下一步|成功|失败|警告|错误|跳过|✓|✗|✅|⚠|💡|\[(OK|INFO|WARN|ERR|STEP|FAIL|SKIP)\]|format_status|Results:', snippet):
            continue
        coverage.append({
            'file': filename, 'line': line, 'method': method, 'snapshot': 'current',
            'source': snippet, 'owner': owner(filename),
            'disposition': 'migrated' if 'format_status' in snippet else 'preserved',
            'reason': 'Shared formatter rendered on the existing stdout path.' if 'format_status' in snippet else 'Reviewed direct print: prompt/help/table detail or neutral field/summary; retain layout and input behavior.',
        })

inventory['review'] = {
    'date': '2026-10-07', 'baseline_count': len(baseline_rows),
    'baseline_counts': dict(Counter(row['disposition'] for row in baseline_rows)),
    'current_count': len(coverage),
    'current_source_sha256': {path: hashlib.sha256(Path(path).read_bytes()).hexdigest() for path in paths},
    'note': 'Baseline head/counts/files/callsite identities retained. Current coverage is a separate source snapshot, not runtime acceptance.',
}
inventory['current_source_coverage'] = coverage
write_json(PARENT / 'output-callsite-inventory.json', inventory)

semantic_path = PARENT / 'semantic-review-inventory.json'
semantics = [row for row in read_json(semantic_path) if row.get('snapshot') != 'current']
for row in semantics:
    row['disposition'], row['reason'], row['owner'] = disposition(row, semantic=True)
    row['snapshot'] = 'baseline'
semantic_methods = STATUS | {'key_value', 'print_next_steps', 'println!', 'print!'}
semantics.extend(row for row in coverage if row['method'] in semantic_methods)
write_json(semantic_path, semantics)
write_json(CHECKS / '2026-10-07-inventory-summary.json', {
    'baseline_calls': len(baseline_rows), 'baseline_semantic_entries': len(semantics) - sum(row['method'] in semantic_methods for row in coverage),
    'current_calls': len(coverage),
    'baseline_call_dispositions': dict(Counter(row['disposition'] for row in baseline_rows)),
    'baseline_call_owners': dict(Counter(row['owner'] for row in baseline_rows)),
    'coverage_scope': paths,
})
print(json.dumps(inventory['review'] | {'current_source_sha256': 'recorded in parent inventory'}, ensure_ascii=False))
