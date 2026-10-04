"""Resolve the proposed VSCE 4 migration in task-owned manifest/lock copies only."""
import datetime as dt
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
spec = importlib.util.spec_from_file_location('receipt', OUT / 'resume-2026-10-04-vsix-run.py')
receipt = importlib.util.module_from_spec(spec)
spec.loader.exec_module(receipt)
CANDIDATE = OUT / 'resume-2026-10-04-vsix-candidate'
CANDIDATE.mkdir(exist_ok=False)
original = json.loads((ROOT / 'ccr-vscode/package-lock.json').read_bytes())
manifest = (ROOT / 'ccr-vscode/package.json').read_bytes()
assert manifest.count(b'"@vscode/vsce": "^3.9.2"') == 1
(CANDIDATE / 'package.json').write_bytes(manifest.replace(b'"@vscode/vsce": "^3.9.2"', b'"@vscode/vsce": "^4.0.0"', 1))
lock = (ROOT / 'ccr-vscode/package-lock.json').read_bytes()
assert lock.count(b'"@vscode/vsce": "^3.9.2"') == 1
(CANDIDATE / 'package-lock.json').write_bytes(lock.replace(b'"@vscode/vsce": "^3.9.2"', b'"@vscode/vsce": "^4.0.0"', 1))
assert receipt.run('candidate-resolve', ['npm', 'install', '--package-lock-only', '--ignore-scripts'], CANDIDATE) == 0
audit_exit = receipt.run('candidate-audit', ['npm', 'audit', '--json'], CANDIDATE)
after = json.loads((CANDIDATE / 'package-lock.json').read_bytes())
added = sorted(set(after['packages']) - set(original['packages']))
removed = sorted(set(original['packages']) - set(after['packages']))
changed = []
for node in sorted(set(original['packages']) & set(after['packages'])):
    old, new = original['packages'][node], after['packages'][node]
    if old != new:
        changed.append({'node': node, 'old_version': old.get('version'), 'new_version': new.get('version'),
                        'changed_fields': sorted(field for field in set(old) | set(new) if old.get(field) != new.get(field))})
semver_code = '''const s=require('./ccr-vscode/node_modules/semver');const fs=require('node:fs');const d=JSON.parse(fs.readFileSync(process.argv[1],'utf8'));const failures=Object.entries(d.packages).filter(([k,v])=>v.engines?.node&&!s.satisfies('24.20.0',v.engines.node)).map(([k,v])=>({node:k,version:v.version,engines:v.engines}));console.log(JSON.stringify(failures));'''
engine_failures = json.loads(subprocess.check_output(['node', '-e', semver_code, str(CANDIDATE / 'package-lock.json')], cwd=ROOT))
changes = {'captured_utc': dt.datetime.now(dt.timezone.utc).isoformat(), 'candidate_only': True,
           'manifest_change': {'@vscode/vsce': '^3.9.2 -> ^4.0.0'},
           'before_package_count': len(original['packages']), 'after_package_count': len(after['packages']),
           'added': [{'node': node, 'version': after['packages'][node].get('version')} for node in added],
           'removed': [{'node': node, 'version': original['packages'][node].get('version')} for node in removed],
           'changed': changed, 'candidate_audit_exit_code': audit_exit,
           'fixed_node_version': '24.20.0', 'node_engine_failures': engine_failures,
           'approved_targets_retained': {node: after['packages'].get(node, {}).get('version') for node in
                                        ['node_modules/brace-expansion', 'node_modules/fast-uri', 'node_modules/undici']},
           'candidate_sha256': {name: hashlib.sha256((CANDIDATE / name).read_bytes()).hexdigest() for name in ['package.json', 'package-lock.json']}}
receipt.write('candidate-diff.json', changes)
print(json.dumps({'candidate_audit_exit': audit_exit, 'added': len(added), 'removed': len(removed), 'changed': len(changed), 'engine_failures': engine_failures}))
