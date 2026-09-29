"""Install pinned CI tools into a task-local Linux directory."""
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import urllib.request
import zipfile
from datetime import datetime, timezone

ROOT = Path('/tmp/ccr-architecture-tools-01a0e781')
BIN = ROOT / 'bin'
BIN.mkdir(parents=True, exist_ok=True)
ASSETS = [
    ('bun', 'https://github.com/oven-sh/bun/releases/download/bun-v1.4.0/bun-linux-x64.zip', '2d03fb5fb83ac8b567aca0a281b2ce1a1a19d488f56c2968d88c3f25e92fe452', 'bun-linux-x64/bun'),
    ('just', 'https://github.com/casey/just/releases/download/1.58.0/just-1.58.0-x86_64-unknown-linux-musl.tar.gz', '4a5cc2f53e6f0f8c59092a6cc38291eb729d46a7dd95d3ae582008881b84931d', 'just'),
    ('cargo-llvm-cov', 'https://github.com/taiki-e/cargo-llvm-cov/releases/download/v0.9.0/cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz', 'b068f7c98841aacb9c4f382b4a0c184ae82f49b56a32d442b429b2961c73be15', 'cargo-llvm-cov'),
]
records = []
for name, url, expected, member in ASSETS:
    archive_path = ROOT / url.rsplit('/', 1)[1]
    if not archive_path.exists():
        with urllib.request.urlopen(url, timeout=60) as response:
            archive_path.write_bytes(response.read())
    content = archive_path.read_bytes()
    actual = hashlib.sha256(content).hexdigest()
    if actual != expected:
        raise RuntimeError(f'{name}: checksum mismatch: {actual} != {expected}')
    if url.endswith('.zip'):
        with zipfile.ZipFile(io.BytesIO(content)) as archive:
            binary = archive.read(member)
    else:
        with tarfile.open(fileobj=io.BytesIO(content), mode='r:gz') as archive:
            entry = archive.getmember(member)
            if not entry.isfile():
                raise RuntimeError(f'{name}: expected regular file')
            binary = archive.extractfile(entry).read()
    target = BIN / name
    target.write_bytes(binary)
    target.chmod(0o755)
    arguments = [str(target), 'llvm-cov', '--version'] if name == 'cargo-llvm-cov' else [str(target), '--version']
    result = subprocess.run(arguments, capture_output=True, text=True, check=True)
    records.append({'name': name, 'url': url, 'archive_sha256': actual, 'binary_sha256': hashlib.sha256(binary).hexdigest(), 'version': result.stdout.strip(), 'path': str(target)})
    print(json.dumps(records[-1]), flush=True)
evidence = {'recorded_at_utc': datetime.now(timezone.utc).isoformat(), 'scope': 'Pinned tools only; no Cargo gate or source generation started', 'tools': records, 'rustc': subprocess.check_output(['rustc', '--version'], text=True).strip(), 'rustup_components': subprocess.check_output(['rustup', 'component', 'list', '--installed', '--toolchain', '1.98.0'], text=True).splitlines(), 'disk': subprocess.check_output(['df', '-h', '/tmp'], text=True)}
Path(__file__).with_name('remaining-linux-toolchain.json').write_text(json.dumps(evidence, indent=2) + chr(10), encoding='utf-8')
