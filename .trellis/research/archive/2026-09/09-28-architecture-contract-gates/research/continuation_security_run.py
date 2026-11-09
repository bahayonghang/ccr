"""Record one explicitly selected security validation command without a shell."""
import argparse
import datetime
import json
import os
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument('--label', required=True)
parser.add_argument('--cwd', default='.')
parser.add_argument('--env', action='append', default=[])
parser.add_argument('command', nargs=argparse.REMAINDER)
args = parser.parse_args()
command = args.command[1:] if args.command[:1] == ['--'] else args.command
if not command:
    parser.error('A command is required')
if any(char not in 'abcdefghijklmnopqrstuvwxyz0123456789-' for char in args.label):
    parser.error('Use a lowercase filename label')
research = Path(__file__).resolve().parent
log = research / f'continuation-security-{args.label}.log'
metadata = research / f'continuation-security-{args.label}.json'
if log.exists() or metadata.exists():
    parser.error('Preserve existing evidence; choose a new label')
overrides = dict(pair.split('=', 1) for pair in args.env)
environment = dict(os.environ, **overrides)
started_at = datetime.datetime.now(datetime.timezone.utc).isoformat()
started = time.monotonic()
with log.open('wb') as output:
    result = subprocess.run(command, cwd=args.cwd, env=environment, stdout=output, stderr=subprocess.STDOUT, check=False)
record = {
    'command': command,
    'cwd': str(Path(args.cwd).resolve()),
    'environment_overrides': overrides,
    'started_at': started_at,
    'exit_code': result.returncode,
    'seconds': round(time.monotonic() - started, 3),
    'log': log.name,
}
metadata.write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, ensure_ascii=False))
print('\n'.join(log.read_text(encoding='utf-8', errors='replace').splitlines()[-18:]))
raise SystemExit(result.returncode)
