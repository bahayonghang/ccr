"""Final structural/byte boundary check for the approved actual lock patch."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import subprocess
import zipfile

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
PREFIX = 'resume-2026-10-04-vsix-'
before = json.loads(subprocess.check_output(['git', 'show', 'HEAD:ccr-vscode/package-lock.json'], cwd=ROOT))
after_bytes = (ROOT / 'ccr-vscode/package-lock.json').read_bytes()
after = json.loads(after_bytes)
assert before.keys() == after.keys()
assert {key: value for key, value in before.items() if key != 'packages'} == {key: value for key, value in after.items() if key != 'packages'}
assert set(before['packages']) == set(after['packages'])
changes = []
for node in before['packages']:
    old, new = before['packages'][node], after['packages'][node]
    if old != new:
        fields = sorted(field for field in set(old) | set(new) if old.get(field) != new.get(field))
        assert fields == ['integrity', 'resolved', 'version']
        changes.append({'node': node, 'before': old, 'after': new, 'changed_fields': fields})
assert [change['node'] for change in changes] == ['node_modules/brace-expansion', 'node_modules/fast-uri']
prepare = json.loads((OUT / (PREFIX + 'prepare.json')).read_bytes())
assert hashlib.sha256(after_bytes).hexdigest() == prepare['after_sha256']
sources = {}
for path, expected in prepare['input_sha256_before'].items():
    if path.startswith('ccr-vscode/') and path != 'ccr-vscode/package-lock.json':
        actual = hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
        assert actual == expected, path
        sources[path] = actual
with zipfile.ZipFile(ROOT / 'ccr-vscode/ccr-vscode.vsix') as archive:
    entries = archive.namelist()
assert len(entries) == 15 and len(set(entries)) == 15
assert not any(part in name for name in entries for part in ['.serena', '.abcoder', '.local.', '.map', 'AGENTS', 'CLAUDE', 'code_map'])
result = {'captured_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
          'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
          'actual_lock_sha256': hashlib.sha256(after_bytes).hexdigest(), 'package_count': len(after['packages']),
          'changed_nodes': changes, 'unchanged_extension_source_sha256': sources,
          'undici_version': after['packages']['node_modules/undici']['version'],
          'final_vsix_entries': entries,
          'final_vsix_sha256': hashlib.sha256((ROOT / 'ccr-vscode/ccr-vscode.vsix').read_bytes()).hexdigest(),
          'candidate_changes_applied_to_actual_manifest': False}
with (OUT / (PREFIX + 'final-verification.json')).open('x', encoding='utf-8') as stream:
    json.dump(result, stream, indent=2)
    stream.write('\n')
print(json.dumps({'changed_nodes': [change['node'] for change in changes], 'vsix_entries': len(entries), 'extension_unchanged_files': len(sources), 'undici': result['undici_version']}))
