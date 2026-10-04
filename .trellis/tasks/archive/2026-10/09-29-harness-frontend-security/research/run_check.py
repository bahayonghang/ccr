"""Capture T08 command evidence without modifying shared gate behavior."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent / 'evidence'
INPUTS = ('ccr-ui/package.json', 'ccr-ui/bun.lock',
          'ccr-ui/scripts/frontend-audit-allowlist.json',
          'ccr-ui/scripts/audit-dependencies.mjs', 'justfile')


def hashes():
    return {name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
            for name in INPUTS}


def main():
    sys.stdout.reconfigure(encoding='utf-8', errors='replace')
    sys.stderr.reconfigure(encoding='utf-8', errors='replace')
    parser = argparse.ArgumentParser()
    parser.add_argument('--cwd', default='.')
    parser.add_argument('--skip-icons', action='store_true')
    parser.add_argument('label')
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    command = args.command
    if command and command[0] == '--':
        command = command[1:]
    if not command:
        parser.error('A command is required')
    executable = shutil.which(command[0])
    if not executable:
        parser.error(f'Command not found: {command[0]}')
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    out = EVIDENCE / f'{args.label}.stdout.log'
    err = EVIDENCE / f'{args.label}.stderr.log'
    receipt = EVIDENCE / f'{args.label}.json'
    if any(path.exists() for path in (out, err, receipt)):
        parser.error(f'Evidence label already exists: {args.label}')
    env = dict(os.environ)
    overrides = {'CCR_SKIP_ICON_GENERATION': '1'} if args.skip_icons else {}
    env.update(overrides)
    before = hashes()
    start = datetime.datetime.now(datetime.timezone.utc).isoformat()
    tick = time.monotonic()
    with out.open('wb') as stdout, err.open('wb') as stderr:
        result = subprocess.run([executable, *command[1:]],
                                cwd=ROOT / args.cwd, env=env,
                                stdout=stdout, stderr=stderr, check=False)
    data = {'command': command, 'executable': executable,
            'cwd': str(ROOT / args.cwd), 'platform': platform.platform(),
            'started_utc': start,
            'finished_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'duration_seconds': round(time.monotonic() - tick, 3),
            'environment_overrides': overrides, 'exit_code': result.returncode,
            'input_sha256_before': before, 'input_sha256_after': hashes(),
            'stdout_sha256': hashlib.sha256(out.read_bytes()).hexdigest(),
            'stderr_sha256': hashlib.sha256(err.read_bytes()).hexdigest()}
    receipt.write_text(json.dumps(data, indent=2, ensure_ascii=False) + '\n',
                       encoding='utf-8', newline='\n')
    print(json.dumps({'receipt': str(receipt), 'exit_code': result.returncode,
                      'duration_seconds': data['duration_seconds']}))
    for path in (out, err):
        print(f'{path.name}:')
        print('\n'.join(path.read_text(encoding='utf-8', errors='replace').splitlines()[-16:]))
    return result.returncode


if __name__ == '__main__':
    sys.exit(main())
