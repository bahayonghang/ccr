"""Read official VSCE archive entries without executing package code."""
import base64
import hashlib
import io
import json
from pathlib import Path
import tarfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
EXPECTED = json.loads((ROOT / '.trellis/tasks/09-29-harness-vsix-package/research/resume-2026-10-04-vsix-candidate-tarball.json').read_text())
tarball = HERE / 'resume-2026-10-04-independent-vsce4.tgz'
payload = tarball.read_bytes()
integrity = 'sha512-' + base64.b64encode(hashlib.sha512(payload).digest()).decode()
assert integrity == EXPECTED['integrity']
assert hashlib.sha256(payload).hexdigest() == EXPECTED['sha256']
result = {'tarball': {'source': EXPECTED['url'], 'sha256': EXPECTED['sha256'], 'integrity': integrity},
          'entries': {}, 'package_code_executed': False, 'credential_access': False}
with tarfile.open(fileobj=io.BytesIO(payload), mode='r:gz') as archive:
    for name in ('package.json', 'out/secretLint.js', 'out/package.js', 'out/store.js', 'out/main.js', 'vsce'):
        data = archive.extractfile('package/' + name).read()
        target = HERE / ('resume-2026-10-04-independent-vsce4-' + name.replace('/', '-'))
        with target.open('xb') as output:
            output.write(data)
        result['entries'][name] = {'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data), 'evidence': target.name}
    result['metadata'] = json.loads(archive.extractfile('package/package.json').read())
with (HERE / 'resume-2026-10-04-independent-vsce4-source.json').open('x', encoding='utf-8', newline='\n') as output:
    output.write(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(result['entries']))
