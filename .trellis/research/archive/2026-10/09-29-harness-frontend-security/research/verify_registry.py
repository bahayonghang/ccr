"""Verify the four scoped patch releases against public registry evidence."""
import base64
from concurrent.futures import ThreadPoolExecutor
import datetime
import hashlib
import json
from pathlib import Path
import re
import urllib.request

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent / 'evidence'
TARGETS = {'colord': '2.9.4', 'fast-uri': '3.1.7',
           'js-yaml': '4.3.2', 'undici': '8.10.2'}


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'CCR-T08-audit'})
    with urllib.request.urlopen(request, timeout=45) as response:
        return response.read()


def save(name, data):
    (EVIDENCE / name).write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n',
                                 encoding='utf-8', newline='\n')


def main():
    lock = (ROOT / 'ccr-ui/bun.lock').read_text(encoding='utf-8')
    entries = {}
    for line in lock.splitlines():
        match = re.fullmatch(r'    "([^"]+)": (\[.*\]),?', line)
        if match:
            entries[match[1]] = json.loads(match[2])
    baseline = EVIDENCE / 'lock-before.json'
    if baseline.exists():
        entries = json.loads(baseline.read_text(encoding='utf-8'))
    else:
        save('lock-before.json', entries)
    report = json.loads((EVIDENCE / 'before-audit.stdout.log').read_text())
    changes = []
    for name, version in TARGETS.items():
        registry_path = EVIDENCE / f'{name}-registry.json'
        registry = json.loads(registry_path.read_text(encoding='utf-8-sig'))
        old_entry = entries[name]
        old_version = old_entry[0].rsplit('@', 1)[1]
        old = registry['versions'][old_version]
        new = registry['versions'][version]
        assert old_entry[-1] == old['dist']['integrity'], name
        assert old.get('dependencies', {}) == new.get('dependencies', {}), name
        tarball = fetch(new['dist']['tarball'])
        actual = 'sha512-' + base64.b64encode(hashlib.sha512(tarball).digest()).decode()
        assert actual == new['dist']['integrity'], name
        parents = []
        for parent, entry in entries.items():
            required = entry[2].get('dependencies', {}).get(name)
            if required:
                parents.append({'package': entry[0], 'range': required})
        changes.append({'name': name, 'old_version': old_version, 'new_version': version,
                        'old_integrity': old['dist']['integrity'],
                        'new_integrity': new['dist']['integrity'],
                        'tarball': new['dist']['tarball'], 'tarball_bytes': len(tarball),
                        'verified_tarball_integrity': actual, 'parents': parents,
                        'old_engines': old.get('engines'), 'new_engines': new.get('engines'),
                        'dependencies': new.get('dependencies', {}),
                        'old_published': registry['time'][old_version],
                        'new_published': registry['time'][version],
                        'advisories': report[name]})
        save(f'{name}-selected-releases.json',
             {'source': f'https://registry.npmjs.org/{name}',
              'fetched_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'old': old, 'new': new})
    save('dependency-changes.json', {'package_count_before': len(entries),
                                   'changes': changes})
    advisories = [(name, advisory) for name, group in report.items() for advisory in group]

    def fetch_advisory(item):
        name, advisory = item
        advisory_id = advisory['url'].rsplit('/', 1)[1]
        source = f'https://api.github.com/advisories/{advisory_id}'
        try:
            data = json.loads(fetch(source))
            return {'package': name, 'source': source, 'status': 'fetched',
                    'fetched_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
                    'advisory': data}
        except Exception as error:
            return {'package': name, 'source': source, 'status': 'fetch_failed',
                    'error': str(error)}

    with ThreadPoolExecutor(max_workers=4) as pool:
        official = list(pool.map(fetch_advisory, advisories))
    save('official-advisories.json', official)
    print(json.dumps({'verified_changes': changes,
                      'official_fetched': sum(x['status'] == 'fetched' for x in official),
                      'official_failed': [x for x in official if x['status'] != 'fetched']},
                     ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
