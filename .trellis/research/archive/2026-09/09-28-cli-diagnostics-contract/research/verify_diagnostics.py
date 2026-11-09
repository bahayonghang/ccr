import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

sys.stdout.reconfigure(encoding='utf-8')
root = Path(__file__).resolve().parents[4]
task = root / '.trellis/tasks/09-28-cli-diagnostics-contract'
checks = [
    ('binary-verified', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr', '--test', 'diagnostics_contract']),
    ('doctor-binary-verified', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr', '--test', 'commands', 'doctor']),
    ('validate-binary-verified', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr', '--test', 'commands', 'validate']),
    ('validate-unit-verified', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr-cli', 'validate']),
    ('validator-unit-verified', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr-config', 'validator']),
    ('clippy-verified', ['cargo', 'clippy', '--offline', '--locked', '-p', 'ccr', '-p', 'ccr-cli', '-p', 'ccr-config', '--all-targets', '--all-features', '--', '-D', 'warnings', '-D', 'clippy::unwrap_used']),
]
results = []
for name, command in checks:
    start = time.time()
    with (task / f'{name}.log').open('w', encoding='utf-8') as log:
        completed = subprocess.run(command, cwd=root, env=os.environ.copy(), stdout=log, stderr=subprocess.STDOUT)
    output = (task / f'{name}.log').read_text(encoding='utf-8')
    passed = sum(int(value) for value in re.findall(r'test result: ok\. (\d+) passed;', output))
    record = {'name': name, 'command': command, 'exit_code': completed.returncode, 'passed': passed, 'seconds': round(time.time()-start, 2)}
    results.append(record)
    (task / 'verification-results.json').write_text(json.dumps({'checks': results, 'cargo_lock_sha256': hashlib.sha256((root/'Cargo.lock').read_bytes()).hexdigest()}, indent=2), encoding='utf-8')
    print(json.dumps(record), flush=True)
    if completed.returncode or ('test' in command and passed == 0):
        print(output[-8000:], flush=True)
        sys.exit(completed.returncode or 1)
