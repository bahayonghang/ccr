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
env = os.environ.copy()
for key in ['GROK_HOME', 'CODEX_HOME', 'CCR_DATA_DIR', 'CCR_CONFIG_PATH']:
    env.pop(key, None)
checks = [
    ('check-doctor-binary', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr', '--test', 'commands', 'doctor', '--', '--skip', 'export_bindings']),
    ('check-validate-binary', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr', '--test', 'commands', 'validate', '--', '--skip', 'export_bindings']),
    ('check-doctor-service', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr-cli', 'doctor', '--', '--skip', 'export_bindings']),
    ('check-validate-service', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr-cli', 'validate', '--', '--skip', 'export_bindings']),
    ('check-config-validator', ['cargo', 'test', '--offline', '--locked', '-p', 'ccr-config', 'validator', '--', '--skip', 'export_bindings']),
    ('check-rust-types', ['cargo', 'check', '--offline', '--locked', '-p', 'ccr', '-p', 'ccr-cli', '-p', 'ccr-config', '--all-targets', '--all-features']),
    ('check-strict-clippy', ['cargo', 'clippy', '--offline', '--locked', '-p', 'ccr', '-p', 'ccr-cli', '-p', 'ccr-config', '--all-targets', '--all-features', '--', '-D', 'warnings', '-D', 'clippy::unwrap_used']),
]
final_run = '--final' in sys.argv
if final_run:
    checks = [(name + '-final', command) for name, command in checks if name in {
        'check-validate-service', 'check-rust-types', 'check-strict-clippy'
    }]
result_name = 'check-results-final.json' if final_run else 'check-results.json'
results = []
for name, command in checks:
    started = time.time()
    with (task / (name + '.log')).open('w', encoding='utf-8') as log:
        result = subprocess.run(command, cwd=root, env=env, stdout=log, stderr=subprocess.STDOUT)
    output = (task / (name + '.log')).read_text(encoding='utf-8')
    passed = sum(int(value) for value in re.findall(r'test result: ok\. (\d+) passed;', output))
    row = {'command': command, 'log': name + '.log', 'exit_code': result.returncode, 'passed': passed, 'seconds': round(time.time() - started, 2)}
    results.append(row)
    data = {'checks': results, 'cargo_lock_sha256': hashlib.sha256((root / 'Cargo.lock').read_bytes()).hexdigest()}
    (task / result_name).write_text(json.dumps(data, indent=2), encoding='utf-8')
    print(json.dumps(row), flush=True)
    if result.returncode or ('test' in command and passed == 0):
        print(output[-9000:], flush=True)
        sys.exit(result.returncode or 1)
