"""Capture the post-VSCE-4 full aggregate without replacing the earlier receipt."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
RESEARCH = Path(__file__).resolve().parent
LABEL = 'resume-2026-10-04-vsce4-full-ci'
CAPTURE = ROOT / '.trellis/tasks/09-29-harness-frontend-security/research/run_check.py'
RECEIPT = ROOT / f'.trellis/tasks/09-29-harness-frontend-security/research/evidence/{LABEL}.json'


def save(name, value):
    path = RESEARCH / name
    if path.exists():
        raise RuntimeError(f'Refusing to replace evidence: {path}')
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8'))


def snapshot():
    output = subprocess.check_output(['git', 'ls-files', '-co', '--exclude-standard', '-z'], cwd=ROOT)
    names = sorted(set(output.decode('utf-8').split('\0')) - {''})
    return {
        name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
        for name in names
        if not name.startswith(('.trellis/tasks/', '.trellis/workspace/'))
        and (ROOT / name).is_file()
    }


if RECEIPT.exists():
    raise SystemExit('Aggregate label already exists')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
before = snapshot()
save(f'{LABEL}-source-before.json', before)
environment = os.environ.copy()
environment.update({'PYTHONUTF8': '1', 'PYTHONIOENCODING': 'utf-8', 'RUSTUP_AUTO_INSTALL': '0'})
result = subprocess.run([sys.executable, str(CAPTURE), LABEL, '--', 'just', 'ci'],
                        cwd=ROOT, env=environment, check=False)
after = snapshot()
save(f'{LABEL}-source-after.json', after)
receipt = json.loads(RECEIPT.read_text(encoding='utf-8')) if RECEIPT.exists() else None
changes = [name for name in sorted(set(before) | set(after)) if before.get(name) != after.get(name)]
data = {
    'command': ['just', 'ci'], 'head': head, 'os': 'Windows local',
    'outer_exit_code': result.returncode,
    'aggregate_exit_code': receipt['exit_code'] if receipt else None,
    'command_receipt': str(RECEIPT.relative_to(ROOT)),
    'source_file_count_before': len(before), 'source_file_count_after': len(after),
    'changed_source_paths': changes, 'source_bytes_preserved': before == after,
    'scope': 'Tracked and unignored deliverables; task evidence and journals excluded.',
    'boundary': 'Post VSCE 4 local aggregate. Fixed Node 24.20.0 extension gates are separate. Hosted CI, native clients, and exception approval retain independent status.',
}
save(f'{LABEL}.json', data)
print(json.dumps(data, ensure_ascii=False))
raise SystemExit(result.returncode)
