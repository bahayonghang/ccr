"""Validate planning artifacts only; never run product checks or start tasks."""
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[4]
PARENT = '.trellis/tasks/10-26-ccr-ui-airy-glass'
DIRS = [PARENT] + [
    '.trellis/tasks/' + name
    for name in (
        '10-26-ccr-ag01-foundation',
        '10-26-ccr-ag02-workspaces',
        '10-26-ccr-ag03-observability',
        '10-26-ccr-ag04-acceptance',
    )
]
COMMON = [
    ('.trellis/spec/ccr-ui/frontend/index.md', 'Frontend contract entrypoint'),
    (PARENT + '/research/surface-inventory.md', 'Route families and frozen behavior'),
    (PARENT + '/research/validation-contracts.md', 'Theme, material and evidence audit'),
    (PARENT + '/research/catalog/DESIGN.md', 'Original catalog; approved CCR adaptation takes precedence'),
    ('.trellis/spec/ccr-ui/frontend/layering-contracts.md', 'Dependency boundaries'),
    ('.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md', 'Rendering and subscription discipline'),
    ('.trellis/spec/ccr-ui/frontend/confirm-interaction-contracts.md', 'Confirmation and focus contracts'),
    ('.trellis/spec/ccr-ui/frontend/usage-chart-stability-contracts.md', 'Chart lifetime and CSS contracts'),
]


def main():
    import sys

    configure = '--configure' in sys.argv
    records = []
    parent_record = json.loads((ROOT / PARENT / 'task.json').read_text(encoding='utf-8'))
    for name in DIRS:
        directory = ROOT / name
        entries = list(COMMON)
        extra = []
        if 'workspaces' in name:
            extra = ['profiles-page-contracts.md', 'raw-config-editor-contracts.md', 'sync-security-contracts.md']
        if 'observability' in name:
            extra = ['agent-session-observability-contracts.md', 'command-job-lifecycle.md', 'checkin-ux-contracts.md']
        if 'acceptance' in name or 'foundation' in name:
            extra = ['development-resource-contracts.md']
        entries += [('.trellis/spec/ccr-ui/frontend/' + item, 'Scope-specific behavior contract') for item in extra]
        if configure:
            for kind in ('implement', 'check'):
                content = ''.join(json.dumps({'file': f, 'reason': r}) + '\n' for f, r in entries)
                (directory / (kind + '.jsonl')).write_text(content, encoding='utf-8')
            task = json.loads((directory / 'task.json').read_text(encoding='utf-8'))
            task['dev_type'] = 'frontend'
            task['scope'] = 'ccr-ui'
            task['notes'] = 'Planning only. Await approval of the latest CCR airy-glass adaptation. Existing GPUI/Rust work is excluded.'
            (directory / 'task.json').write_text(json.dumps(task, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
        task = json.loads((directory / 'task.json').read_text(encoding='utf-8'))
        assert task['status'] == 'planning', name
        if name != PARENT:
            assert task['parent'] == pathlib.Path(PARENT).name, task['parent']
            assert directory.name in parent_record['children'], directory.name
        for document in ('prd.md', 'design.md', 'implement.md'):
            text = (directory / document).read_text(encoding='utf-8')
            assert len(text) > 200 and 'TBD' not in text, (name, document)
        for kind in ('implement', 'check'):
            rows = [json.loads(line) for line in (directory / (kind + '.jsonl')).read_text(encoding='utf-8').splitlines() if line.strip()]
            assert rows and all((ROOT / row['file']).is_file() for row in rows), name
        result = subprocess.run([sys.executable, '.trellis/scripts/task.py', 'validate', name], cwd=ROOT, capture_output=True, text=True, encoding='utf-8')
        records.append(name + '\n' + result.stdout + result.stderr)
        assert result.returncode == 0, records[-1]
    records.append('PASS: 5 planning tasks; parent/child links, 15 planning documents and all context references validated. Product checks NOT_RUN. User approval pending.\n')
    (ROOT / PARENT / 'planning-validation.txt').write_text('\n'.join(records), encoding='utf-8')
    print(records[-1])


if __name__ == '__main__':
    main()
