import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import time
import tomllib
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
DB = Path.home() / '.cargo' / 'advisory-db'
MANIFEST = 'ccr-ui/src-tauri/Cargo.toml'
WARNINGS = [
    ('fxhash', '0.2.1', 'RUSTSEC-2025-0057'),
    ('proc-macro-error', '1.0.4', 'RUSTSEC-2024-0370'),
    ('unic-char-property', '0.9.0', 'RUSTSEC-2025-0081'),
    ('unic-char-range', '0.9.0', 'RUSTSEC-2025-0075'),
    ('unic-common', '0.9.0', 'RUSTSEC-2025-0080'),
    ('unic-ucd-ident', '0.9.0', 'RUSTSEC-2025-0100'),
    ('unic-ucd-version', '0.9.0', 'RUSTSEC-2025-0098'),
    ('glib', '0.18.5', 'RUSTSEC-2024-0429'),
    ('rand', '0.7.3', 'RUSTSEC-2026-0097'),
]
TARGETS = ['x86_64-pc-windows-msvc', 'x86_64-unknown-linux-gnu', 'aarch64-apple-darwin']
INPUTS = ['Cargo.toml', 'Cargo.lock', MANIFEST, 'ccr-ui/src-tauri/Cargo.lock',
          'ccr-ui/src-tauri/command-macros/Cargo.toml', '.cargo/config.toml',
          '.cargo/audit.toml', 'rust-toolchain.toml']
INPUTS += sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / 'crates').glob('*/Cargo.toml'))

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def hashes():
    return {p: sha(ROOT / p) for p in INPUTS}

def save(name, data):
    (OUT / name).write_text(json.dumps(data, ensure_ascii=False, indent=2, default=str) + '\n', encoding='utf-8', newline='\n')

records = []
initial_hashes = hashes()

def run(label, args, env=None):
    started = datetime.now(timezone.utc).isoformat()
    timer = time.monotonic()
    result = subprocess.run(args, cwd=ROOT, env=env, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=180)
    record = {'label': label, 'command': args, 'cwd': str(ROOT), 'started_utc': started,
              'finished_utc': datetime.now(timezone.utc).isoformat(), 'seconds': round(time.monotonic()-timer, 3),
              'exit_code': result.returncode, 'input_sha256': initial_hashes,
              'stdout_sha256': hashlib.sha256(result.stdout.encode('utf-8')).hexdigest(),
              'stderr_sha256': hashlib.sha256(result.stderr.encode('utf-8')).hexdigest(),
              'stdout': result.stdout, 'stderr': result.stderr}
    records.append(record)
    save('tauri-advisory-commands.json', records)
    print(label, 'exit', result.returncode, flush=True)
    return result

metadata = {}
for label, args in [('cargo-version', ['cargo','--version']), ('rustc-version',['rustc','-vV']),
                    ('audit-version',['cargo','audit','--version']), ('head',['git','rev-parse','HEAD']),
                    ('advisory-db-head',['git','-C',str(DB),'rev-parse','HEAD']),
                    ('advisory-db-status',['git','-C',str(DB),'status','--porcelain'])]:
    metadata[label] = run(label,args).stdout.strip()

sources = []
for package, version, advisory in WARNINGS:
    path = DB / 'crates' / package / (advisory + '.md')
    text = path.read_text(encoding='utf-8')
    fields = tomllib.loads(re.search(r'\x60{3}toml\n(.*?)\n\x60{3}', text, re.S).group(1))
    sources.append({'package': package, 'locked_version': version, 'advisory_id': advisory,
                    'source': str(path), 'source_sha256': sha(path), 'fields': fields})
save('tauri-advisory-sources.json', {'metadata': metadata, 'sources': sources,
     'database_fetch': 'not requested; cached official RustSec Git checkout', 'input_sha256': initial_hashes})

# A diagnostic informational-warning query. no-yanked is explicit; this is not a full fresh audit.
audit = run('cached-audit-no-yanked', ['cargo','audit','--file','ccr-ui/src-tauri/Cargo.lock','--no-fetch','--no-yanked','--json'])
if audit.returncode == 0:
    save('tauri-advisory-audit.json', json.loads(audit.stdout))

base = ['cargo','tree','--locked','--offline','--manifest-path',MANIFEST,'--charset','ascii','-f','{p}|features={f}']
summary = []
for target in TARGETS:
    for mode, feature_flags in [('default', []), ('all-features', ['--all-features'])]:
        for edges in ['normal,build', 'normal,no-proc-macro']:
            label = target + '-' + mode + '-' + edges.replace(',','-')
            result = run(label, base + ['--target',target,'--edges',edges,'--prefix','none'] + feature_flags)
            found = {}
            for package, version, advisory in WARNINGS:
                lines = [line for line in result.stdout.splitlines() if line.startswith(package + ' v' + version + '|') or line.startswith(package + ' v' + version + ' ')]
                found[advisory] = {'package': package, 'version': version, 'present': bool(lines),
                                   'feature_lines': sorted(set(lines))}
            summary.append({'target': target, 'features': mode, 'edges': edges, 'exit_code': result.returncode, 'warnings': found})
for package, version, advisory in WARNINGS:
    run('inverse-linux-all-features-' + package, base + ['--target','x86_64-unknown-linux-gnu',
        '--edges','normal,build','--invert',package+'@'+version,'--all-features'])
final_hashes = hashes()
assert initial_hashes == final_hashes, 'Cargo input changed during diagnostic capture'
save('tauri-advisory-reachability.json', {'metadata': metadata, 'matrix': summary, 'input_sha256_before': initial_hashes,
     'input_sha256_after': final_hashes, 'unchanged_inputs': True,
     'scope': 'Static target/feature graphs on a Windows host; not compilation, runtime use, or exploitability proof'})
print('CAPTURE_COMPLETE', len(records), 'commands', flush=True)
