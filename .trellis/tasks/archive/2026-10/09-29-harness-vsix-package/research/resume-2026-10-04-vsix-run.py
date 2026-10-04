"""Scoped T05 continuation runner. Keep each receipt and raw command stream."""
import base64
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
PREFIX = 'resume-2026-10-04-vsix-'
PARENT = ROOT / '.trellis/tasks/09-29-evergreen-harness-audit/research'
LOCK = ROOT / 'ccr-vscode/package-lock.json'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(name, data):
    path = OUT / (PREFIX + name)
    with path.open('x', encoding='utf-8', newline='\n') as stream:
        json.dump(data, stream, indent=2, ensure_ascii=False)
        stream.write('\n')


def inputs():
    paths = subprocess.check_output(['git', 'ls-files', 'ccr-vscode'], cwd=ROOT).decode().splitlines()
    paths += ['justfile', '.trellis/spec/ccr-vscode/frontend/extension-surface-contracts.md',
              '.trellis/spec/ccr/backend/dependency-governance.md']
    return {path: sha((ROOT / path).read_bytes()) for path in paths if (ROOT / path).is_file()}


def run(label, argv, cwd):
    receipt_path = OUT / (PREFIX + label + '.json')
    assert not receipt_path.exists(), receipt_path
    executable = shutil.which(argv[0])
    assert executable, argv[0]
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    before = inputs()
    timer = time.monotonic()
    with (OUT / (PREFIX + label + '.stdout.log')).open('xb') as stdout:
        with (OUT / (PREFIX + label + '.stderr.log')).open('xb') as stderr:
            process = subprocess.run([executable, *argv[1:]], cwd=cwd, stdout=stdout, stderr=stderr)
    streams = {}
    for channel in ['stdout', 'stderr']:
        path = OUT / (PREFIX + label + '.' + channel + '.log')
        raw = path.read_bytes()
        streams[channel] = {'path': path.name, 'bytes': len(raw), 'sha256': sha(raw)}
    text = (OUT / (PREFIX + label + '.stdout.log')).read_text(encoding='utf-8', errors='replace')
    summary = [line for line in text.splitlines() if any(marker in line for marker in
               ['tests ', 'pass ', 'fail ', 'all files', 'file check passed', 'vulnerabilities', 'Packaged:'])]
    receipt = {'label': label, 'argv': argv, 'resolved_executable': executable,
               'cwd': str(cwd), 'started_utc': started,
               'finished_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
               'elapsed_seconds': round(time.monotonic() - timer, 3), 'os': platform.platform(),
               'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
               'versions': {tool: subprocess.check_output([shutil.which(tool), '--version'], cwd=ROOT).decode().strip()
                            for tool in ['node', 'npm', 'just']},
               'input_sha256_before': before, 'input_sha256_after': inputs(),
               'exit_code': process.returncode, 'streams': streams, 'summary': summary,
               'boundary': 'local Windows; existing default parallelism and thresholds'}
    if 'audit' in label:
        audit = json.loads(text)
        receipt['vulnerability_counts'] = audit.get('metadata', {}).get('vulnerabilities')
    write(label + '.json', receipt)
    print(json.dumps({'label': label, 'exit_code': process.returncode, 'summary': summary,
                      'vulnerability_counts': receipt.get('vulnerability_counts')}, ensure_ascii=True))
    return process.returncode


