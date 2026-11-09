"""Read-only checks for the T10-owned spec and evidence surfaces."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
owned = [
    '.trellis/spec/ccr/backend/dependency-governance.md',
    '.trellis/spec/ccr/backend/tauri-handler-registry.md',
    '.trellis/spec/ccr/backend/typed-ipc-bindings.md',
    '.trellis/spec/ccr/backend/index.md',
    '.trellis/spec/ccr/backend/llmusage-provider-adapter.md',
    '.trellis/spec/ccr/backend/usage-job-lifecycle.md',
    'code_map.md', 'ccr-ui/code_map.md',
]
files = []
links = []
for name in owned:
    path = ROOT / name
    raw = path.read_bytes()
    if name.startswith('.trellis/spec/'):
        assert len(raw) <= 32768, (name, len(raw))
    files.append({'path': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
    for chunk in raw.decode('utf-8').split('](')[1:]:
        target = chunk.split(')', 1)[0].split('#', 1)[0]
        if not target or '://' in target or not target.startswith('.'):
            continue
        assert (path.parent / target).exists(), (name, target)
        links.append({'source': name, 'target': target})

manifest = json.loads((ROOT / 'ccr-ui/src/api/generated/command-manifest.json').read_text(encoding='utf-8'))
counts = {key: manifest[key] for key in ('base_command_count', 'windows_command_count', 'typed_command_count', 'exact_wire_type_count')}
assert tuple(counts.values()) == (340, 348, 278, 278)
assert len(manifest['commands']) == 348
registry = (ROOT / 'ccr-ui/src-tauri/src/commands/handler_registry.rs').read_text(encoding='utf-8')
assert 'assert_eq!(COMMAND_MODULES.len(), 38);' in registry
assert 'assert_eq!(manifest.typed_command_count, 278);' in registry
assert 'assert_eq!(manifest.exact_wire_type_count, 278);' in registry
for name in ('ccr-ui/src/config/appMeta.ts', 'ccr-ui/src/shell/MainLayoutChrome.tsx', 'ccr-ui/src/shell/Titlebar.tsx', 'crates/ccr-usage/src'):
    assert (ROOT / name).exists(), name
for name in ('MainLayoutChrome.tsx', 'Titlebar.tsx'):
    assert '@/config/appMeta' in (ROOT / 'ccr-ui/src/shell' / name).read_text(encoding='utf-8')
for name in ('.trellis/spec/ccr/backend/index.md', 'code_map.md', 'ccr-ui/code_map.md'):
    assert 'ccr-usage' in (ROOT / name).read_text(encoding='utf-8')

extraction = json.loads((OUT / 'spec-convergence-evidence.json').read_text(encoding='utf-8'))['extraction']
assert hashlib.sha256((ROOT / extraction['source']).read_bytes()).hexdigest() == extraction['after_sha256']
# This byte-level identity includes all pre-existing analytics, pricing and Home Insights content.
context_results = []
for task in ('09-28-usage-job-lifecycle', '09-28-architecture-contract-gates'):
    command = ['python', '.trellis/scripts/task.py', 'validate', '.trellis/tasks/' + task]
    run = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace')
    (OUT / ('spec-context-' + task + '.log')).write_text(run.stdout + run.stderr, encoding='utf-8')
    assert run.returncode == 0, run.stdout + run.stderr
    assert 'exceeds' not in run.stdout.lower() and '32768' not in run.stdout, run.stdout
    context_results.append({'task': task, 'command': command, 'exit_code': run.returncode})

baseline = json.loads((ROOT / '.trellis/tasks/09-28-cli-tauri-architecture/research/implementation-baseline.json').read_text(encoding='utf-8'))
protected = []
for name, expected in baseline['protected_files_sha256'].items():
    current = hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
    assert current == expected, name
    protected.append({'path': name, 'sha256': current, 'unchanged': True})
states = {}
for name, expected in [('09-24-home-insights-redesign', 'planning'), ('09-24-home-insights-frontend', 'in_progress')]:
    data = json.loads((ROOT / '.trellis/tasks' / name / 'task.json').read_text(encoding='utf-8'))
    assert data['status'] == expected, name
    states[name] = data['status']
result = {'date': '2026-09-28', 'status': 'passed', 'owned_files': files, 'resolved_relative_links': links, 'manifest_counts': counts, 'base_modules': 38, 'prior_analytics_pricing_home_insights_bytes_unchanged': True, 'context_validations': context_results, 'protected_files': protected, 'existing_task_statuses': states, 'limits': 'Checks links, exact source bytes, per-file context budget and protected baseline. No product behavior or full-gate acceptance is inferred.'}
(OUT / 'spec-validation.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + chr(10), encoding='utf-8')
print(json.dumps({'status': result['status'], 'files': len(files), 'links': len(links), 'contexts': len(context_results), 'protected_files': len(protected)}))
