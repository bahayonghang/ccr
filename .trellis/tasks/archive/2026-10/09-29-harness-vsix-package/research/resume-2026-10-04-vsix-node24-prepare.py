"""Stage one official Node executable in an owned Temp directory, without global installation."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import tempfile
import urllib.request

OUT = Path(__file__).resolve().parent
ROOT = OUT.parents[3]
PREFIX = 'resume-2026-10-04-vsix-node24-'
started = dt.datetime.now(dt.timezone.utc).isoformat()
lock_before = hashlib.sha256((ROOT / 'ccr-vscode/package-lock.json').read_bytes()).hexdigest()
temporary_root = Path(tempfile.mkdtemp(prefix='ccr-vscode-node24-')).resolve()
assert temporary_root.is_relative_to(Path(tempfile.gettempdir()).resolve())
base = 'https://nodejs.org/dist/v24.20.0/'
with urllib.request.urlopen(base + 'SHASUMS256.txt', timeout=30) as response:
    manifest = response.read()
    manifest_status = response.status
with (OUT / (PREFIX + 'SHASUMS256.txt')).open('xb') as stream:
    stream.write(manifest)
matches = [line.split()[0] for line in manifest.decode().splitlines() if line.split()[1] == 'win-x64/node.exe']
assert len(matches) == 1
with urllib.request.urlopen(base + 'win-x64/node.exe', timeout=60) as response:
    binary = response.read()
    binary_status = response.status
actual = hashlib.sha256(binary).hexdigest()
assert actual == matches[0]
executable = temporary_root / 'node.exe'
executable.write_bytes(binary)
version = subprocess.run([str(executable), '--version'], capture_output=True, cwd=temporary_root)
assert version.returncode == 0 and version.stdout.strip() == b'v24.20.0'
for channel, raw in [('stdout', version.stdout), ('stderr', version.stderr)]:
    with (OUT / (PREFIX + 'version.' + channel + '.log')).open('xb') as stream:
        stream.write(raw)
lock_after = hashlib.sha256((ROOT / 'ccr-vscode/package-lock.json').read_bytes()).hexdigest()
assert lock_before == lock_after
receipt = {'started_utc': started, 'finished_utc': dt.datetime.now(dt.timezone.utc).isoformat(),
           'os': platform.platform(), 'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(),
           'manifest_url': base + 'SHASUMS256.txt', 'manifest_http_status': manifest_status,
           'manifest_sha256': hashlib.sha256(manifest).hexdigest(),
           'binary_url': base + 'win-x64/node.exe', 'binary_http_status': binary_status,
           'binary_bytes': len(binary), 'binary_sha256': actual, 'expected_sha256': matches[0],
           'temporary_executable': str(executable), 'argv': [str(executable), '--version'],
           'actual_version': version.stdout.decode().strip(), 'exit_code': version.returncode,
           'input_lock_sha256_before': lock_before, 'input_lock_sha256_after': lock_after,
           'global_installation': False, 'actual_extension_node_modules_changed': False}
with (OUT / (PREFIX + 'receipt.json')).open('x', encoding='utf-8') as stream:
    json.dump(receipt, stream, indent=2)
    stream.write('\n')
print(json.dumps(receipt, indent=2))