def prepare():
    before = LOCK.read_bytes()
    parsed = json.loads(before)
    capture = json.loads((PARENT / 'new-advisories-capture.json').read_text(encoding='utf-8'))
    assert sha(before) == capture['sources_after']['ccr-vscode/package-lock.json']
    source_hashes = {}
    for record in capture['results']:
        if 'source' in record:
            path = PARENT / ('new-advisories-' + record['name'] + '.json')
            assert sha(path.read_bytes()) == record['sha256'], path
            source_hashes[path.name] = record['sha256']
    proposal_bytes = (PARENT / 'new-advisories-version-proposal.json').read_bytes()
    proposal = json.loads(proposal_bytes)
    previous = json.loads((PARENT / 'new-advisories-independent-tarball-check.json').read_text(encoding='utf-8'))
    assert sha(proposal_bytes) == previous['proposal_sha256']
    changes = [entry for entry in proposal['changes'] if entry['file'] == 'ccr-vscode/package-lock.json']
    assert [(entry['node'], entry['old_version'], entry['new_version']) for entry in changes] == [
        ('node_modules/brace-expansion', '5.0.9', '5.0.12'),
        ('node_modules/fast-uri', '3.1.7', '3.1.8')]
    assert parsed['packages']['node_modules/undici']['version'] == '7.29.1'
    revised = before
    checks = []
    for entry in changes:
        node = parsed['packages'][entry['node']]
        assert node['version'] == entry['old_version']
        for parent in entry['parents']:
            actual = parsed['packages'][parent['node']]
            assert actual['dependencies'][entry['package']] == parent['range']
        url = 'https://registry.npmjs.org/' + entry['package'] + '/' + entry['new_version']
        with urllib.request.urlopen(url, timeout=30) as response:
            registry_raw = response.read()
            registry_status = response.status
        metadata = json.loads(registry_raw)
        assert metadata['name'] == entry['package'] and metadata['version'] == entry['new_version']
        assert metadata['dist']['integrity'] == entry['new_integrity']
        assert metadata['dist']['tarball'] == entry['tarball']
        registry_path = OUT / (PREFIX + entry['package'] + '-registry.json')
        if registry_path.exists():
            assert registry_path.read_bytes() == registry_raw
        else:
            with registry_path.open('xb') as stream:
                stream.write(registry_raw)
        with urllib.request.urlopen(entry['tarball'], timeout=30) as response:
            tarball = response.read()
            tarball_status = response.status
        integrity = 'sha512-' + base64.b64encode(hashlib.sha512(tarball).digest()).decode()
        assert integrity == entry['new_integrity']
        old_registry = json.loads((PARENT / ('new-advisories-' + entry['package'] + '-registry.json')).read_text(encoding='utf-8'))
        old = old_registry['versions'][entry['old_version']]
        for field in ['dependencies', 'optionalDependencies', 'peerDependencies', 'engines', 'main', 'exports', 'type']:
            assert old.get(field) == metadata.get(field), field
        start = revised.index(('    "' + entry['node'] + '": {').encode())
        end = revised.index(b'\n    },', start)
        segment = revised[start:end]
        replacements = {'version': entry['new_version'], 'resolved': entry['tarball'], 'integrity': integrity}
        for field, value in replacements.items():
            old_field = json.dumps(field) + ': ' + json.dumps(node[field])
            new_field = json.dumps(field) + ': ' + json.dumps(value)
            assert segment.count(old_field.encode()) == 1
            segment = segment.replace(old_field.encode(), new_field.encode(), 1)
        revised = revised[:start] + segment + revised[end:]
        checks.append({'node': entry['node'], 'old_version': entry['old_version'], 'new_version': entry['new_version'],
                       'parents': entry['parents'], 'registry_url': url, 'registry_http_status': registry_status,
                       'registry_sha256': sha(registry_raw), 'tarball_url': entry['tarball'],
                       'tarball_http_status': tarball_status, 'tarball_bytes': len(tarball),
                       'tarball_sha256': sha(tarball), 'tarball_sha1': hashlib.sha1(tarball).hexdigest(),
                       'tarball_integrity': integrity, 'metadata_fields_unchanged': True})
    after = json.loads(revised)
    changed = []
    assert set(parsed['packages']) == set(after['packages'])
    for node, old in parsed['packages'].items():
        new = after['packages'][node]
        if old != new:
            fields = [field for field in set(old) | set(new) if old.get(field) != new.get(field)]
            assert set(fields) == {'version', 'resolved', 'integrity'}
            changed.append({'node': node, 'fields': sorted(fields)})
    assert [change['node'] for change in changed] == [entry['node'] for entry in changes]
    before_inputs = inputs()
    LOCK.write_bytes(revised)
    write('prepare.json', {'date_utc': dt.datetime.now(dt.timezone.utc).isoformat(), 'os': platform.platform(),
                          'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
                          'before_sha256': sha(before), 'after_sha256': sha(revised),
                          'before_crlf': before.count(b'\r\n'), 'after_crlf': revised.count(b'\r\n'),
                          'package_count': len(parsed['packages']), 'changed_nodes': changed, 'checks': checks,
                          'saved_source_sha256': source_hashes, 'input_sha256_before': before_inputs,
                          'input_sha256_after': inputs(), 'tarball_code_executed': False})
    print('Applied exactly two approved lock nodes; verified fresh registry integrity and tarballs')


if __name__ == '__main__':
    mode = sys.argv[1]
    if mode == 'prepare':
        prepare()
    else:
        commands = {
            'audit-before': (['npm', 'audit', '--json'], ROOT / 'ccr-vscode'),
            'npm-ci': (['npm', 'ci'], ROOT / 'ccr-vscode'),
            'audit-after': (['npm', 'audit', '--json'], ROOT / 'ccr-vscode'),
            'ci': (['just', 'vscode-ci'], ROOT),
            'coverage': (['just', 'vscode-coverage'], ROOT),
            'source-list': (['npx', '--no-install', 'vsce', 'ls', '--no-dependencies'], ROOT / 'ccr-vscode'),
            'final-vsix': (['node', 'scripts/check-package-files.mjs', '--vsix', 'ccr-vscode.vsix'], ROOT / 'ccr-vscode'),
            'diff-check': (['git', 'diff', '--check', '--', 'ccr-vscode/package-lock.json'], ROOT),
        }
        argv, cwd = commands[mode]
        sys.exit(run(mode, argv, cwd))
