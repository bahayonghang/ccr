"""Read-only official metadata capture for a separate VSCE repair proposal."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import urllib.request

OUT = Path(__file__).resolve().parent
PREFIX = 'resume-2026-10-04-vsix-'


def fetch(url):
    with urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': 'CCR-task-evidence'}), timeout=30) as response:
        raw = response.read()
        return json.loads(raw), {'url': url, 'fetched_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
                                 'http_status': response.status, 'bytes': len(raw),
                                 'sha256': hashlib.sha256(raw).hexdigest()}


sources = []
selected = {}
for package in ['@vscode/vsce', 'secretlint', 'globby', 'braces']:
    registry, receipt = fetch('https://registry.npmjs.org/' + package)
    sources.append(receipt)
    versions = []
    for version, metadata in registry['versions'].items():
        if '-' in version:
            continue
        numbers = tuple(map(int, version.split('.')))
        lower = {'@vscode/vsce': (3, 9, 2), 'secretlint': (12, 3, 0), 'globby': (15, 0, 0), 'braces': (3, 0, 3)}[package]
        if numbers >= lower:
            versions.append({key: metadata.get(key) for key in ['name', 'version', 'engines', 'dependencies', 'optionalDependencies', 'dist']})
    selected[package] = {'latest': registry['dist-tags'].get('latest'), 'versions': versions}
advisory, receipt = fetch('https://api.github.com/advisories/GHSA-vfj7-8cjw-p6xm')
sources.append(receipt)
result = {'captured_utc': dt.datetime.now(dt.timezone.utc).isoformat(), 'sources': sources,
          'registry_selections': selected, 'advisory': advisory}
with (OUT / (PREFIX + 'braces-official-metadata.json')).open('x', encoding='utf-8') as stream:
    json.dump(result, stream, indent=2, ensure_ascii=False)
    stream.write('\n')
print(json.dumps({'sources': sources, 'versions': {key: [{field: value.get(field) for field in ['version', 'engines', 'dependencies']} for value in record['versions']] for key, record in selected.items()}, 'advisory': {key: advisory.get(key) for key in ['ghsa_id', 'published_at', 'updated_at', 'vulnerabilities']}}, indent=2))
